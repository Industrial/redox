//! Source positions.
//!
//! A single child today ([`span::Span`]); doc 06 §2.5 permits that where
//! growth is declared, and plan 06's span map — which maps generated `.xrs`
//! offsets back to `.sdx` offsets — belongs here.

/// The `.xrs` to `.sdx` line map, and its sidecar format.
pub(crate) mod map;
/// Where one line of generated Rust came from.
pub(crate) mod origin;
/// A half-open byte range into the source text.
pub(crate) mod span;
