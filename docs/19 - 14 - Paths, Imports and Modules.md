# Plan 14 — Paths, Imports and Modules

Plan ID: `paths-colon-colon`
Depends on: 03 (`parser-core`), 13 (`haskell-surface`)
Milestone: 4 — *specification-correct*
Status: `done`

---

## 1. What this plan is

`import` and `module`, which are the two declarations Redox can describe and not write.

Doc 04's row 14 is `partial`: doc 01 §22's three inconsistent path conventions were
corrected by doc 03 §13.1 and the correction is **already shipped** — doc 18 §5.2's tight
`::` is a Rust path, spaced `::` ascribes, and `.` means field or method and nothing else.
What is outstanding is the half the row names second: *"`::` everywhere **including
imports**"*. There are no imports.

### 1.1 Why this is the next plan and not a later one

Three things point at it, and the third is the one that decides.

- **Milestone 4's rule.** Doc 04 says its rows "land before any surface growth regardless
  of schedule (doc 03 §20)". Row 14 is the first row in the register that is neither `done`
  nor `superseded`, so by doc 04's own ordering it is next.
- **Plan 09 recorded the block.** Cargo integration "found that Redox cannot yet fill a
  `mod`, which needs plan 24's `export`". Plan 24 shipped with doc 18, so the block is
  gone and the thing it blocked is this.
- **Doc 18 §9 does not compile.** §9 is *"A page of it, end to end"*, and its first three
  lines are a module header and two imports. A page the specification offers as the
  language's own demonstration, which the compiler rejects on line 2, is the clearest
  possible statement of what to build next.

  **Corrected after the fact:** the first draft of this bullet added *"every other construct
  on that page now works"*, which is false — §9 also uses `guard` with `<-` in condition
  position (plan 22), string interpolation (plan 23), and a `write!` macro call that has no
  spelling at all. This plan moves the page from failing on line 2 to failing on its first
  `guard`. §10.0 records the cost of having asserted that without checking.

### 1.2 What `module` means, decided here

Doc 18 §7 retires `mod` and arrives `module`, and §8 declines Haskell's module header —
*"`module M (f, g) where` cannot express Rust's per-item, per-field, scoped visibility"*,
so `export` and its scopes stay as doc 03 §9.6 has them, and this plan has no export list
to build.

That leaves one reading, and doc 18 §9's own Rust column confirms it: **`module` is Rust's
`mod`.**

```text
module config                     mod config;
export module parser              pub mod parser;

module tests                      mod tests {
  test "it parses"            →       #[cfg(test)]
    assert 1 == 1                     mod __rdx_tests { … }
                                  }
```

A bodiless `module` declares a sibling file's module; one with an indented body is doc 03
§13.5's inline block, *"needed for the near-universal test-module convention"*. Both are
passthrough, and neither needs a lowering.

**Doc 18 §9 writes `module config` at the top of the config module itself**, which would
emit `mod config;` inside `config` — a module declaring itself. The lowering is not in
doubt; the example's placement is. §10.1 records which of the two is wrong once the page is
run.

### 1.3 Why the import path is captured, not parsed

An import path is `std::io::{self, Read, Write}`, `crate::models::*`, or
`std::collections::HashMap as Map`. Doc 03 §13.1's lowering is `use` plus that text, so the
path is **verbatim source text** — the discipline doc 03 §14.1 applies to types, and for
the identical reason: *"every present and future Rust type form is reachable for free, and
nothing can drift"*. A nested tree Redox parsed would be a nested tree Redox could lag
behind.

