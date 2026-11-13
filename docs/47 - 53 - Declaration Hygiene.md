# Plan 53 — Declaration Hygiene

Plan ID: `declaration-hygiene`
Depends on: 52 (`type-colon`)
Milestone: 9 — *surface coherence*
Status: `done`

---

## 1. What this plan is

Three small corrections to the declaration forms, each of which is a thing the language says and
does not mean.

| Today | The problem | This row |
|---|---|---|
| `data` emits `use T::*`; `errors` does not | two declaration forms, one use-site rule between them | `errors` emits it too |
| `newtype` and `data` lower identically | a keyword implying a distinction it does not make | one field enforced, `#[repr(transparent)]` emitted |
| `-- voltaic: allow unwrap` | semantics in trivia a formatter can delete | a **consumed** `#[voltaic::allow(unwrap)]` |

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
- `crates/voltaic-ast/src/program/item/data_decl.rs`, on the `is_newtype` flag: *"Kept even though
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
>   passed through makes rustc say `unknown lint`, so a program that suppressed a Voltaic lint
>   would stop compiling.
> - Consuming it instead — recognising one attribute and dropping it — makes plan 18's rule
>   conditional. […] a carve-out for a single spelling weakens the rule that made nine item
>   kinds work at once.
> - `#[voltaic::allow(unwrap)]` needs `register_tool`, which is unstable.

The first and third reasons **do not apply to a consumed attribute**, and that is the whole of
the design change. If Voltaic recognises `#[voltaic::allow(unwrap)]` and **does not emit it**, then
rustc never sees it: no `unknown lint`, and no `register_tool`, because the attribute does not
reach the compiler that would need the feature. Plan 38 listed consumption as the second option
and then priced the third option's instability against it as though they were the same route;
they are not.

So the real cost is the second reason alone: **plan 18 §1.3's attributes-are-trivia rule gains
exactly one carve-out.** That is a genuine weakening and this row pays it knowingly.

What makes it worth paying is evidence plan 38 did not have. Plan 48 — eleven rows later — found
`clause.rs`'s four field doc comments **deleted from the `.sdx` by `voltaic fmt`**, because the
lexer's `scan_dashes` misclassified a comment opening a continuation line. AGENTS.md calls
comment preservation load-bearing; the formatter broke it anyway, in the tool whose promise is
not changing the program.

Plan 38's bet was that a comment *"reaches nothing downstream"* and is therefore safe. Plan 48
showed the comment is not safe **upstream** — it can be deleted from the source. A suppression
that a formatter run can silently remove turns a deliberate pragma into a surprise `RDX1001`, or
worse, removes the record of why a panic was acceptable. One conditional rule is cheaper than
that.

It also makes the pragma consistent with where the milestone is going: plan 64 deletes the
keyword family in favour of Rust's macros, and `#[voltaic::allow(…)]` is the attribute-shaped
member of the same family.

---

## 2. Verified preconditions

Measured 2026-11-10.

**The `errors` asymmetry is real and reaches rustc, not the Voltaic diagnostics.** A bare variant
emits a bare path and fails with `E0425: cannot find function, tuple struct or tuple variant`,
pointing into generated Rust. `data`'s glob is emitted; `errors`'s is not.

**`errors` derives only `Debug`.** Anything else must come from a native attribute, and **two
`derive` attributes on one item is legal Rust** — verified, and relied on twice in new code:
`#[derive(PartialEq, Eq, Hash)]` above an `errors` declaration for an id_effect boundary type,
and `#[derive(Clone)]` for a type a `Cache` clones to its waiters. That is the escape hatch this
row leaves in place rather than replacing with an option on `errors`.

**`#[cfg(test)]` on a Voltaic declaration passes through and works**, verified on a helper used
only by a `test` block. So attribute passthrough is sound on the declarations this row touches.

**Corpus scale.** 29 examples; `voltaic: allow` pragmas appear in **2** of them across **4**
sites — `ascription.sdx` once and `fallible.sdx` three times, which is exactly what plan 38 §10
records (*"the four sites carry the pragma"*). A fifth site is a test fixture,
`crates/voltaic-cli/tests/lint/unwrap_suppressed.sdx`, and migrates with them.

---

## 3. Concrete proof this plan targets

1. **`Err (Shape raw)` compiles**, with `VersionError::Shape raw` still compiling beside it —
   the glob adds a spelling and removes none.
