# Plan 45 — Reachability Residue

Plan ID: `reachability-residue`
Depends on: 25 (`traits-and-impls`)
Milestone: 7 — *tooling*
Status: `done` 2026-11-10

---

## 1. What this plan is

Doc 03 §18 keeps a list titled **"Remaining open"** — the Rust constructs a Voltaic program
cannot write. Doc 02 §4.1's standard is that any such construct is *"a direct violation of
doc 01 §26"*, and §26 names **Unsafe Rust**, **FFI** and **Rust macros** among the things
that *"must remain possible"*. This row closes that list.

Six items, and §1.2 is a correction to the premise all six rest on.

| Item | Doc 03 §18 / §23 | This row |
|---|---|---|
| `unsafe` block, `unsafe fn` | §18; §23 item 2 | **spelled** |
| `extern` blocks and FFI declarations | §18; §23 item 2 | **spelled** |
| `union` | §18; §23 item 4 | **spelled** |
| Raw identifiers (`r#type`) | §18; §23 item 4 | **spelled**, and §1.4 is why that is not free |
| Macro **definition** | §18; §23 item 3 | **declined**, which §23 item 3 pre-authorises |
| `async` blocks | §18; §23 item 4 | **already closed** by plan 39 |

### 1.1 Verified preconditions

Measured 2026-11-09 by transpiling one probe per construct, not read off the specification.
Every verdict below is a command's output.

