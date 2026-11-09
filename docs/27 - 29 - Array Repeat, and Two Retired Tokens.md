# Plan 29 — Array Repeat, and Two Retired Tokens

Plan ID: `comprehensions-and-collections`
Depends on: 23 (`strings-and-literals`)
Milestone: 5 — *the language, as specified*
Status: `planned`

---

## 1. What this plan is

The smallest row left, and it is smaller than its register cell says.

```text
buffer = [0u8; 1024]      →     let buffer = [0u8; 1024];
```

That is the whole feature. Doc 03 §10.4 lists it as *"added (§18); pure passthrough"*, and
it is the one entry in that table doc 18 did not delete.

### 1.1 Doc 04's row 29 cell is wrong, and this plan corrects it

The cell reads: *"`%w` and array repeat are outstanding."* `%w` is **not** outstanding — doc
18 §7 deletes it:

| Doc 03 §10.4 | Doc 18 §7 |
|---|---|
| `[1, 2, 3]` → `vec![…]` | **the lowering changed and the table did not say so** — a bare literal is an array, and `[…] :: Vec<T>` is the `Vec`. §10.1 |
| `{k: v}` map literal | **deleted** — `[…] :: C<_>` |
| `#{1, 2, 3}` set literal | **deleted** — `[…] :: C<_>` |
| `[0, ...xs, 9]` spread | **deleted** — `...` leaves the language |
| `%w[get post put]` | **deleted** — `[…] :: C<_>`, and §7's operator list says `%w[` *"leaves"* |
| `[0u8; 1024]` repeat | **kept** — this plan |

Doc 18 §7's closing paragraph names `%w[` and `#{` (set) among the operators that *"leave
outright"*. So the row has one item in it, and the cell is corrected in place with a date,
which is doc 04's own rule 2.

### 1.2 Two tokens outlive their constructs, and that is the second half of the plan

`TokenKind::PercentW` and `TokenKind::HashBrace` are still lexed. Nothing parses either, and
after doc 18 nothing ever will.

Plan 18 faced exactly this with `TokenKind::Attr` when attributes became trivia, and its
§10.2 states the rule: *"a variant nothing produces is one every exhaustive match carries
for nothing"*, and the emitter's matches are exhaustive by policy *"precisely so a real new
variant fails to compile."* Clippy does not flag a `pub` enum variant in a library crate, so
only noticing finds these.

They go. A lexer that still scans `%w[` is a lexer that accepts a construct the language
does not have — and `%w[` currently lexes and then fails to parse with a message about an
unexpected token, which is the worst of both: the author learns the syntax is unknown rather
than that it was removed.

### 1.3 `[0u8; 1024]` is a passthrough, and that is the whole design

Rust's array-repeat expression, character for character. No lowering, no `vec!`, no
`::from`. Doc 03 §10.4 calls it *"pure passthrough"* and §18 is where it came from.

The one thing to get right is that it must **not** become a `vec!`: a repeat expression is a
fixed-size array, which is the point of writing one, and emitting `vec![0u8; 1024]` would
compile, run, and silently allocate on the heap. Doc 02 §5.2's failure mode, in a construct
whose only reason to exist is that distinction.

**This paragraph originally argued it from a premise that was false**, and §10.1 records it:
it said the list literal `[1, 2, 3]` lowers to `vec![1, 2, 3]`, which is what doc 03 §10.4's
first table row says and what doc 18 §7 silently changed. A bare bracketed literal is an
**array**; `:: Vec<T>` is how it becomes a `Vec`, because `Vec::from` takes `[T; N]`. The
requirement is unchanged — a repeat is a stack array either way — and the reasoning is
rebuilt rather than deleted, because the premise being wrong is the more useful fact.

So the two bracket forms are a list and a repeat, both arrays, told apart by the `;`.

---

## 2. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes — there is no lowering. The text is Rust's |
| Nothing outside core/alloc/std? | Yes. An array is a primitive |
| Preserves Rust semantics? | Trivially: it *is* the Rust expression |
| Buys reachability? | Yes. A fixed-size array had no spelling, so no Redox program could ask for one |

---

## 3. Concrete proof this plan targets

`examples/arrays.sdx`, which must **run** and print a sequence only a correct lowering
produces:

