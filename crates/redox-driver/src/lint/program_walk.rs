use redox_ast::{Item, Stmt};

/// What a lint wants to see as the walk descends a program.
///
/// Plan 53 leaf 4 built the descent — items → clauses, plain and guarded →
/// `where` locals → statements → statement-owned blocks, with `module`
/// recursing and class and instance methods walked as functions — and wrote
/// it as three functions that collected suppression spans. Plan 54 leaf 3
/// needed the identical descent for a different payload, so the payload moved
/// here and the descent stayed one.
///
/// # Why one walk and not two
///
/// [`stmts`](Self::stmts) is reached through
/// [`walk_stmts`](super::walk_stmts::walk_stmts), whose descent is
/// [`redox_ast::Expr::statement_blocks`] — an **exhaustive match with no
/// wildcard**, which is the property that makes the descent trustworthy: a
/// new form that owns a block has to be considered rather than quietly
/// excluded. Two copies of that match are two places to remember, and the
/// second one is the one that gets forgotten — so a lint added later sees a
/// smaller program than the lint added first, and nothing says so.
///
/// Both methods are **required**. A consumer that wants only statements says
/// so with an empty [`item`](Self::item) and a comment, which is a decision a
/// reader can see; a defaulted method is the same decision taken by not
/// noticing there was one.
pub(crate) trait ProgramWalk {
  /// One item, before anything it contains.
  ///
  /// Reached for a nested `module`'s items and for a class or instance
  /// method, which *is* a function and is walked as one.
  fn item(&mut self, item: &Item);

  /// One statement run, in source order.
  ///
  /// A clause body, a guard row's value, a clause's `where` locals, a `test`
  /// body, or a block some statement owns. Each call is **one list** — which
  /// is the unit `RDX1003` is defined over, and the reason it cannot see
  /// across a block boundary.
  fn stmts(&mut self, stmts: &[Stmt]);
}
