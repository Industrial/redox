pub(crate) mod breaks;

use crate::emit::condition_head::breaks::breaks;
use crate::emit::emit_condition::emit_condition;
use crate::emit::rust_indent::RUST_INDENT;
use redox_ast::Condition;

/// A control-flow statement's keyword, condition and opening brace — plan 22.
///
/// ```text
/// if n > 0 {
///
/// if let Some(v) = o && v > 2      ← what Redox would write on one line
///
/// if let Some(v) = o               ← what rustfmt writes instead
///     && v > 2
/// {
/// ```
///
/// # rustfmt's rule for let-chains, bisected
///
/// A condition holding a `let` operand is written on one line **only** when it
/// has exactly two operands and the `let` is the second. Every other chain
/// with a `let` is broken one operand per line, with the brace on its own
/// line — and it is broken regardless of width, which is why this is a shape
/// rule and not another width constant.
///
/// | Condition | rustfmt |
/// |---|---|
/// | `if let Some(v) = o` | one line |
/// | `if a && let Some(v) = o` | one line |
/// | `if let Some(v) = o && v > 2` | broken |
/// | `if let Some(v) = o && let Some(w) = p` | broken |
/// | `if a && b && let Some(v) = o` | broken |
/// | `if a && b` | one line — no `let`, so not a chain |
///
/// Established by running rustfmt over all seven shapes rather than reasoning
/// about them, which is how plan 13's five width constants were fixed and for
/// the same reason: *reasoning from half of rustfmt's behaviour* is what plan
/// 14 §10.2 records going wrong.
pub(crate) fn condition_head(keyword: &str, condition: &Condition, indent: usize) -> String {
  if !breaks(condition) {
    return format!("{keyword} {} {{\n", emit_condition(condition, indent));
  }

  let inner = " ".repeat(indent + RUST_INDENT);
  let mut out = format!("{keyword} ");
  for (i, operand) in condition.operands.iter().enumerate() {
    if i > 0 {
      out.push_str(&format!("\n{inner}&& "));
    }
    out.push_str(&emit_condition(
      &Condition {
        operands: vec![operand.clone()],
      },
      indent,
    ));
  }
  out.push_str(&format!("\n{}{{\n", " ".repeat(indent)));
  out
}
