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

`crates/redox-parser/sdx` holds **134 files for 3,659 code lines** — 27 lines
each. `redox-emit` holds 123 for 2,873. Across all thirteen crates the mean is
**23.5 code lines per file**, and **271 of 572 files (47%) hold fewer than 15**.

What fills them is ceremony: **2,358 lines, 18% of all code**, are `import` and
`module` declarations. `redox-fmt` and `redox-parser` are 21% ceremony;
`redox-emit` and `redox-lsp` 20%.

That is the cost of doc 06 §1, and it was a reasonable trade when every file
also held a paragraph explaining itself — the comment was the unit of value and
the file was its envelope. Instruction 3 removes the paragraph. A 15-line file
with no comment in it is an import list and a function, and 271 of those are
not an architecture.

### 1.2 Point-free is free in three positions and costs output transparency in a fourth

Redox already composes: `|>` appears on 612 lines across 159 files, sections on
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
`redox-emit` and `redox-fmt` each carry *parallel families of free functions* over
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
| `class` | 18 §2.5 | **0** | **0** |
| `where` group | **61** | **0** | **0** |

Six constructs the project shipped are used **zero to three times** by the
compiler itself, and `class` — the surface's own abstraction mechanism — is used
nowhere. Each has exactly one `examples/` witness and one golden.

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
| redox-ast | 57 | 3,280 | 2,080 | 49 | 83 | 1,068 | 65% |
| redox-bench | 13 | 582 | 166 | 101 | 32 | 283 | 46% |
| redox-cli | 24 | 1,068 | 331 | 63 | 119 | 555 | 37% |
| redox-diag | 21 | 1,602 | 774 | 112 | 123 | 593 | 55% |
| redox-driver | 49 | 2,492 | 1,094 | 183 | 204 | 1,011 | 51% |
| redox-emit | 123 | 6,396 | 2,682 | 661 | 180 | 2,873 | 52% |
| redox-fmt | 57 | 2,559 | 1,129 | 293 | 69 | 1,068 | 56% |
| redox-fuzz | 6 | 134 | 48 | 50 | 3 | 33 | 73% |
| redox-lexer | 34 | 2,820 | 926 | 310 | 188 | 1,396 | 44% |
| redox-lsp | 29 | 1,273 | 492 | 210 | 62 | 509 | 55% |
| redox-mcp | 19 | 735 | 282 | 104 | 44 | 305 | 53% |
| redox-parser | 134 | 7,629 | 2,770 | 1,042 | 158 | 3,659 | 50% |
| redox-seed | 6 | 288 | 78 | 95 | 15 | 100 | 60% |
| **total** | **572** | **30,858** | **12,852** | **3,273** | **1,280** | **13,453** | **52%** |

Target: **30,858 → ~11,100 lines**, a 64% reduction — 52 points of it comments
and 8 ceremony. Largest file today is `parser.sdx` at 623 code lines, so a
merged layout fits the 1,500-line cap with room.

### 2.2 The bootstrap exposure is total

`git ls-files 'crates/*/src/*'` returns **0**. Every one of the thirteen
generated trees is `.gitignore`d, so on a clean clone `seed/redox1.wasm` must
parse **all 572 `.sdx`**. Syntax the blob cannot read breaks the bootstrap for
everyone — on a clean clone, not here, because the files stay on disk locally.

The remedy is known and was exercised four times during milestone 11: build →
`REDOX_UPDATE=1 … --test port` → `moon run :seed-refresh` →
`REDOX_UPDATE=1 cargo test -p redox-seed --test blob`, in that order, in the
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
1  redox-diag      21 →  4
2  redox-lexer     34 →  6     ← never in the same wave as 3
3  redox-ast       57 →  6
4  redox-parser   134 →  8
5  redox-fmt       57 →  5
6  redox-fuzz       6 →  1
7  redox-bench     13 →  2
8  redox-mcp       19 →  2
9  redox-lsp       29 →  3
10 redox-emit     123 →  7
11 redox-driver    49 →  4
12 redox-cli       24 →  3
13 redox-seed       6 →  1
14 surface-uptake      re-measure §1.3; close the gaps that remain
15 docs                the other 13 normative documents; regenerate STATUS, CONFORMANCE
```

### Why the lexer and the parser are never the same wave

Plan 87 §10.3 records the port deadlock's recovery needing the **generated lexer
hand-patched** for the removal of one keyword. The lexer is the tool that
recovers from a broken anything: `redox build` needs it to regenerate the tree.
Plans 86 and 87 both deadlocked the port and both were recoverable *because the
lexer still worked*.

So wave 2 lands and is committed green before wave 4 begins, and each wave's
procedure ends at a commit for the same reason.

### The target layout

Grouped by **syntactic category and pipeline stage** — the two things the
compiler is actually organised by — never by item kind:

| crate | files |
|---|---|
| redox-diag | `code`, `diagnostic`, `render`, `source` |
| redox-lexer | `token`, `cursor`, `scan`, `layout`, `trivia`, `lex` |
| redox-ast | `expr`, `pattern`, `item`, `block`, `type`, `lib` |
| redox-parser | `parser`, `expr`, `pattern`, `item`, `type`, `layout`, `precedence`, `recover` |
| redox-fmt | `fmt`, `item_text`, `expr_text`, `stmt_text`, `pattern_text` |
| redox-emit | `emit`, `expr`, `item`, `pattern`, `scope`, `layout`, `rust` |
| redox-driver | `driver`, `lint`, `remap`, `compile` |
| redox-lsp | `serve`, `capabilities`, `handlers` |
| redox-cli | `main`, `commands`, `watch` |
| redox-mcp | `mcp`, `tools` |
| redox-bench | `bench`, `corpus` |
| redox-fuzz | `fuzz` |
| redox-seed | `seed` |

**52 files.** `scan`, `layout`, `precedence`, `scope` are stages;
`expr`, `pattern`, `item`, `type`, `block` are categories. Neither is a bucket,
which is what §2.3's surviving test checks.

### Per-wave procedure

1. Rewrite the crate's `.sdx`: merge, delete every comment, apply §1.2's
   point-free rule, reach for milestone 11's constructs where they fit.
2. Hand-patch the generated `src/` to match, or regenerate if the current
   compiler already reads the syntax.
3. `cargo build`.
4. `REDOX_UPDATE=1 cargo test -p redox-cli --test port`, then build again.
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
   `REDOX_UPDATE` on the golden corpus.
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
cd tree-sitter-redox && tree-sitter test
REDOX_UPDATE=1 cargo test -p redox-cli --test port     # per wave
moon run :seed-refresh                                  # per wave, if needed
REDOX_UPDATE=1 cargo test -p redox-seed --test blob
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
9. **A `redox-compiler` skill was written and could not be installed.** Both
   `.claude` and `.claude-setup` are gitlinks into a shared submodule, and the
   harness write guard refuses paths outside this checkout. The content —
   §1 of this plan's port recipe, §1.2a's lowering table, and the false-green
   traps — lands in `AGENTS.md`'s *Known tooling gotchas* section in wave 15
   instead, which is tracked here and loads automatically.

---

## 10. Result

Not started. Awaiting approval.
