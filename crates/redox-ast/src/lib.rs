//! `redox-ast` — syntax tree types. **No logic.**
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 3.
//!
//! Every node carries a byte [`Span`], and every node that can own source
//! lines carries [`Trivia`]. Both are built in from the start on evidence:
//! Emerald's plan 22 found `TypeExpr` carried no span at all, so every
//! diagnostic touching a type hardcoded `(0, 0)` and reported line 1 column 1
//! for an error on line 6 — discovered twenty-one plans after its diagnostics
//! plan shipped, by which point the full fix was judged too large.
//!
//! Deliberately not generic over a span type and deliberately not
//! arena-allocated (doc 01 §30's minimality constraint).
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, and per §2 rule 1 this crate holds no decisions — only types.

mod block;
mod expr;
mod fn_decl;
mod ident;
mod item;
mod program;
mod stmt;
mod stmt_kind;

pub use block::Block;
pub use expr::Expr;
pub use fn_decl::FnDecl;
pub use ident::Ident;
pub use item::Item;
pub use program::Program;
pub use stmt::Stmt;
pub use stmt_kind::StmtKind;

// Re-exported so downstream crates naming a span or a trivium need not add a
// second dependency edge onto `redox-lexer` (doc 06 §4).
pub use redox_lexer::{Span, Trivia, TriviaKind};
