# Project Conventions

Repo-level conventions for agents working in this codebase. The harness pointer surface
lives in `.maestro/AGENTS.md`; this file holds code conventions, build commands, and
feature boundaries.

## Build / test / verify

Run these through Moon, from inside the devenv shell:

```bash
moon run :check          # cargo check --all-features --all-targets
moon run :build          # cargo build
moon run :test           # cargo nextest run, then the doctests
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :format         # treefmt (Rust, Nix, shell, JS/TS, YAML, TOML)
moon run :audit :coverage :docs :check-docs
```

**`:check-docs` is red, and has been for a long time.** Measured 2026-11-10:
**495** `missing_docs_in_private_items` errors across eleven crates — 121 in
`redox-parser`, 98 in `redox-emit`, 56 in `redox-driver`, 54 in `redox-fmt`,
46 in `redox-ast`, 42 in `redox-lexer`, then the rest. The task opts into the
lint deliberately (`-W clippy::missing_docs_in_private_items` under
`-D warnings`), so this is a convention the project chose and then stopped
running: `:check-docs` is in no gate, and nothing else runs it.

`redox-diag` was brought to zero at plan 56 — 23 module declarations, six
struct fields and one associated function — because its declaration lists are
the first thing a reader of that crate meets, and a grouping module with no
doc comment contradicts the layout rule below.

**The remaining 468 are a decision, not a chore.** Filling them thinly would
be worse than leaving them: most are private fields and helpers whose names
already say what they are, and "match the surrounding comment density" is a
rule in this file. The two honest options are to document them properly, crate
by crate, or to stop listing `:check-docs` as a command. Picking one is owed;
until then this paragraph is why the target fails.

```bash

**Moon runs cargo. blaze is for development.** That is the standard: blaze is
the inner loop you drive by hand while writing a feature, and cargo is what
moon and CI run to reach a verdict. The `:cargo-check` and `:cargo-test`
targets that used to exist to double-check blaze are gone — `:check` and
`:test` *are* rustc's answer now.

### The Rust targets are per-crate, and cached

`:check`, `:build`, `:test` and `:lint` are defined **once** in
`.moon/tasks/tag-crate.yml` and inherited by each crate through its `tags: ['crate']`. So
`moon run :check` is eleven `cargo check -p <crate>` runs, not one `--workspace` run, and
moon skips the crates whose inputs have not changed. Only workspace-wide work —
`:format`, `:ci-format`, `:audit`, `:coverage`, `:docs`, `:check-docs`, `:fix`, `:bench` —
still lives on the root project in `moon.yml`.

They all used to be `blaze … --workspace` with `cache: false`, which meant a one-line edit
re-checked, re-built and re-tested all eleven crates on every commit with nothing ever
skipped, and blaze needs 12–16 GB to hold the whole workspace at
`--all-features --cfg-test --dev-deps`. The journal records 35 OOM kills of blaze-engine in
one day. Measured, cold then repeated, on the full push gate:

| | `blaze --workspace`, uncached | per-crate blaze, cached | **per-crate cargo, cached** |
|---|---|---|---|
| cold full gate | 12–16 GB, OOM-prone | 203s, 22.6 GB peak | **24s, 0.4 GB peak** |
| no-op gate | the whole workspace again | 1.3s, 67/68 cached | **0.6s, 45/46 cached** |
| one leaf crate edited | the whole workspace | that crate alone | **that crate alone** |
| a universal dependency edited | the whole workspace | all 11, correctly | **all 11, correctly** |

The middle column is why the standard is what it is. Scoping blaze per crate made its
*cold* residency worse, not better — a cold `moon run :check` left 6 daemon workers holding
7.4 GB where one `blaze check --workspace` leaves 3 holding 3.3 GB, because the daemon is
designed to hold one big graph and this asked it to hold eleven small ones. cargo holds
nothing between invocations, so the same per-crate caching costs a fiftieth of the memory.
blaze keeps its advantage exactly where it is still used: a warm inner loop driven by hand.

**Two things keep this safe, and both are easy to break.**

`deps: ['^:check']` is **not** what makes the caching correct. In moon 2.5.0 a task's `deps`
set the order tasks run in, not the hash they are cached under: editing
`crates/redox-diag/src/lib.rs` — which every other crate depends on — re-ran redox-diag
alone and reported the other ten cached. What closes that hole is the `dependency_sources`
file group, which each crate's `moon.yml` overrides with its **transitive** dependencies so
those files are real inputs to its hash. `crates/redox-cli/tests/moon_project_graph.rs`
fails if that list drifts from `Cargo.toml`.

`dependsOn` is deliberately absent from every crate's `moon.yml`: moon's rust toolchain reads
the edges out of `Cargo.toml` itself (`moon query projects` reports them with
`source: "implicit"`), so declaring them would be a second copy of the graph free to drift.

Two smaller traps, both already paid for:

- **`.moon/tasks/tag-crate.yml` is applied to every project, not only the tagged ones**, in
  moon 2.5.0 — `moon task project-template:check` resolved and reported the per-crate
  description. The root therefore opts out by name under `workspace.inheritedTasks.exclude`,
  or `moon run :check` would also try `blaze check -p project-template`.
- **`crates/.gitkeep` made moon treat `crates/` itself as a twelfth project** (toolchain
  `system`, no manifest), which then inherited the per-crate tasks. It is gone, and
  `.moon/workspace.yml` now spells the project sources out instead of globbing.

### blaze is the development loop, by hand

[blaze](https://ix.dev/blaze) is pinned by `cursor.features.program-blaze` in
`devenv.nix`. It keeps the workspace compiled in a per-user daemon and rebuilds
only what changed, so `build`, `check`, `run` and `test` are roughly an order of
magnitude faster than cargo on an incremental edit.

**Use it while developing a feature, typing it yourself:**

```bash
blaze check          # the fast inner-loop verdict while editing
blaze test           # unit, integration and doctests in one command
blaze run -p redox-cli -- build examples/hello.sdx
```

**Moon does not invoke it, and neither does CI.** That split is deliberate and
measured: the daemon's speed comes from holding the build graph resident, and
asking it for one crate at a time makes it hold several graphs instead of one —
15 orphaned workers on 23.3 GB in a single session, and a cold gate at 22.6 GB
against cargo's 0.4 GB for the same work. The table above has the numbers.
blaze is worth that cost when a human is iterating in seconds; it is not worth
it for a gate that wants one answer and then exits.

So: `blaze` by hand, `moon run :check` / `:test` / `:lint` for a verdict. If
`blaze` and `moon` ever disagree, moon is cargo and cargo is rustc, so moon
wins.

Two things to know before reaching for it:

- **blaze is not a cargo wrapper.** It ships its own toolchain and never runs
  rustc, cargo, clippy or rustfmt. It has no `clippy`, `fmt`, `audit`,
  `nextest`, `llvm-cov`, `add` or `update` subcommand — an unknown subcommand
  exits 101 rather than falling back. `blaze doc` panics as of 0.20261030.1.
  Everything in that list stays cargo's, which is why the toolchain is still
  installed.
- **rustc is still the final word.** blaze's verdict is its own front end's and
  not yet all of rustc's — which is why **moon runs cargo**, not blaze, and why
  `:check` and `:test` are rustc's answer rather than blaze's. There are no
  `:cargo-check` / `:cargo-test` targets any more; the paragraph above the
  command list says so, and this bullet used to contradict it.

  Corrected 2026-11-10, after `moon run :cargo-test` reported **success having
  run nothing**: `crates/redox-cli/moon.yml` still carried an inputs-only
  `cargo-test` override of a task that no longer existed, which moon resolves
  to `Command: noop`. A target that passes without running is worse than a
  target that is missing — `moon run :cargo-check` at least said *"No tasks
  found"*. Both are gone.

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
cargo run -p redox-cli -- check examples/hello.sdx   # run rustc, report against the .sdx
```

