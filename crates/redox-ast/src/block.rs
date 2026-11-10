pub(crate) mod stmt;

use crate::{Expr, Stmt};
use redox_lexer::Span;

/// An indented sequence of statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
  pub stmts: Vec<Stmt>,
  pub span: Span,
}

impl Block {
  /// A block of one statement, for a body written as a single expression.
  ///
  /// What the one-line `if c then a else b` needs since plan 55 leaf 1 made
  /// its branches blocks: the inline spelling is a spelling, not a second
  /// kind of branch, so it is built as the block it means.
  pub fn of(expr: Expr) -> Self {
    let span = expr.span();
    Self {
      stmts: vec![Stmt {
        expr,
        span,
        leading: Vec::new(),
        trailing: None,
      }],
      span,
    }
  }

  /// This block's sole expression, when that is all it holds.
  ///
  /// `Some` for a block of exactly one element that is a plain value
  /// expression — not a binding, not a loop, not an `if`. Three printers ask
  /// it, each because the form it is writing has nowhere to put a second
  /// line: a one-line `if`/`else` branch, a collapsed `match` arm, and
  /// `unsafe { e }`.
  ///
  /// Trivia is **not** considered here. A comment must reach the output on a
  /// line of its own (doc 02 §10.1), and whether this block's one line can
  /// carry one depends on the form being written — so the caller that cares
  /// adds the test and the callers that do not are not made to.
  pub fn sole_expression(&self) -> Option<&Expr> {
    match self.stmts.as_slice() {
      [stmt] if !stmt.expr.is_statement_form() => Some(&stmt.expr),
      _ => None,
    }
  }
}
