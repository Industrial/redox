# Plan 10 — Incremental Cache

Plan ID: `incremental-cache`
Depends on: 09 (`cargo-integration`)
Milestone: 3 — *the compiler is fast enough to live in an edit loop*
Status: `done`

---

## 1. What this plan is

A content-addressed cache behind `compile_dir`, so a `.sdx` whose bytes have not changed is
not re-parsed and not re-emitted.

Doc 02 §7.1 calls `emerald-driver/src/cache.rs` *"the highest-value code reuse in the
project"* and names two details to steal outright. Both are stolen, and the governing
principle is quoted here because everything else follows from it:

> **A cache MISS always falls back to calling the real function verbatim — a cache hit is
> strictly an optimization, never a correctness dependency.**

That is what makes a cache safe to add at plan 10 rather than feared until plan 40. Delete
the cache directory and the compiler behaves identically, only slower. Every test in this
plan is written to hold whether or not a hit occurs.

### 1.1 The two details worth stealing

- **`blake3::Hash` as the key, with no wrapper.** Already `Copy`, `Eq`, `Hash`, and
  `Display`-as-hex, so it is a filename and a map key without ceremony.
- **The compiler's own binary is hashed into every key.** Doc 02 §7.1 states the reason:
  *"so that rebuilding the compiler invalidates every previously-cached result instead of
  silently reusing a stale `.o` a different compiler produced."* For Redox the stale artifact
  would be generated Rust from an older emitter — which compiles, runs, and is wrong. This
  is three lines and it is the difference between a cache and a trap.

### 1.2 `blake3` is the second dependency

The first was `miette` (plan 05). The rule doc 02 §5.5 sets is that a dependency needs a
written reason, so: a cache key must be a strong hash of file contents, hashing is exactly
the library function doc 02 forbids implementing, and `blake3` is what Emerald already uses
for the same job. The alternative — `std`'s `DefaultHasher` — is explicitly not
collision-resistant and is documented as unstable across releases, which for a *persisted*
key means last week's cache silently mismatches this week's.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `compile_dir` is the single entry point to put a cache behind | `redox-driver/src/compile_dir.rs` | One function, called by `build.rs` and tests |
| Plan 09 promised the signature would not change | plan 09 §1.1 | *"adds the `blake3` cache behind the same function, and no caller changes"* |
| A cache needs somewhere to live that Cargo manages | Cargo | `OUT_DIR` is per-crate, per-profile, and cleaned by `cargo clean` |
| `std::env::current_exe` is available for the fingerprint | std | Yes; fallible, so a failure must degrade to always-miss rather than panic |

---

## 3. Concrete proof this plan targets

1. **The second compile of an unchanged file is a hit, and produces byte-identical
   output.** Both halves asserted: a hit that produced different bytes would be worse than
   no cache.
2. **Changing one byte of the source is a miss.**
3. **Changing the compiler fingerprint is a miss** — §1.1's reason, asserted rather than
   trusted, by injecting a fingerprint rather than rebuilding the compiler.
4. **Deleting the cache changes nothing but speed.** The same output, from the same input,
   with the cache wiped between runs.
5. **A corrupt cache entry is a miss, not a failure.** Truncated and garbage entries both,
   because a cache that can break a build is not an optimization.

---

## 4. Leaves

### Leaf 1 — `leaf-cache-key`

`redox-driver/src/cache/cache_key.rs`. `cache_key(source: &str, fingerprint: &Fingerprint) ->
blake3::Hash`, hashing the source bytes and the fingerprint together.

### Leaf 2 — `leaf-fingerprint`

`cache/fingerprint.rs`. A newtype over `blake3::Hash` built from `current_exe`'s bytes,
with a constructor taking arbitrary bytes so tests can vary it. A read failure yields a
random-per-process value, which degrades to always-miss — the safe direction.

### Leaf 3 — `leaf-store`

`cache/store.rs`. Reads and writes entries under `<out>/.redox-cache/<hash>`. Every
failure path returns `None`, never an error: by §1's principle a store that cannot answer
is a miss.

