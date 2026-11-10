//! The content-addressed compile cache.
//!
//! Plan 10 (`docs/15 - 10 - Incremental Cache.md`). Doc 02 §7.1 calls
//! `emerald-driver/src/cache.rs` *"the highest-value code reuse in the
//! project"*, and the rule it quotes governs everything in here:
//!
//! > A cache MISS always falls back to calling the real function verbatim — a
//! > cache hit is strictly an optimization, never a correctness dependency.
//!
//! So nothing in this module returns an error. A key that cannot be computed,
//! an entry that cannot be read, a directory that cannot be written: each is a
//! miss, and a miss costs only time. Delete `<OUT_DIR>/.redox-cache/` and the
//! compiler behaves identically.
//!
//! A grouping module (doc 06 §1.2): it declares the cache's parts and holds no
//! construct of its own, because there is no single "cache" object — the key,
//! the fingerprint, the store and the reporter are four independent concerns
//! that `compile_dir` composes.

/// The cache key for one source file.
pub(crate) mod cache_key;
/// A hash of the compiler that produced a cached result.
pub(crate) mod fingerprint;
/// Where the cache reports hits and misses.
pub(crate) mod reporter;
/// The on-disk cache under `<out>/.redox-cache/`.
pub(crate) mod store;
