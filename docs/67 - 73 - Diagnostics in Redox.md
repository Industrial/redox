# Plan 73 — Diagnostics in Redox

The first of row 72's ten successor rows: `redox-diag`'s 21 files, written in
Redox, with the generated `src/` untracked.

It is the dependency root — every other crate names `Diagnostic`, `Code`,
`Lint` and `Severity` — so this is the port whose compiled result the whole
workspace type-checks against. Row 68 claimed `redox-ast` was the root; row 72
corrected that, and this row is the correction landing.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/redox-diag/sdx/**` — 21 files |
| Generated | `crates/redox-diag/src/**`, untracked from this commit |
| Harness | `crates/redox-cli/tests/port.rs`, unchanged in shape |
| List | `seed/PORTED` gains `redox-diag 73`, and becomes the only copy |
| Seed | refreshed in this commit, 914,011 bytes |

Shape, against its two predecessors: `redox-ast` was 53 files of data
declarations and no logic, `redox-fmt` 57 files of one function each. This crate
is **both at once** — four `data` declarations carrying 180 constructors between
them, and `miette` rendering with real control flow — which is why it found
defects neither of them could.

---

## 2 What it found

Twelve findings. Four are in the compiler, five in the port toolchain, two in
the repository's own checks, and one is a gap in the surface that has no answer
yet.

### 2.1 The translator round-tripped the emitter's own output

Plan 68 made a `data` declaration emit `#[allow(unused_imports)] use X::*;`
itself. The translator reads *generated* Rust, so it read that pair back as
author source: the `use` became an `import`, which the writer hoists to the
top where no allow covers it, and the attribute stayed behind as a leading
attribute for whatever construct followed. rustc called the first
`unused_imports`; clippy called the second `empty_line_after_outer_attr`.
**Eight of the sixteen lint failures, from one mechanism.**

The first fix paired the attribute with the `use` below it and dropped both.
Four warnings survived, and the reason is the finding worth keeping: **the
input was already contaminated.** The Rust being translated was a previous
round's output, and *that* round's orphan was in it as an ordinary attribute on
an `impl`, with no `use` below it to pair with. Pairing is not a fixpoint.

So the rule is not about pairing. That attribute is **emitter output by
construction** — it can only reach a `.rs` file from the emitter — and on an
`impl` it is meaningless. It is dropped on sight.

### 2.2 The translator silently truncated a file it could not parse

`#![allow(…)]` does not start with `#[`, so it matched no branch and fell
through to `--partial`'s *"TODO plan 68: hand-port from line N"*. That is not a
parse error. It is a **marker, with everything below it dropped** — and
`lib.sdx` lost its three `module` declarations and every re-export to it.

The only symptom was a TODO count going from 0 to 1. The crate did not compile,
but the port harness blesses before the compile check, so the broken `lib.rs`
was written out first and the failure arrived one step later wearing a different
costume.

None of the nine remaining crates has an inner attribute in `lib.rs`, which is
exactly why this would have gone unnoticed a second time. An inner attribute
passes through verbatim now.

**The general lesson: a tool that cannot parse its input must say so rather
than write a shorter file.**

### 2.3 The port read its own output

`tools/port/port.sh` rebuilt `sdx/` from `crates/redox-diag/src`, which holds
hand-written Rust *until the port is blessed* and generated Rust afterwards. So
every run but the first read back what the previous run produced.

Two visible consequences, both initially mistaken for something else: batch 10
aborted the build because its "before" text had already been replaced by its own
"after"; and §2.1's four survivors looked like a failed fix rather than a
contaminated input.

The input is now a **named commit** — `03cbd72`, the last one that has the
hand-written Rust — and not `HEAD`, because this very commit untracks that tree.
Every batch is also stated as a **destination** rather than a diff: it applies
the edit when the before-state is present and checks the after-state when it is
not, so the build is a fixpoint and reproducible from any starting state.

### 2.4 `let __rdx_t: T = (e);` — a parenthesis the author cannot drop

Doc 18 §6's ascription lowering emitted the inner expression verbatim, so
`(Box::new h) : Box<dyn Display>` produced `let __rdx_t: Box<dyn Display> =
(Box::new(h));` — clippy's `unnecessary_parens`, and an error under
`-D warnings`.

The parenthesis is **not the author's to remove**: without it the ascription
binds to `h` rather than to the call. It is not load-bearing in the lowering
either, because the value lands in `let` position where no expression needs
bracketing. So the lowering looks through it.

Plan 69's `Unary` arm is the same fix for the same reason — a parenthesis that
the surface needs and the output does not — which makes this a shape rather
than a special case, and `expr.rs` now has two instances of it.
`crates/redox-emit/tests/ascription.rs` pins it.

### 2.5 A head-trailing doc comment separated from its item

The body writer appended a blank line between the translated head and the
hand-written body whenever the head had content. For `version.sdx` the head
*ends with* the doc comment that documents the body's `const`, so the blank
line orphaned it: `empty_line_after_doc_comment`.

A comment at the end of the head belongs to the first thing in the body. The
trailing blanks come off and no separator goes back.

### 2.6 Doc comments were doubled, and nobody had counted

Fixing §2.5's neighbour — the orphan-strip that drops the head's trailing
comment when the body supplies its own — **halved** the `-- |` count in seven
files. `render_map.sdx` went 34 → 17, `plain_one.sdx` 34 → 17,
`parse_map.sdx` 30 → 17.

That reads as catastrophic loss, and it is the opposite: the previously
committed `.sdx` carried every doc comment **twice**. `port.rs` could not see
it, because it compares the `.sdx`'s `-- |` count against the generated `.rs`'s
`///` count and both were doubled.

Checked against the ground truth instead — the hand-written Rust at `03cbd72` —
**21 files, zero losses**, and three files grew by the comments this plan added.
That comparison is the one `port.rs` structurally cannot make, because it has
no access to the pre-port Rust once the port lands.

### 2.7 The ported-crate list was in four places and three were stale

| Place | Said |
|---|---|
| `crates/redox-cli/tests/port.rs` | three crates — correct |
| `crates/redox-cli/tests/conformance.rs` | *"Two crates"*, and *"110 files"*, in prose |
| `crates/redox-seed/tests/bootstrap.rs` | two crates |
| `bootstrap.sh` | two crates |

**The last two are the dangerous pair, and they are dangerous together.**
`bootstrap.sh` is the clean-clone path, so a crate missing from its list is
simply not regenerated from the seed; `the_generated_trees_are_ignored` is the
assertion that would have noticed, and it read an equally stale list of its own.
A duplicated list does not merely drift — **it drifts consistently, so the check
and the thing checked agree while both are wrong.**

The list is now `seed/PORTED`, one `crate plan` per line, read by all four. The
paths in `.gitignore` stay a separate copy on purpose: that is the one copy
something *asserts* against, per crate named in the file.

### 2.8 A generated document that contradicted its generator

`docs/CONFORMANCE.md`'s *"Two crates are no longer a count"* section was prose
in `conformance.rs` while the list lived in `port.rs`. With three crates ported
it claimed two, named two of the three, and counted 110 files against 131.

Every number in that section is now derived: the count, the names with their
plan numbers, the file total, and how many crates are left. Row 70 recorded the
same failure one level up — *"a generated document carrying a stale claim is
worse than a hand-written one, because the regeneration looks like a refresh"* —
and this is that sentence coming true inside the generator itself.

The `as`-cast count also **fell**, 12 to 11, with `redox-diag` going 1 to 0:
the port expressed in Redox what the Rust had spelled as a cast. A falling
blocker count is good news and still a gate failure, which is the design.

### 2.9 Redox cannot put a helper inside its own test module

**This is a gap in the surface, not a fault in the port, and it has no answer
in this row.**

A `test "name"` declaration emits a `#[test] fn` inside a generated
`mod __rdx_tests`. There is **no way to put anything else in there**: no
test-module construct, and a helper several tests share cannot live inside one
of them. `redox-diag` has three such helpers — `sample`, `map`, `origin`.

So they sit at module scope, where `#[cfg(test)]` is the only thing that stops
them being dead code the gate rejects — and that collided head-on with
`one_construct_per_file.rs`, which counted each as a second construct in its
file. Two machine-checked rules with no spelling that satisfies both.

The exemption is the right side to give and is not a weakening: doc 06 §7
already exempts `tests/` because *"an integration test's helpers are not part of
the crate's vocabulary"*, and a `#[cfg(test)]` item is not in the crate at all.
Four inline cases pin it, including that a second **non-exempt** construct still
fails — an exemption without that assertion is a hole.

**A successor row owes the surface a test-scoped helper.** Every remaining crate
with shared test fixtures will meet this.

The first attempt also got the state machine wrong in a way worth recording: the
scan was a line filter, so *"this line is skipped"* and *"this line ends the
attribute run"* were the same step, and the flag was cleared by the very
attribute line that set it. It is a loop with `continue` now, where a skipped
line is visibly not a terminator.

### 2.10 The version-skew rule, exercised by an output change

Plan 72 leaf 6's rule reads: *a surface row that the seed cannot parse must
refresh the seed in the same commit.* Its first real exercise was not a parse
failure. §2.4's emitter fix changed the *output*: the seed still emits
`= (Box::new(h));` where the current compiler emits `= Box::new(h);`, so
`a_clone_without_the_generated_rust_bootstraps_from_the_seed` failed on one
line of one file.

The remedy is identical, so **the rule is wider than its wording** — any change
to what the compiler emits for existing source refreshes the seed, not only a
change to what it accepts. The seed is refreshed here, `tests/blob.rs` owns the
new hash, and the refresh is deterministic: two consecutive runs of
`moon run :seed-refresh` produced byte-identical 914,011-byte artifacts.

### 2.11 Five surface facts the bodies had to learn

Found writing the `.sdx`, each costing a rustc error first:

- **`?` binds inside a parenthesised argument.** `.nth (n - 1)?` is
  `.nth((n - 1)?)`; the `?` has to go outside the whole call.
- **A range in argument position needs parentheses**, like an array literal and
  a prefix `*` before it. `.contains (&n) 1..=39` parses as
  `(.contains (&n) 1)..=39`. Three different brackets, one rule: anything with a
  second reading in argument position takes parentheses.
- **`Some *lint` multiplies.** `*` is an operator before it is a dereference.
- **A nullary function is `name : ReturnType`**, with no arrow. `: () -> T`
  declares a *unit parameter*.
- **A lambda parameter is a bare name** — neither a pattern nor a type
  annotation. Plan 69 recorded the pattern half; the annotation half is new, and
  it is load-bearing: the Rust's `|f: &str, what: &str|` annotated for
  inference, so without annotations the closure cannot be written at all. Three
  ascribed bindings say the same thing.

**And `redox-fmt`'s `param_text` prints `(name : type)` for a lambda
parameter**, which the parser rejects. The formatter can emit a form the parser
will not read back; one of the two is wrong and either answer is a surface
decision, so it is recorded for a successor rather than taken here.

### 2.12 Bless mode reports a failure that is not the real one

`REDOX_BLESS=1 cargo test --test port` runs four tests in parallel. One of them
writes the generated `src/`; another compares the `.sdx`'s doc comments against
it. The comparison can read the **pre-bless** tree and fail with a doc-comment
discrepancy that does not exist — which cost a diagnosis here, because a
doc-comment failure is exactly the shape of a real defect this crate had already
had twice.

A second run is clean, so this is friction rather than a false verdict. It is
worth knowing before the next port blesses.

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 21 files are Redox; `crates/redox-diag/sdx` is the source | yes |
| 2 | `redox-diag` is in `seed/PORTED`, and `port.rs`'s four assertions cover it | yes |
| 3 | `crates/redox-diag/src/` is gitignored and out of the index | yes — 21 files left it |
| 4 | No escape hatch: `port.rs`'s `the_port_uses_no_escape_hatch` covers all 131 files | yes |
| 5 | Every doc comment in the hand-written Rust survives | yes — 21 files, 0 losses, measured against `03cbd72` |
| 6 | `moon run :check :lint :test :sdx-format-check` is green | yes |
| 7 | A clean clone bootstraps `redox-diag` from the seed | yes — seed refreshed this commit |
| 8 | `stage1 == stage2` still holds | yes |
| 9 | The ported-crate list has one copy | yes — `seed/PORTED` |
| 10 | A test-scoped helper exists in the surface | **no — deferred, §2.9** |

Criterion 10 is **declined rather than deferred silently**: the surface gap is
real, the workaround is sound and tested, and closing it is a surface decision
that belongs in its own row. Saying so here is the point — row 70 found plan 68
shipping two criteria unmet and undisclosed.

---

## 4 Owed to successor rows

Recorded, not fixed:

- **A test-scoped helper** (§2.9). Every remaining crate with shared fixtures
  needs it.
- **`param_text` and the parser disagree** about an annotated lambda parameter
  (§2.11).
- **`redox fmt` diverges on a nullary lambda in argument position**, adding one
  parenthesis per pass, unbounded. Carried from plan 69.
- **`RDX1001` has no scoped or file-level allow**, and fires inside `test`
  blocks.
- **The `no_effect` / plan 55 §1.6 contradiction.**
- **`compile_dir` does not descend** — carried from plan 72 leaf 1.

---

## 5 Next

`redox-lexer`, 3,008 lines. Then `redox-emit`, `redox-parser`, `redox-driver`,
`redox-cli`, `redox-bench`, `redox-fuzz`, `redox-lsp`, `redox-mcp` — roughly
24,000 lines across nine crates.

The toolchain is in better shape for them than it was for this one: the
translator no longer truncates, no longer round-trips its own attributes, and
reads a pinned input; the writer no longer orphans a doc comment or doubles it;
and the ported-crate list is in one place, so adding a row to it wires the
crate into the port harness, the clean-clone proof, the ignore assertion and
the conformance document at once.