### Leaf 4 — `leaf-reporter`

`cache/reporter.rs`. The `CacheReporter` trait doc 02 §7.1 names, reporting
`HIT`/`MISS` per file, *"with no opinion on stdout"* — a build script prints to stderr, a
test collects into a `Vec`, and the cache knows about neither.

### Leaf 5 — `leaf-wire-compile-dir`

`compile_dir` consults the store; `compile_dir_with` — at `compile_dir/with.rs`, where doc
06 §2.3's parent-plus-stem rule names it — takes a reporter. Plan 09's signature is kept
exactly, delegating with a silent reporter.

### Leaf 6 — `leaf-proof-and-gate`

`redox-cli/tests/cache.rs` for §3's five properties, the gate, and the docs.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | An unchanged file is a hit on the second compile, with byte-identical output |
| 2 | A changed source byte is a miss |
| 3 | A changed compiler fingerprint is a miss |
| 4 | With the cache wiped, output is identical — a hit is never a correctness dependency |
| 5 | A corrupt or truncated entry is a miss, not a failure |
| 6 | `compile_dir`'s signature is unchanged and `examples/cargo-project` still builds |
| 7 | `CacheReporter` has no opinion on where its output goes |
| 8 | `blake3` reaches `redox-driver` only |
| 9 | Full gate green, `cargo audit` clean with the new dependency |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Caching inside the CLI's `build`/`check` | later | `compile_dir` is the build-script path. `redox build` writes one file the user asked for, where a cache saves nothing a human would notice |
| Parallelism | later | Doc 02 §7 pairs caching with parallel scheduling. Caching alone is testable; the two together would make a failure hard to attribute |
| Cache eviction | — | `OUT_DIR` is cleaned by `cargo clean`, so the cache's lifetime is Cargo's problem. A long-lived cache outside `OUT_DIR` would need a size bound |
| A `salsa`-style query graph | **rejected** | Doc 02 §7.1 quotes Emerald's own reasoning: whole-file granularity does not need it |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**Lifted as a design, with its principle quoted:** `crates/emerald-driver/src/cache.rs` —
the content-addressed key, the compiler fingerprint in every key, the `CacheReporter`
trait, and above all the MISS-falls-back-verbatim rule (§1).

**Deliberately smaller.** Emerald's is 742 lines over three queries. Redox has one query —
"this file's bytes to this file's generated Rust" — because the transpiler's unit of work
*is* the file. Copying the three-query structure would be copying a shape without its
reason.

---

## 9. Decision log

- **A hit is an optimization, never a correctness dependency** (§1), which is why criterion
  4 asserts identical output with the cache wiped.
- **The compiler's binary is in every key** (§1.1). Without it a stale emitter's output
  survives a rebuild, compiles, and is wrong.
- **Every store failure is a miss.** No error type, no propagation: a cache that can break
  a build has negative value.
- **`compile_dir` keeps its signature**, as plan 09 promised. `compile_dir_with` adds the
  reporter for callers that want it.
- **The fingerprint is injectable.** Asserting criterion 3 by rebuilding the compiler is
  not a test anyone would run; a constructor taking bytes makes it a unit test.
- **The cache lives in `OUT_DIR`.** Per-crate, per-profile, and `cargo clean` empties it —
  so the cache inherits Cargo's lifetime rules instead of inventing its own.

---

## 10. Result

`done`. 286 tests pass; `fmt`, `clippy -D warnings`, the full gate and `cargo audit` (75
crates) are green.

The value proposition in two lines, from the worked project:

```console
$ cargo build -vv            # first build
[cache] greeting.sdx MISS
$ touch src/greeting.sdx && cargo build -vv
[cache] greeting.sdx HIT
```

