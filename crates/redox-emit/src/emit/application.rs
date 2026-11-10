use crate::emit::closure::closure;
use crate::emit::closure_param::closure_param;
use crate::emit::expr_list::expr_list;
use crate::emit::missing_arity::missing_arity;
use crate::emit::receiver::receiver;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// Lower doc 18 §2.1's application: a saturated call, or the closure an
/// under-applied one becomes.
///
/// Plan 56 (`docs/50 - 56 - Currying.md`) leaves 1, 2 and 4.
///
/// ```text
/// add 1 2      add(1, 2)
/// add 1        |__rdx_1| add(1, __rdx_1)
/// tagged "x"   |__rdx_1, __rdx_2| tagged("x", __rdx_1, __rdx_2)
/// add 1 $ 2    add(1, 2)
/// op 7 8       op(7, 8)
/// ```
///
/// Three things are deliberate about the shape.
///
/// **The closure is flat, not nested.** Haskell would read a one-of-three as
/// three closures, and §3 case 2 says *"nested closures"*; it cannot be, and
/// the reason is on the **consumer** side rather than this one. An
/// under-applied value is bound to a name, and a call of that name is a call
/// of an *unknown-arity* name — `inc 5` emits `inc(5)` and `two 2 3` emits
/// `two(2, 3)`, because no signature in the file names them. A nested closure
/// would reject both. So the closure binds every parameter still missing, in
/// one list, which is the shape the rest of the language already calls.
///
/// **`f a $ b` is flattened back into `f(a, b)`.** §11.6's decision: `$` is
/// Haskell's loose application and carries no meaning beyond precedence, so
/// if `add 1 $ 2` allocated a closure and `add 1 2` did not, `$` would have
/// become semantically significant and a reader would have to know which
/// spelling allocates. The flatten is gated on [`missing_arity`] — *the inner
/// call would have become a closure that is immediately called* — so it
/// collapses exactly the allocation it was about to make, and a saturated
/// inner call (`add 1 2 $ 3`) is left alone for rustc to judge.
///
/// **The callee goes through [`receiver`].** `(` binds as tightly as `.`, so a
/// callee that is a closure needs its parentheses, and §11.5 found that it did
/// not get them: the Call arm called bare `emit_expr` and never consulted
/// `wraps_as_receiver`, which does list `Expr::Lambda` and `Expr::Section`.
/// `print ((+ 1) 2)` emitted `|__rdx_1| __rdx_1 + 1(2)` — broken Rust — and
/// `print ((\_ -> 7) 1)` only worked because a *written* paren leaves an
/// `Expr::Paren` behind. A synthesized closure has no `Paren`, so this row
/// would have inherited the first shape; §11.5 asks for the fix in the row
/// that creates synthesized lambdas, and this is it.
pub(crate) fn emit_application(callee: &Expr, args: &[Expr], indent: usize) -> String {
  if let Expr::Call {
    callee: inner,
    args: inner_args,
    ..
  } = unparen(callee)
    && missing_arity(inner, inner_args).is_some()
  {
    let mut flattened = inner_args.clone();
    flattened.extend(args.iter().cloned());
    return emit_application(inner, &flattened, indent);
  }

  let head = receiver(callee, indent);
  let Some(missing) = missing_arity(callee, args) else {
    return format!("{head}({})", expr_list(args, indent));
  };

  let params: Vec<String> = (0..missing).map(closure_param).collect();
  let body = format!("{head}({}, {})", expr_list(args, indent), params.join(", "));
  closure(&params, &body)
}
