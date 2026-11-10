# Plan 68 — The AST in Redox

Plan ID: `ast-in-redox`
Depends on: 48 (`conformance-and-self-hosting`), 64 (`macro-invocation`), 09 (`cargo-integration`)
Milestone: 10 — *self-hosting*
Status: `planned`

---

## 1. What this plan is

**`crates/redox-ast` is written in Redox.** Its 53 files become 53 `.sdx` files, the
generated Rust is committed in `src/` exactly where the hand-written Rust is today, and
the workspace compiles against it unchanged.

This is the first row of the dogfood effort, and it is the first time the compiler is
compiled from Redox.

### 1.1 Why this crate, and why now

Two facts decide it, and both are measured.

**It is the only crate besides `redox-fmt` with no blockers at all** — 53 of 53 files, 1,411
code lines, measured 2026-11-10 by the same five tests `crates/redox-cli/tests/conformance.rs`
applies. Plan 48 found 42 of 48 files clean and the figure was moving in the wrong direction
for a port; plan 64 moved it to all of them.

**It is the dependency root.** Every other crate depends on `redox-ast` and it depends on
`redox-lexer` alone. So if the port compiles, the whole workspace type-checks against
Redox-generated types, and the entire 845-test suite runs against them. Plan 48's port was
45 lines of 24,159 — 0.2%, as its own §1.3 says — and its oracle was `syn::parse_file`
because *"the port is one module lifted out of a crate, so `crate::Stmt` cannot resolve and
rustc would reject it for a reason that is about the extraction rather than about Redox."*

That sentence is this row's target. **A port nothing compiles against proves that the
generated Rust parses.** A crate the workspace is built from proves it type-checks, that its
public API is unchanged, and that 845 tests still pass. The difference is the whole value.

### 1.2 The bootstrap never breaks, and that is doc 02 §4.8's design

Doc 02 §4.8 says two things that sound opposed and are not:

> Commit the generated `.xrs` so the bootstrap never breaks.

> Keep the Rust implementation **permanently rather than as a "fallback"**.

Both hold here. The `.sdx` becomes the **editable source** and the generated `.rs` stays
committed in `crates/redox-ast/src/` as a build artifact that cargo compiles exactly as it
compiles it today. Nothing in the build runs `redox` — there is no `build.rs`, no `OUT_DIR`,
no `include!`.

**That is not timidity; a `build.rs` is impossible here.** `compile_dir` lives in
`redox-driver`, which depends on `redox-ast`. A `build.rs` in `redox-ast` calling it is a
dependency cycle cargo rejects outright. Plan 09 built the `build.rs` path for a *user's*
crate, where no cycle exists, and `examples/cargo-project/` is its witness; the compiler's
own crates cannot use it, and a row that discovered this halfway through would have built
the wrong thing first.

So the regeneration is a **test**, not a build step: `redox build` over
`conformance/redox-ast/**/*.sdx` must reproduce what is committed. The gate catches a
`.sdx` edited without regenerating, and a hand edit to the generated `.rs`, with the same
assertion.

### 1.3 Every shape in the crate is expressible, probed rather than assumed

Measured 2026-11-10 against `redox` at `8e1d2c4`, by writing the shapes and running
`redox check` on them. `.tmp/p64/ast1.sdx` through `ast4.sdx` are the probes.

| Shape | Spelling | Emits |
|---|---|---|
| An enum with **struct variants** | `data Expr = Num { text : String, span : Span } \| …` | `enum Expr { Num { text: String, … } }` |
| Recursion through `Box` | `lhs : Box<Expr>` | verbatim |
| Derives | `deriving (Debug, Clone, PartialEq, Eq)` | `#[derive(…)]` |
| A braced struct with per-field visibility | `data Ident = Ident { export text : String }` | `pub struct` with `pub` fields |
| A type alias | `type Span = (usize, usize)` | `type Span = (usize, usize);` |
| **An inherent `impl`** | `instance Expr` with no class name | `impl Expr { … }` |
| A `pub` method on one | `export span : &self -> Span` | `pub fn span(&self) -> Span` |
| A multi-line doc comment | `-- |` on **every** line | `///` on every line |
| A crate root | `-- !`, `module block`, `export import expr::Expr` | `//!`, `mod block;`, `pub use` |

