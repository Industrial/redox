# Plan 33 — Destructuring

Plan ID: `destructuring-and-splat`
Depends on: 21 (`switch-and-patterns`), 22 (`is-and-guard`), 29 (`comprehensions-and-collections`)
Milestone: 6 — *the language, completed*
Status: `planned`

---

## 1. What this plan is

The rest of the pattern grammar: **tuple**, **slice** and **record** patterns, the
**as-pattern**, and the **rest pattern**. Doc 03 §6.4 and §6.5.

```text
(a, b) = point                    let (a, b) = point;
[first, .., last] = row       →   let [first, .., last] = row;
[head, rest @ ..] = row           let [head, rest @ ..] = row;
Point { x, y } = p                let Point { x, y } = p;
Point { x = 0, y } -> y           Point { x: 0, y } => y,
whole @ Move x y -> whole         whole @ Move(x, y) => whole,
```

Every one of these *"pass[es] through to native Rust syntax"* (§6.4). Plan 22 added
`Pattern::Apply` because no row owned it; this row owns everything else.

### 1.1 The row as doc 04 wrote it, and what doc 18 did to it

Row 33 reads: *"Tuple, slice and struct destructuring; rest patterns; splat in argument
position."*

| Named by the row | Doc 18 | What does its work |
|---|---|---|
| Tuple, slice, struct destructuring | kept | this plan |
| Rest patterns | kept — but **not** spelled `...` | `..` and `rest @ ..`, Rust's own; see §1.3 |
| Splat in argument position | §7's table: *"`...` forwarding — Nothing. `RDX0048` goes; `...` leaves the language"* | **deleted** |

And two things doc 03 §6 specifies are deleted rather than deferred, both by doc 18 §2.3:
**anonymous record patterns** (`{active: true, age: > 18}`) and with them `RDX0042` and
`RDX0043`. Guards do that work — §1.5.

### 1.2 Record patterns take `=`, not `:`

Doc 03 §6.3 writes `Point{x: 0, y}`. Doc 18 §2.4 changed the record *literal* to `=` —
`Point { x = 10, y = 20 }` — and §5.2 made a spaced `::` the type marker, which leaves `:`
with no job inside braces at all.

So a record **pattern** is `Point { x = 0, y }`, matching the literal character for
character, and lowering to Rust's `Point { x: 0, y }`. The alternative — a literal spelled
`=` and a pattern spelled `:` — would be two spellings of one shape differing only by
position, which is the kind of thing doc 18 §5.3 exists to remove.

Field punning is unchanged and needs no spelling: `Point { x, y }` binds both.

### 1.3 `...` is gone, so a rest pattern is Rust's own

Doc 03 §4.2 and §6.5 spell a rest pattern `...rest`:

```text
[first, ...rest] = values   →   let [first, rest @ ..] = values;
```

Doc 18 §7 removes `...` from the language. That is not a loss here, because the two tokens
Rust uses are both already Voltaic's: `..` is the range operator and `@` is the as-pattern,
which doc 18 §4.3 deliberately left as `@`'s *"exactly one meaning"*. So the spelling
becomes Rust's:

```text
[first, rest @ ..] = values   →   let [first, rest @ ..] = values;
[first, .., last] = values    →   let [first, .., last] = values;
```

One fewer token, one fewer lowering, and a pattern that reads the same in both languages.
**This is the third place doc 18's deletions turned out to pay for themselves rather than
cost** — after `print`'s dissolution and the comprehension's single form.

### 1.4 `..` in a pattern is not the range operator, and that is the one real hazard

`1..10` is a range pattern. `[a, ..]` is a rest pattern. `Point { x, .. }` is a struct rest.
The token is the same and the meaning is positional:

| Position | Meaning |
|---|---|
| Between two patterns | range — `1..10` |
| Alone in a slice or tuple pattern | rest |
| Alone in a record pattern | *"and other fields"* |
| After `name @` in a slice | named rest |

All four are decidable from the source text alone, because the distinction is *what
precedes it*: a pattern, or a delimiter. No lookahead and no backtracking — which is the
same property doc 18 §3.1 relies on for `<-`'s four positions.

### 1.5 Anonymous record patterns are deleted, and the replacement is better

Doc 03 §6.3's second form was field-access guards with no type path:

