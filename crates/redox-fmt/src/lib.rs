//! `redox-fmt` — the canonical formatter.
//!
//! Declared empty by plan 01 leaf 1. Filled by **plan 34** (`formatter`).
//!
//! It must be **trivia-preserving**, not a pure AST pretty-printer. Emerald's
//! formatter took the latter approach and loses every non-doc comment, which
//! its own module doc discloses; for Redox that would be a product defect,
//! since comments must reach the generated `.xrs`. The trivia this plan's
//! lexer already collects is what makes the correct approach available.
