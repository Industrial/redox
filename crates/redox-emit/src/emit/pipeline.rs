use crate::emit::expr::emit_expr;
use crate::emit::expr_list::expr_list;
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
  let receiver = emit_expr(lhs, indent);

  match unparen(rhs) {
    // `a |> f x y` — the head names the method, the arguments come after.
    Expr::Call { callee, args, .. } => {
      format!(
        "{receiver}.{}({})",
        emit_expr(unparen(callee), indent),
        expr_list(args, indent)
      )
    }
    // `a |> f` — a bare name is a no-argument method call. This is the case
    // that makes a pipeline read like a chain: `xs |> iter |> sum`.
    other => format!("{receiver}.{}()", emit_expr(other, indent)),
  }
}