2. **`newtype Bad = Bad f64 f64` is a Voltaic diagnostic**, naming the one-field rule and
   suggesting `data`. A two-field `newtype` currently compiles, so this is a new refusal and the
   example corpus must carry the refused form as a `tests/ui/` fixture.
3. **A `newtype`'s generated Rust carries `#[repr(transparent)]`**, asserted in a golden — and
   the `data` form beside it does not, which is what makes the keyword mean something.
4. **`#[voltaic::allow(unwrap)]` suppresses `RDX1001`**, and the comment form is a *deprecation
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

`#[voltaic::allow(unwrap)]` recognised on a declaration and on a statement, **and consumed** — it
must not reach the `.xrs`, which is what makes `register_tool` irrelevant (§1.3). The namespaced
form is chosen so Voltaic's own attributes never collide with a Rust one, and so `#[allow(…)]`
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
`newtype_and_data_agree_on_a_tuple_struct` in `crates/voltaic-emit/tests/data.rs`, which
**asserts the very equality leaf 3 breaks** and will fail until it is rewritten to assert the
difference.

---

## 5. Acceptance criteria

1. An `errors` variant is usable unqualified **and** qualified; both asserted in one example.
2. `newtype` with zero or two-plus fields is a named Voltaic diagnostic with a `data` suggestion,
   and a `tests/ui/` fixture holds the refusal.
3. A `newtype`'s golden contains `#[repr(transparent)]`; the equivalent `data` declaration's
   does not.
4. `newtype` composes with `deriving` and with a native `#[derive(…)]`, with deterministic
   attribute order asserted by the golden.
5. `#[voltaic::allow(unwrap)]` suppresses `RDX1001` on both a declaration and a statement.
6. The comment pragma still works and warns; the warning names the attribute spelling.
7. All 29 examples green; `construct_coverage` green with the two new spellings indexed.
8. No artifact claims `newtype` and `data` lower identically: `examples/data.sdx`,
   `data_decl.rs`'s doc comment, and `crates/voltaic-emit/tests/data.rs`'s
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

**`#[repr(transparent)]` is a guarantee Voltaic now makes.** It constrains what a `newtype` may
wrap — a zero-sized or multi-field payload is rejected by rustc, not by Voltaic. That is the
division of labour working, and it means some `newtype` declarations that compile today will
fail after this row. The one-field check catches the common case at the Voltaic line; the rest is
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
3. **Namespace the attribute as `voltaic::`, and consume it.** An unnamespaced
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

**Done.** `errors` brings its variants into scope; `newtype` is
`#[repr(transparent)]` and is refused unless it is one constructor over one
field; `#[voltaic::allow(unwrap)]` suppresses `RDX1001` on a declaration and on a
statement, and the comment pragma warns with `RDX1002`.

### 10.1 Criterion 4 could not be met as written, and rustfmt is why

Criterion 4 asked for a native `#[derive(...)]`, a `deriving` clause and
deterministic attribute order **asserted by the golden**. The order is
deterministic and the combination compiles. It cannot appear in an example,
because rustfmt's `merge_derives` defaults to true:

```text
#[derive(PartialEq)]       becomes   #[derive(PartialEq, Debug, Clone)]
#[derive(Debug, Clone)]              #[repr(transparent)]
#[repr(transparent)]                 struct Tally(u32);
struct Tally(u32);
```

and `generated_rust_is_rustfmt_clean` runs over `examples/`. Emitting one
merged `#[derive]` instead would mean reading `derive` out of **verbatim
attribute trivia** and splicing the `deriving` names into it — exactly the
per-attribute work doc 03 §13.2's design exists to avoid, on the one attribute
Voltaic already knows about. The order is what the criterion was for, and
rustfmt-stability is a stronger property this one combination does not have.

So the criterion is split: `examples/attributes.sdx` pins the
`deriving`-then-`repr` order in a golden, and
`crates/voltaic-emit/tests/attributes.rs` pins all three attributes where rustfmt
does not run. Both say why in place.

Worth recording that the ordering half was **already true and already correct**
before this row — `emit_data` emits trivia, then the `deriving` derive, then
the struct — and that no test combined the two spellings, which is why nobody
knew.

### 10.2 `newtype` was unvalidated in four shapes, not two

§1 and criterion 2 named the field count. The parser read `is_newtype` *nowhere*
and `constructor_fields` loops unbounded, so four shapes were accepted:

