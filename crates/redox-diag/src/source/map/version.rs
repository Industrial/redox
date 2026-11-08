/// The source map sidecar's format version.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 4. Written into the header so
/// a map produced by a different Redox is rejected outright rather than
/// misread: the records are positional, so a changed field order would parse
/// successfully and resolve wrongly — the worst failure a source map can have,
/// because every position it reports would still look plausible.
pub const VERSION: u32 = 1;
