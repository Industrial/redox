//! `redox-parser` — hand-written recursive descent.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 4, hand-written arm.
//! `docs/COMPILER.md` records the bake-off against LALRPOP that chose this
//! approach, its axes, and its revisit trigger.
//!
//! Grammar for plan 01's slice:
//!
//! ```text
//! Program  := Item* Eof
//! Item     := Ident "=" "->" Newline Indent Stmt+ Dedent
//! Stmt     := Expr Newline
//! Expr     := Ident StrLit        # parenless call, one argument
//!           | StrLit
//!           | Ident
//! ```
//!
//! The one-argument restriction is the minimum this slice needs, applied
//! knowingly rather than inherited by accident — Emerald carried the
//! equivalent restriction for 89 plans because its LALR(1) grammar could not
//! lift it. Plan 19 generalises it to doc 03 §5.1's full call grammar.
//!
//! Errors **recover at item boundaries**, so one pass reports every bad item
//! rather than only the first. This is free in recursive descent and was a
//! retrofit in Emerald (its plan 26 threads an `errors: &mut Vec<_>` through
//! the generated grammar); it is one of the six axes the bake-off compared.

use redox_ast::{Block, Expr, FnDecl, Ident, Item, Program, Stmt, StmtKind};
use redox_lexer::{LexError, Token, TokenKind, Trivia};

/// A parse error, carrying enough to report against the `.sdx` source.
/// `miette` rendering is plan 05.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  pub message: String,
  pub span: redox_lexer::Span,
}

/// Everything that can go wrong turning source text into a [`Program`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFailure {
  Lex(LexError),
  /// One or more parse errors, recovered at item boundaries.
  Parse(Vec<ParseError>),
}

/// Lex and parse `src`.
pub fn parse_source(src: &str) -> Result<Program, ParseFailure> {
  let toks = redox_lexer::lex(src).map_err(ParseFailure::Lex)?;
  parse(&toks, src.len()).map_err(ParseFailure::Parse)
}

/// Parse an already-lexed token stream.
pub fn parse(toks: &[Token], src_len: usize) -> Result<Program, Vec<ParseError>> {
  let mut p = Parser {
    toks,
    pos: 0,
    errors: Vec::new(),
  };
  let items = p.program();
  if p.errors.is_empty() {
    let trailing = p.eof_trivia();
    Ok(Program {
      items,
      span: (0, src_len),
      trailing,
    })
  } else {
    Err(p.errors)
  }
}

struct Parser<'a> {
  toks: &'a [Token],
  pos: usize,
  errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
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

  fn peek_span(&self) -> redox_lexer::Span {
    self.peek().span
  }

  fn peek_leading(&self) -> Vec<Trivia> {
    self.peek().leading.clone()
  }

  fn prev_span(&self) -> redox_lexer::Span {
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
  /// items (bake-off axis 3).
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

#[cfg(test)]
mod tests {
  use super::*;

  fn prog(src: &str) -> Program {
    parse_source(src).expect("should parse")
  }

  #[test]
  fn parses_the_hello_slice() {
    let p = prog("main = ->\n  print \"hello\"\n");
    assert_eq!(p.items.len(), 1);
    let Item::Fn(f) = &p.items[0];
    assert_eq!(f.name.text, "main");
    assert_eq!(f.body.stmts.len(), 1);
    let StmtKind::Expr(Expr::Call { callee, args, .. }) = &f.body.stmts[0].kind else {
      panic!("expected a call");
    };
    assert_eq!(
      **callee,
      Expr::Ident(Ident {
        text: "print".into(),
        span: (12, 17)
      })
    );
    assert_eq!(args.len(), 1);
  }

  /// Acceptance criterion 6, at the AST level: a node's span indexes the real
  /// source text.
  #[test]
  fn nodes_carry_spans_that_index_the_source() {
    let src = "main = ->\n  print \"hello\"\n";
    let p = prog(src);
    let Item::Fn(f) = &p.items[0];
    assert_eq!(&src[f.name.span.0..f.name.span.1], "main");
    let StmtKind::Expr(e) = &f.body.stmts[0].kind;
    assert_eq!(&src[e.span().0..e.span().1], "print \"hello\"");
  }

  #[test]
  fn carries_a_leading_comment_onto_the_item() {
    let p = prog("# a note\nmain = ->\n  print \"x\"\n");
    let Item::Fn(f) = &p.items[0];
    assert_eq!(f.leading.len(), 1);
    assert_eq!(f.leading[0].text, "a note");
  }

  #[test]
  fn carries_a_trailing_comment_onto_the_statement() {
    let p = prog("main = ->\n  print \"x\" # why\n");
    let Item::Fn(f) = &p.items[0];
    assert_eq!(
      f.body.stmts[0].trailing.as_ref().map(|t| t.text.as_str()),
      Some("why")
    );
  }

  #[test]
  fn parses_several_items() {
    let p = prog("a = ->\n  print \"1\"\n\nb = ->\n  print \"2\"\n");
    assert_eq!(p.items.len(), 2);
  }

  #[test]
  fn parses_a_file_without_a_trailing_newline() {
    let p = prog("main = ->\n  print \"x\"");
    assert_eq!(p.items.len(), 1);
  }

  #[test]
  fn keeps_a_comment_at_end_of_file() {
    let p = prog("main = ->\n  print \"x\"\n# the end\n");
    // Not `.last()`: the file's own final newline arrives as a `BlankLines`
    // trivium after the comment, so the comment is found by kind.
    assert_eq!(
      p.trailing
        .iter()
        .find(|t| t.kind == redox_ast::TriviaKind::LineComment)
        .map(|t| t.text.as_str()),
      Some("the end")
    );
  }

  #[test]
  fn reports_a_missing_arrow() {
    let err = parse_source("main =\n  print \"x\"\n").expect_err("should fail");
    let ParseFailure::Parse(errs) = err else {
      panic!("expected parse errors")
    };
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("`->`"), "{}", errs[0].message);
  }

  #[test]
  fn reports_an_empty_body() {
    let err = parse_source("main = ->\nother = ->\n  print \"x\"\n").expect_err("should fail");
    let ParseFailure::Parse(errs) = err else {
      panic!("expected parse errors")
    };
    assert!(
      errs[0].message.contains("indented body"),
      "{}",
      errs[0].message
    );
  }

  /// Bake-off axis 3, with evidence: one pass reports both bad items.
  #[test]
  fn recovers_at_item_boundaries_and_reports_every_bad_item() {
    let src = "a =\n  print \"1\"\n\nb =\n  print \"2\"\n\nc = ->\n  print \"3\"\n";
    let err = parse_source(src).expect_err("should fail");
    let ParseFailure::Parse(errs) = err else {
      panic!("expected parse errors")
    };
    assert_eq!(errs.len(), 2, "both malformed items should be reported");
  }

  #[test]
  fn surfaces_a_lex_error_distinctly_from_a_parse_error() {
    let err = parse_source("main = ->\n\tprint \"x\"\n").expect_err("should fail");
    assert!(matches!(err, ParseFailure::Lex(_)), "{err:?}");
  }
}
