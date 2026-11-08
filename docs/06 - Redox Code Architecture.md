# Redox — Code Architecture

Version: 2.0.0
Status: normative. Enforced by `crates/redox-cli/tests/one_construct_per_file.rs`,
`crates/redox-cli/tests/directory_structure.rs` and `crates/redox-cli/tests/line_cap.rs`.
Applies to: every crate in the workspace, including crates that are currently empty, and to
every crate added later.

This document governs how Redox's Rust source is organised. It is normative, not advisory:
the central rules are machine-checked and a violation fails the ordinary test gate.

**Version 2.0.0** adds §2, the directory-structure rule, and the path-aware naming it
implies. §1 is unchanged.

---

## 1. The rule: one construct per file

> **Every construct lives in its own file, named after it in `snake_case`.**

A **construct** is a named, top-level item that forms part of a crate's vocabulary:

| Item kind | Example | File |
|---|---|---|
| `struct` | `Token` | `token.rs` |
| `enum` | `TokenKind` | `token/kind.rs` |
| `union` | `Payload` | `payload.rs` |
| `trait` | `Reporter` | `reporter.rs` |
| `type` alias | `Span` | `source/span.rs` |
| `fn` | `lex` | `lex.rs` |
| `const` / `static` | `RUST_INDENT` | `emit/rust_indent.rs` |

**Exactly one** such item may appear at the top level of a file. This holds regardless of
visibility — a private helper function is a construct and gets its own file.

### 1.1 What travels with a construct

These are part of the construct, not separate constructs, and belong in its file:

- **`impl` blocks for the type**, inherent and trait alike. A type and its behaviour are one
  unit; splitting them is how you get a type whose invariants are enforced in three places.
- **Derive and attribute macros** on the item.
- **The construct's own tests**, as `#[cfg(test)] mod tests` — but see §7, which prefers
  integration tests for anything reachable through the public API.

Nothing else. A constant used by two functions is its own construct in its own file.

### 1.2 Module roots and grouping modules hold no constructs

`lib.rs` and `main.rs` are **module roots**, not construct files. They contain only:

1. the crate-level doc comment (`//!`);
2. `mod` / `pub mod` declarations;
3. `pub use` re-exports that form the crate's public surface (§5);
4. crate-level attributes (`#![forbid(unsafe_code)]`).

`main.rs` additionally holds `fn main`, which has no other legal home.

A **grouping module** is the same thing one level down: a file that declares child modules
and a doc comment explaining what the group is for, and holds no construct of its own.
`redox-lexer/src/source.rs` is one — it groups `span.rs` and will group plan 06's span map.

A grouping module is legal **only** with a sibling directory; a file declaring no construct
and no children is dead. Prefer a module head (§2.1) where the group has a natural
head construct, and reach for a grouping module when it genuinely does not.

### 1.3 No `mod.rs`

Use the 2018-and-later layout: a module with children is `foo.rs` beside a `foo/`
directory, never `foo/mod.rs`. A tree full of files all named `mod.rs` is unnavigable in an
editor tab bar and in a diff.

### 1.4 What this costs, stated plainly

The rule is not free, and pretending otherwise would make it easy to abandon under pressure:

- **More files.** `redox-lexer` is twelve source files where it was one.
- **Cohesive small types get separated.** §5's re-exports are what keep this from leaking
  into every caller's `use` block.
- **Visibility widens slightly.** A helper that was a private `fn` in one file now needs
  `pub(crate)` to be reachable from the construct it serves. Use the narrowest visibility
  that compiles — `pub(super)` before `pub(crate)`, `pub(crate)` before `pub` — and never
  widen a helper to `pub` to satisfy the rule.

What it buys: every name has exactly one home, so finding a construct never requires
searching; two people editing two constructs never touch the same file, so the diffs cannot
conflict; and a file's length becomes a direct signal that one construct is doing too much.

### 1.5 The failure mode this prevents

Emerald — the comparable compiler audited in `docs/02 - Redox Update Blueprint.md` — has
**34,429 lines in `crates/emerald-codegen/src/lib.rs`** and **22,589 in
`emerald-sema/src/lib.rs`**, each a single file. Doc 02 §5.4 identifies this as the one clear
structural defect in an otherwise well-organised repository, and notes the contrast: its
`emerald-driver` is 2,658 lines across four named modules and is by far the most readable
crate in it.

