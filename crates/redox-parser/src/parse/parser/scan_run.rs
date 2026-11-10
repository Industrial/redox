use crate::ParseError;
use redox_ast::{Expr, Stmt};
use redox_diag::Code;

/// Walk a statement run. `in_tail` is whether these statements are already
/// inside a generated closure.
///
/// Descent is `Expr::statement_blocks`, which is the set of blocks that are
/// statement lists in the *same* scope: an `if`'s branches, the three loops'
/// bodies and a `guard`'s else. A lambda's body and a `try` block's are
/// closures of their own, so a `return` inside one leaves that closure and
/// not this one — which is the exclusion `return_in_tail`'s doc argues for,
/// and which that method's own doc keeps.
pub(crate) fn scan_run(stmts: &[Stmt], in_tail: bool, errors: &mut Vec<ParseError>) {
  for (i, stmt) in stmts.iter().enumerate() {
    if let Expr::Draw { .. } = &stmt.expr {
      // Everything after a `with` is its closure's body, and the `with`
      // itself ends this run — so the rest is scanned as a tail and nothing
      // here follows it.
      scan_run(&stmts[i + 1..], true, errors);
      return;
    }

    if in_tail && let Expr::Return { .. } = &stmt.expr {
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
    for nested in stmt.expr.statement_blocks() {
      scan_run(&nested.stmts, in_tail, errors);
    }
  }
}
