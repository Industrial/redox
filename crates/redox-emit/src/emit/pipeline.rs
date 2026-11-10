use crate::emit::expr::emit_expr;
use crate::emit::expr_list::expr_list;
use crate::emit::pipeline::callable::callable;
use crate::emit::receiver::receiver;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

pub(crate) mod callable;

/// Lower `|>` as **reverse application**: `x |> f` is `f x`, for any `f`.
///
/// ```text
/// raw |> normalise |> parse        parse(normalise(raw))
/// xs |> .map f |> .sum             xs.map(f).sum()
/// 5 |> double >>> bump             (|__rdx_1| bump(double(__rdx_1)))(5)
/// ```
///
/// Plan 58 (`docs/52 - 58 - The Reverse Pipeline.md`). Before this row the
/// operator was *receiver sugar* — `x |> f` emitted `x.f()` — which is not
/// what the token means anywhere else it exists, and the difference was not
/// cosmetic: **you could not pipe into your own functions.** `5 |> double`
/// emitted `5.double()` and earned `E0599: no method named `double` found for
/// `{integer}``, and the dangerous case was quieter still — if any trait in
/// scope happened to have a method of that name, the pipeline compiled and
/// called the method instead, so adding an import could change what an
/// existing pipeline meant.
///
/// # The piped value is the **last** argument
///
/// Plan 57 §1.2 is the reason, and it is the only order that lets reverse
/// application and partial application share one spelling: a method is a
/// function whose receiver is its last argument, and partial application
/// supplies arguments from the left. So `a |> f x` is `f(x, a)` and not
/// `f(a, x)` — which is also what makes `a |> f x` and `f x a` the same
/// program, as reverse application requires.
///
/// # Why two of the four arms did not change
///
/// `.name` and `.name args` already put their receiver last, so plan 57's
/// lowering *is* reverse application for a method and needed nothing here.
/// What changed is the two arms that read a bare name as a method: those are
/// now ordinary calls, which is the whole of this row's output.
///
/// This is also the row that made plan 66's composition usable. A composition
/// piped into fell to the bare-name arm and emitted
/// `5.|__rdx_1| bump(double(__rdx_1))()` — not valid Rust at all — because a
/// closure is not a method name. Composition landed first and was unreachable
/// through a pipeline until now.
///
/// (The wrapping above avoids starting a line with the forward operator:
/// rustdoc reads a leading `>` as a blockquote marker and `rustdoc::…`'s
/// quote-line lint then fires on every line after it.)
pub(crate) fn pipeline(lhs: &Expr, rhs: &Expr, indent: usize) -> String {
  match unparen(rhs) {
    // `a |> .name args` and `a |> .name` — plan 57's method-as-function,
    // matched **before** the two arms below. §11.1 is the trap: the bare-name
    // arm appends the piped value as an argument, so a zero-argument
    // `x |> .name` falling through to it would lower to a call of a closure.
    //
    // `receiver` rather than `emit_expr` for the left operand here, and only
    // here: `1 .. 3 |> .count` is `(1..3).count()`, not `1..3.count()`, which
    // is a question a method call asks and a function call does not.
    Expr::MethodRef { name, .. } => format!("{}.{}()", receiver(lhs, indent), name.text),
    Expr::Call { callee, args, .. } if let Expr::MethodRef { name, .. } = unparen(callee) => {
      format!(
        "{}.{}({})",
        receiver(lhs, indent),
        name.text,
        expr_list(args, indent)
      )
    }
    // `a |> f x y` — the piped value joins the argument list, last.
    Expr::Call { callee, args, .. } => {
      let mut supplied: Vec<String> = args.iter().map(|a| emit_expr(a, indent)).collect();
      supplied.push(emit_expr(lhs, indent));
      format!(
        "{}({})",
        callable(unparen(callee), indent),
        supplied.join(", ")
      )
    }
    // `a |> f` — reverse application proper, and the case the row exists for.
    other => format!("{}({})", callable(other, indent), emit_expr(lhs, indent)),
  }
}