```text
if u is {active: true, age: > 18}   →   if u.active && u.age > 18
```

Doc 18 §2.3 deletes it along with the relational patterns it depended on. The replacement is
to write the condition, which is what it lowered to anyway:

```text
if u.active && u.age > 18
```

`RDX0042` and `RDX0043` go with it. A construct whose lowering is *shorter than itself* was
never carrying its weight, and doc 18 §2.3 says so.

### 1.6 Destructuring is a binding, and the binding parser is where it lands

`(a, b) = point` is doc 03 §6.5's destructuring, and it is a **binding whose left side is a
pattern** rather than a name. Today `Binding` holds an `Ident`.

So the one structural change this plan makes is that a binding's left side becomes a
`Pattern`. A plain `x = 1` is then a `Pattern::Binding`, which emits exactly what it emits
now — asserted, because that path is every binding in every existing golden.

---

## 2. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes, and §1.4 is the only place it needed arguing |
| Nothing outside core/alloc/std? | Yes. Patterns are rustc's own grammar |
| Preserves Rust semantics? | Yes — §6.4: *"All pass through to native Rust syntax"* |
| Buys reachability? | Yes. Tuple, slice and struct patterns had no spelling, so `(a, b) = f ()` could not be written at all |

---

## 3. Concrete proof this plan targets

`examples/destructuring.sdx`, which must **run** and print a sequence only a correct
lowering produces:

- a tuple destructuring whose two bindings are both used, so a lowering that bound one name
  to the pair fails;
- a slice pattern with a **middle** rest — `[first, .., last]` — over an array long enough
  that `first` and `last` differ from the second and second-to-last;
- a **named** rest — `[head, rest @ ..]` — with `rest.len()` printed, so a lowering that
  bound the whole slice gives a different number;
- a record pattern in a `case` arm **with a literal field**, so the arm tests as well as
  binds and a wrong lowering takes a different arm;
- a record pattern with `..`, over a type with more fields than the pattern names;
- an as-pattern whose bound name is used **beside** the bindings from the pattern it names,
  since that is the whole point of `@`;
- a nested pattern — a record inside a tuple inside a slice — because each form is
  independent and a lowering that worked only at the top level would pass every test above.

The slice-with-middle-rest case is the one a golden could not catch: `[a, .., z]` and
`[a, z]` differ only at run time, and only on an array of more than two elements.

---

## 4. Leaves

### Leaf 0 — `leaf-tuple-expression`

**Added during implementation, not planned.** `Expr::Tuple` — no row owned a tuple
*expression*, and without one a tuple pattern can be fed only by a Rust function that
already returns a tuple. §10.1.

### Leaf 1 — `leaf-tuple-and-slice`

`Pattern::Tuple { items }` and `Pattern::Slice { items }`, with `Pattern::Rest` and
`Pattern::NamedRest { name }` as elements. Emitted as Rust's.

### Leaf 2 — `leaf-record-pattern`

`Pattern::Record { path, fields, rest }`, with a field being a pun or a `name = pattern`
pair. Emitted as `Path { name: pattern, .. }`.

### Leaf 3 — `leaf-as-pattern`

`Pattern::As { name, inner }` — `whole @ Move x y`. Infix, so it reads after a name and
before a pattern, and `@`'s one remaining meaning is this.

### Leaf 4 — `leaf-destructuring-binding`

A binding's left side becomes a `Pattern`. A plain name still emits what it emits today.

### Leaf 5 — `leaf-proof-and-gate`

`examples/destructuring.sdx`, emit tests per form, a golden, a run expectation, and the gate.

---

## 5. Acceptance criteria

1. `(a, b) = p` emits `let (a, b) = p;`, and a nested tuple pattern works.
2. `[a, b] = xs` emits `let [a, b] = xs;`.
3. `[a, .., z]` emits `[a, .., z]` and `[a, rest @ ..]` emits `[a, rest @ ..]`.
4. `Point { x, y } = p` emits `let Point { x, y } = p;`, and `Point { x = 0, y }` emits
   `Point { x: 0, y }` — the `=`-to-`:` change is §1.2's.
5. `Point { x, .. }` emits `Point { x, .. }`.
6. `whole @ Move x y` emits `whole @ Move(x, y)`.
7. Every form works in **all three** pattern positions — a `case` arm, an equation's row
   (parenthesised, per plan 22 §10.2), and a `<-` condition — because the vocabulary is
   shared and sharing it is the point.
