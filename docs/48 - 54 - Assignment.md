# Plan 54 — Assignment

Plan ID: `assignment`
Depends on: 52 (`type-colon`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

`:=` reassigns. `=` continues to bind, and only to bind.

Plan 20 §10.2 wrote this row's specification, in a result section, four years of plans ago in
project time and one milestone ago in fact:

> Plain reassignment appears in none of them. `counter = 2` is a binding wherever it is written.
> […] If the limit ever has to go, the spelling is the decision, not the implementation: a
> distinct token (`:=`) costs one lexer arm and no analysis, which is the shape of answer this
> project can afford. That belongs in a surface-design doc, not in a plan.

This row is that decision, taken, with the surface-design note written into doc 03 and the lexer
arm added. The reason the limit has to go is in §1.2.

### 1.1 The current behaviour is the only silent defect in the language

Measured across the new corpus — 12 programs, 1,501 lines, 905 of them code: of sixteen
constructs that refuse or reinterpret, **fifteen are caught by something** — eleven by a named
Redox diagnostic, four by rustc on the generated code. Exactly one is caught by nothing:

```
mut i = 0
i = j            -- emits `let i = j;`  -- a new binding, shadowing
```

Plan 20 §10.1 found the loop form of it first — `while n < 3 { let n = n + 1; }`, an infinite
loop that compiles — and closed that case with compound assignment. What remains is everything
compound assignment cannot express.

### 1.2 What `+=` cannot express, and why that is not a corner

Plan 20 §10.2's disclosure is precise: *"a loop can advance a numeric accumulator and cannot
rebind anything else."* Three shapes fall outside it, and all three are ordinary:

| Shape | What you must write today |
|---|---|
| A swap | `a += b - a` then `b += tmp - b`, with `tmp` bound first |
| A conditional update | `if n > best` then `best += n - best` |
| Moving a cursor | accumulate into a `String` char by char rather than `i = j` |

The third is from a real program. A tokeniser scanning a run of digits wants `i = j` after
finding the end of the run; the written form accumulates the text and advances `i += 1` inside
the inner loop, because the natural spelling silently shadows. The code is fine. The reason it
is shaped that way is not visible in it, which is the cost.

The first two are worse than verbose — `best += n - best` is arithmetic standing in for
assignment, and it is only correct because `best` is numeric. For a non-numeric binding there is
no workaround at all.

### 1.3 `:=` and not the alternatives

Three other spellings were weighed.

**`=` assigns when the name is an existing `mut` binding.** Shortest, reads best, and requires
knowing whether a name is already bound — scope tracking. AGENTS.md: *every check rustc can
make, rustc makes*, and Redox has no scope table. Plan 20 §10.2 refused this on exactly that
ground and the refusal still holds.

**`set i = j`.** A word rather than a sigil, greppable, and `=` untouched. Declined on weight:
it is the only statement form in the language that would need a keyword to say something every
other language says with punctuation, and the milestone is already adding `is`, `>|`, `|<` and
`\->`.

**Stay assignment-free and make the shadow an error.** The declarative option, and the one
closest to Haskell. Declined because Redox has `mut`, `while`, `until`, `for` and compound
assignment already — the language is not pure, and refusing the one operation that makes its
existing mutation vocabulary usable would be a purity it does not have anywhere else.

`:=` is one lexer arm, no analysis, and carries its meaning to anyone who has met Pascal, Go,
Algol or pseudocode. It is what plan 20 §10.2 named.

### 1.4 Shadowing stays available, and stays spelled `=`

`i = j` continues to mean a new binding. That is deliberate and it is the reason this row is
cheap: nothing about the existing statement grammar changes, and no program written before this
row changes meaning.

The cost is that the silent case remains *expressible*. Someone who means to assign and writes
`=` still shadows. What changes is that the correct spelling now exists, so the diagnostic in
§4's leaf 3 can name it — and a lint can see the shape that was previously invisible.

---

## 2. Verified preconditions

Measured 2026-11-10.

**`i = j` emits a shadowing `let` and nothing complains.** The only signal is a downstream
`unused variable` warning on the *outer* binding, pointing at the declaration rather than the
assignment:

```
mut i = 0
...
i = j
->  let i = j;          // rustc: unused variable: `i`  (on the outer one)
```

**Compound assignment works, on a binding and on a field.** `n += 1`, and doc 18 §2.5's
`increment self = self.value += 1`. Five operators, per plan 20.

**A swap requires three statements and arithmetic.** Verified running:

```
mut a = 1
mut b = 2
mut tmp = a
tmp += 0
a += b - a
b += tmp - b          -- prints "2 1"
```

**`:=` is unused in every position.** Absent from `operator_table.rs` and from `TokenKind`, so
the lexer arm is additive and collides with nothing. `:` is **already** a token — plan 52 gives
it a type-annotation *role*, it does not create it — and longest-match ordering is already
enforced by `scan_operator`'s `longer_operators_are_tried_before_their_prefixes`. The dependency
on plan 52 is about the role, not about tokenhood.

---

## 3. Concrete proof this plan targets

**A swap, written as a swap, in an example that runs.**

```
mut a = 1
mut b = 2
tmp = a
a := b
b := tmp
```

Three cases chosen because they fail differently:

1. **The swap.** Prints `2 1`. The arithmetic workaround prints the same thing, so only the
   source distinguishes them — which is why the example carries both and a comment saying why.
2. **A conditional update of a non-numeric binding.** `best := name` inside a loop over
   `Vec<String>`. There is no `+=` for this, so it is the case that is currently *impossible*
   rather than merely awkward.
3. **`:=` on a name that is not `mut`.** rustc rejects it — `cannot assign twice to immutable
   variable` — and that is the division of labour working. Asserted as a `tests/ui/` fixture so
   the message a reader meets is a known one.

---

## 4. Leaves

### Leaf 1 — `leaf-lexer`

`:=` as a token. One arm, longest-match against `:` from plan 52.

### Leaf 2 — `leaf-assignment-statement`

The statement form, and its emission as Rust's `=`. Target is a place expression: a name, a
field, an index, a deref — all four, because `*slot := v` and `row[i] := v` are the shapes a loop
actually needs and omitting them would leave the row half-delivered in the way plan 20 §10.1
describes.

### Leaf 3 — `leaf-shadow-lint`

A lint on a bare `=` that rebinds a name already bound **in the same statement list**. This is
not scope tracking: it is one list, in the parser, with no nesting and no resolution — which is
why it is affordable where the `=`-assigns option in §1.3 was not. It suggests `:=`, and it is a
warning rather than an error because shadowing is legal and sometimes meant.

### Leaf 4 — `leaf-corpus`

`examples/control.sdx` gains the swap and the non-numeric conditional update. Plan 20 wrote that
file *within* the limit this row removes, and the record of the limit is **not** in the file — it
is plan 20 §10.2's prose and the emit test `a_plain_equals_is_still_a_binding` in
`crates/redox-emit/tests/control.rs`. That test asserts the boundary this row moves, so it is a
migration target: rewritten to assert that `=` still binds **and** that `:=` assigns, which is
the boundary after this row rather than before it.

### Leaf 5 — `leaf-docs`

Doc 03 gains the surface-design note plan 20 §10.2 asked for. Doc 01 §4.3 and doc 18 §2.5 are
amended to show both operators. `docs/GRAMMAR.md` **gains** a `SUGARED` row for assignment — it
has none today, only `let` bindings and shadowing — so `docs/STATUS.md`'s gap table does not
move, because that table is derived from `UNREACHABLE` rows and assignment was never one.

---

## 5. Acceptance criteria

1. `name := e` emits Rust's `name = e`, for a name, a field, an index and a deref target.
2. A swap and a non-numeric conditional update each appear in a running example with asserted
   stdout.
3. `:=` against a non-`mut` binding is a `tests/ui/` fixture carrying rustc's message.
4. `i = j` still binds, and the new lint fires on it with a `:=` suggestion.
5. The lint does **not** fire across statement lists — a `=` inside a nested block shadowing an
   outer name is legal and silent, asserted, because the alternative is scope tracking.
6. Compound assignment is unchanged; all five operators — `+=`, `-=`, `*=`, `/=`, `%=` — still
   work, asserted.
7. `a_plain_equals_is_still_a_binding` is rewritten to assert both halves of the new boundary,
   not deleted.
7. `construct_coverage` green with `:=` indexed.
8. Plan 20 §10.2's disclosure is marked resolved in doc 04's register, naming this row.

---

## 6. Out of scope, and what it discloses

**The silent case is narrowed, not closed.** `=` still shadows, by design (§1.4). The lint
covers one statement list; a shadow across a block boundary is invisible to it and legal. That is
the honest residue, and the alternative — full scope tracking — is the thing AGENTS.md forbids
and plan 20 §10.2 already refused.

**No destructuring assignment.** `(a, b) := (b, a)` is Rust's and would be the natural way to
spell a swap. It is declined here because plan 33 made destructuring a *binding* form, and
giving the same patterns an assignment meaning doubles the pattern grammar's interpretations for
one idiom. Recorded as a candidate, not a gap.

**No `+=` on the new token.** `:+=` is not a thing. The five compound operators keep `=`,
because they already mean assignment and always did — this row does not need to make them
consistent with a token that exists to disambiguate from binding.

**Expression-position assignment is plan 55's.** Under expression orientation `a := b` is an
expression of type `()`, as it is in Rust. Here it is a statement, which is all the current
grammar has a place for.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted; the
non-`mut` refusal in `tests/ui/`; `construct_coverage` green.

---

## 8. Emerald reuse

Emerald had assignment from the start and therefore never recorded why. The reusable item is
Redox's own: **plan 20 §10.1's finding that a row's scope can be smaller than the thing it
claims to deliver** — a `while` whose body cannot change its condition is not a loop — and that
is the fourth such case plan 20 counted.

This row is the fifth instance of the same lesson and the first to be *planned* from it rather
than discovering it at the gate. The disclosure plan 20 wrote is why.

---

## 9. Decision log

1. **`:=`, as plan 20 §10.2 named it.** The alternatives are in §1.3; two cost analysis Redox
   must not grow, and one costs a keyword for no gain.
2. **`=` keeps shadowing.** Changing it would make every existing program's `=` ambiguous and
   would require the scope table that rules out the `=`-assigns design.
3. **Place expressions, all four, in one leaf.** Plan 20 §10.1's lesson applied before the gate
   rather than after it: a row that ships assignment-to-names-only has shipped something a loop
   cannot use.
4. **The lint is one statement list, deliberately.** It catches the common case with no
   resolution. Extending it would be the first step toward the thing AGENTS.md forbids, and the
   boundary is asserted by criterion 5 so a future row cannot drift across it quietly.
5. **No destructuring assignment.** §6. One idiom is not worth doubling the pattern grammar's
   meanings.

---

## 10. Result

*Not yet executed.*
