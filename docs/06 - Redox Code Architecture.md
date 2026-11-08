# Redox — Code Architecture

Version: 1.0.0
Status: normative. Enforced by `crates/redox-cli/tests/one_construct_per_file.rs` and
`crates/redox-cli/tests/line_cap.rs`.
Applies to: every crate in the workspace, including crates that are currently empty.

This document governs how Redox's Rust source is organised. It is normative, not advisory:
the central rule is machine-checked and a violation fails the ordinary test gate.

---

## 1. The rule: one construct per file

> **Every construct lives in its own file, named after it in `snake_case`.**

A **construct** is a named, top-level item that forms part of a crate's vocabulary:

| Item kind | Example | File |
|---|---|---|
| `struct` | `Token` | `token.rs` |
| `enum` | `TokenKind` | `token_kind.rs` |
| `union` | `Payload` | `payload.rs` |
| `trait` | `Reporter` | `reporter.rs` |
| `type` alias | `Span` | `span.rs` |
| `fn` | `lex` | `lex.rs` |
| `const` / `static` | `RUST_INDENT` | `rust_indent.rs` |

**Exactly one** such item may appear at the top level of a file. This holds regardless of
visibility — a private helper function is a construct and gets its own file.

### 1.1 What travels with a construct

These are part of the construct, not separate constructs, and belong in its file:

- **`impl` blocks for the type**, inherent and trait alike. A type and its behaviour are one
  unit; splitting them is how you get a type whose invariants are enforced in three places.
- **Derive and attribute macros** on the item.
- **The construct's own tests**, as `#[cfg(test)] mod tests` — but see §6, which prefers
  integration tests for anything reachable through the public API.

Nothing else. A constant used by two functions is its own construct in its own file.

### 1.2 Module roots hold no constructs

`lib.rs` and `main.rs` are **module roots**, not construct files. They contain only:

1. the crate-level doc comment (`//!`);
2. `mod` / `pub mod` declarations;
3. `pub use` re-exports that form the crate's public surface (§4);
4. crate-level attributes (`#![forbid(unsafe_code)]`).

`main.rs` additionally holds `fn main`, which has no other legal home.

### 1.3 No `mod.rs`

Use the 2018-and-later layout: a module with children is `foo.rs` beside a `foo/`
directory, never `foo/mod.rs`. A tree full of files all named `mod.rs` is unnavigable in an
editor tab bar and in a diff.

```text
src/
  lib.rs
  token.rs          # the construct
  token/            # its children, if any
    kind.rs
```

### 1.4 File naming

The file name is the construct's name in `snake_case`, with no prefix, suffix or
abbreviation. `ParseFailure` → `parse_failure.rs`. `is_ident_start` → `is_ident_start.rs`.

Never name a file after a category — no `types.rs`, `helpers.rs`, `utils.rs`, `common.rs`,
`misc.rs`. A file whose name does not name a construct is a bucket, and buckets are what
this rule exists to prevent.

### 1.5 What this costs, stated plainly

The rule is not free, and pretending otherwise would make it easy to abandon under pressure:

- **More files.** `redox-lexer` is 12 source files where it was 1.
- **Cohesive small types get separated.** `Span`, `Trivia` and `TriviaKind` are closely
  related and now sit in three files. §4's re-exports are what keep this from leaking into
  every caller's `use` block.
- **Visibility widens slightly.** A helper that was a private `fn` in one file now needs
  `pub(crate)` to be reachable from the construct it serves. Use the narrowest visibility
  that compiles — `pub(super)` before `pub(crate)`, `pub(crate)` before `pub` — and never
  widen a helper to `pub` to satisfy the rule.

What it buys: every name has exactly one home, so finding a construct never requires
searching; two people editing two constructs never touch the same file, so the diffs cannot
conflict; and a file's length becomes a direct signal that one construct is doing too much.

### 1.6 The failure mode this prevents

