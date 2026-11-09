# Plan 45 — Reachability Residue

Plan ID: `reachability-residue`
Depends on: 25 (`traits-and-impls`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

Doc 03 §18 keeps a list titled **"Remaining open"** — the Rust constructs a Redox program
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

No program defines `rust`, so this is uncompilable. `redox check` does report it — *"cannot
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
> 3. **Spell macro definition, *or accept permanently that Redox macros require inline
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
anything: it preserves semantics *because* Redox does not interpret it. It is a hole in the
surface by design, and §6 bounds what that hole may do.

---

## 3. Concrete proof this plan targets

```bash
moon run :check :lint :test        # the new constructs, in the corpora
cargo run -p redox-cli -- check examples/reachability.sdx
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
  permanently that Redox macros require inline Rust"*. Honest only because leaf 1 ships
  inline Rust; before this row it was a decline in favour of something that did not exist.
- **Macro invocation.** §1.5. Unreachable, recorded by plans 29 and 39, assigned to no row,
  and not taken here: it needs a surface spelling decision in doc 03 because doc 18 §7
  retired `!`, and this row is about unreachable constructs rather than new syntax. Named
  here so it stops being a finding that two plans have each rediscovered.
- **Procedural macros.** Doc 01 §26 lists *"Procedural macro integrations"*, which are a
  dependency and a build-script concern rather than a surface construct. Using one from
  Redox needs attributes (plan 36, done) and nothing from this row.
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

*Filled in when the plan ships, including anything above that turned out wrong.*
