# Redox — Code Architecture

Version: 3.0.0
Status: normative. Enforced by `crates/redox-cli/tests/directory_structure.rs`,
`crates/redox-cli/tests/line_cap.rs` and `crates/redox-cli/tests/no_comments.rs`.
Applies to: every crate in the workspace, and to every crate added later.

This document governs how Redox's source is organised. It is normative, not advisory: the
central rules are machine-checked and a violation fails the ordinary test gate.

**Version 3.0.0 replaces §§1–2.** Version 2.0.0 required *one construct per file* with
path-aware naming and a three-level depth cap. Plan 91 retired all three and the measurement
is why:

| | 2.0.0 | 3.0.0 |
|---|---|---|
| `.sdx` files | 572 | ~52 |
| mean code lines per file | 23.5 | ~215 |
| files under 15 code lines | 271 (47%) | 0 |
| `import` / `module` ceremony | 2,358 lines, **18% of all code** | one import block per module |
| comments | 16,125 lines, **52% of the source** | **none** |

One construct per file was a sound trade while every file also held a paragraph explaining
itself: the comment was the unit of value and the file was its envelope. With no comments, a
15-line file is an import list and a function, and 271 of those are not an architecture.

§§3–11 are unchanged and still govern crate layering, visibility, re-exports, error
handling, test placement and dependencies.

---

## 1. The rule: one cohesive group per file

> **A module is a syntactic category or a pipeline stage. Everything belonging to it lives
> in one file, and that file holds no comments.**

The compiler is organised by the two things it is actually about:

- **Syntactic categories** — `expr`, `pattern`, `item`, `type`, `block`. These are the
  shapes the language has, and every stage has an opinion about each of them.
- **Pipeline stages** — `scan`, `layout`, `precedence`, `scope`, `render`. These are the
  passes a shape goes through.

A file is named for one of those and holds all of it: the types, the functions over them,
their `impl` blocks and their inline tests.

```text
crates/redox-parser/sdx/
  parser.sdx       state, dispatch, the item loop
  expr.sdx         the Pratt loop and every expression production
  pattern.sdx      every pattern production
  item.sdx         every declaration production
  type.sdx         verbatim type capture
  layout.sdx       the off-side rule as the parser sees it
  precedence.sdx   binding powers and fixity
  recover.sdx      error recovery
```

Eight files for what was 134. The categories are not buckets and the stages are not buckets,
which is the distinction §2.1 enforces: `expr` names a thing the language has, where
`helpers` names where code went when nobody decided.

### 1.1 What a module holds

Everything that belongs to its category or stage:

- the **types** of that category, with their `impl` blocks — a type and its behaviour are
  one unit, and splitting them is how invariants end up enforced in three places;
- the **functions** over them, public and private alike;
- the **constants** they need;
- **inline tests** for private constructs, as `test "…"` blocks — but see §7, which prefers
  integration tests for anything reachable through the public API.

A construct used by two modules belongs to the one that owns the concept, and is imported by
the other. A construct used by *every* module is the crate's vocabulary and belongs in the
module root.

### 1.2 Module roots

`lib.sdx` and `main.sdx` contain only `module` declarations, `export import` re-exports, the
crate's vocabulary types if it has any, and `main`.

### 1.3 No `mod.rs`

The 2018-and-later layout: `foo.sdx` beside `foo/`, never `foo/mod.rs`. Enforced by
`no_module_uses_mod_rs`.

A module with children is the common case now that files are groups — `expr.sdx` with
`expr/precedence.sdx` beneath it is fine where the child is genuinely a sub-stage. It is not
an excuse to recreate 572 files one level down, and §2.2's line cap is what bounds it.

### 1.4 No comments

> **No `.sdx` under `crates/` contains a comment.**

Not doc comments, not line comments, not a commented-out line. Enforced per crate by
`no_comments.rs`.

What this costs is real and is stated here rather than discovered: the reasoning that was in
16,125 comment lines is **gone, not relocated**. What remains is this document and the plan
records — `docs/NN - MM - Title.md`, each with a §10 recording what its row measured and
corrected. `plan_orientation.rs` asserts that every orientation is cited by the plan it
corrects, so the citation path survives; the convenience of reading it beside the code does
not.

The one thing that did **not** change: a *user's* comments are still trivia that reach their
generated `.xrs`. Doc 02 §10.1 is intact. This rule is about the compiler's own source.

---

## 2. Directory structure

> **Directories name domain concepts. A file is a group, and a group has a size limit.**

### 2.1 Group by role, never by item kind

Directory names are **domain nouns, singular**, naming the concept that owns the children.
Never group by what an item *is*:

| Banned | Why |
|---|---|
| `types/`, `structs/`, `enums/`, `traits/`, `impls/` | Groups by item kind. Tells a reader nothing about the domain, and every change touches every directory. |
| `helpers/`, `utils/`, `common/`, `misc/`, `shared/`, `core/` | Buckets. A helper belongs with the concept it serves. |
| `mod/`, `internal/`, `private/` | Visibility is a keyword, not a directory (§4). |