The inherent `impl` is the one that had to be checked before anything else. `docs/GRAMMAR.md`
grades *"Impls, inherent and trait"* as `SUGARED` through `instance` and the corpus witnesses
only the trait form, so the honest reading was that an inherent one might not be reachable —
and `redox-ast`'s entire API is inherent methods. `instance Expr` with **no class name**
emits `impl Expr { … }`. Verified, because AGENTS.md's own gotcha says a declaration read
correctly is still not a behaviour.

### 1.4 Two shapes the port must get right, and the corpus would not catch

**A doc comment is per line, and the wrong spelling is valid Redox.** `-- |` is a doc comment
and `--` is an ordinary one, decided per line — so

```text
-- | An expression.
-- whether the shapes are expressible.
```

emits `///` and then `//`. The text survives, so nothing that checks for comment **loss**
notices, and the line is no longer documentation. `redox-ast` is roughly 60% doc comment by
line and most of them are multi-line, so a port that gets this wrong strips most of the
crate's rustdoc while passing every existing test. Leaf 4 is the check.

**A `data` enum emits `use Expr::*;` and the hand-written crate has no such line.** Plan 53
leaf 1 brings variants into scope under their own names, as Haskell does, and it is right —
but it adds a glob the current module does not have, which changes what the module exports
and could collide. The port must either accept the glob and prove nothing collides, or §6
discloses it. This is the first construct found whose lowering is correct in isolation and
visible in a port.

### 1.5 What this row is not

It is not a migration. `redox-ast` is one crate of twelve, the other eleven stay Rust, and
doc 02 §4.8's *"permanently"* is not being relitigated. The next row decides whether
`redox-fmt` — the other clean crate — follows, on what this one measures.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `8e1d2c4`.

**`redox-ast` is 53 of 53 files blocker-free**, 1,411 code lines. `redox-fmt` is 57 of 57,
1,411 lines. No other crate is clean: the remaining blockers are **`as` casts and nothing
else**, which is what `docs/CONFORMANCE.md` now says — 12 blocking occurrences, down from
964 before plan 64.

**Every shape in §1.3's table was written and run.** The failures the probes produced were
all ordinary Rust ones that rustc caught and reported against the `.sdx` line — `E0308` on
`&str` where `String` was wanted, `E0382` on a move, `E0449` on `export` inside an enum
variant, where visibility is not permitted. That is *"every check rustc can make, rustc
makes"* working, and it is also the answer to how the port is debugged.

**`generated_rust_is_rustfmt_clean` already holds the standard the committed `.rs` needs.**
Thirty-seven examples assert that `redox build`'s output is what rustfmt would write, so the
generated `src/` will not fight `cargo fmt --check`.

**A `build.rs` is impossible**, per §1.2: `compile_dir` is in `redox-driver`, which depends
on `redox-ast`.

**The architecture tests do not see `.sdx`.** `one_construct_per_file.rs` and
`directory_structure.rs` scan `crates/**/src/**/*.rs`, so they will keep checking the
**generated** files — which is right, because the rules are about the Rust module tree. The
`.sdx` tree mirrors it one-to-one, so they hold transitively; leaf 0 asserts the mirror.

---

## 3. Concrete proof this plan targets

**`cargo check --workspace` compiles `redox-ast` from Rust generated by Redox, and all 845
tests pass against it.** No crate's source changes but `redox-ast`'s, and no test's.

Five more:

1. **The crate's public API is byte-identical.** `cargo public-api`-style diffing is out of
   scope, but `cargo check --workspace --all-targets` is the same assertion with teeth: every
   one of the other eleven crates names `redox_ast` types, and a changed signature fails to
   compile.
2. **Regenerating reproduces what is committed.** `redox build` over the `.sdx` tree, diffed
   against `crates/redox-ast/src/`, byte for byte.
3. **Every doc comment survives as a doc comment.** Counted, `///` and `//!` separately,
   against the pre-port count. §1.4.