| Probe | Emitted at HEAD | Diagnostic |
|---|---|---|
| `newtype Bad = Bad f64 f64` | `struct Bad(f64, f64);` | none |
| `newtype Zero = Zero` | `struct Zero;` | none |
| `newtype Lone` (no `=` row) | `struct Lone;` | none |
| `newtype Shade = Light \| Dark` | `enum Shade { … }` | none |

The last two were not in the plan. A `newtype` lowering to an **enum** is as
wrong as one lowering to a two-field tuple, and `newtype Lone` would have been
a hole straight through a field-count-only check. All four are `RDX0111`, one
code with three messages — a code names a rule, and there is one rule and one
remedy. Ordering is load-bearing: the constructor count is checked first, or
`newtype Shade = Light | Dark` reports `Light`'s missing field, which is true
and the wrong diagnosis.

**And `#[repr(transparent)]` gave the multi-field case a rustc backstop it did
not have.** §11.3 measured that a unit struct with `repr(transparent)` compiles
clean while two fields are `E0690` — so leaf 3 turned one of the four shapes
into something rustc also catches. Zero fields, zero constructors and the bare
`newtype Lone` are caught here or not at all.

### 10.3 Criterion 1 was a build requirement, not an illustration

Leaf 1 is described as one emitter change. Adding `use ConfigError::*;` while
`examples/errors.sdx` still qualified every variant is an `unused import`
warning, and `support.rs` asserts rustc emits nothing for an example. So the
example had to gain an unqualified use in the same leaf. It now exercises both
spellings deliberately — `Missing` bare and `ConfigError::Invalid` qualified —
because the glob must *add* to the long form rather than replace it: a program
with two error types in scope needs the long one.

Also corrected: §1's table implied `data` always emits the glob. It emits it in
the **enum branch only**, and `a_struct_brings_nothing_into_scope` asserts the
absence for a struct. The comparison holds for the case that matters, `errors`
being always an enum.

### 10.4 Leaf 4 was not one match arm — the suppression *key* was wrong

`unwrap_call_sites` keyed suppression on the **trivium's own line**. An
attribute can only sit on the line above what it applies to, so the key could
not work for one. Measured before building anything: a *comment* pragma on the
line above did not suppress either.

The fix splits the lint in two, and the split is the interesting part:

- **Detection stays lexical.** `unwrap_call_sites`'s doc comment argues that an
  `Expr` walker which forgets a variant **silently misses call sites** and
  nothing in the suite would notice. That argument stands.
- **Suppression walks the tree**, because it needs construct spans. The
  asymmetry is what makes it safe: a suppression walk that forgets a variant
  makes a warning **fire that should have been silenced** — loud, not silent —
  which is the opposite failure direction, and is why the argument against
  walking does not transfer.

`lint` therefore takes a `Program`. It takes an **`Option<&Program>`**, which
is a correction to §4: three of the four call sites had one parsed, and
`voltaic-lsp`'s `serve/diagnostics.rs` deliberately discards it — its doc says a
lint must still run on a document that does not parse. A non-optional
parameter would have meant a behaviour regression or a second parse.

Two smaller findings in the same leaf. The four pragma sites in `examples/`
were all **trailing** comments, so every one of them worked under the old key
and none demonstrated the bug — converting them to attributes is what makes
the new key load-bearing in the corpus. And `allow_pragma.rs` cited a test,
`an_unrelated_comment_does_not_suppress`, that did not exist anywhere;
`voltaic-driver` had no `tests/` directory at all. It exists now.

Whitespace inside the attribute is **not** normalised: `#[voltaic :: allow(
unwrap )]` is refused. A spaced `::` is `RDX0110` everywhere else after plan
52, so there is no admitted spelling to be generous to, and an unrecognised one
fails loudly — it is not consumed, reaches rustc, and `E0433` names it on the
author's line.

### 10.5 Five artifacts claimed something that is no longer true

§8 named three; §11.7 found three more, and all six are corrected rather than
deleted. Three said `newtype` and `data` lower identically — true until leaf 3.
Three more said a `newtype` is one-field **"by construction"**, which was not
true of the code at all until leaf 2: `crates/voltaic-fmt/src/item_text/data_text.rs`
relied on it to skip a multi-line shape, and `newtype Multi = A | B` falsified
it outright.

Two of those three were written *ahead* of the check, in the past tense, and
now match the code — which is the hazard of describing an invariant a plan
intends rather than one the code keeps.

### 10.6 Criterion 7's second clause had nothing to index