Emerald — the comparable compiler audited in `docs/02 - Redox Update Blueprint.md` — has
**34,429 lines in `crates/emerald-codegen/src/lib.rs`** and **22,589 in
`emerald-sema/src/lib.rs`**, each a single file. Doc 02 §5.4 identifies this as the one clear
structural defect in an otherwise well-organised repository, and notes the contrast: its
`emerald-driver` is 2,658 lines across four named modules and is by far the most readable
crate in it.

Nothing about that outcome was a decision. It is what happens when a file has no rule
telling it when to stop growing.

---

## 2. Crate layering

Dependencies flow one way. A crate may depend only on crates below it.

```text
redox-cli            binary: argument parsing, process exit codes
  ├── redox-fmt      formatter                      (plan 34)
  ├── redox-driver   orchestration, caching         (plan 10)
  ├── redox-emit     AST → Rust source text
  ├── redox-parser   tokens → AST
  ├── redox-diag     diagnostic rendering           (plan 05)
  ├── redox-ast      syntax tree types, no logic
  └── redox-lexer    source text → tokens
```

Rules:

1. **`redox-ast` contains no logic.** Types, spans, trivia, and at most trivial accessors.
   Any function that *decides* something belongs in a stage crate.
2. **No upward dependency, ever.** `redox-lexer` must not know what a parser is.
3. **No sibling dependency between stage crates** except through the AST. `redox-emit`
   depends on `redox-ast`, never on `redox-parser`.
4. **A cycle is a design error**, not a thing to work around with a shared crate.

### 2.1 Crates declared before they are needed

A crate may exist empty, with a `lib.rs` whose doc comment names the plan that fills it.
Crate boundaries are the one structural decision that is genuinely painful to introduce
late, and an empty crate costs a three-line manifest.

---

## 3. Visibility

Use the narrowest visibility that compiles, and widen deliberately:

| Visibility | Use for |
|---|---|
| (none) | items used only within their own file |
| `pub(super)` | items used by the parent module only |
| `pub(crate)` | items used across the crate |
| `pub` | the crate's public API, and nothing else |

A `pub` item is a promise. Before adding one, ask which other crate needs it; if the answer
is none, it is `pub(crate)`.

---

## 4. Re-export policy

Because of §1, a crate's constructs are scattered across many files. **The module root
re-exports them into one flat namespace**, so callers see a crate, not a filesystem:

```rust
// crates/redox-lexer/src/lib.rs
mod lex;
mod span;
mod token;

pub use lex::lex;
pub use span::Span;
pub use token::Token;
```

Callers then write `use redox_lexer::Token`, never
`use redox_lexer::token::Token`. Two consequences:

- **Private modules, public items.** Declare modules `mod`, not `pub mod`, and re-export
  what is public. The module path is an implementation detail and must not appear in a
  caller's `use`.
- **Moving a construct between files is not a breaking change**, because no caller named its
  file.

A downstream crate re-exporting a type it merely passes through (`pub use redox_ast::Span`)
is acceptable where it spares callers a dependency edge, and should carry a comment saying
so.

---

## 5. Error handling

1. **Library crates return `Result`; they do not panic.** A panic in a stage crate is a bug
   in Redox, not a diagnostic for the user.
2. **Each crate owns its error type**, named for the crate's job: `LexError`, `ParseError`.
   A type that aggregates several — `ParseFailure` — is its own construct.
3. **Every error carries a `Span`.** This is not optional. Emerald's plan 22 found a node
   type with no span twenty-one plans after its diagnostics plan shipped, by which point the
   full fix was judged too large and it settled for a workaround with a disclosed residual
   gap. Spans are cheap at construction and near-impossible to retrofit.
4. **No `unwrap()` or `expect()` on a fallible operation in library code.** `expect` is
   permitted only to assert an invariant the surrounding code guarantees, and its message
   must state the invariant — `expect("indent stack is never empty")`, not
   `expect("should work")`.
5. **`panic!` in tests is fine.** Tests are the one place a panic is the correct outcome.