4. **The `.sdx` tree mirrors the `.rs` tree**, file for file, so the architecture rules in
   `docs/06` keep applying.
5. **`docs/CONFORMANCE.md`'s worked-port section names a crate rather than a module**, and its
   oracle is rustc rather than `syn`.

And one negative: **nothing in the build invokes `redox`.** Asserted, because the opposite —
a `build.rs` — is the shape a reader will assume and §1.2 is why it cannot be.

---

## 4. Leaves

### Leaf 0 — `leaf-port-harness`

`conformance/redox-ast/` mirroring `crates/redox-ast/src/` one file for one file, a sweep that
runs `redox build` over all of it, and the test that the output matches what is committed.

The harness comes first because it is what makes the other leaves reviewable: a port of 53
files is only safe if the diff against the hand-written Rust is visible at every step.

Two decisions this leaf carries. The generated extension is **`.rs`, committed in
`crates/redox-ast/src/`** and not `.xrs` beside the `.sdx` — cargo compiles `src/*.rs` and
nothing else, and plan 09 already drew that line for a user's crate: *"rustc accepts `.rs`;
`.xrs` stays the canonical human-facing artifact"*. And the sweep is **one `redox build` per
file** rather than `compile_dir`, because `compile_dir` flattens its output into one
directory and this tree is three levels deep.

### Leaf 1 — `leaf-pure-types`

The files with no `impl` block: `data`, `newtype`, `type` and their derives and doc comments.
This is the bulk by file count and the easiest by line, and it is where the doc-comment
convention is established for the rest.

### Leaf 2 — `leaf-inherent-impls`

The files carrying logic — `expr.rs`'s `span`, `is_statement_form` and `blocks`,
`pattern.rs`'s `binders` and `rust_text`, `where_edges`, `where_cycle`, `block/stmt.rs`'s
`where_entry_name`, `item.rs`, `binary_op.rs`'s precedence table.

`expr.rs` is 363 code lines and the largest file in the crate; it is also a 48-variant enum
with a `match` over every variant three times. If anything in this row is going to find a
gap, it is here, and it should be ported **first within the leaf** rather than last.

### Leaf 3 — `leaf-module-tree`

`lib.rs` and the eight grouping modules: `module`, `import`, `export import`, and the inner
doc comments that make the crate's rustdoc front page.

### Leaf 4 — `leaf-doc-fidelity`

The check §1.4 is the finding for: the generated `.rs` carries the same number of `///` and
`//!` lines as the hand-written crate did, asserted against a figure recorded when the port
starts. A port that silently demotes a doc comment passes every other test in the gate.

Worth a test of its own rather than a count in another one, because it is the only property
here that no existing test could notice.

### Leaf 5 — `leaf-docs-and-register`

`docs/CONFORMANCE.md`'s worked-port section is rewritten around a crate; `docs/GRAMMAR.md`
grades the inherent `impl` against a real witness rather than `examples/class.sdx`'s trait
one; `docs/STATUS.md` regenerates. Milestone 10 gets its header and this row gets its cell.

---

## 5. Acceptance criteria

1. `crates/redox-ast/src/**/*.rs` is **generated**, and `conformance/redox-ast/**/*.sdx` is
   its source, one file for one file.
2. `cargo check --workspace --all-targets` passes with no change to any other crate.
3. All 845 tests pass, unchanged.
4. Regenerating from the `.sdx` tree reproduces the committed `.rs` byte for byte, asserted
   by a test that fails if either side is edited alone.
5. The `///` and `//!` line counts match the pre-port figures. §1.4, leaf 4.
6. `cargo fmt --check` and `cargo clippy -D warnings` pass on the generated crate — the
   standard `generated_rust_is_rustfmt_clean` already holds for examples.
7. **No escape hatch.** No `rust """…"""` and no `rust` block anywhere in the `.sdx` tree,
   asserted the way plan 48 asserts it for `clause.sdx`: doc 02 §4.8's criterion is that the
   source is expressible *without* it.