One token has to be understood rather than copied: `as`. It is a reserved word, so the
scanner stops at it the way `capture_type` already does — a refinement plan 17 added for
`errors` rows and that this plan reuses unchanged.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `import` is reserved | `keyword.rs` | `Keyword::Import`, from doc 03's set |
| `module` is reserved | `keyword.rs` | `Keyword::Module`, added by the keyword commit under plan 13 |
| `import` does not parse today | `redox build` on a one-line fixture | `RDX0103: expected a name to begin a declaration` at column 1 |
| `export` is available to prefix a declaration | plan 24, shipped under plan 13 | `visibility()` reads the marker before the dispatch, so `export import` costs no new reader |
| `capture_type` already stops at `as` | plan 17's `errors` rows | `TokenKind::Kw(Keyword::As) => true` in the terminator set |
| A `use` is already emittable | plan 35, shipped under plan 13 | `use super::*;` in the test module — the one `use` Redox emits today |
| `mod` with a body was blocked and is not | plan 09's §2.2 note, and `Visibility` | the block was `export`; `export` is `done` |

---

## 3. Concrete proof this plan targets

Doc 18 §9's page, compiled and run. It is the only acceptance criterion that cannot be
satisfied by a construct working in isolation, and it is the one the specification offers
as its own demonstration.

`examples/module/` — a directory, because a `module` declaration that names a sibling file
is only meaningful when the sibling exists, and plan 09's `compile_dir` already builds a
directory:

```text
examples/module/main.sdx     module greeting, import, and a call across the boundary
examples/module/greeting.sdx the sibling the declaration names
```

Doc 04's three-corpora rule applies unchanged: the golden asserts the text, the run
expectation asserts what it prints.

---

## 4. Leaves

### Leaf 1 — `leaf-import-path`

Capture an import path as verbatim text: segments, `::`, `{…}` trees, `*`, and `self`.
Stops at `as`, at a line break, and nowhere else. One construct,
`crates/redox-parser/src/parse/parser/import_path.rs`.

### Leaf 2 — `leaf-import-decl`

`Item::Import(ImportDecl { path, alias, visibility, span, leading })`, parsed and emitted
as `use path;` / `use path as Alias;` / `pub use path;`.

### Leaf 3 — `leaf-module-decl`

`Item::Module(ModuleDecl { name, items, visibility, span, leading })`. No body emits
`mod name;`; a body emits `mod name { … }` with its items at one indent level, which means
the item emitter has to become recursive — see §6.

### Leaf 4 — `leaf-ordering`

Rust requires nothing of `use` ordering and rustfmt reorders nothing across blank lines, so
items are emitted **in source order**, including imports. Asserted, because the tempting
alternative — gather the imports to the top, as the test module is gathered — would reorder
the author's file for no reason and break the source map's one-line-per-item claim.

### Leaf 5 — `leaf-proof-and-gate`

`examples/module/`, doc 18 §9's page as a golden, and the gate.

---

## 5. Acceptance criteria

1. `import std::collections::HashMap` emits `use std::collections::HashMap;`.
2. Doc 03 §13.1's five forms all emit their stated Rust: plain, `{self, Read, Write}`,
   `as Map`, `::*`, and `export import` → `pub use`.
3. `module name` emits `mod name;`; `export module name` emits `pub mod name;`.
4. `module name` with an indented body emits `mod name { … }`, its items indented one
   level, and a nested `module` inside it indented two.
5. A `test` block inside a `module` body lands in **that** module's test module, not the
   file's — or, if it does not, §10 says so and why.
6. Items keep source order, imports included.
7. `examples/module/` compiles under `compile_dir`, runs, and matches its `.stdout`.
8. Doc 18 §9's page is a golden and is rustfmt-clean.
9. Every new diagnostic has an `RDX` code reached by the UI corpus — plan 05's
   `code_coverage.rs` requires it.

---

## 6. Out of scope, and what it discloses

- **Module-scope bindings** are doc 03 §13.4 and doc 04 row 16, not this plan. A `module`
  body that needs a `const` will not be writable until that row lands, which bounds how
  useful criterion 4 is and is worth saying rather than discovering.
- **`import` resolution.** Redox does not check that an imported path exists, and must not:
  AGENTS.md's rule is that every check rustc can make, rustc makes. An unresolvable import
  is rustc's error, reported against the `.sdx` through plan 07's map.
