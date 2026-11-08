pub(crate) mod arguments;
pub(crate) mod capture_type;
pub(crate) mod parameter;
pub(crate) mod parse_expr;
pub(crate) mod parse_signature;
pub(crate) mod path_or_ident;
pub(crate) mod postfix;
pub(crate) mod prefix;
pub(crate) mod starts_argument;
pub(crate) mod stmt_call;
pub(crate) mod token_text;
pub(crate) mod unary_op;

use crate::ParseError;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::parse_signature::parse_signature;
use crate::parse::parser::stmt_call::stmt_call;
use redox_ast::{Binding, Block, FnDecl, Ident, Item, Program, Stmt, StmtKind, TypeExpr};
use redox_lexer::{Keyword, Span, Token, TokenKind, Trivia};

/// The recursive-descent parser.
///
/// Private: [`crate::parse`] and [`crate::parse_source`] are the only ways in.
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) grew it from plan 01's three
/// productions to the expression, signature and binding surface, with the
/// expression grammar a Pratt loop over `crate::parse::precedence`.
///
/// Errors recover at **two** levels: an item boundary, and a statement
/// boundary within a block. The second is what stops one bad statement from
/// discarding its siblings — which matters for a language whose bodies are
/// indented blocks, and which `docs/COMPILER.md` §1 records LALRPOP never
/// offering at all.
pub(crate) struct Parser<'a> {
  toks: &'a [Token],
  pos: usize,
  errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
  pub(crate) fn new(toks: &'a [Token]) -> Self {
    Self {
      toks,
      pos: 0,
      errors: Vec::new(),
    }
  }

  /// Parse the whole token stream, consuming the parser.
  pub(crate) fn run(mut self, src_len: usize) -> Result<Program, Vec<ParseError>> {
    let items = self.program();
    if self.errors.is_empty() {
      let trailing = self.eof_trivia();
      Ok(Program {
        items,
        span: (0, src_len),
        trailing,
      })
    } else {
      Err(self.errors)
    }
  }

  fn program(&mut self) -> Vec<Item> {
    let mut items = Vec::new();
    self.skip_separators();
    while !self.at_eof() {
      match self.item() {
        Ok(item) => items.push(item),
        Err(e) => {
          self.errors.push(e);
          self.recover_to_item();
        }
      }
      self.skip_separators();
    }
    items
  }

  /// `Item := Ident "=" Signature? "->" Newline Indent Stmt+ Dedent`
  fn item(&mut self) -> Result<Item, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;

    let name = self.ident("a declaration must start with a name")?;
    self.expect(&TokenKind::Eq, "expected `=` after the declaration name")?;
    let sig = parse_signature(self)?;
    self.expect(&TokenKind::Arrow, "expected `->` after the signature")?;
    let trailing = self.newline_with_trailing("expected a line break after `->`")?;
    self.expect(&TokenKind::Indent, "expected an indented body")?;

    let body_start = self.peek_span().0;
    let mut stmts = Vec::new();
    loop {
      self.skip_separators();
      if self.check(&TokenKind::Dedent) || self.at_eof() {
        break;
      }
      // Statement-level recovery: a bad statement is reported and skipped,
      // and its siblings are still parsed.
      match self.stmt() {
        Ok(stmt) => stmts.push(stmt),
        Err(e) => {
          self.errors.push(e);
          self.recover_to_stmt();
        }
      }
    }
    let body_end = self.prev_span().1;
    self.expect(&TokenKind::Dedent, "expected the body to end")?;

    if stmts.is_empty() && self.errors.is_empty() {
      return Err(ParseError {
        message: "a declaration body must contain at least one statement".to_string(),
        span: (body_start, body_start),
      });
    }

    let end = self.prev_span().1;
    Ok(Item::Fn(FnDecl {
      name,
      sig,
      body: Block {
        stmts,
        span: (body_start, body_end),
      },
      span: (start, end),
      leading,
      trailing,
    }))
  }

  /// `Stmt := Binding | "return" Expr? | Expr`, each closed by a `Newline`.
  fn stmt(&mut self) -> Result<Stmt, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;

    let kind = if self.check(&TokenKind::Kw(Keyword::Return)) {
      self.advance();
      if self.check(&TokenKind::Newline) || self.check(&TokenKind::Dedent) || self.at_eof() {
        StmtKind::Return(None)
      } else {
        StmtKind::Return(Some(parse_expr(self, 0)?))
      }
    } else if let Some(binding) = self.try_binding()? {
      StmtKind::Binding(binding)
    } else if let Some(call) = stmt_call(self)? {
      StmtKind::Expr(call)
    } else {
      StmtKind::Expr(parse_expr(self, 0)?)
    };

    let end = self.prev_span().1;
    let trailing = self.newline_with_trailing("expected a line break after the statement")?;
    Ok(Stmt {
      kind,
      span: (start, end),
      leading,
      trailing,
    })
  }

  /// A binding, if the cursor is on one.
  ///
  /// Recognised by lookahead rather than by backtracking: `mut` opens one
  /// outright, and otherwise an `Ident` followed by `=` or `:` is a binding
  /// while anything else is an expression statement.
  fn try_binding(&mut self) -> Result<Option<Binding>, ParseError> {
    let is_mut = self.check(&TokenKind::Kw(Keyword::Mut));
    if !is_mut {
      let looks_like = matches!(self.peek().kind, TokenKind::Ident(_))
        && matches!(
          self.peek_at(1).map(|t| &t.kind),
          Some(TokenKind::Eq) | Some(TokenKind::Colon)
        );
      if !looks_like {
        return Ok(None);
      }
    }

    let start = self.peek_span().0;
    if is_mut {
      self.advance();
    }
    let name = self.ident("expected a name in a binding")?;

    let ty = if self.check(&TokenKind::Colon) {
      self.advance();
      Some(self.type_expr("expected a type after `:`")?)
    } else {
      None
    };

    self.expect(&TokenKind::Eq, "expected `=` in a binding")?;
    let value = parse_expr(self, 0)?;
    Ok(Some(Binding {
      name,
      ty,
      value,
      is_mut,
      span: (start, self.prev_span().1),
    }))
  }

  /// Capture a type as verbatim source text (plan 03 §1.1).
  pub(crate) fn type_expr(&mut self, message: &str) -> Result<TypeExpr, ParseError> {
    capture_type(self, message)
  }

  // ---- token helpers, shared with the grammar functions ----

  pub(crate) fn peek(&self) -> &'a Token {
    // `lex` always terminates the stream with `Eof`, so the last token is a
    // safe clamp rather than an out-of-bounds risk.
    self.toks.get(self.pos).unwrap_or_else(|| {
      self
        .toks
        .last()
        .expect("the token stream always contains at least Eof")
    })
  }

  /// The token `n` positions ahead — the bounded lookahead `try_binding`
  /// needs, and which plan 19's call grammar will need too.
  pub(crate) fn peek_at(&self, n: usize) -> Option<&'a Token> {
    self.toks.get(self.pos + n)
  }

  pub(crate) fn peek_span(&self) -> Span {
    self.peek().span
  }

  fn peek_leading(&self) -> Vec<Trivia> {
    self.peek().leading.clone()
  }

  pub(crate) fn prev_span(&self) -> Span {
    if self.pos == 0 {
      return (0, 0);
    }
    self.toks[self.pos - 1].span
  }

  pub(crate) fn at_eof(&self) -> bool {
    matches!(self.peek().kind, TokenKind::Eof)
  }

  pub(crate) fn check(&self, kind: &TokenKind) -> bool {
    &self.peek().kind == kind
  }

  pub(crate) fn advance(&mut self) {
    if !self.at_eof() {
      self.pos += 1;
    }
  }

  pub(crate) fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<(), ParseError> {
    if self.check(kind) {
      self.advance();
      Ok(())
    } else {
      Err(ParseError {
        message: message.to_string(),
        span: self.peek_span(),
      })
    }
  }

  /// Consume `kw` if it is there, reporting whether it was.
  pub(crate) fn eat_keyword(&mut self, kw: Keyword) -> bool {
    if self.check(&TokenKind::Kw(kw)) {
      self.advance();
      return true;
    }
    false
  }

  pub(crate) fn ident(&mut self, message: &str) -> Result<Ident, ParseError> {
    if let TokenKind::Ident(text) = &self.peek().kind {
      let ident = Ident {
        text: text.clone(),
        span: self.peek_span(),
      };
      self.advance();
      Ok(ident)
    } else {
      Err(ParseError {
        message: message.to_string(),
        span: self.peek_span(),
      })
    }
  }

  /// Consume a `Newline`, returning any comment that trailed its line.
  fn newline_with_trailing(&mut self, message: &str) -> Result<Option<Trivia>, ParseError> {
    if self.check(&TokenKind::Newline) {
      let trailing = self.peek().trailing.clone();
      self.advance();
      Ok(trailing)
    } else if self.at_eof() || self.check(&TokenKind::Dedent) {
      // A file ending without a trailing newline, or a block whose last
      // statement is immediately followed by a dedent, is well-formed.
      Ok(None)
    } else {
      Err(ParseError {
        message: message.to_string(),
        span: self.peek_span(),
      })
    }
  }

  fn skip_separators(&mut self) {
    while self.check(&TokenKind::Newline) {
      self.advance();
    }
  }

  /// Trivia left over at `Eof` — a comment at the end of the file.
  fn eof_trivia(&self) -> Vec<Trivia> {
    self
      .toks
      .last()
      .filter(|t| matches!(t.kind, TokenKind::Eof))
      .map(|t| t.leading.clone())
      .unwrap_or_default()
  }

  /// Skip to the next item boundary: a top-level name with the indent stack
  /// unwound.
  fn recover_to_item(&mut self) {
    let mut depth = 0usize;
    loop {
      match &self.peek().kind {
        TokenKind::Eof => return,
        TokenKind::Indent => {
          depth += 1;
          self.advance();
        }
        TokenKind::Dedent => {
          depth = depth.saturating_sub(1);
          self.advance();
        }
        TokenKind::Ident(_) if depth == 0 => return,
        _ => self.advance(),
      }
    }
  }

  /// Skip to the end of the current statement, so its siblings still parse.
  fn recover_to_stmt(&mut self) {
    let mut depth = 0usize;
    loop {
      match &self.peek().kind {
        TokenKind::Eof => return,
        TokenKind::Newline if depth == 0 => {
          self.advance();
          return;
        }
        TokenKind::Dedent if depth == 0 => return,
        TokenKind::Indent => {
          depth += 1;
          self.advance();
        }
        TokenKind::Dedent => {
          depth = depth.saturating_sub(1);
          self.advance();
        }
        _ => self.advance(),
      }
    }
  }
}