Enforced by `no_directory_groups_by_item_kind`, and **this is the rule that matters most
under 3.0.0**. Merging 572 files into 52 has exactly one lazy failure mode, and it is a
`helpers/` directory. The check that refuses it is the one check from 2.0.0 §2 that was kept
when the other four were retired.

### 2.2 A file has a size limit

**No generated `.rs` file exceeds 1,500 lines**, enforced by `line_cap.rs`.

Under 2.0.0 this cap was nearly unreachable and never bound anything except
`parse/parser.rs`. Under 3.0.0 it is the only thing standing between *a module per group*
and *one file per crate*. The measurement says there is room: the largest group projects to
roughly 600 generated lines.

A module that reaches the cap is a module whose category has two categories in it. Split on
the seam, not by line count.

### 2.3 A directory earns its existence

- **Two or more children** is the normal case.
- **One child** is permitted but is a smell, justified only when the module is a named place
  with known growth ahead of it.
- **Zero children** is not a directory, it is a deleted directory. Enforced by
  `no_directory_is_empty`.

---

## 3. Crate layering

Dependencies flow one way. A crate may depend only on crates below it.

```text
redox-cli            binary: argument parsing, process exit codes
  ├── redox-fmt      formatter                      (plan 34)
  ├── redox-driver   orchestration, caching         (plan 10)
  ├── redox-emit     AST → Rust source text
  ├── redox-parser   tokens → AST
  ├── redox-lexer    source text → tokens
  ├── redox-ast      syntax tree types, no logic
  └── redox-diag     spans, diagnostic codes, rendering
```

`redox-diag` is the **lowest** crate, below the lexer, which is not where plan 05 first put
it. The order in that listing is load-bearing: a crate may depend only on crates below it.

Rules:

1. **`redox-ast` contains no logic.** Types, spans, trivia, and at most trivial accessors.
   Any function that *decides* something belongs in a stage crate.
2. **No upward dependency, ever.** `redox-lexer` must not know what a parser is.
3. **No sibling dependency between stage crates** except through the AST. `redox-emit`
   depends on `redox-ast`, never on `redox-parser`.
4. **A cycle is a design error**, not a thing to work around with a shared crate.
   When two crates each want something the other owns, one of the two things is in the
   wrong crate. Plan 05 hit this exactly: `redox-diag` needed `Span` to describe where a
   diagnostic points, and `redox-lexer` needed `Code` to label the errors it raises. The
   fix was not a third crate and not a duplicated `Span` — it was recognising that a span
   is a *diagnostic* concept the lexer happens to produce, moving it down into
   `redox-diag`, and re-exporting it (`pub use redox_diag::Span;`) so no caller changed.
   The re-export matters: it means a layering correction need not be a breaking one.
5. **Reach for a crate before a module that holds two categories** (§2.2).
6. **One computation, one home.** If two crates compute the same thing, the copies will
   disagree, and the disagreement will surface as a wrong answer rather than a failed
   build. `Cursor` tracked a line and column while `redox-diag` derived them from the
   span; the copy in the lexer was wrong for any scanner running over a sub-slice, and
   nothing noticed until a UI test printed the wrong line number. Deleting the duplicate
   removed the class of bug, not just the instance.

### 3.1 Crates declared before they are needed

A crate may exist empty, with a `lib.rs` whose doc comment names the plan that fills it.
Crate boundaries are the one structural decision that is genuinely painful to introduce
late, and an empty crate costs a three-line manifest.

---

## 4. Visibility

Use the narrowest visibility that compiles, and widen deliberately:

| Visibility | Use for |
|---|---|
| (none) | items used only within their own file |
| `pub(super)` | items used by the parent module only |
| `pub(crate)` | items used across the crate |
| `pub` | the crate's public API, and nothing else |

A `pub` item is a promise. Before adding one, ask which other crate needs it; if the answer
is none, it is `pub(crate)`.

§2's nesting makes `pub(super)` genuinely useful: a helper under `lexer/` that only its head
needs is `pub(super)`, not `pub(crate)`.

---

## 5. Re-export policy

Because of §§1–2, a crate's constructs are spread across a module tree. **The module root
re-exports them into one flat namespace**, so callers see a crate, not a filesystem:

```rust
// crates/redox-lexer/src/lib.rs
mod lex;
mod source;
mod token;

pub use lex::lex;
pub use source::span::Span;
pub use token::Token;
pub use token::kind::TokenKind;
```

Callers then write `use redox_lexer::TokenKind`, never
`use redox_lexer::token::kind::TokenKind`. Three consequences:

- **Private modules, public items.** Declare modules `mod`, not `pub mod`, and re-export
  what is public. The module path is an implementation detail and must not appear in a
  caller's `use`.
- **Moving a construct between files is not a breaking change**, because no caller named its
  file. This is what makes §2 safe to apply to an existing crate.
- **The module tree can be as deep as the design needs** without taxing callers.