8. Nothing in the build invokes `redox`: no `build.rs` in `redox-ast`, and `redox-driver` is
   not a build-dependency of it. §1.2.
9. `docs/CONFORMANCE.md` regenerated, with the worked-port section naming a crate and rustc
   as its oracle.
10. `conformance/clause.sdx` is **retired**, with its `syn`-only tests, because this row
    supersedes it: a module lifted out of a crate was the best available proof and is no
    longer.

---

## 6. Out of scope, and what it discloses

**One crate, not twelve.** `redox-fmt` is equally clean and is deliberately left for the next
row, which should decide on what this one measures rather than on the argument that both are
clean. Nothing else is close: every other crate has `as` casts, which `docs/GRAMMAR.md` grades
`UNREACHABLE` and `tests/unreachable/as_cast.sdx` asserts still fails.

**`as` casts stay unreachable.** This row does not spell them. They are 12 occurrences across
the remaining eleven crates and the nearest working thing is `u8::try_from n`, which is not a
cast — `as` truncates and `try_from` is fallible, so the two are different programs. A row
that wants the rest of the compiler ported has to take that decision, and it is a surface
decision rather than a port.

**The generated `use Expr::*;` is a real divergence.** §1.4. The port's modules will export a
glob the hand-written ones do not. If nothing collides it is accepted and recorded; if
anything does, the row stops and says so rather than working around it.

**`#[derive]` ordering and attribute placement may move.** Attributes are trivia and reach the
`.xrs`, so they survive — but the emitter decides where a derive sits relative to a doc
comment, and the hand-written crate's choices were made by hand. A diff here is cosmetic and
expected; criterion 4's byte-for-byte assertion is against the **port's own** output, not
against today's Rust.

**This is not a migration**, per §1.5. A reader seeing one compiler crate in Redox will assume
the rest follows; doc 02 §4.8 says the Rust implementation stays permanently, and eleven
crates staying Rust is the design and not a backlog.

---

## 7. Gate

```bash
moon run :check          # the whole workspace, against the generated redox-ast
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # 845 tests, unchanged
moon run :format         # treefmt
```

Plus: the regeneration test; the doc-comment count; no `rust` hatch in the `.sdx` tree; the
mirror assertion; `docs/CONFORMANCE.md` and `docs/STATUS.md` re-blessed and the diffs
reviewed.

---

## 8. Emerald reuse

Emerald never did this. Doc 02 §4.8 chose the property — *"can the compiler's own source be
expressed in Redox, without the inline-Rust escape hatch?"* — and called it *"the property
actually worth proving"*, and plan 48 built the measurement. What is transferable is the
sequence rather than any code: **plan 45 found the gap, plan 48 priced it, plan 64 spelled
it, and this row spends the result.** A count is what turned an unreachability note into a
priority, and nothing but a count would have.

---

## 9. Decision log

1. **`redox-ast`, because it is clean *and* the dependency root.** `redox-fmt` is equally
   clean and proves less: nothing depends on it, so a port of it is type-checked by its own
   crate alone. `redox-ast` is named by all eleven others.
2. **The generated `.rs` is committed and cargo compiles it; no `build.rs`.** Not a
   preference — §1.2 — a `build.rs` in `redox-ast` calling `redox-driver` is a cycle cargo
   rejects. Plan 09's path is for a user's crate and stays correct there.
3. **Regeneration is a test, not a build step.** It catches both failure directions with one
   assertion: a `.sdx` edited without regenerating, and a hand edit to the generated Rust.
4. **The inherent `impl` was probed before the row was written.** `docs/GRAMMAR.md` grades it
   `SUGARED` with a trait-form witness, which would have been a reasonable thing to trust and
   is exactly what AGENTS.md's gotcha warns against. `instance Expr` with no class name emits
   `impl Expr`.
5. **Leaf 4 exists because no other test could fail.** A demoted doc comment keeps its text,
   so comment-preservation checks pass and the crate loses its documentation.
6. **`conformance/clause.sdx` is retired rather than kept.** Two worked ports would be two
   places to maintain the same claim, and the weaker one's oracle is `syn`.

---

## 10. Result

*Not yet executed.*