- a repeat expression used where a **fixed-size array** is required by a signature, so a
  `vec!` lowering fails to compile;
- a list literal beside it, so the two bracket forms are both exercised and the `;` is shown
  to be what separates them;
- a repeat whose count is a `const`, because Rust allows that and a lowering that only
  handled a literal would pass a narrower test;
- a repeat nested in a list and a list nested in a repeat, since neither needs a special
  case and asserting it is how plan 22 found its three-operand chain bug.

---

## 4. Leaves

### Leaf 1 — `leaf-array-repeat`

`Expr::ArrayRepeat { value, count, span }`, read where the list literal is read: after the
first element, a `;` means repeat and a `,` means list. Emitted verbatim as `[value; count]`.

### Leaf 2 — `leaf-retire-two-tokens`

`TokenKind::PercentW` and `TokenKind::HashBrace` removed, with their scanners. Every
exhaustive match over `TokenKind` loses two arms it was carrying for nothing.

### Leaf 3 — `leaf-proof-and-gate`

`examples/arrays.sdx`, emit tests, a golden, a run expectation, and the gate.

---

## 5. Acceptance criteria

1. `[0u8; 1024]` emits `[0u8; 1024]` — **not** `vec![0u8; 1024]`, asserted as the thing that
   must not happen, because the wrong form compiles and runs.
2. `[1, 2, 3]` still emits `[1, 2, 3]`, and `[1, 2, 3] :: Vec<i32>` still emits
   `Vec::from([1, 2, 3])` — both asserted, so the new form does not change either old path.
   *Corrected from "emits `vec![1, 2, 3]`" — see §10.1.*
3. A repeat whose count is a `const` works.
4. A repeat nested in a list, and a list nested in a repeat, both work.
5. The example uses a repeat where a fixed-size array type is required, so criterion 1 is
   enforced by rustc as well as by text.
6. `%w[` and `#{` no longer lex, and `TokenKind` has two fewer variants.
7. `examples/arrays.sdx` compiles with **zero** rustc output, runs, and matches its
   `.stdout`; a golden covers the text.
8. Any new diagnostic has an `RDX` code reached by the UI corpus.

---

## 6. Out of scope, and what it discloses

- **`%w`, the set literal, the map literal and the spread** are deleted by doc 18 §7, not
  deferred. `[…] :: C<_>` does the work of the first three and is already implemented; `...`
  leaves the language entirely.
- **A `[T; N]` *type*** already works, because doc 03 §14.1 carries a type as verbatim text.
  Only the expression was missing — which is why `zeros :: [u8; 4]` parses today and
  `zeros = [0u8; 4]` does not.
- **A growable repeat** needs no spelling of its own: `[v; n] :: Vec<T>` reaches it through
  doc 18 §7's ascribed-literal form, since `Vec::from` takes an array.

  > **Corrected 2026-11-09 by plan 39.** This bullet also said *"`vec![0; n]` stays
  > available as an ordinary macro call"*, which is false: **Redox has no macro-call
  > syntax**. `vec![1, 2]` does not parse. Macros are *emitted* — `println!`, `assert!`,
  > `panic!`, `format!` — and never written, because doc 18 §7 retired `!` as a suffix and
  > left it meaning boolean negation. Plan 39 §1.4 found this by needing `pin!` for an
  > executor and reaching for `Box::pin` instead.
  >
  > The ascribed form is therefore the *only* way to a growable repeat, which makes this
  > bullet's first sentence load-bearing rather than merely true.
- **Removing a token is a breaking change to nothing**, since neither construct ever parsed.
  If that is ever untrue for a future retirement, the removal needs a deprecation path and
  this plan is not the precedent for skipping one.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald had no fixed-size arrays; every sequence was its growable list type, which