*"`construct_coverage` green with the two new spellings indexed."* The test
indexes `docs/GRAMMAR.md`'s rows, and that document grades **Rust**
constructs — which is what plan 46 built it to do. Both of this row's new
spellings are Voltaic-only: `#[voltaic::allow(unwrap)]` has no Rust counterpart,
and `#[repr(transparent)]` is not a construct a Voltaic program writes, it is
something `newtype` emits. So there was no row to add and the clause is
vacuous as written.

What it *did* surface is a note that had gone stale. The `Attributes, inner and
outer | PASSTHROUGH` row reads *"Rust's own `#[…]`"*, and after leaf 4 there is
exactly one attribute that is **not** passed through: `#[voltaic::allow(unwrap)]`
is consumed by `emit/trivia.rs` and never reaches the `.xrs`. The grade is
still correct — one consumed attribute does not make the construct anything
but passthrough — and the note now says which one and why.

### 10.7 Open

- `docs/GRAMMAR.md:62`'s tuple-struct row wording is now stale about
  `newtype`. `grammar_inventory` only checks the cited file exists, so it will
  not catch it; plan 51's `construct_coverage` indexes the row, so it has a
  second reader.
- The comment pragma is deprecated, not removed. `RDX1002`'s help says so:
  *"the comment still suppresses `RDX1001` for this milestone; the attribute is
  what plan 64 keeps."*
- `crates/voltaic-lsp/tests/session.rs` embeds `.sdx` source as Rust string
  literals, which AGENTS.md forbids and `no_literal_expectations` does not
  catch (it guards *expectations*). Not this row's to convert, but counted.

---

## 11. Corrections from orientation

Written after reading the code this row targets, before implementing it. The
three core claims hold: `errors` emits no glob, `newtype` and `data` lower
identically with no validation whatsoever, and
`newtype_and_data_agree_on_a_tuple_struct` exists and says what §1 quotes. Six
things do not, and four of them change what a leaf has to do.

### 11.1 Criterion 1 is load-bearing, not illustrative

Leaf 1 is described as one emitter change. It cannot be. Adding
`use ConfigError::*;` to the generated Rust while `examples/errors.sdx` still
qualifies every variant produces

```
warning: unused import: `ConfigError::*`
```

and `crates/voltaic-cli/tests/support.rs`'s `run()` asserts
`out.stderr.is_empty()` — *"rustc should emit nothing for generated code"*. So
`examples/errors.sdx` **must** gain an unqualified use in the same leaf or
`every_example_compiles_and_runs` goes red. Criterion 1's "both asserted in one
example" is a build requirement, not a nicety.

Scale, for calibration: there is exactly **one** `errors` declaration in the
whole `.sdx` corpus, `examples/errors.sdx:9`.

### 11.2 §1's table row overstates what `data` does

`data.rs:94-95` emits `use T::*;` in the **enum branch only** — the struct
branch returns at `data.rs:60`, and `a_struct_brings_nothing_into_scope`
asserts `!out.contains("use ")` for it. `data Meters = Meters f64` emits no
glob either. The comparison still holds for the case that matters, because
`errors` always lowers to an enum, but the row as written implies a rule `data`
does not have.

The right emission for leaf 1 is therefore `data.rs:94-95` verbatim: a
module-level, **private** `use T::*;` at the declaration's indent. Not
`use Self::*` in an impl — the `Display` arms already write fully-qualified
`T::V`, and a glob inside an impl would not reach user code. Not `pub use` —
`data`'s is plain, and `pub use` would re-export the variants.

### 11.3 §6's rustc backstop does not exist for the zero-field case

§6 says a zero-sized or multi-field `newtype` payload "is rejected by rustc,
not by Voltaic". Measured:

```rust
#[repr(transparent)] pub struct Unit;                   // compiles clean
#[repr(transparent)] pub struct Two(pub f64, pub f64);  // error[E0690]
```

Only the multi-field half has a backstop. For `newtype Foo = Foo` the Voltaic
check in leaf 2 is the **only** thing that catches it, so §6's "the rest is
rustc's" needs qualifying and criterion 2's zero-field half is not redundant
with rustc.

### 11.4 A `newtype` can lower to an enum today, which §1 does not mention

Measured at HEAD:

| Probe | Emitted | Diagnostic |
|---|---|---|
| `newtype Bad = Bad f64 f64` | `struct Bad(f64, f64);` | none, exit 0 |
| `newtype Zero = Zero` | `struct Zero;` | none, exit 0 |
| `newtype Multi = A \| B` | `enum Multi { A, B }` + `use Multi::*;` | none, exit 0 |

