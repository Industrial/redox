use crate::parse::parser::Parser;
use crate::parse::parser::token_text::token_text;
use redox_ast::Pattern;
use redox_lexer::{Span, TokenKind};

/// A literal pattern, grown into a range if `..` or `..=` follows it.
///
/// `18..=64` and `18..64`, doc 18 §2.3's `case` example. Both endpoints are
/// carried as **verbatim source text**, the discipline doc 03 §14.1 applies
/// to types: Redox never has to agree with rustc about what a literal means,
/// and a range is two literals and a token.
///
/// A range with no high end — Rust's `18..` — is not read here. The low end
/// has already been consumed when this is called, so an absent high end would
/// have to be distinguished from a malformed one, and doc 18 §2.3 uses
/// neither form. Plan 33 owns the full pattern grammar.
pub(crate) fn range_or_literal(p: &mut Parser, lo: String, span: Span) -> Pattern {
  let inclusive = match &p.peek().kind {
    TokenKind::DotDotEq => true,
    TokenKind::DotDot => false,
    _ => return Pattern::Literal { text: lo, span },
  };
  p.advance();

  let hi = token_text(&p.peek().kind);
  let end = p.peek_span().1;
  p.advance();

  Pattern::Range {
    lo,
    hi,
    inclusive,
    span: (span.0, end),
  }
}