**Three blessable corpora**, each asserting a different thing about a program. There is no
`--test golden` target — the goldens are checked by `examples.rs`, which is where plan 08
found this command had been wrong since it was written:

```bash
REDOX_BLESS=1 cargo test -p redox-cli --test examples  # tests/golden/*.xrs — the text
REDOX_BLESS=1 cargo test -p redox-cli --test ui        # tests/ui/*.stderr — the diagnostic
REDOX_BLESS=1 cargo test -p redox-cli --test run       # *.stdout — what it prints
```

Blessing rewrites the expectations and then fails the run, so a blessed diff is always
reviewed rather than absorbed into a green build.

A run expectation lives **beside its program**: `examples/phase1.sdx` has
`examples/phase1.stdout`, and `tests/run/` holds cases too narrow to be an example. Never
write an expectation as a string literal in a Rust test — `no_literal_expectations.rs`
fails if you do, and plan 08 §10.2 records the two places it had already happened.

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

- **`SIGKILL` during blaze's *"load workspace"* means the machine ran out of memory, not
  that anything is wrong with the code.** Seen for real at plan 07: `could not build
  <every crate in the workspace>: the daemon could not load the workspace (signal: 9
  (SIGKILL): load workspace: 3.9s)`, while `cargo check`, `cargo test`, `clippy` and
  `fmt` were all green and `moon run :cargo-check` and `:cargo-test` passed. The box had
  6 GB available of 61, consumed by a runaway process tree in an unrelated repository.
  Clearing it made the identical gate pass unchanged.

  So: a blaze failure naming *every* crate, with `SIGKILL` and no diagnostic, is an
  environment report. **Run `free -h` before reading the diff**, and distinguish it from
  the stale-state case above, which names one crate and fails at link time.

