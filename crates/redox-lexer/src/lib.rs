//! `redox-lexer` — the off-side-rule lexer.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`) leaf 2 built the minimum;
//! plan 02 (`docs/07 - 02 - Lexical Surface.md`) finished the lexical surface
//! of `docs/03 - Redox Surface Design.md` §4 and §11.
//!
//! Hand-written rather than built on `logos`: a regex-dispatch lexer cannot
//! express an off-side rule, and continuation suppression is lexer *state*
//! carried across lines (`docs/COMPILER.md` §2).
//!
//! Three properties are mandatory and must not be weakened by later plans:
//!
//! 1. **Every token carries a byte [`Span`]** — including tokens inside a
//!    string interpolation, whose spans index the original file.
//! 2. **Comments, doc comments and fences become [`Trivia`] attached to the
//!    following token, never discarded.**
//! 3. **Every token records whether whitespace preceded it**
//!    ([`Token::ws_before`]), which doc 03 §5.1's call grammar and §5.4's
//!    `(-1)`-versus-`(- 1)` rule both depend on.
//!
//! This crate decides *what tokens exist*, not what they mean. A token no
//! parser yet accepts is expected, not dead.
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, grouped into a module tree, modules private, public items
//! re-exported here so callers see a flat namespace.

mod lex;
mod lexer;

mod token;
mod trivia;
mod verbatim_triple;

pub use lex::error::LexError;
pub use lex::lex;
pub use redox_diag::Span;
pub use token::Token;
pub use token::keyword::Keyword;
pub use token::kind::TokenKind;
pub use token::str_part::StrPart;
pub use trivia::Trivia;
pub use trivia::kind::TriviaKind;
pub use verbatim_triple::verbatim_triple;
