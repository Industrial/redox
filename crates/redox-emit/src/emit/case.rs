use crate::emit::emit_condition::emit_condition;
use crate::emit::expr::emit_expr;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::unparen::unparen;
use redox_ast::{CaseArm, Expr};

/// Lower doc 18 §2.3's `case … of` to Rust's `match`, starting at `indent`.
///
/// ```text
/// case age of                     match age {
///   n | n > 65      -> "senior"       n if n > 65 => "senior",
///   18..=64         -> "adult"        18..=64 => "adult",
///   Teen or PreTeen -> "young"        Teen | PreTeen => "young",
///   _               -> "newborn"      _ => "newborn",
///                                 }
/// ```
///
/// A spelling, not a lowering: the arms, the guards and the alternation are
/// Rust's already. What changes is the arrow — `->` rather than `=>` — which
/// is the single reason `=>` is free to mean a constraint (doc 18 §7), and
/// doc 03's `switch` keyword leaves with it.
///
/// Every arm is written `pat => value,` on one line, because that is what
/// rustfmt does to an arm whose body fits; an arm body that is itself
/// multi-line hangs from the `=>`, for the same reason a `println!`'s block
/// argument does.
pub(crate) fn emit_case(scrutinee: &Expr, arms: &[CaseArm], indent: usize) -> String {
  let arm_pad = " ".repeat(indent + RUST_INDENT);
  let close_pad = " ".repeat(indent);

  let mut out = format!("match {} {{\n", emit_expr(unparen(scrutinee), indent));
  for arm in arms {
    out.push_str(&arm_pad);
    out.push_str(&arm.pattern.rust_text());
    if let Some(guard) = &arm.guard {
      out.push_str(" if ");
      // A pattern test in an arm guard emits `if let` inside the match
      // guard — `if_let_guard`, stable as of rustc 1.97.1 and verified before
      // plan 22 was written.
      out.push_str(&emit_condition(guard, indent + RUST_INDENT));
    }
    out.push_str(" => ");
    out.push_str(&emit_expr(unparen(&arm.body), indent + RUST_INDENT));
    out.push_str(",\n");
  }
  out.push_str(&close_pad);
  out.push('}');
  out
}
