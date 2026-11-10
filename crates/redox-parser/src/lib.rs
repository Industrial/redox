//! `redox-parser` — hand-written recursive descent.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 4, hand-written arm.
//! `docs/COMPILER.md` §1 records the bake-off against LALRPOP that chose this
//! approach, its six axes, and its revisit trigger. The decisive finding:
//! LALRPOP cannot produce `Indent`/`Dedent` at all, so a generated parser
//! would need a hand-written pre-lexer anyway.
//!
//! Grammar for plan 01's slice:
//!
//! ```text
//! Program  := Item* Eof
//! Item     := Ident "=" "->" Newline Indent Stmt+ Dedent
//! Stmt     := Expr Newline
//! Expr     := Ident StrLit        # parenless call, one argument
//!           | StrLit
//!           | Ident
//! ```
//!
//! The one-argument restriction is the minimum this slice needs, applied
//! knowingly rather than inherited by accident — Emerald carried the
//! equivalent restriction for 89 plans because its LALR(1) grammar could not
//! lift it. Plan 19 generalises it to doc 03 §5.1's full call grammar.
//!
//! Errors **recover at item boundaries**, so one pass reports every bad item
//! rather than only the first. That is free in recursive descent and was a
//! whole plan in Emerald (its plan 26).
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: everything parsing
//! owns sits under `parse/`, headed by the entry point.

/// Parse an already-lexed token stream.
mod parse;

pub use parse::error::ParseError;
pub use parse::failure::ParseFailure;
pub use parse::max_nesting::MAX_NESTING;
pub use parse::parse;
pub use parse::source::parse_source;
