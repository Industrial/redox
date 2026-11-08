/// What a cache hit returns: the generated Rust and its map, as text.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 3. Both halves, always
/// together — a hit that produced Rust without its map would hand plan 07's
/// remapping an entry with no positions, and the resulting diagnostic would
/// point nowhere while looking exactly like one that pointed somewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cached {
  pub(crate) rust: String,
  pub(crate) map: String,
}
