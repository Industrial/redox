# `redox1.wasm` — the bootstrap seed

Plan 72 (`docs/66 - 72 - The Seed Compiler.md`) leaf 3.

This directory holds **one binary**, and it is the only binary this repository
tracks. It is `crates/redox-seed` compiled to `wasm32-wasip1`: `.sdx` in, `.rs`
out, and nothing else — no `check`, no `fmt`, no `watch`, no colour.

## What it is for

`crates/redox-ast` and `crates/redox-fmt` are **written in Redox**. The Rust
under each one's `src/` is generated from the `.sdx` beside it. Something has
to break the resulting cycle, because `redox-cli` is built from `redox-fmt`,
which is built from `redox-ast` — so on a clean clone with those trees
untracked, nothing exists that could generate them.

This blob is that something. Every self-hosting language needs one; what
differs is the form. rustc downloads the previous beta, Go requires a previous
toolchain in a documented chain, OCaml commits bytecode, and Zig commits
`zig1.wasm` — which is the model this follows, including the part that matters
most: **the seed was built from the Rust implementation before that
implementation was replaced.** Zig's C++ compiler produced the first
`zig1.wasm` and was deleted afterwards. A seed built the other way round is
verified by a compiler the seed is supposed to underwrite.

`docs/64 - 70 - The Source Beside the Crate.md` §1 is the full survey.

## Identity

| | |
|---|---|
| built from commit | `151df7a97a1789dda0b500f8db69f2e9cc0b0dd9` |
| size | 928389 bytes |
| blake3 | `635a1784b5dc6a4a80a05295bd3664cfa873870abd8165b1992a8a9b096373e3` |

`crates/redox-seed/tests/blob.rs` recomputes the hash and fails if it does not
match this table, so a hand-edited blob and a stale row are the same failure.
It is blessed, not hand-edited:

```bash
REDOX_BLESS=1 cargo test -p redox-seed --test blob
```

Blessing rewrites this table and then **fails**, which is deliberate and is how
every other expectation in this repository behaves: a changed hash is a changed
compiler and has to be looked at rather than absorbed into a green build.

## How to rebuild it

```bash
moon run :seed-refresh
```

which is exactly:

```bash
cargo build --release -p redox-seed --target wasm32-wasip1
wasm-opt -Oz --enable-bulk-memory \
  -o seed/redox1.wasm \
  target/wasm32-wasip1/release/redox-seed.wasm
```

`wasm-opt` is not optional and not cosmetic: the release build is 1,351,423
bytes and `-Oz` takes it to 913,549, so a blob built without it is a third
larger for no reason. `devenv.nix` pins both `binaryen` (for `wasm-opt`) and
`wasmtime` (to run the result).

## When to rebuild it

**Whenever the seed can no longer parse this repository's own `.sdx`.** That is
the version-skew rule, and `AGENTS.md` carries it:

> A surface row that the seed cannot parse must refresh the seed in the same
> commit.

rustc answers the same problem with `cfg(bootstrap)` double implementations and
Go with a version chain bumped about once a year. Both are expensive because
their seeds are *releases*. This one is a build artifact, so the answer is one
command and one re-committed file — and that is only cheap while the seed is
unreleased, which is also the window in which Redox's surface still moves.

Refreshing it for any other reason is churn: a new blob is ~900 KB of history
that does not delta-compress, so the rule is a trigger and not a cadence.

## Running it

```bash
wasmtime run --dir . seed/redox1.wasm crates/redox-ast/sdx <out-dir>
```

`--dir .` is required — WASI grants no filesystem access by default, so without
a preopened directory the seed cannot read a single `.sdx`.

The output is **not** what is committed until rustfmt has run on it. Redox
targets rustfmt's defaults and this repository sets `tab_spaces = 2`, so the
pipeline is seed, then the native rustfmt with the repo's config. `bootstrap.sh`
does both.