| Probe | Verdict today |
|---|---|
| `rust """ … """` (doc 01 §23.1's escape hatch) | **transpiles**, to `rust("let x = 1;")` |
| `unsafe` as a block | `RDX0102: expected an expression` |
| `unsafe` as a function marker | `RDX0101` |
| `extern "C"` block | `RDX0101` |
| `union Pair` | `RDX0101` |
| `r#type` | `RDX0004: a raw string must open with `"` after `r` and any `#`` |
| `macro twice` | `RDX0101` |

`async` blocks are **not** in that table because plan 39 shipped them; doc 03 §18's entry for
them is stale and this row's §10 should say so.

Two of those rows are worth more than their diagnostic.

### 1.2 The escape hatch does not exist, which inverts the row's premise

Doc 03 §18 ends *"Each is reachable today only through doc 01 §23.1's escape hatch"*, and
doc 03 §23 items 2 and 3 are both phrased against that resting place. **There is no such
resting place.** `rust """ … """` is not a construct: `rust` lexes as an ordinary identifier,
`"""…"""` as a multi-line string, and the pair parses as a **function call**. The probe emitted

```rust
fn main() {
    rust("let x = 1;");
    println!("{}", 1);
}
```

No program defines `rust`, so this is uncompilable. `voltaic check` does report it — *"cannot
find function"* — which is the *"every check rustc can make, rustc makes"* corollary working
exactly as designed, and it is also the only thing standing between a user and a silent
mistranslation of their own source into a call.

So the five open constructs are not escape-hatch-only. They are **unreachable, and the escape
hatch is a trap**: it accepts what looks like the documented syntax and produces something
else. That makes leaf 1 the prerequisite for the whole row rather than a convenience — every
*decline* in §6 is only honest once inline Rust actually works.

### 1.3 The specification draws the line itself, differently for FFI and for macros

This row does not have to invent a scope. Doc 03 §23 states the two cases in different words,
and the difference is the design:

> 2. **Spell `unsafe`, `extern` and FFI.** §18 lists these as unreachable, and doc 01 §26
>    names FFI as a requirement, **so the escape hatch is not an acceptable resting place.**
>
> 3. **Spell macro definition, *or accept permanently that Voltaic macros require inline
>    Rust*** (§13.3).

Item 2 forbids the hatch; item 3 offers it. So `unsafe`, `extern` and FFI get native
spellings, and macro definition is declined — by the specification's own permission, not by
this row's convenience. §6 records it as a decline with that citation rather than as a gap.

That argument has a precondition, which is §1.2: *"require inline Rust"* is only a resting
place if inline Rust exists. It does not, so leaf 1 comes first and item 3's permission is
unusable until it lands.

### 1.4 `r#` is already taken, and the collision is the interesting part

`r#type` does not fail with *"unexpected token"*. It fails with **`RDX0004: a raw string must
open with `"` after `r` and any `#`"** — the lexer has already committed to reading a raw
string by the time it sees `t`.

So raw identifiers are not merely unspelled: their natural Rust spelling is **claimed by
another construct**, and plan 23 shipped that construct. The fix is the one Rust's own lexer
uses — after `r` and a run of `#`, the next character decides: `"` is a raw string, an
identifier start is a raw identifier. The branch point already exists; it is where `RDX0004`
is raised, which is why this is a small change and not a lexer rewrite.

Recording it because the cheap assumption — *"add `r#ident` to the identifier rule"* —
produces a lexer that reads `r#"x"` as the raw identifier `r#x` followed by a stray quote,
and the failure would surface as a mangled string rather than as a syntax error.

### 1.5 Macro *invocation* is also unreachable, and is not this row's to fix

Doc 03 §18 lists macro **definition**, which implies invocation works. It does not. Measured:
`vec![1, 2, 3]`, `vec! (1)`, `format! "hi"`, `(vec![1, 2, 3])`, `vec![1, 2] :: Vec<i32>` and
`vec![1, 2, 3]` as a function body are **all** rejected, while `std::vec::Vec::new ()` is
accepted — so paths work and macros do not.

The cause is a shipped surface decision, not an omission: doc 18 §7 **retired `!` as a
suffix** and left it meaning boolean negation. `assert! true` appears to work and is a
misparse — it reads as the `assert` keyword applied to `!true` and emits `assert!(!true)`.

Plans 29 §6 and 39 §10.5 both found and recorded this; **neither assigned it to a row**, and
this plan does not take it either. It needs a *spelling* decision in doc 03 — a sigil that
`!` no longer provides — and that is a surface change, where everything else in this row is an
unreachable-construct fix. §6 names it and leaves it, which is the honest half of closing
§18: the list is closed, and a second gap next to it is now written down where a row can pick
it up.

What makes that defensible rather than convenient is leaf 1. With inline Rust working,
`vec![1, 2]` is reachable, and doc 01 §26's *"Rust macros"* requirement is met the way doc 03
§23 item 3 says it may be.

---

## 2. Admission tests (doc 03 §2)

Every construct here is **reachability-positive** by construction — test 4 is the row's whole
subject. The other three:

| Construct | Type-free | No implied dependency | Semantics-preserving |
|---|---|---|---|
| `unsafe` block / fn | Yes — one keyword, one Rust keyword out | `core` only | It *is* Rust's `unsafe`; nothing is added |
| `extern` block, FFI decl | Yes — the ABI string is the user's | None | Verbatim |
| `union` | Yes — fields are declared, as `data` is | None | Verbatim |
| Raw identifier | Yes — a lexical form | None | The identifier is the user's |
| Inline Rust | Yes — the text is emitted unchanged | Whatever the user names, which is the user's choice | Nothing is parsed, so nothing can be altered |

Inline Rust deserves a note against test 3, because it is the one construct that can carry
anything: it preserves semantics *because* Voltaic does not interpret it. It is a hole in the
surface by design, and §6 bounds what that hole may do.

---

## 3. Concrete proof this plan targets

```bash
moon run :check :lint :test        # the new constructs, in the corpora
cargo run -p voltaic-cli -- check examples/reachability.sdx
```

A new `examples/reachability.sdx` exercising every item in §1's table, **compiled by rustc**
rather than only transpiled — because the whole row is about producing Rust that is valid, and
a golden `.xrs` would assert only that the emitter is consistent with itself.

Plus, for inline Rust specifically, a `tests/ui/` case asserting that the **old** behaviour is
gone: `rust """…"""` must no longer emit a call to a function named `rust`. That is the one
regression this row can cause that a user would meet as a mistranslation.

---

## 4. Leaves

### Leaf 1 — `leaf-inline-rust`

Doc 01 §23.1's escape hatch, as a real construct in item and statement position: the contents
of the `"""` block emitted verbatim into the `.xrs`, with the source map (plan 06) carrying
the offsets so a rustc diagnostic inside the block points at the `.sdx` lines that produced
it. The prerequisite for every decline in §6, and the fix for §1.2's trap.

### Leaf 2 — `leaf-unsafe`

`unsafe` as a block and as a function marker. doc 01 §25.4 *"gestures at native Rust syntax
or the inline Rust escape hatch without choosing"*; this leaf chooses native syntax, per §23
item 2.

### Leaf 3 — `leaf-extern-ffi`

`extern` blocks with an ABI string, foreign function declarations inside them, and the
attribute spelling an exported function needs. This is the leaf doc 01 §26 requires by name.

### Leaf 4 — `leaf-union-and-raw-identifiers`

`union` declarations, and raw identifiers with §1.4's lexer lookahead.

---

## 5. Acceptance criteria

1. `unsafe` blocks and `unsafe fn` parse, emit Rust's own `unsafe`, and appear in a corpus
   example that **rustc compiles**.
2. An `extern` block with an ABI string and at least one foreign declaration does the same.
3. A `union` declaration does the same.
4. A raw identifier parses, and `r#"…"#` **still lexes as a raw string** — the collision in
   §1.4 is asserted in both directions, because fixing one by breaking the other is the
   failure mode.
5. Inline Rust emits its contents verbatim, and a rustc error inside the block is reported
   against the `.sdx` line it came from.
6. `rust """…"""` no longer emits a call to a function named `rust`, asserted as a UI case.
7. Doc 03 §18's "Remaining open" list is **edited** — items closed here struck, `async`
   blocks marked closed by plan 39, macro definition moved to a declined list with §23 item
   3's citation, and macro invocation added as the open item §1.5 found.
8. Macro definition is **not** implemented, and §6 says so with its citation.

Criterion 7 is a documentation criterion on purpose. A row that closes a list and leaves the
list saying otherwise has not closed it, and doc 03 §18 is the inventory plan 46 will grade.

---

## 6. Out of scope, and what it discloses

- **Macro definition.** Declined under doc 03 §23 item 3's explicit alternative — *"accept
  permanently that Voltaic macros require inline Rust"*. Honest only because leaf 1 ships
  inline Rust; before this row it was a decline in favour of something that did not exist.
- **Macro invocation.** §1.5. Unreachable, recorded by plans 29 and 39, assigned to no row,
  and not taken here: it needs a surface spelling decision in doc 03 because doc 18 §7
  retired `!`, and this row is about unreachable constructs rather than new syntax. Named
  here so it stops being a finding that two plans have each rediscovered.
- **Procedural macros.** Doc 01 §26 lists *"Procedural macro integrations"*, which are a
  dependency and a build-script concern rather than a surface construct. Using one from
  Voltaic needs attributes (plan 36, done) and nothing from this row.
- **Lifetimes beyond what types already carry.** Doc 03 §14.1 carries a type as verbatim
  text, so `&'static str` already works; there is no separate lifetime construct to spell.
- **Making inline Rust analysable.** The block is emitted, not parsed. No formatting, no
  source map inside the block beyond line offsets, no diagnostics about its contents — all of
  which would mean implementing a Rust parser, which doc 01 §30's constraint exists to
  prevent. What rustc says about it is what the user gets.
- **`unsafe` on a trait or an impl.** `unsafe trait` and `unsafe impl` are a narrower case
  than §18 lists, and the spelling leaf 2 chooses should extend to them; if it does not, §10
  records that rather than this section promising it.

---

## 7. Gate

```bash
moon run :format :check :lint :build :test :audit
```

Moon runs cargo; `blaze check` / `blaze test` are the by-hand inner loop. See AGENTS.md.

---

## 8. Emerald reuse

**Nothing, and that is the finding.** Emerald had no escape hatch, no `unsafe` spelling and
no FFI surface — its generated code was the only Rust in the project, and anything Emerald
could not express simply was not written. This row has no prior art to take, which is also
why doc 03 §18 is a list of open items rather than a list of ported ones.

The one transferable thing is **doc 02 §4.1's standard itself**: grade the host grammar and
treat an unreachable construct as a defect. That standard is Emerald's lesson stated as a
rule, and it is what makes this row exist.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Inline Rust first, as leaf 1 | Spell the constructs and leave the hatch | §1.2: the hatch does not exist and silently emits a call to a function named `rust`. Every decline in §6 rests on it, and it is a live mistranslation until fixed |
| 2 | Native spellings for `unsafe`, `extern`/FFI, `union` | Leave them to the hatch | Doc 03 §23 item 2 forbids the hatch for exactly these, and doc 01 §26 names FFI and Unsafe Rust as requirements |
| 3 | Decline macro definition | Spell it | Doc 03 §23 item 3 offers the decline in its own words; a macro-definition surface is a second grammar and doc 01 §30's constraint is against exactly that |
| 4 | Raw identifiers by lexer lookahead after `r` and `#` | A new sigil; or adding `r#ident` to the identifier rule | Rust's own lexer does the lookahead, and the branch point already exists where `RDX0004` is raised. The naive alternative misreads `r#"x"` as an identifier plus a stray quote (§1.4) |
| 5 | Leave macro invocation to a future row, named | Fix it here; or say nothing | It is a surface spelling decision caused by doc 18 §7 retiring `!`, not an unreachable-construct fix. Two plans have rediscovered it; naming it stops a third (§1.5) |
| 6 | Edit doc 03 §18's list as a criterion | Record the closures only in this plan | §18 is the inventory plan 46 grades. A closed list that still says otherwise is a trap for the next row |
| 7 | Compile the corpus example with rustc, not a golden `.xrs` | A golden | The row's claim is *valid Rust*, and a golden asserts only that the emitter agrees with itself |

---

## 10. Result

Shipped 2026-11-10. All four leaves, `examples/reachability.sdx` compiled **and run** by the
corpus, and doc 03 §18's list edited — closed items struck, `async` blocks credited to plan
39, macro definition moved to a declined list with §23 item 3's citation, and macro
invocation added as the open item §1.5 found.

§1.2's finding survived contact and was the right shape for the row: implementing inline
Rust first made every decline in §6 a decline rather than a gap.

### 10.1 The formatter turned a `union` into a `struct`, and a property test caught it

The most valuable thing this row found is not in its own code. `union` rides on `data` —
§1.3's argument, with `newtype` as the in-surface precedent — so `DataDecl` gained an
`is_union` flag beside `is_newtype`. The parser set it, the emitter read it, every test
passed.

Except `voltaic-fmt`'s `data_text`, which chooses its keyword from `is_newtype` alone and
therefore wrote `union Word` back out as **`data Word`**. Running the formatter on a union
turned it into a struct: not a formatting difference, a **semantic** one, silent, and in the
tool whose whole promise is that it does not change the program.

`formatting_preserves_meaning` caught it on the first gate run. That property is plan 43's
one delivered leaf, and this is it paying for itself — nothing else in the suite was looking,
and a reviewer reading the diff would have had no reason to.

**The transferable lesson**: adding a keyword to a *shared* declaration node means four
places, not three — parser, emitter, formatter, grammar. Three of them fail loudly and the
formatter fails quietly.

### 10.2 §1.1's `union` probe asked the wrong question

The preconditions table reports `union Pair` → `RDX0101`, which is true and was measured
with **Rust's** spelling. `voltaic-ast`'s own doc comment says doc 18 §§2.4–2.5 *"collapsed six
of doc 03's declaration keywords into four. `struct`, `enum`, `union` and the tuple-struct
spelling are all `data`"* — which reads as though `union` were already reachable.

It was not. That sentence is about which declaration *keywords* doc 18 deleted, and `data`
was left with no mechanism for a union at all: doc 18 §2.4's rule reads a **shape** off the
constructors, and a union is not a shape, it is a struct's shape with different storage. So
the row's question was never *"does `union` parse"* but *"which spelling"*, and §1.1 would
have been sharper for saying so.

### 10.3 Four things rustc and its edition had to be asked rather than assumed

- **Edition 2024 requires `unsafe extern`.** A bare `extern "C" { … }` is rejected with
  *"extern blocks must be unsafe"*. The `unsafe` is emitted and is **not** part of the
  surface: there is no safe form to choose between, and doc 03 §2's *"the user writes which"*
  rule is about choices.
- **rustfmt collapses a one-expression `unsafe` block.** `unsafe {\n    w.halves\n}` becomes
  `unsafe { w.halves }`, and `generated_rust_is_rustfmt_clean` compares byte for byte — so
  `emit_unsafe_block` collapses exactly that case and no other. Guessing rustfmt's width rule
  for anything longer would put a second formatter in the emitter.
- **Rust's qualifier order is `const? async? unsafe? extern?`**, from the reference's
  `FunctionQualifiers`. The emitter writes them in that order so two markers together produce
  valid Rust rather than a permutation rustc rejects for a reason the author did not write.
- **A union field read is unsafe**, which is why the example's `low_half` needs leaf 2 to
  demonstrate leaf 4 at all. The two leaves turned out to need each other.

### 10.4 §1.4's collision, and the half of it that would have gone unnoticed

The `r#` lookahead landed where §1.4 predicted — the branch that raises `RDX0004` — and
criterion 4's *both directions* earned its wording: the example asserts `r#type` is an
identifier **and** that `r#"back\slash"#` is still a raw string, in the same file, because a
one-character lookahead invites breaking the second while fixing the first.

The grammar needed the same rule and did not get it from the parser: plan 41's
`every_example_parses_with_no_error` found an `ERROR` node at every `r#type`. The grammar's
`identifier` was `/[a-z_][A-Za-z0-9_]*/`, and the `r#` alternative has to come **first** or
the engine reads the `r` as an ordinary identifier's first character and stops.

That is plan 41 §1.2's constraint working as designed — the grammar is a second reader of the
same language and the corpus is what keeps them agreeing.

### 10.5 What the architecture tests rejected

`verbatim_triple.rs` was written as a root-level file holding two functions.
`every_root_level_file_is_a_module_head` rejected it: a file directly under `src/` must be the
module root, a module head with a sibling directory, or a grouping module. It became a module
head with `verbatim_triple/dedent_triple_body.rs` under it, which is also the better shape —
the dedent rule is now one named thing that `scan_string` and the hatch both call, rather than
a helper sharing a file with its only caller.

### 10.6 Amended by plan 46: `unsafe trait` and `unsafe impl` were not checked

§6 said of `unsafe` on a trait or an impl that *"the spelling leaf 2 chooses should extend to
them; if it does not, §10 records that rather than this section promising it"*.

**It does not, and this section did not record it.** Plan 46 §1.2 probed it: `unsafe class`
and `unsafe instance` both fail with *"expected a name to begin a declaration"*, because leaf
2 put the marker in the signature's `::` run and in the expression prefix, and a declaration
keyword is neither place.

Added here rather than quietly fixed because §6 made a conditional promise and the condition
came true. The gap is graded `UNREACHABLE` in `docs/GRAMMAR.md` with
`tests/unreachable/unsafe_class.sdx` holding it, so it cannot be forgotten a third time.

### 10.7 Declined, with the specification's words

Macro **definition** is not implemented, under doc 03 §23 item 3's own alternative. Macro
**invocation** is named in §6 and in doc 03 §18 and left: it needs a spelling decision, not an
implementation, because doc 18 §7 retired `!`. Writing it into §18 is the part that matters —
plans 29 and 39 each found it and neither left it anywhere a row could pick it up.
