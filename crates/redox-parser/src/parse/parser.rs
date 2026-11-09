pub(crate) mod application;
pub(crate) mod arguments;
pub(crate) mod atom;
pub(crate) mod capture_type;
pub(crate) mod case_arm;
pub(crate) mod case_of;
pub(crate) mod lambda;
pub(crate) mod parse_expr;
pub(crate) mod parse_spine;
pub(crate) mod path_or_ident;
pub(crate) mod pattern;
pub(crate) mod pattern_alt;
pub(crate) mod patterns;
pub(crate) mod postfix;
pub(crate) mod prefix;
pub(crate) mod range_or_literal;
pub(crate) mod signature_from;
pub(crate) mod spaced_colon_colon;
pub(crate) mod starts_argument;
pub(crate) mod tight_colon_colon;
pub(crate) mod token_text;
pub(crate) mod unary_op;

use crate::ParseError;
use crate::parse::parser::capture_type::capture_type;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::parse_spine::parse_spine;
use crate::parse::parser::patterns::patterns;
use crate::parse::parser::signature_from::signature_from;
use redox_ast::{Binding, Block, FnDecl, Ident, Item, Program, Stmt, StmtKind, TypeExpr};
use redox_diag::Code;
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

  /// ```text
  /// Item := Ident "::" Spine Newline
  ///         Ident Binder* "=" Newline Indent Stmt+ Dedent
  /// ```
  ///
  /// Doc 18 §§2.2–2.3 — the signature line and the equation. Two lines where
  /// doc 03 had one, and the trade doc 18 §10.1 item 2 flags as the thing to
  /// watch: a one-line private helper now costs two lines. What it buys is
  /// that the types are written once, in Haskell's order, and the equation is
  /// a row of binders rather than a repeated annotation.
  ///
  /// The two names must match. Catching that here rather than at the emitter
  /// is what makes a mistyped equation a diagnostic about the equation
  /// instead of an undefined-function error from rustc about a line the user
  /// did not write.
  fn item(&mut self) -> Result<Item, ParseError> {
    let leading = self.peek_leading();
    let start = self.peek_span().0;

    // The signature line.
    let name = self.ident("expected a name to begin a declaration")?;
    self.expect(
      &TokenKind::ColonColon,
      "expected `::` after the name, to begin the signature",
    )?;
    let spine = parse_spine(self)?;
    self.newline_with_trailing("expected a line break after the signature")?;
    self.skip_separators();

    // The equations. One or more, each repeating the name — doc 18 §2.3.
    let mut clauses = Vec::new();
    let mut sig = None;
    loop {
      let again = self.ident("expected the equation to repeat the name")?;
      if again.text != name.text {
        return Err(ParseError::with(
          Code::ExpectedName,
          format!(
            "this equation defines `{}`, but the signature above it is for `{}`",
            again.text, name.text
          ),
          again.span,
        ));
      }

      let row = patterns(self)?;
      // Every clause is checked against the same spine, so a row of the wrong
      // length is reported per clause rather than once for the function.
      let built = signature_from(&spine, &row, (name.span.0, self.prev_span().1))?;
      if sig.is_none() {
        sig = Some(built);
      }

      let clause = self.clause(row)?;
      clauses.push(clause);

      // Another clause follows only if the next item is this same name. A
      // different name starts a new declaration, and anything else ends the
      // run — which is how the parser knows where a multi-clause function
      // stops without a terminator.
      self.skip_separators();
      let continues = matches!(&self.peek().kind, TokenKind::Ident(text) if *text == name.text)
        && !self
          .peek_at(1)
          .is_some_and(|t| t.kind == TokenKind::ColonColon);
      if !continues {
        break;
      }
    }

    let end = self.prev_span().1;
    let trailing = clauses.last().and_then(|c| c.trailing.clone());
    Ok(Item::Fn(FnDecl {
      name,
      sig: sig.expect("the loop runs at least once"),
      clauses,
      span: (start, end),
      leading,
      trailing,
    }))
  }

  /// One clause's right-hand side, after its pattern row.
  ///
  /// Either `= body` or doc 18 §2.3's guard ladder. A guard list arrives on
  /// the *next* line, indented, so the newline and `Indent` are consumed
  /// before the `|` can be seen — which is why the two forms are told apart
  /// after the layout tokens rather than before them.
  fn clause(&mut self, patterns: Vec<redox_ast::Pattern>) -> Result<redox_ast::Clause, ParseError> {
    let start = patterns.first().map_or(self.peek_span().0, |p| p.span().0);

    if self.opens_a_guard_list() {
      self.advance();
      self.advance();
      let guards = self.guards()?;
      self.expect(&TokenKind::Dedent, "expected the guard list to end")?;
      return Ok(redox_ast::Clause {
        patterns,
        body: redox_ast::ClauseBody::Guarded(guards),
        span: (start, self.prev_span().1),
        trailing: None,
      });
    }

    self.expect(
      &TokenKind::Eq,
      "expected `=` after the equation's parameters, or an indented `|` guard list",
    )?;

    // Doc 18 §2.3's own examples put short bodies on the `=` line — `fib 0 =
    // 0` — and reserve the indented block for bodies that need statements.
    // Both are the same construct, so both are read here rather than in two
    // places that could drift apart.
    if !self.check(&TokenKind::Newline) {
      let body = self.inline_body()?;
      let trailing = self.newline_with_trailing("expected a line break after the body")?;
      return Ok(redox_ast::Clause {
        patterns,
        body: redox_ast::ClauseBody::Plain(body),
        span: (start, self.prev_span().1),
        trailing,
      });
    }

    let trailing = self.newline_with_trailing("expected a line break after `=`")?;
    self.expect_with(
      Code::EmptyBody,
      &TokenKind::Indent,
      "expected an indented body",
    )?;

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
      return Err(ParseError::with(
        Code::EmptyBody,
        "a declaration body must contain at least one statement",
        (body_start, body_start),
      ));
    }

    Ok(redox_ast::Clause {
      patterns,
      body: redox_ast::ClauseBody::Plain(Block {
        stmts,
        span: (body_start, body_end),
      }),
      span: (start, self.prev_span().1),
      trailing,
    })
  }

  /// One expression as a body, for a clause or guard written on one line.
  ///
  /// A block of one tail statement, so the emitter has one body shape to
  /// handle rather than two — the inline form is a spelling, not a second
  /// kind of body.
  fn inline_body(&mut self) -> Result<Block, ParseError> {
    let start = self.peek_span().0;
    let value = parse_expr(self, 0)?;
    let end = self.prev_span().1;
    Ok(Block {
      stmts: vec![Stmt {
        kind: StmtKind::Expr(value),
        span: (start, end),
        leading: Vec::new(),
        trailing: None,
      }],
      span: (start, end),
    })
  }

  /// Whether a guard list follows: a line break, an indent, then `|`.
  ///
  /// Three tokens of lookahead rather than one, because a line break after
  /// the pattern row is also what a *forgotten* `=` looks like. Deciding on
  /// the break alone made `add a b` with no `=` report "expected at least one
  /// `|` guard" and then cascade into two more errors as the body was read as
  /// fresh items — a worse diagnostic than the one it replaced, for the
  /// likelier mistake. Doc 05 §4's rule is that the first error names the
  /// actual defect.
  fn opens_a_guard_list(&self) -> bool {
    self.check(&TokenKind::Newline)
      && self.peek_at(1).is_some_and(|t| t.kind == TokenKind::Indent)
      && self.peek_at(2).is_some_and(|t| t.kind == TokenKind::Or)
  }

  /// A guard ladder: `| cond = body` rows, in order.
  ///
  /// `otherwise` is Haskell's name for the final catch-all and is recognised
  /// here as a condition of `None`, so the emitter can write a bare `else`.
  /// Lowering it to `if true` instead would leave rustc unable to see that the
  /// ladder is total, and every guarded function would need a return the
  /// author did not write.
  fn guards(&mut self) -> Result<Vec<redox_ast::Guard>, ParseError> {
    let mut rows = Vec::new();
    loop {
      self.skip_separators();
      if !self.check(&TokenKind::Or) {
        break;
      }
      let start = self.peek_span().0;
      self.advance();

      // `otherwise` is a reserved word (doc 18 §7), not an identifier the
      // parser compares against: `| otherwise = …` must not be readable as a
      // comparison against a variable someone happened to name `otherwise`.
      let condition = if self.check(&TokenKind::Kw(Keyword::Otherwise)) {
        self.advance();
        None
      } else {
        Some(parse_expr(self, 0)?)
      };

      self.expect(&TokenKind::Eq, "expected `=` after a guard's condition")?;
      let body = self.inline_body()?;
      self.newline_with_trailing("expected a line break after a guard's body")?;

      rows.push(redox_ast::Guard {
        condition,
        body,
        span: (start, self.prev_span().1),
      });
    }

    debug_assert!(
      !rows.is_empty(),
      "`opens_a_guard_list` guarantees a first `|`"
    );
    Ok(rows)
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
    } else if let Some(draw) = self.try_draw()? {
      draw
    } else if let Some(binding) = self.try_binding()? {
      StmtKind::Binding(binding)
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

  /// A draw arrow, if the cursor is on one.
  ///
  /// Doc 18 §3.1. Recognised by one token of lookahead — `Ident` then `<-` —
  /// which is unambiguous because `<-` is a single token. It was not, until
  /// this plan: `x <- f` lexed as `<` then `-`, parsed as `x < -f`, and
  /// emitted valid Rust that compared `x` against negated `f`. No diagnostic,
  /// wrong program.
  fn try_draw(&mut self) -> Result<Option<StmtKind>, ParseError> {
    let looks_like = matches!(self.peek().kind, TokenKind::Ident(_))
      && self
        .peek_at(1)
        .is_some_and(|t| t.kind == TokenKind::DrawArrow);
    if !looks_like {
      return Ok(None);
    }

    let name = self.ident("expected a name before `<-`")?;
    self.expect(&TokenKind::DrawArrow, "expected `<-`")?;
    let value = parse_expr(self, 0)?;
    Ok(Some(StmtKind::Draw { name, value }))
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
    self.expect_with(Code::ExpectedToken, kind, message)
  }

  /// [`Parser::expect`] with a code other than [`Code::ExpectedToken`].
  ///
  /// A missing token is usually nothing more specific than "the grammar
  /// wanted something else here", but a few are worth their own code because
  /// a reader recognises them: a declaration with no body is `RDX0105`, not a
  /// generic missing `Indent`.
  pub(crate) fn expect_with(
    &mut self,
    code: Code,
    kind: &TokenKind,
    message: &str,
  ) -> Result<(), ParseError> {
    if self.check(kind) {
      self.advance();
      Ok(())
    } else {
      Err(ParseError::with(code, message, self.peek_span()))
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
      Err(ParseError::with(
        Code::ExpectedName,
        message,
        self.peek_span(),
      ))
    }
  }

  /// Consume a `Newline`, returning any comment that trailed its line.
  fn newline_with_trailing(&mut self, message: &str) -> Result<Option<Trivia>, ParseError> {
    if self.check(&TokenKind::Newline) {
      let trailing = self.peek().trailing.clone();
      self.advance();
      Ok(trailing)
    } else if self.at_eof() || self.check(&TokenKind::Dedent) || self.just_dedented() {
      // A file ending without a trailing newline, or a block whose last
      // statement is immediately followed by a dedent, is well-formed — and
      // so is a statement whose expression *was* a block.
      Ok(None)
    } else {
      Err(ParseError::expected(message, self.peek_span()))
    }
  }

  /// Whether the token just consumed closed an indented block.
  ///
  /// Doc 18 §2.3's `case … of` is the first expression with layout of its
  /// own: its arms are an indented block, and the `Dedent` that ends them has
  /// already ended the line. A caller that then demanded a `Newline` would
  /// reject `label age = case age of …` — which is exactly how this was
  /// found, by writing §2.3's example into an example file.
  ///
  /// Expressed as "the previous token was a `Dedent`" rather than "the body
  /// was a `case`", because the next construct with its own layout should not
  /// have to be added here too.
  pub(crate) fn just_dedented(&self) -> bool {
    self
      .pos
      .checked_sub(1)
      .and_then(|i| self.toks.get(i))
      .is_some_and(|t| t.kind == TokenKind::Dedent)
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
