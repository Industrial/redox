//! Operator binding powers.
//!
//! Plan 03 (`docs/08 - 03 - Parser Core.md`) leaf 2. A grouping module
//! (doc 06 §1.2): a table and two thresholds are peers, with no natural head.
//!
//! Doc 02 §5.2 fixes the table as **Rust's unchanged** — a Redox expression
//! that reads like Rust and binds differently is the worst available failure
//! mode, worse than an unfamiliar operator.

/// The minimum binding power that stops a Pratt loop at a bare `|`.
pub(crate) mod above_bitor;
/// Which operator a token spells, and its `(left, right)` binding powers.
pub(crate) mod binary_power;
/// Binding power of the prefix operators — `-`, `!`, `&`, `*`.
pub(crate) mod prefix_power;
