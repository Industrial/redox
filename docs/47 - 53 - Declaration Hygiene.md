# Plan 53 — Declaration Hygiene

Plan ID: `declaration-hygiene`
Depends on: 52 (`type-colon`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

Three small corrections to the declaration forms, each of which is a thing the language says and
does not mean.

| Today | The problem | This row |
|---|---|---|
| `data` emits `use T::*`; `errors` does not | two declaration forms, one use-site rule between them | `errors` emits it too |
| `newtype` and `data` lower identically | a keyword implying a distinction it does not make | one field enforced, `#[repr(transparent)]` emitted |
| `-- redox: allow unwrap` | semantics in trivia a formatter can delete | a **consumed** `#[redox::allow(unwrap)]` |

None of the three is large. They are one row because they are one kind of defect — a declaration
that is not what it appears to be — and because each is cheap enough that three separate rows
would cost more in gates than in code.

### 1.1 `errors` is the only declaration whose constructors need qualifying

Measured while writing a program against the current surface: `data` emits `use T::*`, so a
constructor is in scope under its own name — which is doc 18 §6's rule (*"In Haskell a
constructor is in scope under its own name; in Rust it is not"*), with §2.4 supplying the
example. `errors` does not, so:

```
data Req = Exact Version | Caret Version | Tilde Version
errors VersionError
  Shape String  "expected major.minor.patch, got `#{0}`"

Ok (Caret v)                    -- works
Err (Shape raw)                 -- E0425: cannot find tuple variant `Shape`
Err (VersionError::Shape raw)   -- what you must write
```

**No document gives a reason.** Doc 03 §12.5 specifies what `errors` generates — the enum, the
`Display`, the `Error` impl, the `From` rows — and says nothing about scope. The glob is doc 18
§6's fourth disclosed lowering (`data T = A | B` → the `enum` *plus* `use T::*;`), which that
section records as *"not in this document's first draft"*; `errors` was shipped by a different
row and did not inherit it. Plan 20 §10.1 lists *"plan 18's `use T::*`"* among four rows whose
scope was smaller than what they claimed — the attribution there is to the plan, where the
lowering is doc 18 §6's.

So this is an omission rather than a decision, and the fix is to make the two forms agree.

### 1.2 `newtype` currently means nothing

`newtype Meters = Meters f64` and `data Meters = Meters f64` emit the same Rust, and **the
language's own source says so in two places** — neither of them a specification:

- `examples/data.sdx`: *"`newtype` and `data` lower identically here, because Rust has one such
  form and not two."*
- `crates/redox-ast/src/program/item/data_decl.rs`, on the `is_newtype` flag: *"Kept even though
  it changes no output […] would let a later plan reject `newtype` with more than one field."*

Doc 18 §2.4 only *shows* the identical lowering in its code blocks; it never claims it as a rule.
So the flag was kept **for this row**, which is as close to a specification as the distinction
has.

In Haskell they are not the same. `newtype` has **exactly one constructor with exactly one
field**, and its operational promise is **no runtime wrapper**. Both halves are available here:

- the shape is a syntactic check, needing no resolver and no types;
- the promise is `#[repr(transparent)]`, which is Rust's spelling of the same guarantee.

Making both real is cheaper than the alternative, which is a keyword that exists to look like
Haskell. The retirement option — delete `newtype`, keep `data` — was weighed and declined in §9.

### 1.3 A lint pragma in a comment is semantics in trivia — and plan 38 already refused the fix

**This reverses a recorded decision, and the reversal needs new evidence rather than a
preference.** Plan 38 §1.3 chose the comment pragma and gave three reasons against the
attribute:

> - Attributes are **trivia** (plan 18 §1.3) and reach the `.xrs` verbatim. `#[allow(unwrap)]`
>   passed through makes rustc say `unknown lint`, so a program that suppressed a Redox lint
>   would stop compiling.
> - Consuming it instead — recognising one attribute and dropping it — makes plan 18's rule
>   conditional. […] a carve-out for a single spelling weakens the rule that made nine item
>   kinds work at once.
> - `#[redox::allow(unwrap)]` needs `register_tool`, which is unstable.

The first and third reasons **do not apply to a consumed attribute**, and that is the whole of
the design change. If Redox recognises `#[redox::allow(unwrap)]` and **does not emit it**, then
rustc never sees it: no `unknown lint`, and no `register_tool`, because the attribute does not
reach the compiler that would need the feature. Plan 38 listed consumption as the second option
and then priced the third option's instability against it as though they were the same route;
they are not.

So the real cost is the second reason alone: **plan 18 §1.3's attributes-are-trivia rule gains
exactly one carve-out.** That is a genuine weakening and this row pays it knowingly.

What makes it worth paying is evidence plan 38 did not have. Plan 48 — eleven rows later — found
`clause.rs`'s four field doc comments **deleted from the `.sdx` by `redox fmt`**, because the
lexer's `scan_dashes` misclassified a comment opening a continuation line. AGENTS.md calls
comment preservation load-bearing; the formatter broke it anyway, in the tool whose promise is
not changing the program.

Plan 38's bet was that a comment *"reaches nothing downstream"* and is therefore safe. Plan 48
showed the comment is not safe **upstream** — it can be deleted from the source. A suppression
that a formatter run can silently remove turns a deliberate pragma into a surprise `RDX1001`, or
worse, removes the record of why a panic was acceptable. One conditional rule is cheaper than
that.

It also makes the pragma consistent with where the milestone is going: plan 64 deletes the
keyword family in favour of Rust's macros, and `#[redox::allow(…)]` is the attribute-shaped
member of the same family.

---

## 2. Verified preconditions

Measured 2026-11-10.

**The `errors` asymmetry is real and reaches rustc, not the Redox diagnostics.** A bare variant
emits a bare path and fails with `E0425: cannot find function, tuple struct or tuple variant`,
pointing into generated Rust. `data`'s glob is emitted; `errors`'s is not.

**`errors` derives only `Debug`.** Anything else must come from a native attribute, and **two
`derive` attributes on one item is legal Rust** — verified, and relied on twice in new code:
`#[derive(PartialEq, Eq, Hash)]` above an `errors` declaration for an id_effect boundary type,
and `#[derive(Clone)]` for a type a `Cache` clones to its waiters. That is the escape hatch this
row leaves in place rather than replacing with an option on `errors`.

**`#[cfg(test)]` on a Redox declaration passes through and works**, verified on a helper used
only by a `test` block. So attribute passthrough is sound on the declarations this row touches.

**Corpus scale.** 29 examples; `redox: allow` pragmas appear in **2** of them across **4**
sites — `ascription.sdx` once and `fallible.sdx` three times, which is exactly what plan 38 §10
records (*"the four sites carry the pragma"*). A fifth site is a test fixture,
`crates/redox-cli/tests/lint/unwrap_suppressed.sdx`, and migrates with them.

---

## 3. Concrete proof this plan targets

1. **`Err (Shape raw)` compiles**, with `VersionError::Shape raw` still compiling beside it —
   the glob adds a spelling and removes none.
2. **`newtype Bad = Bad f64 f64` is a Redox diagnostic**, naming the one-field rule and
   suggesting `data`. A two-field `newtype` currently compiles, so this is a new refusal and the
   example corpus must carry the refused form as a `tests/ui/` fixture.
3. **A `newtype`'s generated Rust carries `#[repr(transparent)]`**, asserted in a golden — and
   the `data` form beside it does not, which is what makes the keyword mean something.
4. **`#[redox::allow(unwrap)]` suppresses `RDX1001`**, and the comment form is a *deprecation
   diagnostic* rather than a silent no-op, because 5 example files and an unknown amount of
   outside code use it.

---

## 4. Leaves

### Leaf 1 — `leaf-errors-glob`

`errors` emits `use T::*` alongside its enum, exactly as `data` does. One emitter change; the
shared declaration node makes it a small one.

### Leaf 2 — `leaf-newtype-shape`

The one-constructor-one-field check, as a parser-level refusal with a named code and a help line
pointing at `data`. No resolver, no types — the shape is visible in the declaration.

### Leaf 3 — `leaf-newtype-repr`

`#[repr(transparent)]` on the emitted struct. Note the interaction asserted in §5: `repr` and a
`deriving` clause coexist, and the attribute order must be deterministic for the goldens
(plan 04's determinism rule).

### Leaf 4 — `leaf-pragma-attribute`

`#[redox::allow(unwrap)]` recognised on a declaration and on a statement, **and consumed** — it
must not reach the `.xrs`, which is what makes `register_tool` irrelevant (§1.3). The namespaced
form is chosen so Redox's own attributes never collide with a Rust one, and so `#[allow(…)]`
continues to mean rustc's lint and nothing else. This leaf is where plan 18 §1.3's
attributes-are-trivia rule gains its one carve-out, and the rule's text is amended to say so
rather than left to contradict the code.

### Leaf 5 — `leaf-pragma-deprecation`

The comment form keeps working for this milestone and emits a deprecation diagnostic naming the
attribute. Removal is plan 64's, where the rest of the comment-carried semantics goes.

### Leaf 6 — `leaf-corpus-and-docs`

Migrate the 4 pragma sites plus the test fixture; add the `newtype` refusal fixture; amend
doc 03 §12.5, doc 03 §12.7 and plan 18 §1.3. **Three code sites carry the identical-lowering
claim and all three must move**: `examples/data.sdx`'s comment, `data_decl.rs`'s `is_newtype`
doc comment, and — load-bearing — the emit test
`newtype_and_data_agree_on_a_tuple_struct` in `crates/redox-emit/tests/data.rs`, which
**asserts the very equality leaf 3 breaks** and will fail until it is rewritten to assert the
difference.

---

## 5. Acceptance criteria

1. An `errors` variant is usable unqualified **and** qualified; both asserted in one example.
2. `newtype` with zero or two-plus fields is a named Redox diagnostic with a `data` suggestion,
   and a `tests/ui/` fixture holds the refusal.
3. A `newtype`'s golden contains `#[repr(transparent)]`; the equivalent `data` declaration's
   does not.
4. `newtype` composes with `deriving` and with a native `#[derive(…)]`, with deterministic
   attribute order asserted by the golden.
5. `#[redox::allow(unwrap)]` suppresses `RDX1001` on both a declaration and a statement.
6. The comment pragma still works and warns; the warning names the attribute spelling.
7. All 29 examples green; `construct_coverage` green with the two new spellings indexed.
8. No artifact claims `newtype` and `data` lower identically: `examples/data.sdx`,
   `data_decl.rs`'s doc comment, and `crates/redox-emit/tests/data.rs`'s
   `newtype_and_data_agree_on_a_tuple_struct` are all updated, the last to assert the
   `#[repr(transparent)]` difference instead.

---

## 6. Out of scope, and what it discloses

**`errors` gains no `deriving` clause.** The native attribute covers it, verified twice in §2,
and adding a second way to spell a derive would be the kind of redundancy this milestone is
removing. Disclosed because the asymmetry with `data` is now the *only* one left between the two
forms, and a reader will notice.

**The emit test is a migration target, not a casualty.**
`newtype_and_data_agree_on_a_tuple_struct` asserts today's behaviour correctly; leaf 6 rewrites
it to assert the new difference. A row that breaks a test and deletes it has removed the
evidence; a row that breaks a test and inverts it has moved the evidence.

**`#[repr(transparent)]` is a guarantee Redox now makes.** It constrains what a `newtype` may
wrap — a zero-sized or multi-field payload is rejected by rustc, not by Redox. That is the
division of labour working, and it means some `newtype` declarations that compile today will
fail after this row. The one-field check catches the common case at the Redox line; the rest is
rustc's.

**Nothing here touches `type`.** It is already Rust's alias and already honest.

**The comment pragma is not removed.** Two spellings coexist for the rest of the milestone. That
is a deliberate cost, paid because 5 examples and an unknown amount of outside code depend on
the old one, and plan 64 is where the family is cleaned up in one pass.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
goldens re-blessed and reviewed; the `newtype` refusal fixture in `tests/ui/`;
`construct_coverage` green.

---

## 8. Emerald reuse

None directly. Emerald had no `errors` construct and no `newtype`; its lint suppression was a
compiler flag rather than a source annotation, which is the design this row is avoiding — a
file-level flag cannot say *this* `unwrap` is intended.

The transferable item is negative and worth stating: Emerald's lint configuration lived outside
the source, so a suppression outlived the line it was written for. A per-site attribute cannot.

---

## 9. Decision log

1. **Make `errors` match `data`, rather than making `data` match `errors`.** Requiring
   qualification everywhere would be the more conservative change and would break
   `examples/data.sdx` and `examples/destructuring.sdx`, both of which use bare constructors as
   doc 18 §6 specifies. The asymmetry is the bug, and `data`'s side is the specified one.
2. **Keep `newtype` and make it mean something, rather than deleting it.** Deletion was the
   other real option. It loses a word Haskell readers expect for no gain, where the
   one-field-plus-`repr` version costs one check and one attribute and makes the keyword carry
   information. If it had required a resolver, the answer would have been deletion.
3. **Namespace the attribute as `redox::`, and consume it.** An unnamespaced
   `#[allow(unwrap)]` passed through makes rustc say `unknown lint` — plan 38 §1.3's first
   objection, and correct. Consuming a namespaced attribute answers both that and the
   `register_tool` objection at once, because nothing reaches rustc. The cost is plan 18's
   carve-out, which §1.3 argues is worth paying and leaf 4 records in the rule.
4. **Deprecate the comment pragma rather than remove it.** §6. Removal in the same row would
   make this a migration row, and it is not.
5. **Three corrections in one row.** Each is a few hours; each needs the full gate. Bundling is
   the cheaper arrangement and the three share a subject.

---

## 10. Result

*Not yet executed.*
