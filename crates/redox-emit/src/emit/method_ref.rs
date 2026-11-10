use crate::emit::closure::closure;
use crate::emit::closure_param::closure_param;
use crate::emit::expr_list::expr_list;
use crate::emit::receiver::receiver;
use redox_ast::{Expr, Ident};

/// Lower plan 57's method-as-function: `.name` and its application.
///
/// ```text
/// .trim raw          raw.trim()
/// .cmp y x           x.cmp(y)
/// .insert k v m      m.insert(k, v)
/// .name              |__rdx_1| __rdx_1.name()
/// ```
///
/// **The receiver is the last argument**, which §1.2 settles rather than
/// prefers: `|>` feeds its left operand as the last argument and partial
/// application supplies arguments from the left, so receiver-last is the only
/// order on which both work from one spelling. Read bare, `.cmp y x` puts the
/// receiver in a surprising place; read in a pipeline, which is where it will
/// almost always be read, it is in the only place it could be.
///
/// Applied to **nothing** it is a value awaiting its receiver, and the closure
/// it becomes is byte-identical to what `(|> name)` emitted — which is why
/// plan 57 retires that spelling rather than keeping two surfaces for one
/// construct (§1.3's decision block).
///
/// The receiver goes through [`receiver`] for the reason every other receiver
/// does: `.` binds tighter than every binary operator, so `1 .. 3 |> count`
/// is `(1..3).count()`. Nothing else here needs a precedence decision —
/// `expr_list` writes the arguments exactly as a call's.
pub(crate) fn method_ref(name: &Ident, args: &[Expr], indent: usize) -> String {
  let Some((recv, rest)) = args.split_last() else {
    let param = closure_param(0);
    let body = format!("{param}.{}()", name.text);
    return closure(&[param], &body);
  };
  format!(
    "{}.{}({})",
    receiver(recv, indent),
    name.text,
    expr_list(rest, indent)
  )
}
