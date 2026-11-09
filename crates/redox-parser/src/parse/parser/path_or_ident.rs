use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::tight_colon_colon::tight_colon_colon;
use redox_ast::Expr;

/// An identifier, or a `::`-joined path carried verbatim.
///
/// Doc 03 §13.1 makes `::` the only path separator, so `.` unambiguously means
/// field or method access — which is what lets `postfix` treat every `.` the
/// same way.
///
/// **A spaced `::` is not a path separator** (doc 18 §5.2), and this function
/// is where that matters most. Without the check it eats the ascription's
/// marker: `x :: fmt::Result` became the path `x::fmt::Result`, and
/// `xs :: HashSet<i32>` became `xs::HashSet` followed by a comparison that
/// then failed to parse. Both were silent — the first emitted a path that
/// does not exist, which rustc would have reported against a line the author
/// did not write.
pub(crate) fn path_or_ident(p: &mut Parser) -> Result<Expr, ParseError> {
  let first = p.ident("expected a name")?;
  if !tight_colon_colon(p) {
    return Ok(Expr::Ident(first));
  }

  let start = first.span.0;
  let mut text = first.text;
  while tight_colon_colon(p) {
    p.advance();
    let next = p.ident("expected a name after `::`")?;
    text.push_str("::");
    text.push_str(&next.text);
  }
  Ok(Expr::Path {
    text,
    span: (start, p.prev_span().1),
  })
}
