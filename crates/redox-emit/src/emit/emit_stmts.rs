use crate::emit::expr::emit_expr;
use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::stmt::emit_stmt;
use crate::emit::trivia::emit_trivia;
use redox_ast::{Stmt, StmtKind};

/// Emit a run of statements, flattening doc 18 §3.1's draw arrows.
///
/// Every other statement lowers on its own, which is why `emit_stmt` takes one
/// at a time. A draw arrow cannot: its lowering is a **callback whose body is
/// every statement after it**, so the unit of translation is the run rather
/// than the statement.
///
/// ```text
/// conn <- pool.with_connection      pool.with_connection(|conn| {
/// tx <- conn.transaction        →       conn.transaction(|tx| {
/// commit tx                                 commit(tx);
///                                       })
///                                   })
/// ```
///
/// Nesting falls out of the recursion: the second draw arrow is simply the
/// first statement of the first one's body. That is doc 03 §7.6's lowering
/// character for character, and the reason doc 18 §3.1 could delete `use`,
/// `is`, `defer` and the comprehension's `for` in favour of one token — the
/// flattening was already the hard part, and it was already written.
///
/// `tail` applies to the **last** statement of the whole run, which after a
/// draw arrow is the last statement of the innermost closure: that is where
/// the function's value comes from.
pub(crate) fn emit_stmts(out: &mut Sink<'_>, stmts: &[Stmt], indent: usize, tail: bool) {
  let Some((first, rest)) = stmts.split_first() else {
    return;
  };

  let StmtKind::Draw { name, value } = &first.kind else {
    let last = rest.is_empty();
    emit_stmt(out, first, indent, tail && last);
    emit_stmts(out, rest, indent, tail);
    return;
  };

  out.mark(first.span);
  emit_trivia(out, &first.leading, indent);
  pad(out, indent);
  out.push_str(&emit_expr(value));
  out.push_str("(|");
  out.push_str(&name.text);
  out.push_str("| {\n");

  emit_stmts(
    out,
    rest,
    indent + crate::emit::rust_indent::RUST_INDENT,
    tail,
  );

  pad(out, indent);
  out.push_str("})");
  // The closing `;` is the draw-arrow statement's own, and is omitted when the
  // callback's value is the function's value.
  if !tail {
    out.push(';');
  }
  out.push('\n');
}
