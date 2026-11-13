# Plan 91 — The Great Rewrite

Plan ID: `great-rewrite`
Depends on: 68–83 (the thirteen ports), 84–90 (milestone 11's surface)
Milestone: 12 — *the compiler written in the language it became*
Status: `planned` — awaiting approval; §4 is thirteen waves and §2 is measured

---

## 1. What this plan is

Every `.sdx` in `crates/` is rewritten: merged into modules grouped by what they
do rather than split one construct to a file, point-free where the surface
allows it, **with no comments at all**, using the constructs plans 84–90 added.
The normative documents are rewritten with it.

Four instructions, and each is an axis with its own measurement in §2:

| | Instruction | What it changes | Measured cost today |
|---|---|---|---|
| 1 | Best possible architecture; one-construct-per-file retired | 572 files become ~52 | 2,358 ceremony lines, mean 23.5 code lines per file |
| 2 | Point-free wherever possible | composition replaces named intermediates | `>>>` appears on **11 lines in 5 files** |
| 3 | No comments | 16,125 lines deleted | **52% of the source** |
| 4 | Latest language version | the compiler uses its own newest surface | five of the newest seven constructs appear **zero to three times** |

### 1.1 The architecture is the measurement, not a preference

`crates/voltaic-parser/sdx` holds **134 files for 3,659 code lines** — 27 lines
each. `voltaic-emit` holds 123 for 2,873. Across all thirteen crates the mean is
**23.5 code lines per file**, and **271 of 572 files (47%) hold fewer than 15**.

What fills them is ceremony: **2,358 lines, 18% of all code**, are `import` and
`module` declarations. `voltaic-fmt` and `voltaic-parser` are 21% ceremony;
`voltaic-emit` and `voltaic-lsp` 20%.

That is the cost of doc 06 §1, and it was a reasonable trade when every file
also held a paragraph explaining itself — the comment was the unit of value and
the file was its envelope. Instruction 3 removes the paragraph. A 15-line file
with no comment in it is an import list and a function, and 271 of those are
not an architecture.

### 1.2 Point-free is free in three positions and costs output transparency in a fourth

Voltaic already composes: `|>` appears on 612 lines across 159 files, sections on
891 across 265. Those are point-free and they lower to **direct method calls**,
so the generated Rust is unchanged by them.

`>>>` is different. Plan 66 lowers composition to a **closure**, so
`describe = .len >>> show` emits `|x| show(x.len())` where `describe x = show
(.len x)` emits `show(x.len())`. The goldens and
`generated_rust_is_rustfmt_clean` compare bytes, and doc 02 §5.2's claim is that
the generated Rust is the Rust a person would have written.

So the rule this plan adopts, and §10 reports against:

- **Point-free wherever the output is unchanged** — pipelines, sections, method
  references in adaptor position, `$` instead of parentheses. Unbounded.

  > **Correction, wave 14: there are no method references in adaptor position.**
  > `(.f)` is a *field* section and emits `|__rdx_1| __rdx_1.f` with no call,
  > because plan 57 §1.1 made `.` field access and nothing else. `MethodRef`
  > exists but the grammar reaches it only in `over`'s function slot. Under
  > `.map` / `.filter` / `.all` an explicit lambda is already the minimal form,
  > and eta-reducing one to a section is a type error. §10.3.
- **Point-free in a definition where it inserts a closure** — taken where the
  definition is a pure composition of named functions and the closure is what a
  person would have written anyway; **declined** where a direct call exists,
  because that is paying output transparency for source brevity and doc 02 §5.2
  is the property the README scores 9/10 on.

§5 criterion 4 is what holds this honest: every golden stays byte-identical, or
each divergence is listed with its cause. A rewrite that changes the emitted
Rust has changed the compiler, not its source.

### 1.2a Haskell patterns are admissible exactly as far as the lowering is honest

Rust has no higher-kinded types. A pattern is therefore admissible only if it
lowers to Rust a person would have written — doc 02 §5.2 again, and the same
test §1.2 applies to point-free.

| Pattern | Verdict | Lowers to |
|---|---|---|
| ADT + exhaustive `case` | **take** | `enum` + `match`, exhaustiveness rustc's |
| `newtype` for a domain distinction | **take** | `#[repr(transparent)] struct` |
| **Type class + instances** | **take — the headline** | `trait` + `impl` |
| Smart constructor returning `Result` | **take** | a function |
| Fold over a recursive type | **take** | structural recursion |
| `do .method` for `Result`/`Option` plumbing | **take** | nested `.and_then` |
| Monoidal accumulation (the `Sink`) | **name it** | the existing `Sink` |
| `Functor`/`Traversable` as abstractions | **decline** | needs HKTs |
| Monad transformers over the parser's state | **decline** | needs HKTs; `Parser` is already a `State` by construction and naming it buys nothing rustc can check |

**The type class is the headline because the measurement says so: `class` is used
zero times in 13,453 lines.** And there is an obvious place for it.
`voltaic-emit` and `voltaic-fmt` each carry *parallel families of free functions* over
the same three types — `emit_expr` / `emit_item` / `pattern_text`,
`expr_text` / `item_text` / `stmt_text` / `pattern_text`. That is a type class
waiting to be named:

```
class Render t
  render : usize -> &self -> String
```

with instances for `Expr`, `Pattern`, `Item`, `Stmt`. It replaces a naming
convention with a signature, it is what a Haskell programmer would have written
first, and **it changes no output** — a `trait` with one method per type emits the
same bodies the free functions did.

The two declines are declines on the lowering, not on taste. A `Traversable`
abstraction over `Vec<Result<_, _>>` has no Rust form that is not a lie about
what the compiler checks, and doc 02 §12.4 is the standing refusal that makes
that matter.

### 1.3 "Latest version of the language" is the dogfooding gap, and it is wide

Measured over the compiler's own 13,453 code lines:

| Construct | Plan | Lines | Files |
|---|---|---:|---:|
| `\|>` | 58 | 612 | 159 |
| section `(.f)` | 57 | 891 | 265 |
| `$` | doc 18 §4 | 210 | 102 |
| `case … of` | 03 | 349 | 211 |
| `instance` | 18 §2.5 | 76 | 56 |
| `_` hole | **85** | 78 | 58 |
| `guard … else` | 20 | 64 | 47 |
| `>>>` / `<<<` | **66** | **11** | **5** |
| `over (.a.b)` | **88** | **3** | **2** |
| `do .method` | **86** | **2** | **2** |
| `given <…>` | **84** | **1** | **1** |
| pattern synonym | **87** | **0** | **0** |
| `deriving template` | **89** | **0** | **0** |
| `infixl` | **90** | **0** | **0** |
| `class` | 18 §2.5 | **0** | **0** | ← **wrong, see below** |
| `where` group | **61** | **0** | **0** |

Six constructs the project shipped are used **zero to three times** by the
compiler itself, and `class` — the surface's own abstraction mechanism — is used
nowhere. Each has exactly one `examples/` witness and one golden.

> **Correction, wave 14: the `class` row is wrong.** `voltaic-driver` already had
> two — `export class CacheReporter` and `export(crate) class ProgramWalk` —
> both present at the commit this plan was written against. The probe that
> produced this table matched `^class` and so missed every `export`-prefixed
> declaration. The row is **2 / 2**, not 0 / 0, and the sentence above
> overstates the gap for that one construct. The rest of the table re-measures
> exactly, which is what makes the error attributable to the probe rather than
> to the tree. §10.2 has the corrected table and §10.8 lists the consequences;
> the identical probe bug cost wave 11 a dropped public re-export.

That is the real content of instruction 4. A construct exercised by one
hand-written example is a construct whose failure modes are unknown; the same
construct carrying part of a 13,000-line compiler is one that has been tested.
Plan 68 §1 made this argument for porting the crates at all —
*"porting a crate puts shapes in its own corpus that its authors never wrote"* —
and plans 73, 78 and 82 each found real defects that way. This applies it to
the surface instead of to the language.

### 1.4 What this plan does not get to decide

**Doc 02 §12.4 stays.** No type checker, no borrow checker, no exhaustiveness
checker. The rewrite changes how the compiler is written and not what it
checks.

**The output stays.** §5 criterion 4. If the goldens move, the rewrite is wrong.

---

## 2. Preconditions, measured

Every number below was taken before this document was written. That is a
departure: plans 84, 89 and 90 each recorded in their own §10 that their
preconditions were *"unmeasured"* and that their cost estimates were
consequently wrong — three times, in the same direction.

### 2.1 The corpus

| crate | files | lines | doc `-- \|` | `--` | blank | code | cmt % |
|---|---:|---:|---:|---:|---:|---:|---:|
| voltaic-ast | 57 | 3,280 | 2,080 | 49 | 83 | 1,068 | 65% |
| voltaic-bench | 13 | 582 | 166 | 101 | 32 | 283 | 46% |
| voltaic-cli | 24 | 1,068 | 331 | 63 | 119 | 555 | 37% |
| voltaic-diag | 21 | 1,602 | 774 | 112 | 123 | 593 | 55% |
| voltaic-driver | 49 | 2,492 | 1,094 | 183 | 204 | 1,011 | 51% |
| voltaic-emit | 123 | 6,396 | 2,682 | 661 | 180 | 2,873 | 52% |
| voltaic-fmt | 57 | 2,559 | 1,129 | 293 | 69 | 1,068 | 56% |
| voltaic-fuzz | 6 | 134 | 48 | 50 | 3 | 33 | 73% |
| voltaic-lexer | 34 | 2,820 | 926 | 310 | 188 | 1,396 | 44% |
| voltaic-lsp | 29 | 1,273 | 492 | 210 | 62 | 509 | 55% |
| voltaic-mcp | 19 | 735 | 282 | 104 | 44 | 305 | 53% |
| voltaic-parser | 134 | 7,629 | 2,770 | 1,042 | 158 | 3,659 | 50% |
| voltaic-seed | 6 | 288 | 78 | 95 | 15 | 100 | 60% |
| **total** | **572** | **30,858** | **12,852** | **3,273** | **1,280** | **13,453** | **52%** |

Target: **30,858 → ~11,100 lines**, a 64% reduction — 52 points of it comments
and 8 ceremony. Largest file today is `parser.sdx` at 623 code lines, so a
merged layout fits the 1,500-line cap with room.

### 2.2 The bootstrap exposure is total

`git ls-files 'crates/*/src/*'` returns **0**. Every one of the thirteen
generated trees is `.gitignore`d, so on a clean clone `seed/voltaic1.wasm` must
parse **all 572 `.sdx`**. Syntax the blob cannot read breaks the bootstrap for
everyone — on a clean clone, not here, because the files stay on disk locally.

The remedy is known and was exercised four times during milestone 11: build →
`VOLTAIC_UPDATE=1 … --test port` → `moon run :seed-refresh` →
`VOLTAIC_UPDATE=1 cargo test -p voltaic-seed --test blob`, in that order, in the
same commit as the surface change.

### 2.3 Three of the seven retiring assertions should survive

Instruction 1 retires doc 06 §1. The two gates that enforce it hold **seven**
tests, and they do not all depend on it:

| Test | File | Verdict |
|---|---|---|
| `every_file_declares_exactly_one_construct` | `one_construct_per_file.rs` | **delete** — it *is* the rule |
| `every_file_is_named_after_its_construct` | `one_construct_per_file.rs` | **delete** — meaningless once a file holds many |
| `every_root_level_file_is_a_module_head` | `directory_structure.rs` | **delete** — a merged root file is the module |
| `no_file_is_deeper_than_the_depth_cap` | `directory_structure.rs` | **delete** — ~52 files are two levels at most anyway |
| `no_directory_groups_by_item_kind` | `directory_structure.rs` | **keep** — forbids `types/`, `helpers/`, `utils/`; independent of the rule and the main hazard of merging |
| `no_module_uses_mod_rs` | `directory_structure.rs` | **keep** — independent |
| `no_directory_is_empty` | `directory_structure.rs` | **keep** — independent |
| `no_source_file_exceeds_the_line_cap` | `line_cap.rs` | **keep unchanged** — §2.1 shows the merged layout fits |

Deleting both files wholesale would discard three checks that have nothing to do
with one-construct-per-file, and `no_directory_groups_by_item_kind` is precisely
the thing that goes wrong when 572 files become 52: the lazy merge is a
`helpers/` directory.

### 2.4 One gate stops asserting, silently

`port.rs`'s `every_doc_comment_reaches_the_generated_rust` compares the `.sdx`
tree's `-- |` count against the generated tree's `///` count. At zero comments
that is `0 == 0`: **the test passes and checks nothing.** Its own doc comment
names the hazard — *"this is how a narrowed check quietly becomes no check at
all"* — and instruction 3 walks into it.

It is **deleted** rather than left green, per §5 criterion 2. A test that cannot
fail is worse than a missing one.

### 2.5 The doc surface is 14 files, not 70

Seventy documents mention the retiring rules, but 63 are **plan records** —
`docs/NN - MM - Title.md`, whose §10 sections state what was true when that plan
ran. Rewriting them would be falsifying history and is not in scope (§6).

Normative, and therefore in scope: `AGENTS.md`, `README.md`, and
`docs/{01, 02, 03, 04, 06, 18, 30, COMPILER, GRAMMAR, DECLINED, CONFORMANCE,
STATUS}`. **Doc 06 is the heavy one** — 496 lines, 20 mentions of the module
tree; it *is* the specification being retired and needs rewriting rather than
amending.

---

## 3. Concrete proof this plan targets

**The compiler rebuilds itself from the rewritten source, byte-for-byte, from a
clean clone.**

Six claims, each failing differently:

1. **Every golden is byte-identical.** `tests/golden/*.xrs` unchanged, with no
   re-blessing. This is the one that catches a rewrite that changed behaviour.
2. **A clean clone bootstraps.** `./bootstrap.sh` in a fresh checkout with no
   `src/`, then `cargo build`.
3. **Zero comment lines**, asserted by a gate rather than by inspection.
4. **File count ≤ 60**, no file over the 1,500-line cap.
5. **Each of milestone 11's seven constructs appears in the compiler's own
   source**, with the count recorded — §1.3's table re-measured.
6. **`no_directory_groups_by_item_kind` green**, which is the merge going right
   rather than becoming a bucket.

And one that must **not** change: `moon run :test` reports the same test count
and the same passes. A rewrite that drops a test is plan 74's defect —
`scan_number.sdx` lost seven inline tests and every gate stayed green, because
a check comparing generated output to its source cannot see something absent
from both.

---

## 4. Leaves — thirteen waves in dependency order

One crate per wave, in `seed/PORTED` order, which is dependency order. Each
wave: rewrite `.sdx` → cross the port → gates → commit. A wave is revertible on
its own.

```
0  gates-and-docs      retire §2.3's four tests, delete §2.4's, rewrite doc 06
1  voltaic-diag      21 →  4
2  voltaic-lexer     34 →  6     ← never in the same wave as 3
3  voltaic-ast       57 →  6
4  voltaic-parser   134 →  8
5  voltaic-fmt       57 →  5
6  voltaic-fuzz       6 →  1
7  voltaic-bench     13 →  2
8  voltaic-mcp       19 →  2
9  voltaic-lsp       29 →  3
10 voltaic-emit     123 →  7
11 voltaic-driver    49 →  4
12 voltaic-cli       24 →  3
13 voltaic-seed       6 →  1
14 surface-uptake      re-measure §1.3; close the gaps that remain
15 docs                the other 13 normative documents; regenerate STATUS, CONFORMANCE
```

### Why the lexer and the parser are never the same wave

Plan 87 §10.3 records the port deadlock's recovery needing the **generated lexer
hand-patched** for the removal of one keyword. The lexer is the tool that
recovers from a broken anything: `voltaic build` needs it to regenerate the tree.
Plans 86 and 87 both deadlocked the port and both were recoverable *because the
lexer still worked*.

So wave 2 lands and is committed green before wave 4 begins, and each wave's
procedure ends at a commit for the same reason.

### The target layout

Grouped by **syntactic category and pipeline stage** — the two things the
compiler is actually organised by — never by item kind:

| crate | files |
|---|---|
| voltaic-diag | `code`, `diagnostic`, `render`, `source` |
| voltaic-lexer | `token`, `cursor`, `scan`, `layout`, `trivia`, `lex` |
| voltaic-ast | `expr`, `pattern`, `item`, `block`, `type`, `lib` |
| voltaic-parser | `parser`, `expr`, `pattern`, `item`, `type`, `layout`, `precedence`, `recover` |
| voltaic-fmt | `fmt`, `item_text`, `expr_text`, `stmt_text`, `pattern_text` |
| voltaic-emit | `emit`, `expr`, `item`, `pattern`, `scope`, `layout`, `rust` |
| voltaic-driver | `driver`, `lint`, `remap`, `compile` |
| voltaic-lsp | `serve`, `capabilities`, `handlers` |
| voltaic-cli | `main`, `commands`, `watch` |
| voltaic-mcp | `mcp`, `tools` |
| voltaic-bench | `bench`, `corpus` |
| voltaic-fuzz | `fuzz` |
| voltaic-seed | `seed` |

**52 files.** `scan`, `layout`, `precedence`, `scope` are stages;
`expr`, `pattern`, `item`, `type`, `block` are categories. Neither is a bucket,
which is what §2.3's surviving test checks.

### Per-wave procedure

1. Rewrite the crate's `.sdx`: merge, delete every comment, apply §1.2's
   point-free rule, reach for milestone 11's constructs where they fit.
2. Hand-patch the generated `src/` to match, or regenerate if the current
   compiler already reads the syntax.
3. `cargo build`.
4. `VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test port`, then build again.
5. `moon run :format :check :lint :test`; goldens asserted **unchanged**.
6. `moon run :seed-refresh` + blob hash, if the wave used syntax the blob
   cannot read.
7. Commit the wave.

---

## 5. Acceptance criteria

1. **Zero comments**, enforced *incrementally* so a rewritten crate cannot
   regress while the next is in flight. `no_comments.rs` carries a `REWRITTEN`
   list and asserts those crates hold no line whose first non-space characters
   are `--`; a second test asserts `REWRITTEN` covers every crate in
   `seed/PORTED`, and is red until wave 13. Two tests that check each other is
   `HATCH_ALLOWED`'s shape, for `HATCH_ALLOWED`'s reason: a list and a check
   that agree while both are narrow is no check at all.
2. §2.3's four tests are deleted, §2.4's vacuous test is deleted, and the three
   survivors plus `line_cap.rs` are green **unmodified**.
3. ≤ 60 `.sdx` files under `crates/`; none generating over 1,500 lines.
4. **Every `tests/golden/*.xrs` byte-identical to its pre-rewrite content**, or
   each divergence listed in §10 with its cause and its justification. No
   `VOLTAIC_UPDATE` on the golden corpus.
5. **No test disappears except the five §2.3 and §2.4 name**, and the count
   delta is recorded in §10 wave by wave. The five sanctioned deletions are
   `every_file_declares_exactly_one_construct`,
   `every_file_is_named_after_its_construct`,
   `every_root_level_file_is_a_module_head`,
   `no_file_is_deeper_than_the_depth_cap` and
   `every_doc_comment_reaches_the_generated_rust`, **plus the five self-tests
   that retired gate carried** for its own `constructs` matcher and
   `to_snake_case` helper — ten in total, not five. `no_comments.rs` adds three,
   one of them `#[ignore]`d, so the workspace goes **900 → 892 run + 1
   skipped**.

   The five self-tests were missed by a first count that used `^#\[test\]`
   and so skipped every `#[test]` indented inside a `mod` block. The reconciling
   measurement is a **name-set diff** of `cargo nextest list` before and after,
   which is the only form of this check that cannot be fooled by a regex:

   ```bash
   cargo nextest list --workspace | sed 's/\x1b\[[0-9;]*m//g' | grep '::' | sort -u
   ```

   Two traps found while taking it, both worth avoiding on later waves.
   `git stash` **leaves untracked files in place**, so the first HEAD baseline
   still had `no_comments.rs` on disk and reported 902 with `status_is_current`
   red — a contaminated control, not a real failure. Use `git stash -u`. And
   `nextest list` omits `#[ignore]`d tests, so the added count reads as two.
   Anything else that vanishes is plan 74's defect — `scan_number.sdx` lost
   seven inline tests and every gate stayed green, because a check comparing
   generated output to its source cannot see something absent from both.

   The first version of this criterion read *"the same test count or more,
   never fewer"*, which contradicted §2.3 on the same page.
6. A clean clone bootstraps: `./bootstrap.sh` then `cargo build`, in a checkout
   with no `crates/*/src/`.
7. §1.3's table re-measured and recorded in §10, with every milestone-11
   construct at ≥ 1 use — or a statement of why a construct genuinely does not
   fit the compiler's own code. **`class` at ≥ 1**, per §1.2a.
8. The 14 normative documents of §2.5 updated; the 63 plan records **untouched**,
   asserted by `git diff --stat` naming none of them.
9. `moon run :format :check :lint :test` green, `tree-sitter test` 20/20, port
   and seed and bootstrap green.
10. Line count recorded per crate in §10 against §2.1's table.

---

## 6. Out of scope, and what it discloses

**The 63 plan records are not rewritten.** They record what was true when each
plan ran, including rules this plan retires. A reader of plan 68 should see doc
06 §1 as it was, with this plan's row as the correction — which is how every
other correction in the register works.

**No behaviour changes.** Not one defect fixed, not one construct added. §5
criterion 4 is the enforcement. Anything found along the way is a register row,
not a hitchhiker.

**The comments are deleted, not relocated.** The reasoning in 16,125 lines is
not migrated into the documents — the instruction is that it goes, and a
migration would reproduce it elsewhere while claiming to have removed it. What
the documents get instead is §2.5's rewrite: the rules that apply *now*, stated
once, in the place a reader looks.

Two consequences, disclosed rather than discovered later:

- **The `.xrs` loses its commentary.** Doc 02 §10.1 made comments trivia that
  reach the generated Rust *because the `.xrs` is a human-facing artifact*. With
  no comments in the `.sdx` there are none in the `.xrs`, and that rule becomes
  moot rather than violated. The mechanism stays — a user's comments still
  reach their output.
- **Six findings per plan §10 were previously duplicated in a doc comment near
  the code.** After this, the plan document is the only copy. `plan_orientation.rs`
  already asserts that every orientation is cited by the plan it corrects, so
  the citation path survives; the convenience of reading it beside the code does
  not.

---

## 7. Gate

```bash
moon run :format :check :lint :test
cargo nextest run --workspace --no-fail-fast
cd tree-sitter-voltaic && tree-sitter test
VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test port     # per wave
moon run :seed-refresh                                  # per wave, if needed
VOLTAIC_UPDATE=1 cargo test -p voltaic-seed --test blob
git diff --stat -- 'docs/[0-9]* - [0-9]* - *'          # must be empty
```

Plus, once, at the end: a clean clone, `./bootstrap.sh`, `cargo build`.

---

## 8. Emerald reuse

Negative, and specifically. Emerald's one syntax cutover **missed 113 fixtures**
that lived as string literals inside test modules, which is why `AGENTS.md`
requires every fixture to be a file. This rewrite touches 572 files of compiler
source, and the same failure shape is available: a construct that exists only
inside a comment-documented example, moved without being re-run.

What protects against it here is §5 criterion 4 — the goldens are the fixtures
and they are byte-exact — and criterion 5, which is plan 74's lesson: a check
comparing generated output to its source cannot see something absent from both.

---

## 9. Decision log

1. **Comments are deleted, not migrated.** Raised as a concern once, reaffirmed,
   and that is the decision. §6 records what goes with them.
2. **Point-free is unbounded where the output is unchanged and bounded where it
   inserts a closure** (§1.2). Composition lowers to a closure, and doc 02 §5.2
   is a property the project scores itself on. §10 reports the count either way.
3. **Three of the seven retiring assertions survive** (§2.3). Retiring the two
   gate *files* would discard checks that have nothing to do with the rule.
4. **`every_doc_comment_reaches_the_generated_rust` is deleted, not left
   green** (§2.4). A test that cannot fail is worse than a missing one.
5. **Thirteen waves, one crate each, committed separately** (§4), with the lexer
   never sharing a wave with the parser. The port deadlock is recoverable only
   while the lexer works.
6. **The line cap stays.** §2.1's measurement says the merged layout fits, so
   there is no reason to relax it, and it is the one thing standing between
   "merged" and "one file per crate".
7. **No behaviour changes**, enforced by criterion 4 rather than promised.
8. **Haskell patterns to the limit of an honest lowering** (§1.2a). Type classes
   are taken and are the headline — `class` is used zero times today and two
   crates carry parallel free-function families that are a class by another
   name. HKT-dependent abstractions are declined on the lowering.
9. **A `voltaic-compiler` skill was written and could not be installed.** Both
   `.claude` and `.claude-setup` are gitlinks into a shared submodule, and the
   harness write guard refuses paths outside this checkout. The content —
   §1 of this plan's port recipe, §1.2a's lowering table, and the false-green
   traps — lands in `AGENTS.md`'s *Known tooling gotchas* section in wave 15
   instead, which is tracked here and loads automatically.

---

## 10. Result

**Done.** Thirteen waves, thirteen crates, one commit each.

### 10.1 What the rewrite measures

| | before | after | |
|---|---:|---:|---:|
| files under `crates/*/sdx` | 572 | **78** | −86% |
| lines | 30,858 | **13,216** | −57% |
| doc and comment lines | 17,293 | **0** | −100% |
| code lines | 13,565 | **11,970** | −12% |
| mean code lines per file | 23.5 | **153** | ×6.5 |
| largest generated `.rs` | 1,478 of 1,500 | **828** | |
| tests | 892 + 1 skipped | **893, none skipped** | |

Per crate, files then lines:

| Crate | Files | Lines | Wave |
|---|---|---|---:|
| voltaic-diag | 21 → 5 | 1,602 → 641 | 1 |
| voltaic-lexer | 34 → 8 | 2,820 → 1,518 | 2 |
| voltaic-ast | 57 → 7 | 3,280 → 1,000 | 3 |
| voltaic-parser | 134 → 13 | 7,629 → 3,371 | 4 |
| voltaic-fmt | 57 → 6 | 2,559 → 976 | 5 |
| voltaic-fuzz | 6 → 2 | 134 → 33 | 6 |
| voltaic-bench | 13 → 5 | 582 → 299 | 7 |
| voltaic-mcp | 19 → 5 | 735 → 316 | 8 |
| voltaic-lsp | 29 → 4 | 1,273 → 530 | 9 |
| voltaic-emit | 123 → 10 | 6,396 → 2,715 | 10 |
| voltaic-driver | 49 → 6 | 2,492 → 1,094 | 11 |
| voltaic-cli | 24 → 5 | 1,068 → 621 | 12 |
| voltaic-seed | 6 → 2 | 294 → 102 | 13 |

**Code fell 12% while files fell 86%**, which is the number that says what the
merge actually removed: 1,595 lines of `import` blocks and `module`
declarations, not logic. §1.1 measured that ceremony at 2,358 lines; the rest
of it is the imports a merged file still needs.

### 10.2 §1.3's table, re-measured — and §1.3 was wrong about `class`

Probes that reproduce §1.1 and §1.3's own figures exactly on the `before`
column (572 files, 13,453 code), so the two columns are comparable:

| Construct | before | after |
|---|---:|---:|
| `\|>` | 612 / 159 | 616 / 52 |
| `(.f)` in parens | 891 / 265 | 891 / 56 |
| `$` | 210 / 102 | 212 / 44 |
| `case … of` | 349 / 211 | 349 / 54 |
| `instance` | 72 / 53 | **89 / 30** |
| `guard … else` | 64 / 47 | 64 / 23 |
| `>>>` / `<<<` | 11 / 5 | 11 / 4 |
| `over (.a.b)` | 3 / 2 | 3 / 2 |
| `do .method` | 2 / 2 | **3 / 3** |
| `given <…>` | 1 / 1 | 1 / 1 |
| pattern synonym | 0 / 0 | 0 / 0 |
| `deriving template` | 0 / 0 | 0 / 0 |
| `infixl` | 0 / 0 | 0 / 0 |
| `class` | **2 / 2** | **4 / 4** |
| `where` group | 0 / 0 | 0 / 0 |

**§1.3 recorded `class` as 0 / 0, and that was false when it was written.**
`voltaic-driver` already had two: `export class CacheReporter` in
`cache/reporter.sdx` and `export(crate) class ProgramWalk` in
`lint/program_walk.sdx`, both present at `505f46d`, the commit this plan was
written against. The probe that produced the table must have matched `^class`
and so missed every `export`-prefixed declaration — the same probe bug this
plan hit from the other side at wave 11, where `export newtype Fingerprint`
was invisible to the re-export detector.

The claim built on that row is therefore also wrong, and it is repeated in two
commit messages that cannot be amended: wave 5's *"§1.3's dogfooding gap had
`class` at zero uses across all thirteen crates"* and wave 10's *"the zero it
has been for 90 plans"*. The truth is that `class` was used **twice** before
this plan and is used **four** times after, with 17 instances where there were
72 lines' worth before. The gap was real and narrower than stated.

The honest reading of the rest of the table: **point-free was already
saturated.** 612 pipelines and 891 parenthesised `.f` calls across a
13,453-line compiler is not a codebase with point-free headroom, and the
instruction to prefer it moved `\|>` by 4 lines and `$` by 2. What the plan
added that was genuinely new is the two type classes and their 17 instances.

### 10.3 Point-free in method position does not exist, and that is a surface gap

I found 39 sites of the form `(\x -> .f x)` and rewrote them to `(.f)` as
eta-reduction. **That is wrong, and the port caught it.**

`(.f)` emits `|__rdx_1| __rdx_1.f` — a *field* access with no call. Plan 57
§1.1 made `.` field access and nothing else, so `.f x` is a method call with
the receiver last and `(.f)` is a field section; they are different
constructs, and conflating them turned `.all (\c -> .is_ascii_digit c)` into
`.all(|__rdx_1| __rdx_1.is_ascii_hexdigit)` — E0615, *"method, not a field"*.
Probed four ways afterwards (std method, user method, under `.map`, under
`.all`): all four emit a field access. The sweep is reverted in full.

What caught it is worth naming: `VOLTAIC_UPDATE=1 … --test port` **compiles the
generated tree before writing it**, so a generation that does not compile
leaves `src/` untouched. The 39 bad rewrites never reached a committed state.

So **§1.2 is wrong where it lists "method references in adaptor position"
among the free point-free positions.** There is a `MethodRef` node and a
`method_ref` emitter, but the grammar only reaches them in `over`'s function
slot — `over (.addr.city) .to_uppercase u` — where plan 88 reads the function
as an atom. In `.map` / `.filter` / `.all` position there is no spelling for
"pass this method as a function", and an explicit lambda is already the
minimal form. Field sections work and are used; method sections do not exist.

That is a real finding about the surface rather than about the rewrite, and it
is exactly what §1.3 argued dogfooding would surface.

### 10.4 The four constructs still at zero

`pattern synonym`, `deriving template`, `infixl` and `where` group are
unchanged at 0 / 0, and this plan does not move them. Each is design-bound
rather than effort-bound: the compiler has no place where a pattern synonym
abbreviates a pattern it matches more than once, no derivable shape repeated
across its own types, no domain operator worth declaring, and no `where` group
that a top-level function does not express as well. Inventing a site to raise
the count would be the token use §1.3 argues against — a construct carrying
*part of a compiler* is the thing with value, and a contrived one carries
nothing.

### 10.5 Where the two classes landed

`Render` (wave 5, voltaic-fmt) is `show` to `expr_text`'s `showsPrec`: four
instances over `Item`, `Stmt`, `Expr` and `Pattern`, three of which already
shared the signature, and `format` calls `.render 0 source item`.

`Emit` (wave 10, voltaic-emit) is the stronger of the two. Fourteen functions
already shared `&mut Sink<'_> -> &T -> usize -> ()` across thirteen types —
open recursion written out by hand — and `emit_item`'s `case` now dispatches
through the class in ten of its arms.

**An `instance` must be in the same file as its `class`.** `Parser` carries
`class_heads`, filled by `record_head` when a `class` is parsed and read by
`borrowed_head` when an instance member is parsed, so the parser learns a
member's signature from the class declaration — and each file gets a fresh
`Parser`. An instance elsewhere parses as a run of top-level declarations and
fails with RDX0101 on every name. It is a constraint, not a defect: the parser
needs the member's *shape*, not its types, so it does not cross doc 02
§12.4's line. It is in AGENTS.md as of wave 15.

### 10.6 What the merge tool cost, and what that says

The merge is mechanical, so it is a tool. Eleven bugs in it, each found by
running it and each of which would have recurred in every later wave:

| # | Bug | Wave |
|---:|---|---:|
| 1 | braced imports opaque to dedup and the defined-here check | 2 |
| 2 | `crate::X` and `voltaic_diag::X` both kept — same type, two paths | 2 |
| 3 | fully-qualified paths in *bodies* not rewritten, only imports | 2 |
| 4 | counting `"""` stuck on a string literal containing one | 2 |
| 5 | `lib.sdx` hand-maintained, so it drifted from the groups | 3 |
| 6 | `top_names` blind to `export`-prefixed functions | 3 |
| 7 | a group named after a keyword — `module class` cannot parse | 4 |
| 8 | `main.sdx` skipped entirely, so its comments survived | 7 |
| 9 | aliased imports (`as`) left pointing at deleted modules | 9 |
| 10 | names inside a `rust """` hatch invisible to collision detection | 10 |
| 11 | `X as _` kept beside bare `X` — rustc calls the pair unused | 10 |
| 12 | disambiguation ignored case convention: `SRC_narrow` | 11 |
| 13 | `newtype` missing from the re-export detector | 11 |
| 14 | `main.sdx`'s `module` list not regenerated | 12 |
| 15 | a `lib.sdx` created for a binary-only crate — 25 dead_code | 12 |

Fifteen, not eleven. The one that mattered most was not in the tool at all:
**the driver script piped the merge into `grep`, so the pipeline carried
grep's exit status** and a reported `UNASSIGNED` went on to retire the crate's
subtrees anyway. A guard whose whole job is to stop a destructive step has to
be checked before that step runs. Tracked files made it a reset.

Two of the fifteen produced a *silently wrong* result rather than a failure —
#8 left 43 comment lines that `no_comments` caught, and #13 dropped a public
re-export that only `voltaic-cli`'s cache test noticed. After #13 I audited all
eleven rewritten crates for the same class of loss; `Fingerprint` was the only
one.

### 10.7 Criteria

| | Criterion | Verdict |
|---|---|---|
| 1 | every crate rewritten | **met** — 13/13, `the_rewrite_is_complete` armed |
| 2 | no comments | **met** — `no_comments` over 13 crates, 0 lines |
| 3 | one cohesive group per file | **met** — doc 06 3.0.0; 572 → 78 |
| 4 | goldens byte-identical | **met** — 0 changes at every wave |
| 5 | no fewer tests than the sanctioned ten | **met** — 893, none skipped |
| 6 | line cap | **met** — 828 of 1,500 |
| 7 | surface uptake re-measured | **met** — §10.2, with §1.3 corrected |
| 8 | `class` ≥ 1 | **met** — 4, and it was already 2 |
| 9 | docs updated | **met** — wave 15 |
| 10 | gates green | **met** — `:format :check :lint :test`, port, seed blob |

### 10.8 What the plan got wrong

- **§1.3's `class` row was 0 / 0 and the truth was 2 / 2.** §10.2. A probe that
  missed `export` prefixes, and the same bug cost wave 11 a re-export.
- **§1.2 lists method references among the free point-free positions.** They
  exist only in `over`'s function slot. §10.3.
- **§4 estimated 8 groups for voltaic-parser and 6 for voltaic-ast.** Both were
  low: the parser needs 13 because the 1,500-line cap is on the *generated*
  Rust at ~1.6 lines per line of source, and voltaic-ast needs 7 because two
  enums sharing a variant name cannot share a file.
- **Nothing in §1 anticipated that flattening breaks privacy.** A private
  method is visible to its module's *descendants*; crate-root siblings are not
  descendants. Twelve of `Parser`'s members are now `export(crate)` — the set
  the compiler named, not a blanket promotion.
- **I refreshed the seed on a wrong diagnosis at wave 5** and reverted it. The
  blob does not lag the compiler on instance bodies; same-file placement was
  the issue, and the refreshed seed failed identically.
