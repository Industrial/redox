use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Expr;
use redox_diag::Code;
use redox_lexer::TokenKind;

/// An identifier, or a `::`-joined path carried verbatim.
///
/// Doc 03 §13.1 makes `::` the only path separator, so `.` unambiguously means
/// field or method access — which is what lets `postfix` treat every `.` the
/// same way.
///
/// **`::` joins a path and nothing else** — plan 52. Doc 18 §5.2 used to make
/// this the one place in Redox where a space changed meaning: a *spaced* `::`
/// ascribed a type, so a path separator had to be told from an ascription
/// marker by its whitespace. Plan 52 gave ascription its own token, `:`, which
/// retires the whitespace rule.
///
/// # A spaced `::` is refused rather than accepted
///
/// Retiring the rule does not retire the old spelling from anybody's fingers,
/// and `y = x :: u64` is the dangerous case: `u64` is a valid segment, so
/// taking every `::` unconditionally would consume it as the path `x::u64` and
/// leave rustc to report an unresolved path against a line the author did not
/// write. That is precisely what §5.2's whitespace rule existed to prevent, so
/// the rule is replaced by a diagnostic and not by silence.
///
/// The test is positional and needs no token flag: a gap between the previous
/// token's end and the `::`'s start is whitespace. Nothing legitimate is
/// caught by it, because a path is written tight in Rust and in Redox alike,
/// and a `::` opening a continuation line cannot arise — an identifier does
/// not continue a line, so `Foo\n  ::bar` is two statements and never one
/// path.
pub(crate) fn path_or_ident(p: &mut Parser) -> Result<Expr, ParseError> {
  let first = p.ident("expected a name")?;
  if !p.check(&TokenKind::ColonColon) {
    return Ok(Expr::Ident(first));
  }

  let start = first.span.0;
  let mut text = first.text;
  let mut tight_through = first.span.1;
  while p.check(&TokenKind::ColonColon) {
    let separator = p.peek_span();
    if separator.0 > tight_through {
      return Err(ParseError::with(
        Code::SpacedPathSeparator,
        "`::` joins a path, so it takes no space before it — write `:` to \
         annotate a type, or close up the `::` to name a path",
        separator,
      ));
    }
    p.advance();
    let next = p.ident("expected a name after `::`")?;
    text.push_str("::");
    text.push_str(&next.text);
    tight_through = next.span.1;
  }
  Ok(Expr::Path {
    text,
    span: (start, p.prev_span().1),
  })
}
