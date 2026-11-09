//! `redox-bench` — doc 03's zero-cost claim, measured.
//!
//! Plan 44 (`docs/38 - 44 - Zero-cost Benchmarks.md`). Two things, in two
//! categories, and the split is §1.2's correction to the row's own register
//! cell:
//!
//! | What | How |
//! |---|---|
//! | Generated versus hand-written Rust | **identical optimised IR**, asserted in the gate |
//! | Transpile throughput | timed, reported in `benchmarks/REPORT.md` |
//!
//! # Why not a timing comparison
//!
//! Doc 04's row says *"asserted within noise"*. A timing assertion is a flaky
//! gate: plan 23 §10.8, doc 30 §8.7 and plan 43 §10.3 are three rows that
//! wrote *unexplained* into their results because a gate failed for a
//! load-dependent reason.
//!
//! Identical optimised IR is deterministic, exact — *within noise* becomes
//! *identical*, which is the claim doc 03 makes — and actionable, because a
//! timing regression says *slower* where an IR diff says **what was added**.
//!
//! What it is not is a runtime measurement, and §6 says so: identical IR is
//! sufficient for zero cost, and a timing comparison is not necessary once you
//! have it.

pub mod ir;
pub mod pair;
mod throughput;

pub use ir::function_ir::function_ir;
pub use ir::rustc_path::rustc_path;
pub use pair::Pair;
pub use pair::pairs::pairs;
pub use throughput::throughput;