A downstream crate re-exporting a type it merely passes through (`pub use redox_ast::Span`)
is acceptable where it spares callers a dependency edge, and should carry a comment saying
so.

---

## 6. Error handling

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

## 7. Test placement

| Test kind | Location | Rationale |
|---|---|---|
| Behaviour reachable through the public API | `crates/<crate>/tests/<construct>.rs` | Tests the contract, not the implementation; survives refactors within the crate |
| A private construct's own logic | `#[cfg(test)] mod tests` in its file | No other code can reach it |
| End-to-end, through the real binary | `crates/redox-cli/tests/` | The only place the whole pipeline is observable |
| Golden output | `tests/golden/`, driven by `crates/redox-cli/tests/golden.rs` | Exact-text comparison needs a stable fixture directory |

Prefer the first row. A test that only needs the public API should not live inside the
crate, because an inline test can reach private internals and will eventually start doing so.

`tests/` directories are exempt from §§1–2: an integration test file is many `#[test] fn`s
by design, and its name describes the behaviour under test rather than a construct.

**Fixtures are files, never string literals.** Every `.sdx` fixture lives under `examples/`
or `tests/golden/` and tests reference it by path. Emerald's one syntax cutover missed **113
fixtures** embedded as string literals inside Rust test modules; that cost is paid once per
syntax change, forever.

---

## 8. Dependencies

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

## 9. Workspace conventions

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

## 10. Enforcement

Run by the ordinary gate. None is optional and none may be weakened to make a change land.

| Check | Test |
|---|---|
| No comments in any rewritten crate's `.sdx` (§1.4) | `no_comments.rs` |
| Every name in that gate's list is a real crate | `no_comments.rs` |
| No kind-based or bucket directory names (§2.1) | `directory_structure.rs` |
| No directory without children (§2.3) | `directory_structure.rs` |
| No `mod.rs` (§1.3) | `directory_structure.rs` |
| File length under 1,500 lines (§2.2) | `line_cap.rs` |
| Every declared diagnostic code is reached by a UI case | `code_coverage.rs` |
| No `.stderr` expectation without its `.sdx` | `ui.rs` |
| Every diagnostic prints a well-formed `RDX####` code | `ui.rs` |
| Every generated line resolves through the source map, or is an enumerated synthetic line | `source_map.rs` |
| rustc's diagnostics land on the `.sdx` line they belong to | `check.rs` |
| Every runnable program prints exactly its recorded `.stdout` | `run.rs` |
| No test embeds generated Rust or program output as a string literal | `no_literal_expectations.rs` |
| A Redox crate builds and runs under plain Cargo | `cargo_project.rs` |
| A cache hit produces byte-identical output, and a wiped or corrupt cache changes nothing | `cache.rs` |
| The CLI surface is exactly doc 01 §27's six commands | `cli_surface.rs` |
| The watcher holds no timing constant, and survives a compile error | `watch.rs` |

Hidden directories (`.hermes` and similar tooling artifacts) are skipped, and `tests/`
directories are exempt per §7.

All live in `crates/redox-cli/tests/`.

**The line cap stopped being a backstop and became a primary rule.** Under 2.0.0 it was
nearly unreachable, because a file held one construct. Under 3.0.0 it is the only thing
between *a module per group* and *one file per crate*, and §2.2 says what to do when a module
reaches it: split on the seam, not by line count.

Four checks were retired with 2.0.0's §§1–2 and one was deleted for a different reason.
`every_doc_comment_reaches_the_generated_rust` compared the `.sdx` tree's `-- |` count to the
generated tree's `///` count; at zero comments that is `0 == 0`, so it passed while asserting
nothing. Its own doc comment named that hazard. A test that cannot fail is worse than a
missing one, so it is gone rather than green.

### 10.1 Why enforce rather than document

Doc 02 §3.1 records Emerald's habit of correcting stale plan labels in place, because a
convention that is only written down drifts. A rule with a test attached does not drift — it
either holds or the build is red.

The value is not hypothetical, and it cuts both ways. The one-construct check caught two
real violations within a minute of being written, and the violation turned out to be in the
*checker's* own matcher rather than the layout. That is the argument for writing the check.
Its retirement at 3.0.0 is the argument for measuring the rule: 271 of 572 files held under
15 code lines, which no amount of enforcement would have told anyone.

---

## 11. Checklist for a new construct

1. Pick the crate, obeying §3's layering.
2. Decide which **category or stage** owns it (§1) and open that module's file. Creating a
   new file means claiming a new category or stage, which is a bigger decision than adding a
   construct — most additions do not need one.
3. Put it there with its `impl` blocks. **No comment** (§1.4) — if it needs explaining, the
   explanation belongs in the plan document for the row that adds it.
4. Give it the narrowest visibility that compiles (§4), and `export import` it from the
   module root if it is public (§5).
5. If it is an error type, make sure it carries a `Span` (§6).
6. Put its tests in `crates/<crate>/tests/` if they use only the public API (§7); inline
   `test "…"` blocks are for private constructs.
7. Run the gate: `moon run :format :check :lint :test`.