8. `x = 1` emits `let x = 1;` — asserted, because a binding's left side changing type must
   not change the path every existing golden takes.
9. `1..10` is still a range pattern and `..` alone is still a rest — §1.4's positional rule,
   asserted in both positions.
10. `...rest` is **not** accepted: `...` left the language with doc 18 §7. The error names
    the correction rather than the token.
11. `examples/destructuring.sdx` compiles with **zero** rustc output, runs, and matches its
    `.stdout`; a golden covers the text.
12. `(3, 4)` is a tuple **expression**, told from a group by its comma, and `(a,)` keeps
    Rust's trailing comma in both positions — asserted beside the group, because a test for
    either alone passes whatever the other does. *Added with leaf 0; see §10.1 and §10.3.*
13. Any new diagnostic has an `RDX` code reached by the UI corpus.

---

## 6. Out of scope, and what it discloses

- **Splat in argument position** is deleted by doc 18 §7 with `...`. Not deferred.
- **Anonymous record patterns**, `RDX0042` and `RDX0043` are deleted by doc 18 §2.3 — §1.5.
- **Exhaustiveness** stays rustc's, per doc 03 §6.6: *"Voltaic never synthesises a wildcard
  arm."* This plan adds pattern forms and no analysis, so a missing variant is still
  reported by rustc against the author's own arms.
- **Reference patterns** — `&x`, `ref x` — are not in doc 03 §6's grammar and are not added.
  Rust's match ergonomics make them rarely necessary, and a form with no specification is a
  form with no agreed meaning.
- **`|` as pattern alternation** stays `or`, for doc 18 §5.4's reason: the bar is the guard,
  unambiguously, and that is what `or` buys.
- **Binding a tuple field by index** — `p.0` on the left of a `=` — is not a pattern in
  Rust and is not one here.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None usable.** Emerald's destructuring bound into its own environment rather than lowering
to a host pattern, so the mechanism does not transfer. Its transferable lesson is the one
plan 22 already paid for: Emerald's pattern vocabulary grew one form per plan across eleven
plans, and each addition had to be threaded through every pattern position by hand. One
`Pattern` enum reached from one function in each position is what avoids that, and it is
already the shape here.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | A record pattern takes `=` | Doc 03 §6.3's `:` | Doc 18 §2.4 made the literal `=` and §5.2 left `:` no job in braces. Two spellings of one shape differing only by position is what §5.3 exists to remove |
| 2 | A rest pattern is `..` and `rest @ ..` | Doc 03 §6.5's `...rest` | `...` left the language with doc 18 §7, and both of Rust's tokens are already Voltaic's. One fewer token and one fewer lowering — §1.3 |
| 3 | `..`'s meaning is positional | A second token for the rest | All four positions are decidable from what precedes the token, with no lookahead — the same property `<-`'s four positions have. A second token would be a second thing to learn for no gain |
| 4 | A binding's left side becomes a `Pattern` | A separate destructuring statement | `(a, b) = p` *is* a binding; a second node would duplicate every binding behaviour. Criterion 8 guards the plain-name path |
| 5 | `...rest` gets its own diagnostic | Let it fail as an unexpected token | An author following doc 03 §6.5 wrote the published spelling. The message names the correction, as `RDX0106` does for doc 01 §4.3's module-scope binding |

---

## 10. Result

