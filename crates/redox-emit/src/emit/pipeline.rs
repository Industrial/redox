use crate::emit::expr::emit_expr;
use crate::emit::expr_list::expr_list;
use crate::emit::receiver::receiver;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// Lower doc 18 §4's receiver pipeline: `a |> f` becomes `a.f()`.
///
/// ```text
/// numbers |> iter |> map f |> sum   numbers.iter().map(f).sum()
/// ```
///
/// The right operand is an **application** (doc 18 §2.1), and the lowering
/// takes it apart: its head becomes the method name and its arguments become
/// the call's arguments, with the left operand inserted as the receiver. So
/// `xs |> map f` is `xs.map(f)` — one method call, not a call of a call.
///
/// This is the pipeline doc 03 §8.3 specified and doc 18 §4 keeps unchanged.
/// Doc 18 explains why it is `|>` and not Haskell's spelling: composition is
/// `.`, which in Redox is field access — the one Haskell token that cannot be
/// had at any price — and reverse application is `&`, which is Rust's
/// reference sigil. So the pipeline keeps Elm's and F#'s spelling.
///
/// Nesting falls out of recursion, because the left operand of a `|>` is
/// itself whatever the previous `|>` produced.
pub(crate) fn pipeline(lhs: &Expr, rhs: &Expr, indent: usize) -> String {
  // Parenthesised when `.` would bind tighter than the left operand does:
  // `1 .. 3 |> count` is `(1..3).count()`, not `1..3.count()`. The defect
  // was here from the day the pipeline landed and went unexercised because
  // no example piped from an operator expression.
  let left = receiver(lhs, indent);

  match unparen(rhs) {
    // `a |> .name args` and `a |> .name` — plan 57's method-as-function,
    // matched **before** the two arms below. §11.1 is the trap: the `other =>`
    // arm appends `()`, so a zero-argument `x |> .name` falling through to it
    // lowers to `x.(|__rdx_1| __rdx_1.name())()`. The two readings agree on
    // the text — the pipeline inserts the receiver, and under plan 58's
    // reverse application `.name args` awaits its last argument — which is
    // why 57 need not assume either meaning of `|>`.
    Expr::MethodRef { name, .. } => format!("{left}.{}()", name.text),
    Expr::Call { callee, args, .. } if let Expr::MethodRef { name, .. } = unparen(callee) => {
      format!("{left}.{}({})", name.text, expr_list(args, indent))
    }
    // `a |> f x y` — the head names the method, the arguments come after.
    Expr::Call { callee, args, .. } => {
      format!(
        "{left}.{}({})",
        emit_expr(unparen(callee), indent),
        expr_list(args, indent)
      )
    }
    // `a |> f` — a bare name is a no-argument method call. This is the case
    // that makes a pipeline read like a chain: `xs |> iter |> sum`.
    other => format!("{left}.{}()", emit_expr(other, indent)),
  }
}