Cargo reruns the build script because the **mtime** changed. The cache answers HIT because
the **bytes** did not. That is the whole difference between a content-addressed cache and
the timestamp heuristic it replaces, and it is why doc 02 §7.1 called this the
highest-value reuse in the project.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Hit on second compile, byte-identical output | `cache.rs`: `an_unchanged_file_hits_and_produces_identical_output` — both halves |
| 2 | A changed byte misses | `a_changed_source_misses`, which also asserts the *new* output appears |
| 3 | A changed compiler fingerprint misses | `a_different_compiler_invalidates_the_cache`, plus `cache_key.rs`'s unit test |
| 4 | Wiped cache, identical output | `wiping_the_cache_changes_nothing_but_speed` |
| 5 | A corrupt entry is a miss, not a failure | `a_corrupt_cache_entry_is_a_miss`; `store.rs` adds truncation, half-entry and unwritable-directory cases |
| 6 | `compile_dir`'s signature unchanged | `compile_dir_still_takes_two_paths`, and `examples/cargo-project` builds untouched |
| 7 | The reporter owns its output | `the_cache_reports_through_the_caller_not_stdout` |
| 8 | `blake3` confined to `redox-driver` | its manifest alone |
| 9 | Gate and audit green | `moon run :check :lint :test :cargo-check :cargo-test :audit` |

### 10.1 Testing an optimization honestly

An optimization is awkward to test, because the thing it changes is supposed to be
invisible. The split used here is worth naming, because it generalises:

- **Prove the behaviour is unchanged.** Criteria 1, 4 and 5 all assert *identical output* —
  warm cache, wiped cache, corrupted cache. Every one of them would pass with the cache
  deleted from the codebase, which is exactly the property plan 10 §1 claims.
- **Prove separately that work was skipped.** Criteria 1, 2 and 3 assert the HIT/MISS
  events. These are the only tests that would fail if the cache were removed.

Writing it the other way round — asserting hits and assuming correctness — is how a cache
becomes load-bearing without anyone deciding that it should.

`Outcome::was_cached` exists only to be *reported*, never acted on. No code path differs
because a result came from cache, which is what makes the rule true by construction rather
than by discipline.

### 10.2 One consequence of the key worth knowing

The key covers content and the compiler, **not the path**. So two files with identical
bytes share an entry, and the second is a hit on its *first* compile —
`identical_contents_share_an_entry_across_names` asserts it. This is correct and might
surprise a reader, which is why it is a test rather than a comment: a renamed file keeps
its cached result, and a copied file inherits one.

### 10.3 Four layout corrections

The architecture tests named every one; none was visible while writing:

- `store.rs` declared `Store` and `Cached` → `store/cached.rs`.
- `reporter.rs` declared the trait and `Silent` → `reporter/silent.rs`.
- `compile_file.rs` declared `compile_file` and `write_outputs` → `compile_file/write_outputs.rs`.
- `compile_dir_with.rs` sat at `src/` root with no children → `compile_dir/with.rs`, where
  §2.3's parent-plus-stem rule makes `compile_dir_with` the *correct* name rather than a
  permitted one. That is the second time (after plan 06's `emit/with_map.rs`) the rule has
  picked the home rather than merely allowed it.

`write_outputs` being shared is not only a rule's doing: the hit and miss paths now
*cannot* write different bytes, which is criterion 1's second half enforced structurally
rather than by test.

### 10.4 Disclosures

- **No parallelism.** Doc 02 §7 pairs caching with parallel scheduling; only caching is
  here, because a failure in the two together would be hard to attribute. Deferred with the
  reason rather than silently.
- **No caching in `redox build` or `redox check`.** The cache is on the build-script path.
  `redox build` writes one file a human asked for, where a cache saves nothing anyone would
  notice; `redox check` spends its time in rustc, not in Redox.
- **No eviction, deliberately.** The cache lives in `OUT_DIR`, so `cargo clean` empties it
  and its lifetime is Cargo's problem. A cache outside `OUT_DIR` would need a size bound,
  and that is a different plan.
- **The hit path still writes the output files.** It could check whether last run's files
  survived; a few kilobytes of writing makes the question moot and removes a way for the
  two paths to differ.
- **A fingerprint failure means always-miss.** If `current_exe` cannot be read the
  fingerprint varies per process. Slow and correct, which is the right direction for a
  cache that cannot identify its own compiler.
