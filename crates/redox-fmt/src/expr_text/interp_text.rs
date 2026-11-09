use crate::expr_text::expr_text;
use redox_ast::StrInterpPart;

/// An interpolated string — `"hi #{name}, you owe #{amount:.2}"`.
///
/// The `Text` parts carry their **source** text, escapes and all, exactly as
/// `StrLit` does. That is worth stating because the alternative would be a
/// real hazard: a cooked part would have to be re-escaped on the way out, and
/// an interpolated string has one more escape than an ordinary one — `#{`
/// itself — so a re-escaper that handled `\` and `"` and forgot `#{` would
/// turn a literal brace into an interpolation silently. Nothing here escapes
/// anything, so nothing can.
///
/// A format spec is appended after a `:`, which is where the lexer found it.
pub(crate) fn interp_text(parts: &[StrInterpPart], indent: usize, source: &str) -> String {
  let mut out = String::from("\"");
  for part in parts {
    match part {
      StrInterpPart::Text(text) => out.push_str(text),
      StrInterpPart::Interp { expr, spec } => {
        out.push_str("#{");
        out.push_str(&expr_text(expr, 0, indent, source));
        if let Some(spec) = spec {
          out.push(':');
          out.push_str(spec);
        }
        out.push('}');
      }
    }
  }
  out.push('"');
  out
}
