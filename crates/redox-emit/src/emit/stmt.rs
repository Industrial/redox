use crate::emit::expr::emit_expr;
use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::pad::pad;
use crate::emit::trivia::emit_trivia;
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