---

## 6. Test placement

| Test kind | Location | Rationale |
|---|---|---|
| Behaviour reachable through the public API | `crates/<crate>/tests/<construct>.rs` | Tests the contract, not the implementation; survives refactors within the crate |
| A private construct's own logic | `#[cfg(test)] mod tests` in its file | No other code can reach it |
| End-to-end, through the real binary | `crates/redox-cli/tests/` | The only place the whole pipeline is observable |
| Golden output | `tests/golden/`, driven by `crates/redox-cli/tests/golden.rs` | Exact-text comparison needs a stable fixture directory |

Prefer the first row. A test that only needs the public API should not live inside the
crate, because an inline test can reach private internals and will eventually start doing so.

**Fixtures are files, never string literals.** Every `.sdx` fixture lives under `examples/`
or `tests/golden/` and tests reference it by path. Emerald's one syntax cutover missed **113
fixtures** embedded as string literals inside Rust test modules; that cost is paid once per
syntax change, forever.

---

## 7. Dependencies

Redox currently has **zero third-party dependencies** — `Cargo.lock` holds eight packages,
all first-party. That is a property worth defending, not an accident:

1. **A new dependency needs a reason written down** in the plan that adds it.
2. **Prefer `std`.** `docs/02 - Redox Update Blueprint.md` §5.5 names the few dependencies
   the project expects to acquire: `miette` for diagnostics, `notify` for file watching,
   `lsp-server`/`lsp-types` for the language server. Anything beyond that list is an
   argument, not a default.
3. **No dependency may appear in a lowering.** Generated Rust names only `core`, `alloc` and
   `std`. A construct whose output requires a crate the user did not declare is misdesigned
   — this is what rules out regex literals and property-test blocks
   (`docs/03 - Redox Surface Design.md` §2, §11.3, §15).

---

## 8. Workspace conventions

- **`resolver = "3"`, `edition = "2024"`**, inherited from `[workspace.package]` so each
  crate manifest stays three lines.
- **`[workspace.package]` carries version, edition and licence.** Never repeat them.
- **The binary's name may differ from its crate's** — `redox-cli` ships `redox` — so
  `cargo build -p redox-cli` keeps working.
- **`.tmp/` is excluded from the workspace**, so a throwaway crate there cannot be absorbed
  as a member by `cargo init`.
- **Source style is `rustfmt.toml`'s**, currently two-space indentation. This is independent
  of the four-space indentation Redox *emits*, which matches rustfmt's default so generated
  code does not read as foreign.

---

## 9. Enforcement

Two tests, run by the ordinary gate. Neither is optional and neither may be weakened to make
a change land.

| Check | Test | Limit |
|---|---|---|
| One construct per file | `crates/redox-cli/tests/one_construct_per_file.rs` | exactly 1 top-level construct outside module roots |
| File length | `crates/redox-cli/tests/line_cap.rs` | 1,500 lines |

The line cap is a backstop, not the primary rule. Under §1 a file should rarely approach it;
if one does, the construct inside it is doing too much.

### 9.1 Why enforce rather than document

Doc 02 §3.1 records Emerald's habit of correcting stale plan labels in place, because a
convention that is only written down drifts. A rule with a test attached does not drift — it
either holds or the build is red. The two tests above cost roughly 120 lines together and
remove the need to notice violations in review.

---

## 10. Checklist for a new construct

1. Pick the crate, obeying §2's layering.
2. Create `crates/<crate>/src/<snake_case_name>.rs` containing that one construct, its
   `impl` blocks, and nothing else.
3. Declare it in the module root with `mod`, and `pub use` it if it is public (§4).
4. Give it the narrowest visibility that compiles (§3).
5. If it is an error type, make sure it carries a `Span` (§5).
6. Put its tests in `crates/<crate>/tests/` if they use only the public API (§6).
7. Run the gate: `moon run :check :lint :test :cargo-check :cargo-test :ci-format`.
