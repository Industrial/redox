use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::bracketed::bracketed;
use crate::parse::parser::byte_literal::byte_literal;
use crate::parse::parser::case_of::case_of;
use crate::parse::parser::do_block::do_block;
use crate::parse::parser::if_expression::if_expression;
use crate::parse::parser::interpolated_string::interpolated_string;
use crate::parse::parser::lambda::lambda;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::path_or_ident::path_or_ident;
use crate::parse::parser::rust_string_body::rust_string_body;
use crate::parse::parser::section::section;
use crate::parse::parser::unary_op::unary_op;
use crate::parse::precedence::prefix_power::PREFIX_POWER;
use redox_ast::Expr;
use redox_diag::Code;
use redox_lexer::{Keyword, StrPart, TokenKind};

/// A prefix position: a literal, a name, a grouping, a receiver sigil, or a
/// unary operator.
pub(crate) fn prefix(p: &mut Parser) -> Result<Expr, ParseError> {
  let start = p.peek_span().0;

  if let Some(op) = unary_op(p) {
    let operand = parse_expr(p, PREFIX_POWER)?;
    let span = (start, operand.span().1);
    return Ok(Expr::Unary {
      op,
      operand: Box::new(operand),
      span,
    });
  }

  match p.peek().kind.clone() {
    TokenKind::Int(text) | TokenKind::Float(text) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Num { text, span })
    }
    // A string with no interpolation keeps `raw` — passed through verbatim,
    // which is what plan 02 carried `raw` for. One holding an interpolation
    // becomes `Expr::StrInterp` instead, so every existing path is untouched
    // (plan 23's criterion 6).
    //
    // A `"""` block arrives with its dedent already computed and `raw`
    // holding triple-quoted source, which is not Rust at all — so it takes
    // the dedented value and the emitter escapes it. Plan 23 §1.3.
    TokenKind::Str { parts, raw } => {
      let span = p.peek_span();
      p.advance();
      if parts
        .iter()
        .any(|part| matches!(part, StrPart::Interp { .. }))
      {
        return interpolated_string(&parts, span);
      }
      if raw.starts_with("\"\"\"") {
        let value = parts
          .iter()
          .map(|part| match part {
            StrPart::Text(text) => text.as_str(),
            StrPart::Interp { .. } => unreachable!("checked above"),
          })
          .collect::<String>();
        return Ok(Expr::StrLit {
          value: format!("\"{}\"", rust_string_body(&value)),
          span,
        });
      }
      Ok(Expr::StrLit { value: raw, span })
    }
    // Doc 03 §11.2's four literal kinds, *"all pure passthrough"* — plan 23.
    // Rust's own spellings, which is why one node carries all of them: nothing
    // downstream needs to tell them apart, and none may be escaped,
    // brace-doubled or used as a format template.
    TokenKind::RawStr { text, hashes } => {
      let span = p.peek_span();
      p.advance();
      let pad = "#".repeat(hashes);
      Ok(Expr::RawLit {
        text: format!("r{pad}\"{text}\"{pad}"),
        span,
      })
    }
    TokenKind::ByteStr(text) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::RawLit {
        text: format!("b\"{}\"", rust_string_body(&text)),
        span,
      })
    }
    TokenKind::CStr(text) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::RawLit {
        text: format!("c\"{}\"", rust_string_body(&text)),
        span,
      })
    }
    TokenKind::Byte(value) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::RawLit {
        text: byte_literal(value),
        span,
      })
    }
    // Doc 03 §12.4's `try` block — plan 28. An expression, so it is reached
    // from here exactly as plan 20's `do` is, and it takes the same statement
    // block. The difference between the two is entirely in the emitter.
    TokenKind::Kw(Keyword::Try) => {
      p.advance();
      let body = p.statement_block("expected an indented `try` body")?;
      Ok(Expr::TryBlock {
        body,
        span: (start, p.prev_span().1),
      })
    }
    TokenKind::Char(value) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Char { value, span })
    }
    TokenKind::Kw(kw @ (Keyword::True | Keyword::False)) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Bool {
        value: kw == Keyword::True,
        span,
      })
    }
    // `@field` → `self.field`, `@@name` → `Self::name` (doc 01 §16). Legal
    // only in a method body; plan 25 is where that is checked.
    TokenKind::At => {
      p.advance();
      let name = p.ident("expected a field name after `@`")?;
      let span = (start, name.span.1);
      Ok(Expr::SelfField { name, span })
    }
    TokenKind::AtAt => {
      p.advance();
      let name = p.ident("expected a name after `@@`")?;
      let span = (start, name.span.1);
      Ok(Expr::SelfType { name, span })
    }
    // `[` opens a list literal or doc 18 §3.2's comprehension; `bracketed`
    // tells them apart by what follows the first expression.
    TokenKind::LBracket => bracketed(p),
    TokenKind::LParen => {
      p.advance();
      // `()` — the unit value. Doc 18 §7 replaces the `!` zero-argument
      // suffix with `f ()`, so this is what makes a no-argument call
      // writable: juxtaposition needs something to juxtapose.
      if p.check(&TokenKind::RParen) {
        p.advance();
        return Ok(Expr::Unit {
          span: (start, p.prev_span().1),
        });
      }

      // Doc 18 §4's sections. Tried before the grouping, because `(> 0)` is
      // not an expression at all — and `section` puts the cursor back when
      // what follows turns out to be an ordinary group.
      if let Some(found) = section(p, start)? {
        return Ok(found);
      }
      let inner = parse_expr(p, 0)?;
      p.expect(&TokenKind::RParen, "expected `)` to close the group")?;
      Ok(Expr::Paren {
        inner: Box::new(inner),
        span: (start, p.prev_span().1),
      })
    }
    // Doc 18 §4's lambda, and `move \x -> …` with it: `move` can only open
    // a closure here, so one token of lookahead settles it.
    TokenKind::Backslash => lambda(p),
    TokenKind::Kw(Keyword::Move) => lambda(p),
    // Doc 01 §8's conditional **expression** and doc 03 §7.2's `do` block,
    // both plan 20. `if` here is always the `then` form: the statement form
    // is read by `control_flow`, which `stmt` asks before it ever reaches an
    // expression.
    TokenKind::Kw(Keyword::If) => if_expression(p),
    TokenKind::Kw(Keyword::Do) => do_block(p),
    // Doc 18 §2.3's `case e of`. An expression, not a statement: its value is
    // the matching arm's, which is what lets it be a clause body, an argument
    // or the right-hand side of a binding with no extra form.
    TokenKind::Kw(Keyword::Case) => case_of(p),
    // `self` and `Self` are expressions, because doc 18 §2.5 drops `@name`
    // and `@@name` in favour of `self.name` and `Self::name`. They lex as
    // reserved words, so they need naming here — an `Ident` arm would never
    // see them.
    TokenKind::Kw(Keyword::SelfValue) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Ident(redox_ast::Ident {
        text: "self".to_string(),
        span,
      }))
    }
    TokenKind::Kw(Keyword::SelfType) => {
      let span = p.peek_span();
      p.advance();
      Ok(Expr::Ident(redox_ast::Ident {
        text: "Self".to_string(),
        span,
      }))
    }
    TokenKind::Ident(_) => path_or_ident(p),
    _ => Err(ParseError::with(
      Code::ExpectedExpression,
      "expected an expression",
      p.peek_span(),
    )),
  }
}