- **A gate that exits 1 with no task reporting a failure is blaze's worker running out of
  tmpfs.** This is the mechanism behind the intermittent failure plan 23 §10.8, doc 30 §8.7
  and plan 43 §10.3 each recorded as *unexplained*. Found 2026-11-09 by not filtering
  blaze's output:

  ```
  error: could not build `redox_ast`, `redox`, … `recovery`: the daemon's worker ended
  (exit status: 1: Error: backend: writing the object: No space left on device (os error 28))
  ```

  blaze's worker cache lives under **`/run/user/1000/blaze`**, which is a **6.2 GB tmpfs** —
  RAM, not disk, and far smaller than the 1.9 TB root the 15 GB `target/` sits on. A build
  broad enough to write past it kills the worker mid-run.

  Everything that made it look inexplicable follows from that:

  - it names *every* crate, like the OOM case above, because the worker died rather than any
    crate failing;
  - an immediate re-run often passes, because the cache was reclaimed in between;
  - **each target passes when run alone**, because one target builds less and fits, which is
    exactly the observation that made it look like it could not be reproduced;
  - sometimes one re-run is not enough (doc 30 §8.7), because the reclaim is not instant.

  So: **`df -h /run/user/1000` belongs beside `free -h`** when a gate fails with no named
  task. Clear it — `blaze stop` releases the worker, and its cache directory can be removed —
  then re-run. And read blaze's **last** line rather than grepping it: the diagnosis was in
  the output the whole time, and three plans filtered it out while writing down that there was
  no diagnosis.

  **Correction, 2026-11-09: `/run/user/1000` was only half of it, and the smaller half.**
  `TMPDIR` arrived **unset** in this shell, so blaze, cargo and rustc wrote their scratch
  objects to **`/tmp`, a 2.0 GB tmpfs** — found 90% full with 216 MB free, 1.25 GB of it
  Claude Code session scratchpads under `/tmp/claude-1000`. That is the same `os error 28`,
  on a tmpfs a third the size, and because tmpfs is RAM every object written there also
  counted against the memory that was OOM-killing blaze-engine.

  `devenv.nix` now exports `TMPDIR="$HOME/.cache/redox/tmp"`, on the 630 GB `/home`. Two
  things about that line are load-bearing:

  - it is in `enterShell`, **not** in `env`, because devenv clears `TMPDIR` after applying
    `env` and setting it there leaves it empty;
  - it must stay **outside the repository**. Pointing it at `$DEVENV_STATE/tmp` put it back
    inside the tree, `crates/redox-cli/tests/support.rs` then formatted its generated Rust
    somewhere that inherits this repo's `rustfmt.toml` (`tab_spaces = 2`) rather than
    rustfmt's default four, and `generated_rust_is_rustfmt_clean` failed on every example —
    the same false finding plan 04's probe recorded.

- **blaze's workers accumulate during a session, and `blaze stop` is what reclaims them.**
  Measured 2026-11-09: **15 orphaned `blaze-engine` processes holding 23.3 GB**, one per
  (crate, flag-set) pair the daemon had been asked for. `blaze stop` took all 15 to zero.

  This is the cost of the per-crate build: a cold `moon run :check` across eleven crates
  leaves **6 workers / 7.4 GB** where `blaze check --workspace` leaves **3 / 3.3 GB**. The
  caching is what pays for it — an untouched crate spawns no worker at all, and a fully
  cached push gate runs in 1.3s and starts no toolchain process whatsoever.

  The daemon self-reaps after 30 minutes idle, so this only bites inside an active session.
  When `free -h` looks wrong mid-session, count the workers before believing anything else:

  ```bash
  pgrep -x blaze-engine | wc -l
  ps -o rss= -C blaze-engine | awk '{s+=$1} END {printf "%.1f GB\n", s/1048576}'
  ```

- **`cargo::rustc-env` from a build script is a *compile-time* value — `env!`, not
  `std::env::var`.** `redox-bench`'s `rustc_path()` read its baked `REDOX_RUSTC` with
  `std::env::var`, which always missed, leaving `PATH`'s `stub-ld` as the only candidate. All
  four of its tests failed under `blaze test -p redox-bench` with *"no rustc answered
  `--version` like rustc"* while passing under `cargo nextest`, where `PATH` happens to hold
  a real rustc and hid the mistake for as long as nobody ran the blaze path.

- **A single `watch` test failing at exactly 20 seconds is the load-dependent flake, not your
  change.** `crates/redox-cli/tests/watch.rs` polls the generated `.xrs` for expected content
  with a 20-second `CEILING`, and under the **full** `redox-cli` suite one of its six tests
  intermittently exhausts it. Measured 2026-11-10 while doing plan 50 leaf 1:

  - the failing test **passes alone in ~0.1 s**, and fails only in the full run;
  - it is a **different test each time** — `it_survives_the_file_being_replaced`,
    `a_compile_error_does_not_end_the_loop`, `it_builds_at_start_and_on_change` — which is
    what tells it from a real regression in any one of them;
  - it reproduces **at HEAD with the change stashed**, which is the experiment that settles
    authorship. Two of three runs pass either way.

  It is **not** inotify exhaustion: `max_user_instances` and `max_user_watches` are both
  524288 here with effectively none in use, and that was the first hypothesis. An earlier one
  — that a `PATH` scan for rustc had become expensive — was also measured and false: 264
  `PATH` entries, 2 of them holding a `rustc`, 7.7 ms to resolve.

  So: **run the failing test alone before believing it.** If it passes in a tenth of a second,
  you have this, and the diff is not the cause. Distinguish it from the two cases below, which
  name *every* crate rather than one test.

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
