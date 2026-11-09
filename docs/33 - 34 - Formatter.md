# Plan 34 — Formatter

Plan ID: `formatter`
Depends on: 17 (`comments-as-trivia`), 33 (`destructuring-and-splat`), doc 30 (`<-` is await)
Milestone: 6 — *full surface*
Status: `planned`

---

## 1. What this plan is

`redox fmt`: one canonical spelling per construct, trivia preserved, and therefore the
migration tool doc 02 §4.5 calls for.

It is the **largest row left**, and §1.2 says so plainly rather than discovering it halfway.

### 1.1 Doc 03 §16's table is stale, and rewriting it is the hard part

§16's whole point is that *"every construct has exactly one spelling the formatter emits"*,
and it lists thirteen normalisations. Doc 18 and doc 30 between them deleted or changed
**eleven** of them:

| §16's row | Status now |
|---|---|
| `f(x, y)` → `f x, y` | **changed** — doc 18 §2.1 made juxtaposition the call form, so it is `f x y`, no commas |
| `f()` → `f!` | **deleted** — doc 18 §7 retired the `!` suffix; the form is `f ()` |
| `x.a!.b!.c!` → pipeline | **deleted** with `!` |
| parentheses on a call | **survives**, and is now about juxtaposition rather than §5.1.2 |
| nested parenless call → `RDX0055` | **superseded** — plan 19 dissolved into juxtaposition |
| `(- 1)` / `(-1)` spacing | **deleted** — Haskell has no subtraction section, doc 18 §5.2 |
| `Point{x: 1}` ↔ block form | **deleted** — doc 18 §7 removed the block initialiser |
| `1...10` → `1..=10` | **moot** — `...` left the language, so `1..=10` is the only spelling |
| `await f()` → `f().await` | **deleted twice** — doc 03 §14.3 made it postfix, doc 30 made it `x <- f` |
| `Vec[T]` → `Vec<T>` | **never applied** — a type is verbatim text (doc 03 §14.1), so `Vec[T]` was never accepted |
| `x.map(f)` → pipeline | **survives** |
| repeated receiver → `with` block | **deleted** — and `with` now means doc 30's callback bind, so reusing it here would be a collision |
| `== true` → field | **deleted** with doc 03 §6.3's anonymous record patterns |

Two survive. So this plan's first job is not writing a printer: it is **deciding the
canonical form of the language as it now stands**, and §3 is that table.

That §16 rotted is not a surprise — it was written before doc 18 — but the shape of the
rot is worth naming: every stale row is one where doc 18 *removed an alternative*. A
formatter normalises alternatives away, and a language with fewer alternatives needs less
normalising. **The formatter got smaller while nobody was looking**, and §16 is the only
place that would have said so.

### 1.2 The size, stated honestly

A printer needs an arm per construct. The language has roughly fifteen item kinds, forty-odd
expression variants, a dozen statement kinds and a dozen pattern forms — which is why
`redox-emit` is about eighty files. A Redox-output printer is comparable, and one
construct per file (doc 06 §1) means comparably many.

So this plan is **not** finishable in one sitting at the standard the rest of the project
holds, and pretending otherwise would produce exactly the half-formatter doc 02 §3.2 warns
about. §4's leaves are ordered so each is independently shippable and the row can be
`partial` with the remainder named — which is how rows 21, 22, 28, 29, 30 and 38 were each
carried.

### 1.3 Trivia-preserving is the whole design, and the lexer already paid for it

Doc 02 §3.2: Emerald's formatter *"loses every non-doc comment because its lexer discards
them"*. Redox's lexer keeps all four comment forms and attributes as trivia (plans 17 and
18), attached to the item or statement they belong to, and the emitter already re-emits them
into the `.xrs`.

So the formatter's trivia handling is **the same traversal the emitter already does**, with a
different target language. One investment, two consumers, as §16 says — and the consumer
that came second gets it free.

What that does *not* give free: trivia attached to a construct the formatter moves. A comment
between two arguments of a call has nowhere canonical to go. §6 declines to move such a
comment and keeps the construct unformatted instead, which is the conservative answer and
the one that cannot lose text.

### 1.4 Two properties worth more than any golden

