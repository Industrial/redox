//! `redox-fmt` — the canonical formatter.
//!
//! Declared empty by plan 01 leaf 1. Filled by **plan 34** (`formatter`).
//!
//! It must be **trivia-preserving**, not a pure AST pretty-printer. Emerald's
//! formatter took the latter approach and loses every non-doc comment, which
//! its own module doc discloses; for Redox that would be a product defect,
//! since comments must reach the generated `.xrs`. The trivia this plan's
//! lexer already collects is what makes the correct approach available.
//!
//! Layout follows `docs/06 - Redox Code Architecture.md`: one construct per
//! file, modules private, public items re-exported from here.
//!
//! **Plan 34 leaf 1 is in.** It canonicalises file-level structure — one blank
//! line between items, no leading blank lines, one trailing newline, no
//! trailing whitespace — and passes every item's own text through verbatim,
//! sliced from the source by its span.
//!
//! That ordering is deliberate: plan 34 §1.2 says the row is too large for one
//! sitting, and starting from a formatter that *cannot* lose text or change
//! meaning means its two properties (§1.4) hold at every step. Each later leaf
//! replaces one span-slice with a real printer.

mod format;

pub use format::format;
