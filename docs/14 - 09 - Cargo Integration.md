# Plan 09 — Cargo Integration

Plan ID: `cargo-integration`
Depends on: 06 (`source-map`), 07 (`rustc-remapping`)
Milestone: 2 — *rustc's errors land on the user's source*
Status: `done`

---

## 1. What this plan is

The answer to **"what does a Voltaic project look like?"** — doc 02 §6.6 calls it unresolved
and says it *"must be closed before Phase 2, because it determines what a Voltaic project
even looks like."*

Doc 02 offers three options and recommends (a): a user's crate has a `build.rs` that calls
Voltaic over `src/**/*.sdx`, writes `.rs` into `OUT_DIR`, and a thin `mod` wrapper brings it
in with `include!`. This plan implements (a) and proves it by building and running a real
crate.

Three properties make (a) the right call, and all three are asserted rather than claimed:

1. **Pure Cargo.** No new tool, no build-system replacement — doc 01 §1.3 forbids one.
   Cargo already knows how to rerun a build script when inputs change.
2. **`.xrs` stays the canonical human-facing artifact.** `voltaic build` writes it for
   reading and review; the build path writes `.rs` into `OUT_DIR` because that is the
   extension rustc accepts. The pun survives without fighting the toolchain.
3. **`include!` preserves the path to the generated file**, so the span rustc reports is
   one a map can translate. Doc 02 states this more strongly than it holds — the map is
   written and correct, and plain `cargo build` never reads it. §2.2 measures exactly what
   a user sees, and criterion 6 says what is actually delivered.

### 1.1 Where the API lives, and why not a new crate

In `voltaic-driver`, whose own module doc already calls it *"orchestration"*. A build script
is an orchestration client: "compile this set of files into that directory." Plan 10 then
adds the `blake3` cache behind the same function, and no caller changes.

The alternative — a `voltaic-build` crate — would add a crate boundary to express a
difference in *audience* rather than in *layer*, and doc 06 §3 orders crates by layer.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `voltaic-driver` is empty and declared for this kind of work | `crates/voltaic-driver/src/lib.rs` | Module doc names orchestration; no constructs |
| The pieces a build script needs are already public | `voltaic_parser::parse_source`, `voltaic_emit::emit_with_map`, `voltaic_diag::render` | All `pub` |
| `include!` is usable for this | Rust reference | `include!(concat!(env!("OUT_DIR"), "/x.rs"))` is the standard idiom |
| A nested crate needs its own workspace | Cargo | An empty `[workspace]` table in its manifest detaches it from the parent |

### 2.1 The constraint that shaped the design

A build script **cannot** use `voltaic-cli`: Cargo build scripts depend on *libraries*, and
shelling out to a binary the user may not have installed would make a Voltaic crate depend
on a globally-installed tool. So the API has to be a library function, and `voltaic build`'s
own code must not be where the logic lives.

That is why `build_command` is left alone and the new `compile_dir` is not layered on top
of it — the CLI depends on the driver, not the reverse.

### 2.2 The claim that did not survive measurement

Doc 02 §6.6 says of option (a) that *"`include!` preserves spans to the generated file so
§6.5's map still works."* Half of that is true and the other half needs a qualifier, and
the difference matters to a user.

Measured: a deliberate type error in `src/greeting.sdx`, built with plain `cargo build`,
reports

```text
error[E0277]: cannot multiply `i32` by `&str`
  --> target/debug/build/cargo-project-<hash>/out/greeting.rs:3:7
```

The span does reach the generated file, exactly as doc 02 says. But **nothing consults the
map**, because Cargo invokes rustc directly and there is no hook between them. The map is
written and correct and unread.

So the honest position, and criterion 6 is rewritten to match it:

- `voltaic check` reports against the `.sdx`. That is the authoring workflow, it is plan 07's
  machinery, and it works on a project laid out this way.
- `cargo build` reports against `OUT_DIR`. The file it names is real and openable, with the
  map sitting beside it, but the user is reading generated Rust.

