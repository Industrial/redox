//! `redox-driver` — orchestration: the `blake3` query cache, parallel
//! scheduling, and rustc diagnostic remapping.
//!
//! Declared empty by plan 01 leaf 1. Filled by **plan 10**
//! (`incremental-cache`), which lifts `emerald-driver/src/cache.rs` — the
//! highest-value code reuse in the project (doc 02 §7.1), including the
//! trick of hashing the compiler's own binary into every cache key.
//!
//! Layout, when this crate gains constructs, follows
//! `docs/06 - Redox Code Architecture.md`: one construct per file, modules
//! private, public items re-exported from here.
