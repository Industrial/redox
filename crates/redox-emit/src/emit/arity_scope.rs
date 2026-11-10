use crate::emit::arity_table::ArityTable;
use std::cell::RefCell;

thread_local! {
  /// The table in scope, empty outside an [`ArityScope`].
  ///
  /// Empty is the safe default and that is load-bearing: an arity nobody
  /// installed is an arity nobody knows, which is the case plan 56 §11.1
  /// settles by emitting a call.
  static IN_SCOPE: RefCell<ArityTable> = RefCell::new(ArityTable::default());
}

/// The arity table in scope for the file being emitted.
///
/// Plan 56 (`docs/50 - 56 - Currying.md`) leaf 1. Entered once at the
/// emitter's entry (`emit_with_map`) and restored on drop, so one file's
/// signatures are never visible while another is being written — §1.4's
/// boundary, enforced by the lifetime of this guard rather than by convention.
///
/// # Why the table is ambient and not a parameter
///
/// Both readers are *inside* expression emission — the `Expr::Call` arm and
/// [`wraps_as_receiver`](crate::emit::wraps_as_receiver) — and `emit_expr` has
/// no context argument at all. §11.3 forbids the two obvious alternatives for
/// reasons that are each a measurement rather than a preference:
///
/// - **Not threaded through `emit_expr`.** Twenty-four importers and
///   thirty-six call sites would gain a parameter that all but two of them
///   only pass down.
/// - **Not a rewrite of `Program` in `parse()`.** `redox_fmt::format` consumes
///   the same tree (`redox-cli/src/fmt_command.rs`,
///   `redox-lsp/src/serve/formatting.rs`, `redox-mcp/src/format.rs`), so a
///   `Call`→`Lambda` rewrite there would make `redox fmt` rewrite the author's
///   `add 1` into a closure. Plan 61 states the rule this is a case of: *a
///   pass that rewrites `Program` has two consumers and will surprise one of
///   them.*
///
/// §11.3 then names the remaining split as *"the check rides `parse()`; the
/// rewrite lives at `emit_with_map` entry"* — and the check half went with
/// leaf 3, which §11.1 withdrew, so nothing rides `parse()` and there is no
/// rewrite either. What is left is the smallest thing that does the job: the
/// decision is **local** to the Call arm, which already holds the callee and
/// the argument list, and only the table has to travel. It travels here.
///
/// A `thread_local!` rather than a `static`, so the suite's parallel tests
/// cannot see each other's files; a guard rather than a setter, so an early
/// return cannot leave a stale table behind.
pub(crate) struct ArityScope {
  previous: ArityTable,
}

impl ArityScope {
  /// Install `table` for as long as the returned guard lives.
  pub(crate) fn enter(table: ArityTable) -> Self {
    let previous = IN_SCOPE.with_borrow_mut(|slot| std::mem::replace(slot, table));
    Self { previous }
  }

  /// The declared arity of `name` in the file being emitted.
  pub(crate) fn arity_of(name: &str) -> Option<usize> {
    IN_SCOPE.with_borrow(|table| table.arity_of(name))
  }
}

impl Drop for ArityScope {
  fn drop(&mut self) {
    let restored = std::mem::take(&mut self.previous);
    IN_SCOPE.with_borrow_mut(|slot| *slot = restored);
  }
}
