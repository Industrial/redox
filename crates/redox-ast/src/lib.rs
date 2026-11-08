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
//! Layout follows `docs/06 - Redox Code Architecture.md`. The module tree is
//! the grammar's containment hierarchy: a `Program` holds `Item`s which hold
//! a `FnDecl`; a `Block` holds `Stmt`s which hold a `StmtKind`; an `Expr`
//! holds an `Ident`. Per §3 rule 1 this crate holds no decisions, only types.

mod block;
mod expr;
mod program;

pub use block::Block;
pub use block::stmt::Stmt;
pub use block::stmt::kind::StmtKind;
pub use expr::Expr;
pub use expr::ident::Ident;
pub use program::Program;
pub use program::item::Item;
pub use program::item::fn_decl::FnDecl;

// Re-exported so downstream crates naming a span or a trivium need not add a
// second dependency edge onto `redox-lexer` (doc 06 §6).
pub use redox_lexer::{Span, Trivia, TriviaKind};