Nothing about that outcome was a decision. It is what happens when a file has no rule
telling it when to stop growing.

---

## 2. Directory structure

> **`src/` is not a flat list of files.** Constructs are grouped into modules that mirror
> the crate's internal structure, and the module tree is part of the design.

§1 alone produces a correct but shapeless crate: forty files in one directory, alphabetical,
with no indication of what owns what. The one-construct rule says where a construct *starts*;
this section says where it *belongs*.

### 2.1 Every root-level file is a module head

A file directly under `src/` is legal only if it is:

1. the **module root** — `lib.rs` or `main.rs`;
2. a **module head** — a construct with a sibling directory of the same name holding its
   children; or
3. a **grouping module** (§1.2) — child declarations and a doc comment, no construct, with a
   sibling directory.

```text
src/
  lib.rs            module root
  token.rs          module head — declares Token
  token/            its children
    kind.rs         declares TokenKind
```

Everything else lives at least one level down. A construct that belongs to no group is a
signal, not an exception: either it belongs under an existing module, or it heads a new one.

This is the rule that actually forces structure. Without it, §1 permits a flat pile and the
module tree never gets designed.

### 2.2 Group by role, never by item kind

Directory names are **domain nouns, singular**, naming the concept that owns the children:
`token/`, `trivia/`, `source/`, `emit/`, `build_command/`.

Never group by what an item *is*:

| Banned | Why |
|---|---|
| `types/`, `structs/`, `enums/`, `traits/` | Groups by item kind. Tells a reader nothing about the domain, and every change touches every directory. |
| `helpers/`, `utils/`, `common/`, `misc/`, `shared/`, `core/` | Buckets. A helper belongs under the construct it serves. |
| `impls/` | An `impl` travels with its type (§1.1). |
| `mod/`, `internal/`, `private/` | Visibility is a keyword, not a directory (§4). |

The same prohibition as §1's ban on `types.rs` and `helpers.rs`, one level up. A directory
whose name does not name a domain concept is a bucket.

### 2.3 Naming is path-aware

A directory must pay for itself by shortening the names beneath it. The construct's
`snake_case` name must equal **either**:

- the file stem — `lexer/trim_comment.rs` holds `trim_comment`; or
- the parent directory name joined to the stem by `_` — `token/kind.rs` holds `TokenKind`.

So `token/token_kind.rs` is wrong: it stutters, and the directory has bought nothing. Write
`token/kind.rs`.

```text
source/span.rs              → Span              stem matches
token/kind.rs               → TokenKind         parent + stem
lex/error.rs                → LexError          parent + stem
block/stmt/kind.rs          → StmtKind          parent + stem
lexer/is_ident_start.rs     → is_ident_start    stem matches
```

**One exception, for keywords.** If prefix-dropping would produce a Rust keyword as the
module name, keep the full name: `emit_fn` lives at `emit/emit_fn.rs`, because `emit/fn.rs`
would need `mod r#fn`. Prefer renaming the construct to avoid this where a better name
exists.

### 2.4 Depth

**At most three levels below `src/`.** `block/stmt/kind.rs` is the limit.

Depth mirrors containment, so a fourth level means the crate is modelling something the
crate boundary should have split. Reach for a new crate (§3), not a fourth directory.

### 2.5 A directory earns its existence

- **Two or more children** is the normal case.
- **One child** is permitted but is a smell. It is justified only when the module is a
  named place with known growth ahead of it — `source/` holds only `span.rs` today and will
  hold plan 06's span map. Write the reason in the head file's doc comment.
- **Zero children** is not a directory, it is a deleted directory.

### 2.6 Worked example

The whole workspace, as it stands, read as a demonstration of §§2.1–2.5:

