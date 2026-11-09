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
| `[1, 2, 3]` → `vec![…]` | kept |
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

The one thing to get right is that it must **not** become a `vec!`. The list literal
`[1, 2, 3]` lowers to `vec![1, 2, 3]` — doc 01's choice, so that a list is growable by
default — and a repeat expression is a fixed-size array, which is the point of writing one.
Emitting `vec![0u8; 1024]` would compile, run, and silently allocate on the heap where the
author asked for a stack array. Doc 02 §5.2's failure mode, in a construct whose only reason
to exist is the distinction.

So the two share a bracket and nothing else, and they are told apart by the `;`.

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
2. `[1, 2, 3]` still emits `vec![1, 2, 3]` — asserted, so the new form does not change the
   old path.
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
- **`vec![0; n]`** stays available as an ordinary macro call for the growable case. This
  plan does not give the growable repeat a spelling, because `[…]` already means `vec!` and
  a second one would be a second thing to remember.
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
| 2 | Emit verbatim, never `vec!` | `vec![v; n]`, which is shorter to implement | A repeat expression is a stack array; `vec!` is a heap allocation. Both compile, so only the emitted text tells them apart |
| 3 | Delete the two retired tokens now | Leave them, harmless | Plan 18 §10.2's rule: a variant nothing produces weakens every exhaustive match that carries it. And `%w[` lexing today produces a *syntax* error where a *removed-construct* error is the truth |
| 4 | Correct doc 04's row 29 cell rather than implement `%w` | Implement `%w` as the cell asks | Doc 18 §7 deletes it. Implementing a deleted construct because a stale register cell asks for it is the failure mode doc 04's rule 2 exists to prevent |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