`data_decl.rs:43` sets `is_newtype` and **never consults it again**;
`constructor_fields.rs:40-47` loops `while starts_a_field_type(...)` unbounded.
The Haskell rule is one constructor with one field, and the
**constructor-count** half is as unenforced as the field-count half. Leaf 2 and
criterion 2 mention only arity; both halves belong in the same check.

### 11.5 Criterion 4's ordering half is already true

`data.rs:27-33` emits `emit_trivia(&d.leading)` first, then the
`deriving`-derived `#[derive(...)]`, then the struct line. Native attributes are
`TriviaKind::Attribute` trivia printed verbatim, so

```rust
#[derive(Clone)]
#[derive(Debug)]
struct M(f64);
```

is already deterministic and already compiles. rustfmt does not reorder
attributes. So criterion 4 needs a **test** and no production code — no
existing test combines a native attribute *with* a `deriving` clause, which is
why nobody knew. Only the `repr` interaction is new work.

### 11.6 Leaf 4 is larger than "recognise one more trivium kind"

`unwrap_call_sites.rs` keys suppression on `line_of(source, trivium.span.0)` —
the **trivium's own line**. A Rust attribute can only sit on the line *above*
the statement it applies to. Measured:

```text
main : ()
main =
  raw = Some 7
  -- voltaic: allow unwrap
  port = raw.unwrap ()
```

→ `RDX1001` **still fires**. So the suppression key must change from the
trivium's line to the line of the construct the trivium leads, and the
declaration-level case of criterion 5 needs a **span range**, which the current
model has no notion of at all. That is two changes to
`unwrap_call_sites.rs`, not one match arm in `allow_pragma.rs`.

Two smaller findings in the same area. The pragma's exact text is
`allow_pragma.rs:24`, `trivium.text.trim() == "voltaic: allow unwrap"` — a plain
`LineComment` only, so a doc comment never suppresses. And
`#[voltaic::allow(unwrap)]` reaching rustc is `error[E0433]: cannot find module
or crate 'voltaic'`, a hard error rather than an unknown lint — which strengthens
plan 38 §1.3's must-consume argument rather than weakening it. `register_tool`
appears nowhere in `crates/`; plan 38 refused it and the refusal held.

Supporting §1.3's evidential argument, verified: `voltaic-fmt`'s
`stmt_text/trivia_text.rs:27` already prints `TriviaKind::Attribute` back as
`#[…]`, so the formatter round-trips an attribute and the parser needs no
change — attributes are already trivia on every item and statement.

### 11.7 Two counts, and a criterion that cannot be checked yet

- **Pragma sites: 2 examples, 4 sites, plus one test fixture.**
  `examples/ascription.sdx:25`, `examples/fallible.sdx:124`, `:125`, `:127`,
  and `crates/voltaic-cli/tests/lint/unwrap_suppressed.sdx:14`. §2 has this
  right; §6 and criterion 4 both say "5 examples", which is internally
  inconsistent with §2.
- **Leaf 6 undercounts the artifacts by three.** Beyond the three §8 names,
  these claim *one field by construction*, which is also false today and
  becomes true only after leaf 2:
  `crates/voltaic-fmt/src/item_text/data_text.rs:30-31` (falsified outright by
  the `Multi` probe above), `crates/voltaic-lsp/src/serve/symbols/symbol_of.rs:26`,
  `crates/voltaic-lexer/src/token/keyword.rs:33`.
- **Criterion 3's second half has no existing basis.** `examples/data.sdx` has
  `newtype Meters` but its only `data` is the `Event` enum; there is no
  single-field `data` tuple struct anywhere in the corpus to contrast against.
  It needs a new declaration whose field is actually read, because of the
  zero-rustc-output rule. The two `newtype`s in the corpus are
  `examples/data.sdx:26` and `examples/fallible.sdx:19`, both legal under
  `repr(transparent)`, both forcing a golden re-bless.
- **Criterion 2 needs a `tests/ui/` fixture to exist at all**, not just for
  completeness: `code_coverage.rs`'s `every_code_is_reached_by_the_corpus` has
  `NOT_YET_REACHABLE = &[]`, so a declared `Code` with no fixture fails the
  gate.

Also in scope but unlisted: `docs/GRAMMAR.md:62`'s tuple-struct row is about to
become wrong in its wording. `grammar_inventory` only checks the cited file
exists, so it will not catch it — but plan 51's `construct_coverage` now
indexes that row, so the two documents have a second reader.
