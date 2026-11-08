/// Whether a command works yet.
///
/// Plan 11 (`docs/16 - 11 - CLI Surface.md`) leaf 1. The distinction is
/// user-facing and worth a type (§1.2): `redox fmt` is a *declared* part of
/// doc 01 §27's surface awaiting plan 34, not a typo. Answering both with
/// "unknown command" would tell a user they made a mistake when they did not,
/// and would hide that the feature is coming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
  /// Implemented and dispatched.
  Ready,
  /// Declared by doc 01 §27, implemented by the named plan.
  Planned {
    /// The plan number, e.g. `34`.
    plan: u32,
    /// Its id, e.g. `formatter`.
    id: &'static str,
  },
}
