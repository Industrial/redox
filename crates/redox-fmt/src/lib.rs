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

//! **Leaf 3 is in**, as `expr_text`: one expression in canonical Redox, with
//! §3's expression normalisations and a verbatim slice for the block-bearing
//! forms, which are statement containers and therefore leaf 4's. It is not yet
//! wired into `format`, because an expression is only reachable through the
//! item that holds it and items are leaf 2 — §4's corrected order has
//! expressions first precisely because everything above them delegates down.

//! **Leaf 4's first half is in**, as `pattern_text`. Patterns split off from
//! the rest of leaf 4 cleanly because they carry neither indentation nor
//! trivia — a pattern is always one line — so they need no `indent` and
//! nothing from the source, which the statement printer will need both of.

mod expr_text;
mod format;
mod pattern_text;

pub use expr_text::expr_text;
pub use format::format;
pub use pattern_text::pattern_text;
