# Project Conventions

Repo-level conventions for agents working in this codebase. The harness pointer surface
lives in `.maestro/AGENTS.md`; this file holds code conventions, build commands, and
feature boundaries.

## Build / test / verify

Run these through Moon, from inside the devenv shell:

```bash
moon run :check          # blaze check  — the fast inner-loop verdict
moon run :build          # blaze build
moon run :test           # blaze test   (unit, integration, doctests)
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :format         # treefmt (Rust, Nix, shell, JS/TS, YAML, TOML)
moon run :cargo-check    # cargo check  — rustc's verdict, runs in the push gate
moon run :cargo-test     # cargo nextest run — likewise
moon run :audit :coverage :docs :check-docs
```

### Rust goes through blaze

[blaze](https://ix.dev/blaze) is the default Rust build command here, pinned by
`cursor.features.program-blaze` in `devenv.nix`. It keeps the workspace
compiled in a per-user daemon and rebuilds only what changed, so `build`,
`check`, `run` and `test` are roughly an order of magnitude faster than cargo on
an incremental edit. Use `blaze` wherever you would have typed `cargo`, for
those four commands.

Two things to know before reaching for it:

- **blaze is not a cargo wrapper.** It ships its own toolchain and never runs
  rustc, cargo, clippy or rustfmt. It has no `clippy`, `fmt`, `audit`,
  `nextest`, `llvm-cov`, `add` or `update` subcommand — an unknown subcommand
  exits 101 rather than falling back. `blaze doc` panics as of 0.20261030.1.
  Everything in that list stays cargo's, which is why the toolchain is still
  installed.
- **rustc is still the final word.** blaze's verdict is its own front end's and
  not yet all of rustc's, so the pre-push gate runs `:cargo-check` and
  `:cargo-test` as well. Do not drop them to make a push faster.

`blaze test` reports a cached pass for a test whose inputs have not changed
(`Cached N tests from an earlier run`); `BLAZE_TESTS=all blaze test` forces every
test to run.

`blaze status` shows the daemon and what it holds; `blaze stop` ends it;
`blaze-doctor` prints version, daemon and cache paths. Do not add
`[workspace.metadata.blaze] version` to `Cargo.toml` — blaze would download that
release itself and run it instead of the Nix-pinned one.

## Layout

Redox is a source-to-source compiler: `.sdx` in, `.xrs` containing ordinary Rust out. The
workspace splits along pipeline stages (`docs/02 - Redox Update Blueprint.md` §5.4).

- `crates/redox-lexer/` — off-side-rule lexer; emits spans, trivia and whitespace adjacency
- `crates/redox-ast/` — syntax tree types, no logic
- `crates/redox-parser/` — hand-written recursive descent (see `docs/COMPILER.md` §1)
- `crates/redox-emit/` — Rust source emitter
- `crates/redox-cli/` — the `redox` binary, plus the end-to-end and golden test harnesses
- `crates/redox-diag/`, `redox-driver/`, `redox-fmt/` — declared empty; each crate's module
  doc names the plan that fills it (plans 05, 10, 34)
- `examples/*.sdx` — the CI-checked source of truth for what works today
- `tests/golden/` — `.sdx` beside its expected `.xrs`, compared as exact text
- `docs/` — numbered: 01 specification, 02 blueprint, 03 surface design, 04 plan of plans,
  `05 - NN -` one per plan, 06 code architecture; plus `COMPILER.md` for toolchain decisions
- `.maestro/` — harness state (read `.maestro/AGENTS.md` first)

Within a crate, `src/` is flat and every file is one construct — see the first bullet under
*Conventions that are load-bearing* and `docs/06 - Redox Code Architecture.md`.

### Conventions that are load-bearing, not stylistic

- **One construct per file.** Every `struct`, `enum`, `union`, `trait`, `type`, `fn`,
  `const` and `static` lives in its own file. `impl` blocks travel with their type; `lib.rs`
  and `main.rs` hold only `mod` declarations, `pub use` re-exports and `fn main`.
- **`src/` is a module tree, not a flat directory.** Every file directly under `src/` must
  be the module root, a **module head** (a construct with a sibling `<name>/` holding its
  children), or a **grouping module** (child declarations and a doc comment, no construct).
  Everything else lives at least one level down, at most three. Directories are named for
  domain concepts — never `types/`, `helpers/`, `utils/`, `enums/`, `traits/` or any other
  bucket, at either level.
- **Naming is path-aware.** A construct's `snake_case` name equals the file stem, or the
  parent directory joined to the stem by `_`. So `token/kind.rs` holds `TokenKind` —
  `token/token_kind.rs` stutters and is rejected.
- All three are **machine-checked** by `crates/redox-cli/tests/one_construct_per_file.rs`
  and `crates/redox-cli/tests/directory_structure.rs`. Full rules and rationale:
  `docs/06 - Redox Code Architecture.md`.
- **Public-API tests are integration tests** under `crates/<crate>/tests/`, not
  `#[cfg(test)]` modules. An inline test can reach private internals and eventually will.
  Inline tests are for private constructs only.
- **Every `.sdx` fixture is a file, and tests reference files.** Never embed language source
  as a string literal in a Rust test. Emerald's one syntax cutover missed 113 fixtures that
  lived as string literals inside test modules; that cost is paid once per syntax change,
  forever.
- **Comments are trivia, never discarded.** They must reach the generated `.xrs` — the
  `.xrs` is a human-facing artifact. Doc 02 §10.1 corrects doc 01 §4.1 on this.
- **Every AST node carries a byte span.** Emerald found a whole node type with no span at
  all twenty-one plans after its diagnostics plan shipped, by which point the fix was judged
  too large and it settled for a workaround.
- **Generated identifiers use the `__rdx_` prefix**, and user identifiers may not.
- **Every check rustc can make, rustc makes.** Redox has no type checker, no borrow checker
  and no exhaustiveness checker, and must never grow one.
- **No plan implements a library function.** Redox has crates.io. Emerald spent 112 of its
  203 plans building a standard library; that is the cost this rule avoids.
- **No single `.rs` file over 1,500 lines**, enforced by
  `crates/redox-cli/tests/line_cap.rs`.

### Working on the compiler

```bash
cargo run -p redox-cli -- build examples/hello.sdx   # transpile to examples/hello.xrs
REDOX_BLESS=1 cargo test -p redox-cli --test golden  # regenerate golden expectations
```

Blessing rewrites the expectations and then fails the run, so a blessed diff is always
reviewed rather than absorbed into a green build.

## Conventions

- Match existing code style; use established libraries before adding new ones.
- Surgical edits only — touch what the task requires.
- Bump the relevant version when behavior changes.

## Known tooling gotchas

Environment traps found by hitting them. Each one cost real time or produced a false
result, which is why they are written down rather than rediscovered.

- **Neither obvious source of a `rustc` path is trustworthy here, so verify instead.**
  Two separate shims bite, and each one looks like the other's fix:
  - resolving `rustc` from `PATH` **at test time** finds NixOS's `stub-ld` shim under
    `blaze test` (*"Could not start dynamically linked executable: rustc"*);
  - trusting `RUSTC` **at build time** finds blaze's own shim under `blaze build`, which
    refuses general work with *"`BLAZE_SCRIPT_RUSTC` does not name blaze's rustc"*.

  `crates/redox-cli/build.rs` therefore does not trust a candidate, it **runs
  `--version` and checks the banner**, taking the first that answers like real rustc, and
  bakes that path in as `REDOX_RUSTC`. Any future test invoking a toolchain binary should
  verify rather than assume — a path that exists is not a path that works.

- **A stale blaze daemon or cache produces link errors that look like code defects.** Seen
  for real: `rust-lld: error: undefined symbol: __rust_alloc_zeroed` while linking the test
  binary of a crate that has no tests and no build script. Nothing was wrong with the code.
  Fix: `blaze stop && rm -rf target/blaze`, then re-run. Suspect this before believing a
  link failure that names a crate you did not touch.

- **Do not grep compiler output for `^error` — it reads as green when it is not.** `cargo
  clippy` colourises its output, so the line begins with an ANSI escape and the pattern
  never matches. This produced a confident "clippy clean" report while two
  `collapsible_if` errors were live, caught only when the moon gate ran. **Check exit
  codes, not output text**, when deciding whether a gate passed.

## See also

- `.maestro/MAESTRO.md` — read order, lane policy, daily commands
- `.maestro/docs/HARNESS.md` — product-delta vs harness-delta model
- `.maestro/docs/FEATURE_INTAKE.md` — work-type classification decision tree
- `.maestro/docs/VALIDATION_LADDER.md` — 7-step verification protocol

<!-- maestro-setup:start -->
## Maestro

This project is wired into the Maestro harness. State and config live
under `.maestro/`. Run `./init.sh` to bring a fresh checkout up; run
`maestro doctor` and `maestro status` to see what Maestro knows.

Preserve content outside this managed block; the block is rewritten by
`maestro setup` and the `maestro-setup` skill, but everything else in
this file is yours.
<!-- maestro-setup:end -->
