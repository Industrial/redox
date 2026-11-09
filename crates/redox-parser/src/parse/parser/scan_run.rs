use crate::ParseError;
use crate::parse::parser::blocks_in_the_same_closure::blocks_in_the_same_closure;
use redox_ast::{Stmt, StmtKind};
use redox_diag::Code;

/// Walk a statement run. `in_tail` is whether these statements are already
/// inside a generated closure.
pub(crate) fn scan_run(stmts: &[Stmt], in_tail: bool, errors: &mut Vec<ParseError>) {
  for (i, stmt) in stmts.iter().enumerate() {
    if let StmtKind::Draw { .. } = &stmt.kind {
      // Everything after a `with` is its closure's body, and the `with`
      // itself ends this run — so the rest is scanned as a tail and nothing
      // here follows it.
      scan_run(&stmts[i + 1..], true, errors);
      return;
    }

    if in_tail && let StmtKind::Return(_) = &stmt.kind {
      errors.push(ParseError::with(
        Code::ReturnInCallbackTail,
        "`return` here returns from the closure `with` generates, not from the \
         enclosing function — doc 03 §7.6. Use `?` to propagate, or restructure so the \
         value is the callback's",
        stmt.span,
      ));
    }

    // Descend into the blocks that stay inside this closure. Every one of
    // these is a statement list in the same scope, so a `return` in it leaves
    // the same closure.
    for nested in blocks_in_the_same_closure(&stmt.kind) {
      scan_run(&nested.stmts, in_tail, errors);
    }
  }
}
