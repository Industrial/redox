use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::pattern_alt::pattern_alt;
use crate::parse::parser::pattern_app::pattern_app;
use crate::parse::parser::pattern_series::pattern_series;
use crate::parse::parser::range_or_literal::range_or_literal;
use crate::parse::parser::record_pattern::record_pattern;
use crate::parse::parser::token_text::token_text;
use redox_ast::Pattern;
use redox_lexer::{Keyword, TokenKind};

/// One pattern in an equation's row.
///
/// Doc 18 §2.3, and a sibling of [`super::patterns`] rather than a private
/// helper inside it, because doc 06 §1 permits one construct per file.
pub(crate) fn pattern(p: &mut Parser) -> Result<Pattern, ParseError> {
  let span = p.peek_span();
  match &p.peek().kind {
    TokenKind::Ident(text) if text == "_" => {
      p.advance();
      Ok(Pattern::Wildcard(span))
    }
    // An identifier is a binding, unless `@` follows (an as-pattern, doc 03
    // §6.4) or a `{` does (a record pattern, §6.3). A constructor path with a
    // `{` is read here too, because `pattern_app` hands an uppercase head
    // back only when no argument follows and `{` is not an argument.
    TokenKind::Ident(_) => {
      let name = p.ident("expected a pattern")?;
      if p.check(&TokenKind::At) {
        p.advance();
        // `name @ ..` is a **named rest** and `name @ pattern` is an
        // as-pattern. Rust admits the first only in a slice, and one node for
        // it keeps the parser from building what rustc would reject.
        if p.check(&TokenKind::DotDot) {
          p.advance();
          return Ok(Pattern::NamedRest {
            name,
            span: (span.0, p.prev_span().1),
          });
        }
        let inner = pattern_app(p)?;
        return Ok(Pattern::As {
          name,
          inner: Box::new(inner),
          span: (span.0, p.prev_span().1),
        });
      }
      if p.check(&TokenKind::LBrace) {
        return record_pattern(p, name.text, span.0);
      }
      Ok(Pattern::Binding(name))
    }
    // `..` alone — doc 03 §6.5's rest, spelled Rust's way because doc 18 §7
    // removed `...` from the language. Plan 33 §1.3 and §1.4: the meaning is
    // positional, and this arm is reached only where a *pattern* was expected,
    // which is never where a range's second operand would be.
    TokenKind::DotDot => {
      p.advance();
      Ok(Pattern::Rest(span))
    }
    // `[a, .., z]` — doc 03 §6.4's slice pattern.
    TokenKind::LBracket => {
      p.advance();
      let items = pattern_series(p, &TokenKind::RBracket)?;
      p.expect(
        &TokenKind::RBracket,
        "expected `]` to close the slice pattern",
      )?;
      Ok(Pattern::Slice {
        items,
        span: (span.0, p.prev_span().1),
      })
    }
    // Doc 03 §6.5's `...rest` left the language with doc 18 §7, and an author
    // following §6.5 wrote the published spelling — so the message names the
    // correction rather than the token, as `RDX0106` does for doc 01 §4.3.
    TokenKind::DotDotDot => Err(ParseError::with(
      redox_diag::Code::EllipsisRestPattern,
      "`...` left the language with doc 18 §7 — write a rest pattern Rust's way: `..` to \
       discard, or `rest @ ..` to bind",
      span,
    )),
    // A parenthesised group, which is how a constructor application reaches
    // an equation's *row* — `depth (Meters d) = d`. Juxtaposition in a row is
    // ambiguous without it, exactly as in Haskell, so the parentheses are
    // required there and optional nowhere. Plan 22; see `pattern_app`.
    //
    // A tuple pattern — `(a, b)` — is **not** this: it needs a comma, and it
    // is plan 33's along with the record and slice forms.
    TokenKind::LParen => {
      p.advance();
      // `()` is the unit pattern, which is a tuple of nothing.
      if p.check(&TokenKind::RParen) {
        p.advance();
        return Ok(Pattern::Tuple {
          items: Vec::new(),
          span: (span.0, p.prev_span().1),
        });
      }
      let inner = pattern_alt(p)?;
      // **The comma is the whole difference** between a group and a tuple —
      // plan 33. A group holds one pattern; a tuple holds a comma.
      if p.check(&TokenKind::Comma) {
        p.advance();
        let mut items = vec![inner];
        items.extend(pattern_series(p, &TokenKind::RParen)?);
        p.expect(
          &TokenKind::RParen,
          "expected `)` to close the tuple pattern",
        )?;
        return Ok(Pattern::Tuple {
          items,
          span: (span.0, p.prev_span().1),
        });
      }
      p.expect(
        &TokenKind::RParen,
        "expected `)` to close a parenthesised pattern",
      )?;
      Ok(inner)
    }
    // `self` is a reserved word, not an identifier, so an equation's binder
    // row needs it named here. Doc 18 §2.5's methods are written
    // `describe self = …`, and the binder is dropped by `receiver` — it is
    // the *type* that decides the receiver, and the name is what the body
    // refers to.
    TokenKind::Kw(Keyword::SelfValue) => {
      p.advance();
      Ok(Pattern::Binding(redox_ast::Ident {
        text: "self".to_string(),
        span,
      }))
    }
    // A literal **tests**, and is carried as verbatim source text — the same
    // discipline doc 03 §14.1 applies to types, so Redox never has to agree
    // with rustc about what a literal means.
    kind @ (TokenKind::Int(_)
    | TokenKind::Float(_)
    | TokenKind::Char(_)
    | TokenKind::Byte(_)
    | TokenKind::Kw(Keyword::True)
    | TokenKind::Kw(Keyword::False)) => {
      let text = token_text(kind);
      p.advance();
      Ok(range_or_literal(p, text, span))
    }
    TokenKind::Str { raw, .. } => {
      let text = raw.clone();
      p.advance();
      Ok(Pattern::Literal { text, span })
    }
    _ => Err(ParseError::with(
      redox_diag::Code::ExpectedName,
      "expected a parameter name, a literal pattern, a range, `_`, `(`, `[`, or `=`",
      span,
    )),
  }
}