- **Idempotence.** `fmt(fmt(x)) == fmt(x)`. A formatter that is not idempotent oscillates in
  CI, and the failure is maddening to diagnose from a diff.
- **Meaning preservation.** `parse(fmt(x))` is the same AST as `parse(x)`. This is the one
  Emerald's plan 90 lost: its formatter *"always re-added parentheses because it ran off a
  bare AST with no type information, so example code rewritten in the terser style would be
  silently reverted by the same CI gate that checked it."*

Both are property tests over the **existing corpora**, which is the cheapest large test this
project can buy: every `.sdx` in `examples/`, `tests/ui/` and `tests/lint/` is an input, and
there are more than thirty of them covering every construct that exists.

### 1.5 The corpus becomes the style guide

`redox fmt --check examples/*.sdx` passing means **every example is already canonical**.

That is a strong claim and a cheap test, and it inverts the usual relationship: rather than
the formatter being tested against hand-written expectations, the hand-written corpus is
tested against the formatter. Where they disagree, one of them is wrong and a human decides
which — which is doc 03 §16's *"decide the canonical spelling before writing examples"*
arriving late but arriving.

Expect this to find disagreements. Thirty-odd examples were written over many plans by
someone following the house style by eye.

---

## 2. Admission tests (doc 03 §2)

Not applicable: the formatter adds no construct and no lowering. The governing requirement is
§1.4's two properties, and the governing risk is doc 02 §3.2's — a formatter that silently
drops text is worse than no formatter.

---

## 3. The canonical form of the language as it stands

Decided here, replacing doc 03 §16's table. Each row is a **normalisation**: the left is
accepted and the right is what `fmt` emits.

| Accepted | Canonical | Why |
|---|---|---|
| `f(x)`, `f (x)` for a single argument | `f x` | Doc 18 §2.1: juxtaposition is the call form. Parentheses are grouping |
| `f(x, y)` | `f x y` | Likewise — and the comma in §16's `f x, y` predates juxtaposition |
| parentheses that are only grouping | removed where precedence allows | Doc 18 §2.1's own rule; `receiver`'s existing logic already decides this for the emitter |
| `x.a ().b ().c ()` — three or more stages | `x \|> a \|> b \|> c` | §16's surviving row, now spelled with `()` rather than `!` |
| `x.map (f)` as a chain head | `x \|> map f` | §16's other survivor |
| two spaces or none before a method's `()` | one space — `x.len ()` | The corpus's own habit, made a rule |
| `case` arm bodies aligned with padding | one space after `->` | Alignment breaks on the next edit |
| a binding's `::` type annotation | spaced — `n :: u64 = 0` | Doc 18 §5.2: a spaced `::` ascribes and a tight one is a path. The formatter must never tighten one |
| `data` alternatives on one line | one per line with a leading `\|` when more than one | Doc 18 §2.4's own example |
| guard ladders | one `\|` per line | Doc 18 §2.3's own example |
| indentation | two spaces per level | Every `.sdx` in the corpus |
| a trailing comma in a list, tuple or record | removed, except where a one-element tuple needs it | Plan 33 §10.3: `(a,)` is a tuple and `(a)` is a group |
| `1..=10` and `1..10` | unchanged — both are meaningful | Not a normalisation; listed so nobody adds one |

**Nothing about `<-`, `with`, `=`-in-a-condition or `in`** appears here, because doc 30 left
each with exactly one spelling. A construct with one spelling needs no canonical form, and
that is the §1.1 point restated: the formatter shrank.

---

## 4. Leaves

Ordered so each is shippable alone, and so the row can stop between any two.

### Leaf 1 — `leaf-printer-and-trivia`

The `Printer`: an indent-aware sink, the trivia traversal lifted from the emitter's shape,
and the top-level `format(program, source) -> String`. Plus `redox fmt` and
`redox fmt --check` in the CLI, so the thing is reachable the moment it prints anything.

### Leaf 2 — `leaf-items`

The fifteen item kinds: `data`, `newtype`, `type`, `class`, `instance`, `errors`, `test`,
`import`, `module`, `const`, `static`, and a function's signature line and equations.

### Leaf 3 — `leaf-expressions`

