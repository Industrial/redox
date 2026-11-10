use crate::emit::binding_kind::BindingKind;
use crate::emit::binding_table::BindingTable;
use std::cell::RefCell;

thread_local! {
  /// The table in scope, empty outside a [`BindingScope`].
  static IN_SCOPE: RefCell<BindingTable> = RefCell::new(BindingTable::default());
}

/// The binding table, installed for the duration of one file's emission.
///
/// Plan 59 (`docs/53 - 59 - Value Bindings.md`) leaf 1, and the placement is
/// cited rather than re-derived: the table rides the **emitter's entry** and
/// nothing earlier, because `redox_fmt::format` consumes the same `Program`
/// and must keep printing the author's `block` as `block`. A rewrite visible
/// before this point would make the formatter rewrite the surface.
///
/// A `thread_local!` rather than a `static`, so the suite's parallel tests
/// cannot see each other's files; a guard rather than a setter, so an early
/// return cannot leave a stale table behind. `ArityScope`'s shape exactly —
/// two guards over two tables, entered side by side in `emit_with_map`.
pub(crate) struct BindingScope {
  previous: BindingTable,
}

impl BindingScope {
  /// Install `table` for as long as the returned guard lives.
  pub(crate) fn enter(table: BindingTable) -> Self {
    let previous = IN_SCOPE.with_borrow_mut(|slot| std::mem::replace(slot, table));
    Self { previous }
  }

  /// What `name` means in the file being emitted.
  pub(crate) fn kind_of(name: &str) -> Option<BindingKind> {
    IN_SCOPE.with_borrow(|table| table.kind_of(name))
  }
}

impl Drop for BindingScope {
  fn drop(&mut self) {
    let restored = std::mem::take(&mut self.previous);
    IN_SCOPE.with_borrow_mut(|slot| *slot = restored);
  }
}
