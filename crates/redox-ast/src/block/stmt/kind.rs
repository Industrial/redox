use crate::{Binding, Expr};

/// What a [`crate::Stmt`] is.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) added bindings and `return`;
/// control flow is plan 20.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
  Expr(Expr),
  Binding(Binding),
  /// `return` with an optional value.
  Return(Option<Expr>),
  /// `name <- expr`, doc 18 §3.1's draw arrow in statement position.
  ///
  /// The construct doc 03 §7.6 spelled `use name <- expr`. Its lowering is a
  /// **callback**: `expr` is called with a closure whose parameter is `name`
  /// and whose body is **every statement after this one**. So a draw arrow is
  /// the one statement whose meaning depends on its successors, which is why
  /// the emitter handles a block rather than a statement at a time.
  ///
  /// ```text
  /// conn <- pool.with_connection      pool.with_connection(|conn| {
  /// print conn                   →       println!("{}", conn);
  ///                                  })
  /// ```
  Draw {
    name: crate::Ident,
    /// The callback-taking expression.
    value: Expr,
  },
  /// `assert e`, doc 03 §12.6's assertion spelling.
  ///
  /// A statement rather than a call, because `assert` is a reserved word —
  /// and because the lowering reads the operator the author wrote: `==`
  /// becomes `assert_eq!` and `!=` becomes `assert_ne!`, which print both
  /// values when they differ where `assert!` prints only the expression's
  /// text.
  Assert(Expr),
}
