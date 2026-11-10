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
///
/// An arm body is an **expression**, and since plan 55 leaf 3 an indented
/// block is one — so the braced arm is a body shape rather than a second
/// spelling of an arm. Two rustfmt rules decide how it is written, and the
/// loop below names both: a one-expression block collapses, and a block body
/// takes no trailing comma.
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
    // Plan 55 leaf 3: an arm body written as an indented block is a block
    // expression, and a block of one plain expression collapses to
    // `pat => expr,`. rustfmt *flattens* a one-expression block body, so
    // emitting the braces would be output rustfmt rewrites — which is
    // `emit_arm`'s reason for making the same choice for a clause.
    //
    // A comment on that one line disqualifies it: a collapsed arm has no line
    // to put one on, and a comment must reach the `.xrs` (doc 02 §10.1).
    // `sole_expression` answers the rest, and leaves the trivia test to the
    // caller for exactly this reason.
    let body = match &arm.body {
      Expr::Do {
        label: None, body, ..
      } if body
        .stmts
        .first()
        .is_some_and(|stmt| stmt.leading.is_empty() && stmt.trailing.is_none()) =>
      {
        body.sole_expression().unwrap_or(&arm.body)
      }
      other => other,
    };
    out.push_str(&emit_expr(unparen(body), indent + RUST_INDENT));
    // rustfmt writes **no** trailing comma after an arm whose body is a plain
    // block, and one after every other body — `arm_comma`, and it reads the
    // body's node kind rather than its width. A block body is what leaf 3
    // added, so the comma became a question with it.
    if !matches!(body, Expr::Do { .. }) {
      out.push(',');
    }
    out.push('\n');
  }
  out.push_str(&close_pad);
  out.push('}');
  out
}
