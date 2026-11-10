use crate::emit::condition_head::condition_head;
use crate::emit::emit_stmts::emit_stmts;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use redox_ast::Expr;

/// An `else if`'s nested `if`, written without its leading pad.
///
/// The `} else ` before it already occupies the line, so the nested statement
/// must not pad again. Separate from `emit_control` because the difference is
/// one call and merging them would mean a flag on every arm.
///
/// `tail` carries the same meaning as in `emit_control`, and must be forwarded
/// here too: in `if a then … else if b then … else …` the value can come from
/// any rung of the chain, so dropping it at the first `else if` would make the
/// deeper branches statements again.
pub(crate) fn emit_control_inline(out: &mut Sink<'_>, expr: &Expr, indent: usize, tail: bool) {
  let Expr::If {
    condition,
    then_block,
    otherwise,
    ..
  } = expr
  else {
    unreachable!("only an `if` is written inline")
  };

  out.push_str(&condition_head("if", condition, indent));
  emit_stmts(
    out,
    &then_block.stmts,
    indent + RUST_INDENT,
    tail && otherwise.is_some(),
  );

  let Some(otherwise) = otherwise else {
    out.unmark();
    pad(out, indent);
    out.push_str("}\n");
    return;
  };

  out.unmark();
  pad(out, indent);
  if let [only] = otherwise.stmts.as_slice()
    && matches!(only.expr, Expr::If { .. })
  {
    out.push_str("} else ");
    emit_control_inline(out, &only.expr, indent, tail);
    return;
  }
  out.push_str("} else {\n");
  emit_stmts(out, &otherwise.stmts, indent + RUST_INDENT, tail);
  out.unmark();
  pad(out, indent);
  out.push_str("}\n");
}