```text
crates/redox-lexer/src/
  lib.rs
  lex.rs                      fn lex              entry point, head
  lex/error.rs                LexError
  lexer.rs                    Lexer               the scanner, head
  lexer/is_ident_continue.rs  fn
  lexer/is_ident_start.rs     fn
  lexer/trim_comment.rs       fn
  source/span.rs              Span                single child, growth declared
  token.rs                    Token               head
  token/kind.rs               TokenKind
  trivia.rs                   Trivia              head
  trivia/kind.rs              TriviaKind

crates/redox-ast/src/         # mirrors the containment hierarchy
  lib.rs
  block.rs                    Block
  block/stmt.rs               Stmt
  block/stmt/kind.rs          StmtKind
  expr.rs                     Expr
  expr/ident.rs               Ident
  program.rs                  Program
  program/item.rs             Item
  program/item/fn_decl.rs     FnDecl

crates/redox-parser/src/
  lib.rs
  parse.rs                    fn parse            entry point, head
  parse/error.rs              ParseError
  parse/failure.rs            ParseFailure
  parse/parser.rs             Parser
  parse/source.rs             fn parse_source

crates/redox-emit/src/
  lib.rs
  emit.rs                     fn emit             entry point, head
  emit/emit_fn.rs             fn emit_fn          keyword exception (§2.3)
  emit/expr.rs                fn emit_expr
  emit/inline_comment.rs      fn emit_inline_comment
  emit/pad.rs                 fn pad
  emit/rust_indent.rs         const RUST_INDENT
  emit/stmt.rs                fn emit_stmt
  emit/trivia.rs              fn emit_trivia

crates/redox-cli/src/
  main.rs
  build_command.rs                   fn build_command    head
  build_command/default_output.rs    fn
  build_command/parse_build_args.rs  fn
  build_command/render.rs            fn render           head
  build_command/render/line_col.rs   fn                  owned by render
  run.rs                             fn run              head
  run/usage.rs                       const USAGE
```

Note what the tree tells a reader that a flat directory could not: `line_col` exists only
for `render`; the three `is_ident_*`/`trim_comment` helpers belong to the `Lexer` and to
nothing else; `ParseError` and `ParseFailure` are part of parsing rather than free-floating
error types; and the AST's directory nesting *is* the grammar's containment hierarchy.

---

## 3. Crate layering

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
5. **Reach for a crate before a fourth directory level** (§2.4).

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
| Exactly one construct per file | `one_construct_per_file.rs` |
| File name matches its construct, path-aware (§2.3) | `one_construct_per_file.rs` |
| Every root-level file is a module head or grouping module (§2.1) | `directory_structure.rs` |
| No kind-based or bucket directory names (§2.2) | `directory_structure.rs` |
| At most three levels below `src/` (§2.4) | `directory_structure.rs` |
| No directory without children (§2.5) | `directory_structure.rs` |
| No `mod.rs` (§1.3) | `directory_structure.rs` |
| File length under 1,500 lines | `line_cap.rs` |

Hidden directories (`.hermes` and similar tooling artifacts) are skipped, and `tests/`
directories are exempt per §7.

All live in `crates/redox-cli/tests/`.

The line cap is a backstop, not a primary rule. Under §1 a file should rarely approach it;
if one does, the construct inside it is doing too much.

### 10.1 Why enforce rather than document

Doc 02 §3.1 records Emerald's habit of correcting stale plan labels in place, because a
convention that is only written down drifts. A rule with a test attached does not drift — it
either holds or the build is red.

The value is not hypothetical: the one-construct check caught two real violations within a
minute of being written, and the violation turned out to be in the *checker's* own matcher
rather than the layout — which is itself the argument for writing the check rather than
eyeballing the tree.

---

## 11. Checklist for a new construct

1. Pick the crate, obeying §3's layering.
2. Decide where it belongs in the module tree (§2). If it heads a new module, create
   `<name>.rs` **and** `<name>/`; if it belongs to an existing one, put it inside.
3. Name the file per §2.3 — drop the parent's prefix if it would stutter.
4. Put that one construct in it, with its `impl` blocks, and nothing else (§1).
5. Declare it with `mod` in its parent, and `pub use` it from the module root if it is
   public (§5).
6. Give it the narrowest visibility that compiles (§4).
7. If it is an error type, make sure it carries a `Span` (§6).
8. Put its tests in `crates/<crate>/tests/` if they use only the public API (§7).
9. Run the gate: `moon run :check :lint :test :cargo-check :cargo-test :ci-format`.