Closing that gap needs something between Cargo and rustc — a `RUSTC_WRAPPER`, or doc 02
§6.6's rejected option (c). Both are larger than this plan and one is forbidden by doc 01
§1.3. Recorded in §6 as the open question it is, rather than claimed as solved.

---

## 3. Concrete proof this plan targets

Doc 02 §6.6's standard is that the design "determines what a Voltaic project even looks
like", so the proof is a project:

**A real crate in the repository, with its own `Cargo.toml`, `build.rs`, and a `.sdx`
source, which `cargo run` builds and executes — asserted by running it.** Not a unit test
of the API: an actual cargo build, because the claim is about Cargo.

Plus the property that makes the layout worth choosing: a **type error in the project's
`.sdx` surfaces against the `.sdx` under `voltaic check`**. Not under plain `cargo build` —
§2.2 measured that and says why.

---

## 4. Leaves

### Leaf 1 — `leaf-compile-dir`

`voltaic-driver/src/compile_dir.rs` and children — named for its construct, which doc 06
§2.3 requires and the architecture test caught (§10.3). One entry point:

```rust
pub fn compile_dir(src: &Path, out: &Path) -> Result<Compiled, Vec<Diagnostic>>
```

Walks `src` for `.sdx`, writes `<stem>.rs` into `out` plus the `.xrs.map` sidecar, and
returns what was written. Errors come back as `Vec<Diagnostic>` — the type plan 05
defined — so a build script can render them with `voltaic_diag::render` and Cargo shows
them.

### Leaf 2 — `leaf-rerun-directives`

`Compiled` carries the input paths so a build script can emit
`cargo::rerun-if-changed=<path>` for each. Without this Cargo reruns the script only when
the crate's own files change, and editing a `.sdx` would not rebuild — a silent staleness
bug, which is exactly what doc 02 §6.6 warns option (b) has.

### Leaf 3 — `leaf-example-crate`

`examples/cargo-project/`: a worked project, detached from the workspace, with
`build.rs`, `src/main.rs` holding only the `include!` wrapper, and `src/greeting.sdx`.
This is the artifact a user copies, so it is written to be read.

### Leaf 4 — `leaf-proof`

`voltaic-cli/tests/cargo_project.rs`: `cargo run` the example crate and assert its output,
read back what Cargo wrote into `OUT_DIR`, and assert `voltaic check` names the `.sdx` for a
type error in it (§2.2 for why `cargo build` does not).

### Leaf 5 — `leaf-tests-and-gate`

Gate, doc 04 status, doc 06 §10 row, and AGENTS.md's layout section gains the project
shape.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | `compile_dir` turns a directory of `.sdx` into `.rs` in `OUT_DIR`, with maps |
| 2 | A lex or parse error comes back as `Vec<Diagnostic>`, renderable by `voltaic_diag::render` |
| 3 | `Compiled` names every input, so a build script can emit `rerun-if-changed` |
| 4 | `examples/cargo-project/` builds and runs under plain `cargo run`, asserted by a test |
| 5 | Editing the `.sdx` rebuilds — the rerun directives are emitted and correct |
| 6 | `voltaic check` reports a type error in a project's `.sdx` against the `.sdx`; plain `cargo build` reports against `OUT_DIR`, and that is disclosed (§2.2) |
| 7 | `voltaic build` still writes `.xrs`, unchanged |
| 8 | No new third-party dependency |
| 9 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Caching | **plan 10** | `compile_dir` recompiles every `.sdx` every time the script runs. Cargo's own rerun logic limits how often that is, and plan 10 puts a `blake3` cache behind the same signature |
| Nested directories | — | `compile_dir` is one level. Module support is what would make nesting meaningful, and that is a later plan |
| Mixed `.sdx`/`.rs` in one module tree | partly here | A crate can hold both — `main.rs` is ordinary Rust and the `.sdx` arrives by `include!`. What is *not* supported is a `.sdx` declaring `mod` over a `.rs` file |
| Remapping under plain `cargo build` | open | §2.2. Needs a `RUSTC_WRAPPER` or option (c); the first is a plan of its own and the second is forbidden by doc 01 §1.3. `voltaic check` is the workflow until then |
| `export` → `pub`, so a `.sdx` can fill a `mod` | **plan 24** | Doc 03 §9.6 specifies it and nothing implements it, so a `.sdx`'s items are private and the example includes at the crate root. Found by building the project |
| `cargo voltaic` | **rejected** | Doc 02 §6.6 option (c); doc 01 §1.3 forbids replacing the build system |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

