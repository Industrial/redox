//! `redox-lexer` — the off-side-rule lexer.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 2. Hand-written rather
//! than built on `logos`, because a regex-dispatch lexer cannot express an
//! off-side rule and Redox additionally needs trivia and whitespace-adjacency
//! on every token (`docs/COMPILER.md` §2).
//!
//! Three properties are mandatory and must not be weakened by later plans:
//!
//! 1. **Every token carries a byte [`Span`].**
//! 2. **Comments and blank-line runs become [`Trivia`] attached to the
//!    following token, never discarded.**
//! 3. **Every token records whether whitespace preceded it**
//!    ([`Token::ws_before`]).
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, modules private, public items re-exported here so callers see a flat
//! namespace.

mod is_ident_continue;
mod is_ident_start;
mod lex;
mod lex_error;
mod lexer;
mod span;
mod token;
mod token_kind;
mod trim_comment;
mod trivia;
mod trivia_kind;

pub use lex::lex;
pub use lex_error::LexError;
pub use span::Span;
pub use token::Token;
pub use token_kind::TokenKind;
pub use trivia::Trivia;
pub use trivia_kind::TriviaKind;