- **The recursion criterion 4 forces.** `emit_with_map` is a flat loop over
  `program.items` today, and a `module` body makes it a tree. That is the one piece of this
  plan that is not passthrough, and the risk it carries is the source map: plan 06's
  totality test requires every generated line to be attributed, and a line inside a nested
  module is one more place to get that wrong.
- **`#[cfg(test)]` on a `module`** is doc 03 §13.2's attribute passthrough, row 18, still
  `partial`. Criterion 5 therefore tests where a `test` block *lands*, not whether the
  module it lands in can be attributed.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald compiles to object files and has no generated intermediate source, so its
module handling is a symbol-table concern rather than a text one. Doc 03 §13.1's correction
is Redox's own, and §13.5's three forms are *"all previously unreachable"* — there is
nothing to take.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | `module` is Rust's `mod` | Haskell's file header, naming the module the file *is* | Doc 18 §8 declines the header for the export list it cannot express; §9's Rust column shows `mod config;`, which is a declaration and not a header |
| 2 | The import path is verbatim text | A parsed import tree | Doc 03 §13.1's lowering is `use` plus the text. A tree Redox parsed is a tree Redox could lag — §14.1's argument, applied where it applies equally |
| 3 | Items keep source order | Gather imports to the top, as the test module is gathered | §15's gathering is forced (one module per file); nothing forces this, and reordering an author's file without cause is the opposite of doc 01 §3.4 |
| 4 | No import resolution | Check that the path resolves | AGENTS.md: every check rustc can make, rustc makes. Redox has no name resolution and must not grow one |
| 5 | `module` bodies make the item emitter recursive | Flatten nested modules at parse time | Flattening loses the source structure the formatter (row 34) needs, and the span map would attribute an inner item to the wrong nesting |

---

## 10. Result

Shipped. 517 tests pass; `fmt`, `clippy -D warnings` and the full gate are green.
`examples/module.sdx` joins the corpus, and `examples/cargo-project/` moved to the module
shape it was written wanting.

### 10.0 Which criteria were met, and the two that were not

Criteria 1–6 and 9 are met as written. Two are not, and both are §1.1's fault rather than
the implementation's.

**Criterion 7 was met differently.** §3 asked for `examples/module/` as a *directory*,
because a bodiless `module` only means something when the sibling it names exists.
`examples/cargo-project/` is already that directory, already built through `compile_dir`,
and already carried plan 09's note about why it could not use the module shape — so it is
where the bodiless form belongs, and §10.5 is what that looks like. The single-file
`examples/module.sdx` covers the form with a body, which needs no sibling. A second
directory would have duplicated the harness for nothing.

**Criterion 8 could not be met, and §1.1 was wrong to imply it could.** §1.1 said of doc 18
§9's page that *"every other construct on that page now works"*. It does not. The page also
uses:

| Construct | Plan | Status |
|---|---|---|
| `guard Ok raw <- … else return …` | 22 | `partial` — `<-` in condition position outstanding |
| `#{cfg.path}` string interpolation | 23 | `not-started` |
| `write! f "{}" (…)` | — | no macro-call spelling exists |

So §9's page needs plans 22 and 23 before it can be a golden, and this plan moved it from
*failing on line 2* to *failing on its first `guard`*. That is real progress and it is not
the criterion as written. **The claim should have been checked against the page line by line
before it was made** — the three constructs above are visible in §9's own Redox column, and
§1.1 asserted their absence without looking.

### 10.1 §1.2's doubt about doc 18 §9, resolved

§1.2 said the lowering was not in doubt and the example's placement was. **The placement is
wrong.** `module config` emits `mod config;`, so writing it at the top of the config module
declares a child module named `config` inside `config` — and the file it would then load does
not exist, which rustc says in as many words: *"file not found for module `config`"*.

Doc 18 §9's Rust column has it right and its Redox column does not. What §9 means is that
*some other* file declares `module config`; the page itself needs no header, because doc 18
§8 declined the one Haskell would have given it. The header line is removable without
changing a byte of the Rust.