Shipped. 657 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit` are green. Every
criterion is met, and criterion 8 — the one guarding every existing golden — passed on the
first run: **not one golden changed** when a binding's left side became a `Pattern`.

### 10.1 A tuple *expression* had no owner either

`(a, b) = p` destructures a tuple, and nothing could build one. Doc 03 §6.5's example is
`(a, b) = get_coordinates!`, so a tuple *value* was always assumed to exist; §10.4's literal
table lists arrays, maps, sets and word arrays and **no tuple at all**; and no row's sentence
names one.

Found by writing `sum_pair (3, 4)` in the example — `expected ')' to close the group`. Without
it the whole row is only half reachable: a tuple pattern can be fed only by a Rust function
that already returns a tuple.

Implemented here, told from a parenthesised group by its **comma** — the same rule the tuple
*pattern* uses one position over, which is one rule in two places rather than two rules.

Sixth row whose scope was smaller than the thing it claimed to deliver, after plan 14's doc
03 §13.5, plan 16's `const fn`, plan 18's `use T::*`, plan 20's assignment and plan 22's
`Pattern::Apply`. The pattern across all six is unchanged and now unmistakable: **a surface
the specification uses in an example but no row's sentence names.**

### 10.2 `is_irrefutable` has to stay one rule, and rustc said so

The first version was clever: a tuple is irrefutable when every element is, a slice never is,
an as-pattern inherits from its inner. It compiled, and `first_of (a, _) = a` then failed with

```text
cannot find value `a` in this scope
```

Because that answer decides two things at once. `signature_from` lends a parameter its name
only for a `Pattern::Binding`, so a tuple pattern's parameter is `__rdx_a0` and `a` and `b`
come from the `match` arm `emit_clauses` builds — and calling the pattern irrefutable
**suppressed the `match`**, leaving the body referring to names nothing bound.

So it is `Binding | Wildcard` and nothing else. The cost is a `match` where a `let` would
have done, which rustc accepts with a single irrefutable arm. One rule that cannot produce an
unbound name is worth more than a second lowering that can, and the doc comment now says
which two questions the answer serves — because that coupling is what made the clever version
wrong.

### 10.3 A one-element tuple needs Rust's trailing comma

`(a)` is a parenthesised group; `(a,)` is a tuple. Joining elements with `", "` gives the
first for a one-element tuple, which turns a destructuring of a `(T,)` into a binding of the
whole value.

rustc catches it, because the types differ — but only because they differ. It was caught
here instead by `a_group_is_not_a_tuple`, which asserts the group and the tuple **side by
side**: a test for either alone passes whatever the other does. Both `Pattern::Tuple` and
`Expr::Tuple` emit the comma, and each carries the other's reason.

### 10.4 Two parser collisions, both from a token already in use

- **`pattern_app` intercepted the record pattern.** An uppercase head goes through
  `constructor_path`, which consumes `Point` before `pattern`'s identifier arm could see the
  `{` after it. The record form is therefore recognised in `pattern_app` as well, and `{` is
  deliberately absent from `starts_a_pattern_argument` — it opens a field list, not an
  argument.
- **A record *literal* must not read as a binding.** `p { x = 1 }` has an `=`, so the
  binding lookahead had to scan for one at **depth zero**; the literal's is inside its
  braces. `a_record_literal_is_not_read_as_a_binding` asserts it, and the scan is a
  lookahead that advances nothing — plan 22's condition parser established that shape for
  the same reason.

### 10.5 §1.3's prediction held

With `...` gone, a rest pattern is `..` and `rest @ ..` — Rust's own, because both tokens
were already Voltaic's. No lowering, no new token, and `[first, .., last]` reads the same in
both languages. `RDX0108` catches the published `...rest` spelling and names the correction,
because an author who writes it followed doc 03 §6.5.

That is the third of doc 18's deletions to pay for itself rather than cost, and the first
where the payment was *a construct this plan did not have to build*.

### 10.6 Two things the example ran into, neither this row's

- **`f [a, b]` reads as an index, not an argument.** `[` after an expression is the postfix
  index, so a list literal as a juxtaposed argument needs parentheses: `f ([a, b])`. A
  pre-existing consequence of juxtaposition meeting indexing; it produces a parse error
  rather than a wrong program, so it is a sharp edge and belongs to row 13 if anywhere.
- **Matching a `&Point` binds `&i32`.** Rust's match ergonomics, unchanged — so the example
  writes `*y`. Worth recording because it is doc 01 §25.1's no-silent-clone rule showing
  through: the borrow is visible at the use site rather than hidden in the lowering, which is
  the posture doc 03 §12.3 takes for `?.` as well.

### 10.7 What the run caught that the text could not

`ends ([1, 2, 3, 4, 5])` prints `15` — `first * 10 + last` over a **middle** rest. A lowering
that dropped the `..` would be `[a, z]`, which is a different pattern against a five-element
array and a type error; one that bound the wrong end prints `12`. And `tail_len` prints `4`,
where a lowering binding the whole slice prints `5`.

Both are one character of difference in the emitted text and a different number on screen,
which is what §3 asked the example to be.