---

## 8. Emerald reuse

**Not applicable, and worth saying why.** Emerald compiles to object files and links them;
it has no analogue of generating source for another compiler to consume. Its
`emerald-driver/build.rs` parses Cargo's artifact stream, which plan 07 already borrowed
for a different purpose. This plan is one of the few where being a transpiler makes the
problem *different in kind* rather than merely easier.

---

## 9. Decision log

- **Option (a), as doc 02 recommended**, with its three properties asserted rather than
  assumed (§1).
- **The API lives in `voltaic-driver`, not a new crate** (§1.1): a build script differs from
  the CLI in audience, not in layer.
- **A build script may not shell out to `voltaic`** (§2.1). A Voltaic crate must build with
  `cargo build` and a path dependency, not with a globally-installed binary.
- **`compile_dir` returns `Vec<Diagnostic>`, not a formatted string.** The caller chooses
  the renderer; a build script wants `render(.., plain: true)` because Cargo's output is
  not a terminal.
- **The example crate is detached from the workspace** with an empty `[workspace]` table.
  Including it as a member would make the parent workspace's `cargo check` build it with
  the parent's lockfile and profile, and the point is that it builds the way a user's crate
  does.
- **`.rs` into `OUT_DIR`, `.xrs` for humans** — doc 02 §6.6's own resolution, and the
  reason the `.xrs` convention survives.

---

## 10. Result

`done`. 263 tests pass; `fmt`, `clippy -D warnings`, `cargo audit` and the full gate are
green.

A Voltaic project looks like this, and it is in the repository:

```text
examples/cargo-project/
  Cargo.toml          [build-dependencies] voltaic-driver, voltaic-diag; empty [workspace]
  build.rs            compile_dir("src", OUT_DIR) + rerun directives — 30 lines
  src/greeting.sdx    Voltaic
  src/main.rs         include!(concat!(env!("OUT_DIR"), "/greeting.rs")) — plain Rust
```

`cargo run` prints `hello, world` and `42`. No new tool, no build-system replacement.

| # | Criterion | Evidence |
|---|---|---|
| 1 | `.rs` plus maps into `OUT_DIR` | `cargo_project.rs`: `the_build_script_writes_rust_and_a_map_into_out_dir`, which reads what Cargo actually produced and parses the map |
| 2 | Errors as renderable `Vec<Diagnostic>` | `a_broken_source_returns_renderable_diagnostics` |
| 3 | `Compiled` names every input | `the_build_script_asks_cargo_to_watch_the_sdx` |
| 4 | Builds and runs under plain Cargo | `the_example_project_builds_and_runs`, asserting exact stdout |
| 5 | Editing a `.sdx` rebuilds | criterion 3's directives, plus `cargo::rerun-if-changed=src` for additions |
| 6 | `voltaic check` against the `.sdx`; `cargo build` against `OUT_DIR` | `voltaic_check_reports_a_project_error_against_the_sdx`, and §2.2 for the limit |
| 7 | `voltaic build` unchanged | plan 01–08's tests, untouched |
| 8 | No new dependency | `Cargo.lock` unchanged |
| 9 | Full gate green | `moon run :check :lint :test :cargo-check :cargo-test` |

### 10.1 What building a real project found

