use redox_ast::{Clause, Stmt};

/// A clause's body as one tail expression statement, when that is what it is.
///
/// Used to decide whether a `match` arm can be written `pat => expr,` — see
/// `emit_arm`, which explains why that spelling is required rather than
/// merely tidier.
///
/// A leading comment disqualifies it: a comment must reach the `.xrs` on a
/// line of its own (doc 02 §10.1), and a collapsed arm has no line to put it
/// on. `tail` disqualifies it too — a body that is not the function's value
/// ends in `;`, which rustfmt keeps braced.
pub(crate) fn one_tail_expression(clause: &Clause, tail: bool) -> Option<&Stmt> {
  if !tail {
    return None;
  }
  match clause.body.plain()?.stmts.as_slice() {
    [stmt] if !stmt.expr.is_statement_form() && stmt.leading.is_empty() => Some(stmt),
    _ => None,
  }
}
