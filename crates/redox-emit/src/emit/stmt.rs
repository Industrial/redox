use crate::emit::assert_call::assert_call;
use crate::emit::emit_await::emit_await;
use crate::emit::emit_control::emit_control;
use crate::emit::expr::emit_expr;
use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::pad::pad;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use crate::emit::unparen::unparen;
use redox_ast::{Expr, Stmt};

/// Emit one block element at `indent`.
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
/// # This is the statement position, and that is a choice now rather than a type
///
/// Plan 55 leaf 1 deleted `StmtKind`, so a block element and a subexpression
/// are the same type. What used to be decided by *which enum a node lived in*
/// is decided here instead: the forms `Expr::is_statement_form` names are
/// written by this function, and everything else is handed to `emit_expr`.
/// A new form still has to be considered, because `emit_expr` stays
/// exhaustive with no wildcard for the reason its own doc gives.
pub(crate) fn emit_stmt(out: &mut Sink<'_>, stmt: &Stmt, indent: usize, tail: bool) {
  out.mark(stmt.span);
  emit_trivia(out, &stmt.leading, indent);

  // Plan 20's control flow writes its own padding, its own braces and its own
  // trailing newline, because each form spans lines and an `else if` has to
  // reuse the line its `}` is on.
  if stmt.expr.is_control_flow() {
    emit_control(out, &stmt.expr, indent, tail);
    return;
  }

  pad(out, indent);
  match &stmt.expr {
    // Immutable by default (doc 01 §4.3): `mut` is the opt-in, and the type
    // is emitted only when the author wrote one — rustc infers the rest
    // (doc 02 §6.2).
    Expr::Binding(b) => {
      out.push_str("let ");
      if b.is_mut {
        out.push_str("mut ");
      }
      out.push_str(&b.pattern.rust_text());
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
    Expr::Draw { .. } => unreachable!(
      "a draw arrow must be emitted by `emit_stmts`, which has the following \
       statements its callback body needs"
    ),
    // Doc 03 §12.6. A statement, so it always takes its `;`.
    Expr::Assert { value, .. } => {
      out.push_str(&assert_call(value, indent));
      out.push(';');
    }
    // Doc 30's await-bind. A statement, which is the whole of the decision:
    // `<-` means `await` and nothing else, and every await binds.
    Expr::Await { pattern, value, .. } => {
      out.push_str(&emit_await(pattern, value, indent));
    }
    // Assignment — doc 01 §4.3's compound form from plan 20, and plan 54's
    // plain `:=`. Rust spells both with `=`, differing only by the operator
    // in front of it, so one arm prints all six: `None` is `=` and `Some(op)`
    // is the operator's own Rust text plus `=`.
    //
    // Redox's two spellings collapse to Rust's one here, which is the point
    // of the row: `:=` exists to stop *Redox* having to decide whether `=`
    // means a binding, and Rust never had that problem because `let` says so.
    Expr::Assign {
      target, op, value, ..
    } => {
      let operator = op.map_or("", |op| op.rust_text().unwrap_or("/* unsupported */"));
      out.push_str(&format!(
        "{} {operator}= {};",
        emit_expr(unparen(target), indent),
        emit_expr(unparen(value), indent)
      ));
    }
    Expr::Return { value, .. } => {
      out.push_str("return");
      if let Some(e) = value {
        out.push(' ');
        out.push_str(&emit_expr(e, indent));
      }
      out.push(';');
    }
    // Every other form is a value expression, and a block element that is one
    // is Rust's expression statement: the expression, then a `;` unless it is
    // the block's value. Control flow is written above, before the padding,
    // so none of those forms reaches here.
    value => {
      debug_assert!(
        !value.is_statement_form(),
        "every statement form has an arm of its own above"
      );
      out.push_str(&emit_expr(unparen(value), indent));
      if !tail {
        out.push(';');
      }
    }
  }
  emit_inline_comment(out, stmt.trailing.as_ref());
  out.push('\n');
}