### 10.2 §4's leaf 4 was wrong about rustfmt, in a way that cost a design decision

Leaf 4 asserted that items keep source order because *"rustfmt reorders nothing across blank
lines"*. True — and the emitter had just stopped putting a blank line between adjacent
imports, because grouping a `use` block is Rust's own convention and reads better.

rustfmt's `reorder_imports` defaults to **on** and sorts `use` statements **within a
contiguous block**. So the grouped form was reordered, and the choice was:

1. replicate rustfmt's sort — which is not plain lexicographic, and would be a fifth
   rustfmt rule the emitter has to carry;
2. keep one blank line between every item, which is what the emitter already did, and let
   the question not arise.

(2) ships. It costs one line per import and keeps decision 3 — the author's order is the
author's. Recorded here because the first version of this plan reasoned from half of
rustfmt's behaviour and reached a conclusion that happened to be right for the wrong reason.

### 10.3 Two latent defects, both found by the example

**A lowercase path segment was read as a type variable.** `write_ok :: &mut String ->
fmt::Result` emitted `fn write_ok<FMT>(s: &mut String) -> FMT::Result`, which rustc rejects
with *"associated type `Result` not found"*. Doc 18 §5.3's rule is about a type *variable*,
and a variable is never part of a path — so `type_vars` now skips an identifier adjacent to
`::` on either side. The defect shipped with plan 26's synthesised generics and was
unexercised because no signature had named a module until now.

**A trailing `*` continued the line.** `import crate::models::*` is doc 03 §13.1's glob and
a `*` is the last token of every one of them, so the path scanner swallowed the following
declaration whole — `use crate::models::*exportimportcrate::models::User;`. `*` left
`continues_line`, which is the second token to leave it for this reason: `>` left when a
generic type's close swallowed the next line under plan 13. Twice is a pattern, and it is
now written down where the set is defined: **a token that ends a construct should not
continue a line, however operator-like it looks.**

### 10.4 What §6 disclosed, measured

The recursion was the risk, and it was the cheap part. Lifting `emit_items` out of
`emit_with_map` took the blank-line rule, the source order and the gathered test module with
it, so a module body got all three for nothing — and plan 14's acceptance criterion 5, that
a test inside a module lands in *that* module's test module, fell out of gathering per
item-list rather than per file.

What it cost instead was **nine item emitters gaining an `indent`**. Every one of them had
assumed column zero. None of them was hard; all of them were necessary; and plan 06's span
map needed no change at all, because threading the indent keeps every line attributed by
construction rather than by a later fix-up.

### 10.5 Plan 09's note, resolved

`examples/cargo-project/src/main.rs` carried a note from plan 09 explaining why its
`include!` had to sit at the crate root: Redox emitted no visibility, so inside
`mod greeting { … }` every item was private and unreachable. It named plan 24 as the fix.

Plan 24 shipped `export` with doc 18 and this plan gave the module a declaration to be, so
the crate now uses the shape it was written wanting — `mod greeting { include!(…) }`, with
`greeting::greet` called from `main`. The note is rewritten rather than deleted, because
what it recorded is still the reason the shape is possible.

### 10.6 Disclosures

- **A `module` body cannot hold a `const`.** Doc 03 §13.4's module-scope bindings are row
  16, so a module body is useful for functions, types and tests and not yet for constants.
  §6 said this would bound criterion 4's usefulness; it does.
- **No import is resolved**, by design (decision 4). An unresolvable path is rustc's error,
  reported against the `.sdx` through plan 07's map — which is what `mod config;` with no
  sibling file produced while §10.1 was being settled, and it pointed at the right line.
- **A zero-parameter function's signature takes one type, not `() -> T`.** `sizes :: usize`
  is the zero-parameter form and `sizes :: () -> usize` declares a unit *parameter*. Both
  are legal and they are different functions; the example says which it means. This is not
  new to plan 14 and is the second example to meet it, so it is recorded here rather than
  found a third time.
