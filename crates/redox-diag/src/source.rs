//! Source positions.
//!
//! A single child today ([`span::Span`]); doc 06 §2.5 permits that where
//! growth is declared, and plan 06's span map — which maps generated `.xrs`
//! offsets back to `.sdx` offsets — belongs here.

pub(crate) mod map;
pub(crate) mod origin;
pub(crate) mod span;
