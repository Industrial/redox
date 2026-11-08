use crate::emit_expr::emit_expr;
use crate::emit_inline_comment::emit_inline_comment;
use crate::emit_trivia::emit_trivia;
use crate::pad::pad;
use redox_ast::{Stmt, StmtKind};

/// Emit one statement at `indent`.
pub(crate) fn emit_stmt(out: &mut String, stmt: &Stmt, indent: usize) {
  emit_trivia(out, &stmt.leading, indent);
  pad(out, indent);
  match &stmt.kind {
    StmtKind::Expr(e) => {
      out.push_str(&emit_expr(e));
      out.push(';');
    }
  }
  emit_inline_comment(out, stmt.trailing.as_ref());
  out.push('\n');
}
