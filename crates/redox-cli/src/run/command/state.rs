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
  ///
  /// **Unconstructed since plan 34**, which made `fmt` — the last planned
  /// command — `Ready`. Kept rather than removed, which is the opposite of
  /// what plan 18 did with `TokenKind::Attr`, and the difference is whether
  /// the variant can ever come back: `Attr` could not, because attributes
  /// became trivia for good, and this one is constructed again the moment a
  /// command is declared ahead of its row. Rows 40 (`lsp`), 49
  /// (`editor-integrations`) and 50 (`mcp-server`) each bring one.
  ///
  /// The tests construct it, so the mechanism stays exercised while no real
  /// command needs it.
  #[allow(
    dead_code,
    reason = "the next declared-before-built command uses it; see above"
  )]
  Planned {
    /// The plan number, e.g. `34`.
    plan: u32,
    /// Its id, e.g. `formatter`.
    id: &'static str,
  },
}
