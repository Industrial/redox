use crate::emit::assert_call::assert_call;
use crate::emit::emit_control::emit_control;
use crate::emit::expr::emit_expr;
use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use crate::emit::unparen::unparen;
use redox_ast::{Stmt, StmtKind};

/// Emit one statement at `indent`.
///
/// `tail` marks the block's final expression, which is emitted **without a
/// semicolon** so it becomes the block's value. Doc 01 §5.2's own example
/// requires it: `add = (a, b): i32 -> a + b` must produce `{ a + b }`, and a
/// trailing semicolon makes the function return `()` instead.
///
/// Only a function that *declares* a return type gets a tail expression. One
/// that does not returns `()`, and its last line is an ordinary statement —
/// which is why a one-statement body still emits `println!("x");`.
///
/// The match is exhaustive with no wildcard, for the reason given in
/// `emit_expr`: a new `StmtKind` must fail to compile here.
pub(crate) fn emit_stmt(out: &mut Sink<'_>, stmt: &Stmt, indent: usize, tail: bool) {
  out.mark(stmt.span);
  emit_trivia(out, &stmt.leading, indent);

  // Plan 20's control flow writes its own padding, its own braces and its own
  // trailing newline, because each form spans lines and an `else if` has to
  // reuse the line its `}` is on.
  if matches!(
    stmt.kind,
    StmtKind::If { .. }
      | StmtKind::While { .. }
      | StmtKind::For { .. }
      | StmtKind::Loop { .. }
      | StmtKind::Break { .. }
      | StmtKind::Continue { .. }
      | StmtKind::Guard { .. }
  ) {
    emit_control(out, &stmt.kind, indent);
    return;
  }

  pad(out, indent);
  match &stmt.kind {
    StmtKind::Expr(e) => {
      out.push_str(&emit_expr(unparen(e), indent));
      if !tail {
        out.push(';');
      }
    }
    // Immutable by default (doc 01 §4.3): `mut` is the opt-in, and the type
    // is emitted only when the author wrote one — rustc infers the rest
    // (doc 02 §6.2).
    StmtKind::Binding(b) => {
      out.push_str("let ");
      if b.is_mut {
        out.push_str("mut ");
      }
      out.push_str(&b.name.text);
      if let Some(ty) = &b.ty {
        out.push_str(": ");
        out.push_str(&ty.text);
      }
      out.push_str(" = ");
      out.push_str(&emit_expr(&b.value, indent));
      out.push(';');
    }
    // A draw arrow is flattened by `emit_stmts`, which owns the run of
    // statements its callback body is made of. Reaching it here would mean a
    // caller emitted statements one at a time, which cannot lower it
    // correctly — so this is a bug in Redox, not in the program.
    StmtKind::Draw { .. } => unreachable!(
      "a draw arrow must be emitted by `emit_stmts`, which has the following \
       statements its callback body needs"
    ),
    // Doc 03 §12.6. A statement, so it always takes its `;`.
    StmtKind::Assert(e) => {
      out.push_str(&assert_call(e, indent));
      out.push(';');
    }
    // Doc 01 §4.3's compound assignment — plan 20. The operator's own Rust
    // text plus `=`, so the five share one arm.
    StmtKind::Assign { target, op, value } => {
      out.push_str(&format!(
        "{} {}= {};",
        emit_expr(unparen(target), indent),
        op.rust_text().unwrap_or("/* unsupported */"),
        emit_expr(unparen(value), indent)
      ));
    }
    // Written above, before the padding — plan 20.
    StmtKind::If { .. }
    | StmtKind::While { .. }
    | StmtKind::For { .. }
    | StmtKind::Loop { .. }
    | StmtKind::Break { .. }
    | StmtKind::Guard { .. }
    | StmtKind::Continue { .. } => {
      unreachable!("a control-flow statement is written by `emit_control`")
    }
    StmtKind::Return(value) => {
      out.push_str("return");
      if let Some(e) = value {
        out.push(' ');
        out.push_str(&emit_expr(e, indent));
      }
      out.push(';');
    }
  }
  emit_inline_comment(out, stmt.trailing.as_ref());
  out.push('\n');
}
