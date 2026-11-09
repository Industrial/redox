use notify::EventKind;
use notify::event::ModifyKind;

/// Whether an event kind represents a **change** to a file.
///
/// Plan 12 (`docs/17 - 12 - Watcher.md`) §10.1 — the defect that makes this
/// function exist, and the one correction to the plan's own reasoning.
///
/// `is_relevant` filters by *path*: the generated `.xrs` is not a `.sdx`, so
/// writing it cannot retrigger a build. That is necessary and it is not
/// sufficient, because **the build reads its own input**, and `inotify`
/// reports that read:
///
/// ```text
/// EVENT Access(Open(Any)) ["main.sdx"]
/// ```
///
/// A `.sdx` path, so `is_relevant` says yes, so the loop compiles, which opens
/// the file, which is another event. Measured: **97,442 builds in six
/// seconds** on a file nobody was editing.
///
/// Plan 12 §1.1 claimed the cache meant the watcher *"does not need to be
/// correct about which events matter"*. That is wrong, and this is the
/// correction: a cache makes a redundant compile cheap, but it cannot break a
/// loop whose trigger is the compile itself. Cheap multiplied by unbounded is
/// still unbounded.
///
/// So `Access` is excluded — reading a file is not changing it. So is
/// `Modify(Metadata)`: a `chmod` or an `atime` update is not a content change,
/// and under `relatime` the build's own read can update `atime`, which would
/// reopen the same loop by another door.
pub(crate) fn is_change(kind: &EventKind) -> bool {
  match kind {
    EventKind::Create(_) | EventKind::Remove(_) => true,
    EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_)) => true,
    // An unrecognised kind is treated as a change: a watcher that misses an
    // edit looks broken, and the cache does make the redundant compile cheap
    // *provided* it is not self-triggering. Every self-triggering kind is
    // named above and excluded.
    EventKind::Any | EventKind::Other => true,
    EventKind::Access(_) | EventKind::Modify(_) => false,
  }
}

#[cfg(test)]
mod tests {
  use super::is_change;
  use notify::EventKind;
  use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, ModifyKind, RemoveKind,
    RenameMode,
  };

  /// The event that caused the loop. If this ever returns `true` again, the
  /// watcher will spin on its own reads.
  #[test]
  fn reading_a_file_is_not_a_change() {
    assert!(!is_change(&EventKind::Access(AccessKind::Open(
      AccessMode::Any
    ))));
    assert!(!is_change(&EventKind::Access(AccessKind::Read)));
    assert!(!is_change(&EventKind::Access(AccessKind::Close(
      AccessMode::Read
    ))));
    assert!(!is_change(&EventKind::Access(AccessKind::Any)));
  }

  /// Writing is. `Close(Write)` is an `Access` kind and still excluded — the
  /// `Modify(Data)` event that accompanies the same write is what triggers.
  #[test]
  fn writing_data_is_a_change() {
    assert!(is_change(&EventKind::Modify(ModifyKind::Data(
      DataChange::Any
    ))));
    assert!(is_change(&EventKind::Modify(ModifyKind::Data(
      DataChange::Content
    ))));
  }

  /// A rename over the source is how editors save, and it is a change.
  #[test]
  fn renaming_is_a_change() {
    assert!(is_change(&EventKind::Modify(ModifyKind::Name(
      RenameMode::To
    ))));
    assert!(is_change(&EventKind::Modify(ModifyKind::Name(
      RenameMode::Both
    ))));
  }

  #[test]
  fn creating_and_removing_are_changes() {
    assert!(is_change(&EventKind::Create(CreateKind::File)));
    assert!(is_change(&EventKind::Remove(RemoveKind::File)));
  }

  /// Metadata is excluded, and the reason is not fastidiousness: under
  /// `relatime` the build's own read can update `atime`, which would reopen
  /// the loop by another door.
  #[test]
  fn metadata_alone_is_not_a_change() {
    assert!(!is_change(&EventKind::Modify(ModifyKind::Metadata(
      MetadataKind::Any
    ))));
    assert!(!is_change(&EventKind::Modify(ModifyKind::Metadata(
      MetadataKind::AccessTime
    ))));
  }

  /// An unknown kind errs toward rebuilding, because a missed edit looks like
  /// a broken watcher.
  #[test]
  fn an_unknown_kind_is_treated_as_a_change() {
    assert!(is_change(&EventKind::Any));
    assert!(is_change(&EventKind::Other));
  }
}
