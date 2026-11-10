use crate::emit::bind_mode::BindMode;
use std::cell::Cell;

thread_local! {
  /// What `<-` means right here. [`BindMode::Await`] outside any modifier,
  /// which is the meaning doc 30 gave the token and the one `async fn` keeps.
  static MODE: Cell<BindMode> = const { Cell::new(BindMode::Await) };
}

/// The bind mode, installed for the body of one block modifier.
///
/// Plan 60 (`docs/54 - 60 - The Bind Arrow.md`) leaf 3. A guard rather than a
/// setter, because the modes **nest**: a `try` inside an `async` and an
/// `async` inside a `try` both have to restore the outer meaning when their
/// body ends, and an early return must not leave the wrong one installed.
///
/// `Cell` rather than `RefCell`, because the mode is `Copy` and nothing
/// borrows it across a call. `thread_local!` for `ArityScope`'s reason: the
/// suite's parallel tests must not see each other's blocks.
pub(crate) struct BindScope {
  /// The mode to put back when the guard drops.
  previous: BindMode,
}

impl BindScope {
  /// Install `mode` for as long as the returned guard lives.
  pub(crate) fn enter(mode: BindMode) -> Self {
    Self {
      previous: MODE.replace(mode),
    }
  }

  /// What `<-` means at this point in the emission.
  pub(crate) fn mode() -> BindMode {
    MODE.get()
  }
}

impl Drop for BindScope {
  fn drop(&mut self) {
    MODE.set(self.previous);
  }
}