is the design Redox explicitly does not take.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | A repeat is a distinct node, not a list with a flag | `List { elements, repeat: bool }` | The two emit different macros — one emits none at all — and a flag on a shared node invites the wrong branch. §1.3 is why that matters more than usual here |
| 2 | Emit verbatim, never `vec!` | `vec![v; n]`, which is shorter to implement | A repeat expression is a stack array; `vec!` is a heap allocation. Both compile, so only the emitted text tells them apart. The decision stands; §10.1 records that the argument first given for it rested on a stale table row |
| 3 | Delete the two retired tokens now | Leave them, harmless | Plan 18 §10.2's rule: a variant nothing produces weakens every exhaustive match that carries it. And `%w[` lexing today produces a *syntax* error where a *removed-construct* error is the truth |
| 4 | Correct doc 04's row 29 cell rather than implement `%w` | Implement `%w` as the cell asks | Doc 18 §7 deletes it. Implementing a deleted construct because a stale register cell asks for it is the failure mode doc 04's rule 2 exists to prevent |

---

## 10. Result

Shipped. 625 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit` are green. Every
criterion is met, one of them after being corrected.

The smallest plan in the run found the stalest page.

### 10.1 Doc 03 §10.4's first row has been wrong since doc 18, and nothing noticed

§10.4's table opens with `[1, 2, 3]` → `vec![1, 2, 3]`. It has not been true since doc 18
§7 replaced four of that table's six rows with `[…] :: C<_>` — because that form needs the
bare literal to be an **array**: `Vec::from` and `HashSet::from` both take `[T; N]`.

So a bare bracketed literal emits `[1, 2, 3]`, and `[1, 2, 3] :: Vec<i32>` emits
`Vec::from([1, 2, 3])`. The implementation has been right all along; the specification's
table was describing a lowering that had been replaced four plans earlier, and §7's
retirement list did not mention changing the row it made obsolete.

**It cost this plan its argument.** §1.3 built the case for emitting `[v; n]` verbatim on
the premise that a list means `vec!` — so a repeat emitting `vec!` would be the odd one out.
The premise was gone; both forms are arrays. The *requirement* survived unchanged, because a
repeat is a stack array either way, but the reasoning had to be rebuilt from the type rather
than from the contrast.

Found by writing `growable :: Vec<i32>` in the example and watching rustc say
`expected Vec<i32>, found [{integer}; 3]`. Corrected in §10.4 itself, dated, with the
ascribed row added beside it — the fourth published lowering this run of plans has had to
correct, after doc 18 §2.4's missing `use T::*`, plan 23's `lex_fragment` claim, and plan
28's `.as_ref()` chain.

### 10.2 The register cell was wrong in the other direction

Doc 04's row 29 said *"`%w` and array repeat are outstanding"*. `%w` is deleted by doc 18
§7, twice over — in the table and in the operator list. Had the cell been taken at face
value this plan would have **implemented a construct the language does not have**, which is
exactly what doc 04's rule 2 exists to prevent and the first time in this run that the rule
has had a near-miss rather than a tidy-up.

Two stale documents pointing opposite ways on the same table, found in the same hour. The
useful generalisation: doc 18 §7's retirement list says what leaves, and nothing made it
say what to *change* in the sections it emptied.

### 10.3 Two tokens retired, and the fixture told on itself

`TokenKind::PercentW` and `TokenKind::HashBrace` are gone, with their scanners, and the
parser's bracket-depth scan lost two arms it was carrying for nothing — plan 18 §10.2's
rule, applied a second time.

The lexer's own fixture held `words = %w[get post put]` and `set = #{1, 2, 3}`, replaced by
one `repeated = [0u8; 4]`. That shifted every line after it by one, and
`dedents_a_multiline_block_and_keeps_line_numbers` failed on a written-out line number —
which is precisely what its comment says it is for: *"this number is written out so that
moving the fixture is a visible decision rather than a silent one."* Second time that
mechanism has earned its keep, and the comment now records both.

`#` is now almost empty, as doc 18 §5.1 intended: `#[attr]`, and `#{` inside a string. A
bare `#` is `unexpected character '#'`, which is how two `continuation.rs` cases discovered
they had been using a set literal as a generic delimiter.

### 10.4 What the `;` buys, stated plainly

A list and a repeat share a bracket and differ by one token, and the parser tells them apart
with one token of lookahead after the first element — no backtracking, because the
comprehension's `|` was already ruled out by then. The three bracket forms now read in one
function in the order `|`, `;`, `,`, and the two neighbours are asserted beside the new one
so that inserting the check in the wrong place fails a test rather than a golden.
