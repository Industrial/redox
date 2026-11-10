use crate::emit::unparen::unparen;
use redox_ast::{BinaryOp, Expr};

/// The functions a composition chain applies, in **application order**.
///
/// `f >>> g >>> h` is `[f, g, h]`, and `f <<< g` is `[g, f]` — one lowering
/// for both directions, which is plan 66 §3 case 6's equivalence read off the
/// order rather than asserted twice.
///
/// Flattening rather than letting [`compose`](super::compose::compose) recurse
/// through the emitter is what keeps the lowering **one** closure. A nested
/// composition reached as an operand emits a closure inside its parent's body,
/// immediately applied and binding the same `__rdx_1` the parent already
/// bound. That is *correct* — the argument is evaluated outside the inner
/// closure, so the shadow never bites — and it is unreadable in a file doc 02
/// §10.1 makes human-facing.
///
/// `unparen` so a parenthesised chain flattens too: `(f >>> g) >>> h` is the
/// tree left-associativity already builds, and `f >>> (g >>> h)` is the one an
/// author writes.
pub(crate) fn compose_chain<'a>(
  op: BinaryOp,
  lhs: &'a Expr,
  rhs: &'a Expr,
  out: &mut Vec<&'a Expr>,
) {
  let (first, second) = if matches!(op, BinaryOp::ComposeBack) {
    (rhs, lhs)
  } else {
    (lhs, rhs)
  };
  for side in [first, second] {
    match unparen(side) {
      Expr::Binary {
        op: inner @ (BinaryOp::Compose | BinaryOp::ComposeBack),
        lhs,
        rhs,
        ..
      } => compose_chain(*inner, lhs, rhs, out),
      other => out.push(other),
    }
  }
}
