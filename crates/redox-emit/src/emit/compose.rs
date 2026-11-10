use crate::emit::closure::closure;
use crate::emit::closure_param::closure_param;
use crate::emit::compose_chain::compose_chain;
use crate::emit::expr::emit_expr;
use redox_ast::{BinaryOp, Expr, Ident};

/// Lower plan 66's composition: `f >>> g` and `f <<< g`, as a closure.
///
/// ```text
/// .trim >>> .to_lowercase    |__rdx_1| __rdx_1.trim().to_lowercase()
/// wrap >>> shout             |__rdx_1| shout(wrap(__rdx_1))
/// shout <<< wrap             |__rdx_1| shout(wrap(__rdx_1))
/// ```
///
/// **One lowering, the operands exchanged.** `f <<< g` is `g >>> f`, which is
/// plan 66 §3 case 6's equivalence and the reason keeping both directions is
/// two table entries rather than two lowerings.
///
/// The body is built as an **application of the AST** rather than as text: the
/// chain is folded into nested [`Expr::Call`]s over the parameter and emitted
/// once, like any other application. That is what makes
/// `.trim >>> .to_lowercase` a method chain instead of two closures called in
/// place — plan 57's receiver-last rule writes `__rdx_1.trim()`, and plan 56's
/// arity table decides whether an operand that is itself under-applied
/// becomes a closure. Composition needed neither of those to be re-stated.
///
/// `param` is the index [`closure_param`] binds, so a *section* of composition
/// can ask for the second name: `(>>> g)` is `\f -> f >>> g`, so the closure
/// `emit_section` writes binds `__rdx_1` and this one binds `__rdx_2`.
pub(crate) fn compose(op: BinaryOp, lhs: &Expr, rhs: &Expr, param: usize, indent: usize) -> String {
  let mut chain = Vec::new();
  compose_chain(op, lhs, rhs, &mut chain);
  let name = closure_param(param);
  let mut applied = Expr::Ident(Ident {
    text: name.clone(),
    span: lhs.span(),
  });
  for function in chain {
    applied = Expr::Call {
      callee: Box::new(function.clone()),
      args: vec![applied],
      is_dollar: false,
      span: function.span(),
    };
  }
  closure(&[name], &emit_expr(&applied, indent))
}
