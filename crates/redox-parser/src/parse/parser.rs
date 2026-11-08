use crate::ParseError;
use redox_ast::{Block, Expr, FnDecl, Ident, Item, Program, Stmt, StmtKind};
use redox_lexer::{Span, Token, TokenKind, Trivia};

/// The recursive-descent parser.
///
/// Private: [`crate::parse`] and [`crate::parse_source`] are the only ways in.
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
    // Leading `Newline`s before the first item are separators with nothing to
    // separate; skip them so a file starting with a blank line still parses.
    self.skip_separators();
    while !self.at_eof() {
      match self.item() {
        Ok(item) => items.push(item),
        Err(e) => {
          self.errors.push(e);
          self.recover();
        }
      }
      self.skip_separators();
    }
    items
  }

  /// `Item := Ident "=" "->" Newline Indent Stmt+ Dedent`
  fn item(&mut self) -> Result<Item, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;

    let name = self.ident("a declaration must start with a name")?;
    self.expect(&TokenKind::Eq, "expected `=` after the declaration name")?;
    self.expect(&TokenKind::Arrow, "expected `->` after `=`")?;
    let trailing = self.newline_with_trailing("expected a line break after `->`")?;
    self.expect(&TokenKind::Indent, "expected an indented body")?;

    let body_start = self.peek_span().0;
    let mut stmts = Vec::new();
    loop {
      self.skip_separators();
      if self.check(&TokenKind::Dedent) || self.at_eof() {
        break;
      }
      stmts.push(self.stmt()?);
    }
    let body_end = self.prev_span().1;
    self.expect(&TokenKind::Dedent, "expected the body to end")?;

    if stmts.is_empty() {
      return Err(ParseError {
        message: "a declaration body must contain at least one statement".to_string(),
        span: (body_start, body_start),
      });
    }

    let end = self.prev_span().1;
    Ok(Item::Fn(FnDecl {
      name,
      body: Block {
        stmts,
        span: (body_start, body_end),
      },
      span: (start, end),
      leading,
      trailing,
    }))
  }

  /// `Stmt := Expr Newline`
  fn stmt(&mut self) -> Result<Stmt, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;
    let expr = self.expr()?;
    let end = self.prev_span().1;
    let trailing = self.newline_with_trailing("expected a line break after the statement")?;
    Ok(Stmt {
      kind: StmtKind::Expr(expr),
      span: (start, end),
      leading,
      trailing,
    })
  }

  /// `Expr := Ident StrLit | StrLit | Ident`
  fn expr(&mut self) -> Result<Expr, ParseError> {
    match &self.peek().kind {
      TokenKind::StrLit(raw) => {
        let value = raw.clone();
        let span = self.peek_span();
        self.advance();
        Ok(Expr::StrLit { value, span })
      }
      TokenKind::Ident(_) => {
        let callee = self.ident("expected a name")?;
        // A string literal immediately after a name is a parenless call with
        // one argument. Nothing else can follow a name in this slice.
        if let TokenKind::StrLit(raw) = &self.peek().kind {
          let value = raw.clone();
          let arg_span = self.peek_span();
          self.advance();
          let span = (callee.span.0, arg_span.1);
          Ok(Expr::Call {
            callee: Box::new(Expr::Ident(callee)),
            args: vec![Expr::StrLit {
              value,
              span: arg_span,
            }],
            span,
          })
        } else {
          Ok(Expr::Ident(callee))
        }
      }
      _ => Err(ParseError {
        message: "expected an expression".to_string(),
        span: self.peek_span(),
      }),
    }
  }

  // ---- token helpers ----

  fn peek(&self) -> &'a Token {
    // `lex` always terminates the stream with `Eof`, so the last token is a
    // safe clamp rather than an out-of-bounds risk.
    self.toks.get(self.pos).unwrap_or_else(|| {
      self
        .toks
        .last()
        .expect("the token stream always contains at least Eof")
    })
  }

  fn peek_span(&self) -> Span {
    self.peek().span
  }

  fn peek_leading(&self) -> Vec<Trivia> {
    self.peek().leading.clone()
  }

  fn prev_span(&self) -> Span {
    if self.pos == 0 {
      return (0, 0);
    }
    self.toks[self.pos - 1].span
  }

  fn at_eof(&self) -> bool {
    matches!(self.peek().kind, TokenKind::Eof)
  }

  fn check(&self, kind: &TokenKind) -> bool {
    &self.peek().kind == kind
  }

  fn advance(&mut self) {
    if !self.at_eof() {
      self.pos += 1;
    }
  }

  fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<(), ParseError> {
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

  fn ident(&mut self, message: &str) -> Result<Ident, ParseError> {
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
      // A file that ends without a trailing newline, or a block whose last
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

  /// Skip to the next plausible item boundary: a top-level `Ident` with the
  /// indent stack unwound. This is what lets one pass report several bad
  /// items — bake-off axis 3 in `docs/COMPILER.md` §1.
  fn recover(&mut self) {
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
}