The expression printers, and §3's parenthesis and pipeline normalisations — the only leaf
with real decisions left in it, because dropping a parenthesis is the one place the formatter
can change meaning.

### Leaf 4 — `leaf-statements-and-patterns`

Statement kinds and pattern forms. Mostly mechanical once leaf 3's precedence logic exists.

### Leaf 5 — `leaf-properties`

The two property tests of §1.4 over every `.sdx` in the repository, and `--check` over
`examples/` per §1.5.

### Leaf 6 — `leaf-reconcile-the-corpus`

Whatever §1.5 finds: either the example changes or §3's table does, one row at a time, each
with a reason.

---

## 5. Acceptance criteria

1. `redox fmt FILE` formats in place; `--check` exits non-zero and prints a diff when the
   file is not canonical.
2. **Idempotence** over every `.sdx` in the repository.
3. **Meaning preservation** over every `.sdx`: `parse(fmt(x))` equals `parse(x)`, compared as
   ASTs with spans ignored.
4. **No text lost**: every comment and attribute in the input appears in the output. Asserted
   by counting trivia before and after, because a formatter that drops a comment is doc 02
   §3.2's defect and the one thing this plan may not do.
5. Every normalisation in §3 is asserted, both directions where both are meaningful.
6. `redox fmt --check` passes on every file in `examples/`, so the corpus is the style guide.
7. A construct the formatter does not yet print is passed through **verbatim** rather than
   mangled, and `--check` does not claim it is canonical. This is what makes a partial
   formatter safe to ship.
8. `redox fmt` on a file that does not parse fails with the parse error and changes nothing.

---

## 6. Out of scope, and what it discloses

- **Moving a comment.** A comment inside a construct the formatter reflows has nowhere
  canonical to go, so such a construct is left as written — §1.3. Conservative, and it
  cannot lose text.
- **A line-width limit.** Rust's formatter has one and Redox's layout is significant, so
  wrapping a line can change what a program means. Declined until someone shows a wrap that
  provably cannot.
- **Sorting anything.** Imports keep their order, because plan 14 §10.2 found that
  `reorder_imports` is what made a blank line between imports necessary in the first place.
  A formatter that reorders would reintroduce that.
- **Formatting the generated `.xrs`.** That is rustfmt's, and plan 04's guarantee is that
  the emitter already produces what rustfmt leaves alone.
- **A configuration file.** One canonical form is the point. A width or indent option would
  make `--check` mean something different per repository.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**The goal and its disclosed failure.** `crates/emerald-fmt` wanted exactly this and got two
things wrong, both recorded in Redox's own docs before a line was written: it ran off a bare
AST and so lost every comment (doc 02 §3.2), and it re-added parentheses it could not know
were redundant, so its CI gate silently reverted the style its examples were written in
(§16's own opening).

Redox avoids the first by keeping trivia from the lexer up — already done, for the emitter's
sake. It avoids the second because the emitter already has `receiver` and
`wraps_as_receiver`, which decide exactly when a parenthesis is load-bearing, and the
formatter uses the same functions rather than a second opinion.

That second point is the strongest argument this plan has: the hazard that sank Emerald's
formatter is a solved problem *in this repository* because the emitter had to solve it first.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Rewrite §16's table rather than implement it | Implement §16 as published | Eleven of thirteen rows describe constructs that no longer exist. Implementing them would normalise toward a language doc 18 deleted |
| 2 | Share the emitter's parenthesis logic | A formatter-side copy | Emerald's formatter sank on exactly this, and a second opinion about when a parenthesis matters is a second thing to keep right |
| 3 | Unprinted constructs pass through verbatim | Print everything or nothing | It is what makes a partial formatter safe, and §1.2 says a partial formatter is what this row will have for a while |
| 4 | No line-width limit | Wrap at 100, like rustfmt | Redox's layout is significant. A wrap can change meaning, and no one has shown a safe one |
| 5 | The corpus is the style guide | Hand-written formatter fixtures | Thirty-odd `.sdx` files already cover every construct, and testing them against the formatter finds disagreements a fixture would hide |
| 6 | Properties over goldens | A `.fmt` expectation per case | Idempotence and meaning-preservation are the actual requirements; a golden asserts one output and says nothing about either |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
