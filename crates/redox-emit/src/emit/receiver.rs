use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use crate::emit::wraps_as_receiver::wraps_as_receiver;
use redox_ast::Expr;

/// Render an expression as the **receiver** of a method call, parenthesised
/// when `.` would otherwise bind tighter than it does.
///
/// ```text
/// xs |> count          xs.count()
/// (1 .. 3) |> count    (1..3).count()
/// ```
///
/// `.` binds tighter than every binary operator in Rust, so a receiver that
/// is an operator expression needs its parentheses — `1..3.count()` is
/// `1..(3.count())`, which is not what the author wrote and not even the same
/// type. Doc 02 §5.2 calls output that reads as Rust and means something else
/// the worst available failure mode, and this is exactly that shape.
///
/// Found twice: once in doc 18 §3.2's comprehension, whose generator source
/// is routinely a range, and then in §4's pipeline, which had the same defect
/// unexercised because no example piped from an operator expression.
///
/// The parentheses are added only where they are **needed**, which keeps
/// `xs.iter()` free of noise and keeps plan 04's rustfmt-clean guarantee:
/// rustfmt preserves parentheses, so a redundant pair would survive into the
/// output and read as the author's.
pub(crate) fn receiver(e: &Expr, indent: usize) -> String {
  let bare = unparen(e);
  let text = emit_expr(bare, indent);
  if wraps_as_receiver(bare) {
    return format!("({text})");
  }
  text
}