Three things, none of which a unit test of `compile_dir` would have reached. This is the
argument for doc 02 §6.6's own standard — that the answer "determines what a Voltaic project
even looks like", so the proof has to be a project.

**Voltaic cannot yet produce a reachable item inside a module.** The natural shape is
`mod greeting { include!(…) }`, and it fails: Voltaic emits `fn greet(…)` with no
visibility, so every item is private to that module. Doc 03 §9.6 already specifies the fix
— `export` lowers to `pub`, `export(crate)` to `pub(crate)` — and **plan 24** owns it. The
example includes at the crate root instead, where private items are still crate-visible,
with the reason written in `main.rs` where someone copying it will read it.

This is a *language* gap found by an *integration* plan, and it is the single most useful
thing this plan produced: a transpiler that cannot fill a module is not yet usable for
library work, and nothing before now had tried.

**Doc 02's span claim is half true** (§2.2). `include!` does preserve the path to the
generated file, so rustc's span is one a map can translate — but Cargo invokes rustc
directly, so **nothing reads the map**, and plain `cargo build` reports
`out/greeting.rs:3:7`. Measured, not assumed. Criterion 6 was rewritten to say what is
delivered: `voltaic check` remaps, `cargo build` does not, closing the gap needs a
`RUSTC_WRAPPER` or the option doc 01 §1.3 forbids.

**A fourth toolchain shim.** `env!("CARGO")` under `blaze test` is
`~/.blaze/toolchains/…/bin/cargo`, a generic-linux binary NixOS refuses to run — the same
`stub-ld` class as plan 01's rustc and plan 03's two blaze shims. Worse, once a working
cargo was found, the nested build still failed **with empty stderr**, because it inherited
`RUSTC` and `CARGO_*` pointing into blaze's toolchain.

Both fixes follow the rule AGENTS.md already records: `support::cargo_path` checks the
`--version` banner rather than trusting the name, and the nested build starts from an
environment with every `CARGO*`/`RUST*` variable removed. The second is also more
*correct* and not merely more robust: a test claiming the project "builds the way a user's
crate does" must not hand the child the outer build's environment.

### 10.2 One deduplication, done before the duplicate existed

`diagnose` was in the CLI, where plan 05 put it because the CLI was its only caller. This
plan gave it a second caller that must not depend on the CLI (§2.1), so it moved **down**
into `voltaic-driver` — above the stages, below the CLI — rather than being copied. Doc 06
§3 rule 6 applied in advance instead of after the two copies drifted, which is the only
time applying it is cheap.

### 10.3 Three layout corrections from the architecture tests

The checks named all three; none was visible while writing:

- `compile.rs` declared `compile_dir`, which §2.3 permits only for a file named
  `compile_dir.rs`. Renamed, with its directory.
- `diagnose.rs` sat at `src/` root with no children, failing §2.1. `io_diagnostic` moved
  from `compile/compile_file/` to `diagnose/`, which groups the two `Diagnostic`
  constructors together *and* makes `diagnose.rs` a legitimate module head *and* removes a
  directory level. One move, three rules satisfied — the sign that the grouping was right
  rather than merely legal.
- `compile_file.rs` declared `io_diagnostic` as a second construct, resolved by the same
  move.

### 10.4 Disclosures

- **`compile_dir` is one level deep** and recompiles everything each run. Nesting needs
  module support to mean anything; caching is plan 10, behind this signature.
- **A build script cannot report against the `.sdx` under `cargo build`** (§2.2). Authors
  should run `voltaic check`.
- **The example project builds in place**, so `examples/*/target/` and
  `examples/*/Cargo.lock` are gitignored. Making it a workspace member would have given it
  the parent's lockfile and profile, which is precisely what the plan needs it not to have.
- **`.sdx` declaring `mod` over a `.rs` file is not supported**, and is not the same
  question as plan 24's `export`. A crate *can* mix both kinds of source; what it cannot do
  is have the Voltaic side own the module tree.
