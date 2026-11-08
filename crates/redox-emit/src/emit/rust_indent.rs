/// Indentation of emitted Rust, in spaces.
///
/// Four, matching rustfmt's default, so generated code does not read as
/// foreign. Independent of this repository's own two-space source style
/// (`rustfmt.toml`) and of the source indentation width, which the lexer
/// deliberately does not fix.
pub(crate) const RUST_INDENT: usize = 4;
