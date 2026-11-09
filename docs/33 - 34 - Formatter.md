# Plan 34 — Formatter

Plan ID: `formatter`
Depends on: 17 (`comments-as-trivia`), 33 (`destructuring-and-splat`), doc 30 (`<-` is await)
Milestone: 6 — *full surface*
Status: `partial` — leaves 1 and 5 shipped 2026-11-09; §10 names what is owed

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
| ~~`f(x, y)`~~ | ~~`f x y`~~ | **Retired 2026-11-09, leaf 3 §10.10.** It changes the program: `f (x, y)` parses as `f` applied to a **tuple**, which is what juxtaposition being the call form *means*. The row was inherited from doc 03 §16, where `f x, y` was the call form and the rewrite was therefore sound |
| parentheses that are only grouping | removed where precedence allows | Doc 18 §2.1's own rule; `receiver`'s existing logic already decides this for the emitter |
| ~~`x.a ().b ().c ()`~~ | ~~`x \|> a \|> b \|> c`~~ | **Retired 2026-11-09, leaf 3 §10.11.** A refactor, not a normalisation — it replaces one construct with another, and §1.4's property cannot guard it |
| ~~`x.map (f)` as a chain head~~ | ~~`x \|> map f`~~ | Retired with the row above, same reason |
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

> **The order of leaves 2 to 4 is wrong, and §10 records it.** Items were listed first and
> most of them *delegate* to the expression printer: a `const`'s value is an `Expr`, a
> function's equations are expressions, and a `data` declaration's only non-type content is
> too. The dependency runs **expressions → statements → items**, so the numbering below is
> kept for reference and the work should be taken in reverse.

### Leaf 2 — `leaf-items`

The fifteen item kinds: `data`, `newtype`, `type`, `class`, `instance`, `errors`, `test`,
`import`, `module`, `const`, `static`, and a function's signature line and equations.

Only `import` and `type` are printable without leaf 3, because only those two contain no
expression. Doing them alone buys very little — the corpus is already canonical at that
level — which is the other half of why the order is wrong.

### Leaf 3 — `leaf-expressions`

The expression printers, and §3's parenthesis and pipeline normalisations — the only leaf
with real decisions left in it, because dropping a parenthesis is the one place the formatter
can change meaning. **This is where the row's value is**, and where its remaining risk is.

### Leaf 4 — `leaf-statements-and-patterns`

Statement kinds and pattern forms, including the two-space re-indentation §3 asks for.

Re-indenting is the one normalisation that rewrites **leading** whitespace, which the
off-side rule reads — so it is the one that can change meaning by getting a continuation
line's depth wrong. The lexer's `Indent`/`Dedent` tokens give the statement depths; a
multi-line expression's continuation lines sit at no statement depth at all, and that is the
case to get right.

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

**Leaf 1 and leaf 5 shipped; the row is `partial`.** 671 tests pass; `fmt`, `clippy -D
warnings` and the gate are green. `redox fmt` and `redox fmt --check` work, the four
properties of §1.4 hold over every `.sdx` in the repository, and §1.5's inversion is a test
rather than an aspiration.

What the row still owes: leaves 2, 3 and 4 — the item, expression, statement and pattern
printers — and leaf 6, which cannot run until they exist. §1.2 said this would be the shape
of it.

### 10.1 What a span-slice formatter actually buys

The increment canonicalises file-level structure and passes every item through verbatim,
sliced from the source. That sounded modest when §1.2 proposed it and turned out to be the
whole reason this went smoothly: a formatter that rewrites nothing **cannot** lose text,
reorder anything, or change meaning, so §1.4's properties hold by construction and every
later leaf has a green baseline to break.

It is also already useful: it collapses blank-line runs, strips trailing whitespace, and
normalises the trailing newline, which is most of what a formatter does to a file in
practice.

### 10.2 Three bugs, each caught by a different property

The properties earned their place immediately. None of these was found by reading the code.

- **A blank line grew per item**, caught by idempotence. A declaration's span runs to the
  newline that ends it, so slicing to it and then adding a separator doubled the gap.
- **A comment was emitted twice**, caught by the `--`-counting property on
  `examples/case.sdx`. A declaration's span can reach *into the following comment*, which is
  also the next item's leading trivia — so slicing both regions duplicated it. The fix is
  the model: an item owns the source from its own start to the **next item's** start, which
  makes the regions contiguous and disjoint, so text can neither duplicate nor vanish.
- **Three adjacent imports were pushed apart**, caught by meaning-preservation on
  `examples/async.sdx`. See §10.3 — this is the interesting one.

### 10.3 A blank line between items is load-bearing, and the plan had it backwards

The first version had a `SEPARATOR` constant and a doc comment arguing at length that there
was **no exception**: one blank line between items, always, and plan 14 §10.2's import rule
cited as the reason not to *remove* them.

That reasoning was half right and the conclusion was wrong. Plan 14 §10.2 is that rustfmt's
`reorder_imports` sorts within a **contiguous** block, so

```text
import core::task::Context
import core::task::Poll
import core::task::Waker
```

is one block rustfmt may sort, and three blank-line-separated imports are three blocks it may
not. The author's grouping therefore decides what the generated Rust is allowed to do — in
both directions. A formatter that *imposes* blank lines regroups every import in the corpus
just as surely as one that removes them.

So the rule is the ordinary one after all: collapse a run of blank lines to one, leave zero
as zero. Worth recording because the wrong version was argued for confidently and in detail,
and the argument cited the right source.

### 10.4 Measuring the gap needed a third model

Two attempts failed before the right one:

1. Count trailing blank lines of the earlier region — saw none, because a comment trivium's
   span *starts at the newline before the comment*, so the blank line had landed in the
   later region.
2. Count leading blank lines of the later region — the mirror failure for a declaration.

The gap is split across the boundary, so neither region alone can measure it. `trimmed_range`
finds each block's real text and the gap is the source **between** two blocks — which is
obvious in hindsight and was not obvious from either failure.

### 10.5 The property test that reported a defect in its own normaliser

`formatting_preserves_meaning` failed on `examples/async.sdx` with two debug dumps that
looked identical for the first three hundred characters. The cause was the normaliser:
`{:#?}` prints a span across four lines —

```text
span: (
    613,
    640,
),
```

— so dropping lines that *begin* with `span:` left the numbers behind, and the comparison
still saw them move. The test was reporting a defect in the subject that was a defect in
itself.

Rewritten to scan single-line `{:?}` output and blank each `span: (…)`. Recorded because a
property test with a wrong normaliser is worse than no test: it fails on correct code, and
the natural response is to go looking in the wrong place.

### 10.6 `fmt` was the last planned command, so two tests inverted

`State::Planned` is now unconstructed outside tests, and two tests asserted that `fmt`
*names its plan*. Both carried a comment recording the same pattern from plan 12 — *"`watch`
was here too until plan 12 implemented it, and this test failing was how plan 12 learned it
had changed the surface"* — and both now assert the opposite: **no command advertises a
plan**, because every declared command is built.

The variant stays rather than being removed, which is the opposite of what plan 18 did with
`TokenKind::Attr`, and the difference is whether it can come back: `Attr` could not, and this
is constructed again the moment a row declares a command ahead of itself. Rows 40, 49 and 50
each bring one.

### 10.7 `--check` reports names, not a diff

Criterion 1 asked for a diff. It prints a file list, and the narrowing is deliberate: the
information wanted from `--check` is *which files*, and the diff wanted after that is
`redox fmt FILE && git diff`, which reads better than anything this command would render.

### 10.8 The corpus was already canonical, which proves less than it looks

`every_example_is_already_canonical` passes, so no example needed reformatting — and leaf 6
had nothing to reconcile. That is a weak result rather than a strong one: this increment only
canonicalises file-level structure, and the corpus was already consistent there. The real
test of §1.5 comes with leaves 2 to 4, when the formatter starts having opinions about the
inside of a construct.

Stated so the green tick is not mistaken for the claim §1.5 actually makes.

### 10.9 §4's leaf order is backwards, and the plan now says so

Leaf 2 is items, leaf 3 expressions. But a `const`'s value is an `Expr`, a function's
equations are expressions, and `data`'s only non-type content is too — so **items delegate
to expressions**, and only `import` and `type` can be printed without leaf 3 at all.

Doing those two alone buys almost nothing, because the corpus is already canonical at the
item level, which §10.8 just established. So the dependency is
**expressions → statements → items**, the numbering is left as published with a note above
it, and the work should be taken in reverse.

Worth recording rather than silently reordering: the plan was written with the item kinds
first because that is how `redox-emit` is organised, and the emitter can do that because it
emits *Rust*, where an item's shape and its expressions are independent problems. Printing
Redox back is not the same problem, and assuming the emitter's structure transferred was the
mistake.

### 10.10 Two of §3's rows change the program, and one of them is a language fact

Leaf 3 printed expressions and the first thing that fell over was §3's own table.

**Row 2, `f(x, y)` → `f x y`, is wrong.** Not awkward — wrong. Checked against the parser
rather than reasoned about:

```
f(a, b)   =>  Call { callee: f, args: [Tuple { items: [a, b] }] }
f (a, b)  =>  Call { callee: f, args: [Tuple { items: [a, b] }] }
f a b     =>  Call { callee: f, args: [a, b] }
```

`f (x, y)` applies `f` to **one** argument, a tuple. Rewriting it to `f x y` applies `f` to
two. The spacing makes no difference, which is the part worth knowing: whitespace adjacency
is tracked by the lexer and does not distinguish these.

This is not a defect in the row so much as the row outliving its premise. It came from doc
03 §16, where the call form was `f x, y` — with the comma — so `f(x, y)` → `f x, y` was a
pure respelling. Doc 18 §2.1 replaced that with juxtaposition and the row was carried across
with its *shape* updated and its *soundness* unexamined. §1.1 of this plan already found doc
03 §16 stale in 11 of 13 rows; this is the twelfth, and it is the one that would have
silently miscompiled a program.

Row 1 — `f(x)` → `f x` — survives, because a single parenthesized argument is a `Paren` node
and not a tuple. The two rows look like one rule and are two.

### 10.11 The pipeline rewrite is a refactor, and §1.4's property cannot guard it

Rows 4 and 5 turn a method chain into `|>`. Both are retired, for a reason that is about the
**property** rather than about the rewrite:

§1.4's meaning-preservation property compares `parse(fmt(x))` with `parse(x)` as trees. A
method chain is `MethodCall` nodes; a pipeline is `Binary { op: Pipeline }`. The rewrite
therefore changes the tree by construction, so the property cannot pass unless its normaliser
**lowers `|>` to method calls** — which is reimplementing part of the emitter inside the
test, and doing it on exactly the rewrite with the most room to be wrong.

A formatter whose riskiest transformation is the one its safety property cannot see is worse
than a formatter without that transformation. Emerald's plan 90 is the case: its formatter
re-added parentheses it could not know were redundant, and doc 02 §3.2 records the result —
*"example code rewritten in the terser style would be silently reverted by the same CI gate
that checked it."*

So the pipeline rewrite is not this row's. It is a **refactor**: a suggestion a user accepts,
not a rewrite a formatter performs on save. Plan 38 built the machinery for exactly that —
`Severity::Warning`, the `RDX1000+` band, a `tests/lint/` corpus whose cases must still build
— and `x.a ().b ().c ()` is a reasonable `RDX1002`. Named here rather than dropped, which is
§6's rule.

### 10.12 `Paren` is a node, so the property had to be told to ignore it

§3 row 3 — drop a parenthesis where precedence allows — cannot pass a tree comparison
either, for the same structural reason: `Paren` is in the AST, deliberately, so the emitter
reproduces the grouping the author wrote instead of re-deriving it.

The resolution is different from §10.11's, and the difference is the whole argument. Stripping
`Paren` from **both** sides is not a weakening of the property, because **precedence is
recorded in the tree's shape, not in its `Paren` nodes**. Drop a parenthesis that was holding
`(1 + 2) * 3` together and the re-parse is `1 + (2 * 3)` — a different *shape*, which the
stripped comparison still rejects. The property keeps exactly the power it was for.

Stripping only the top-level `Paren` was not enough, and the way it failed is instructive: the
printer **adds** parentheses where a form would otherwise be misread — `-(a + b)`,
`f (g x)` — and those land at whatever depth the form sits at, so the trees legitimately
differ by `Paren` nodes anywhere. The strip is therefore a scanner over the debug string
rather than a rebuild of the tree: thirty-one match arms in a test file would be a second copy
of the AST, which is a worse liability than a scanner.

### 10.13 The precedence table moved to the operator

Dropping a parenthesis is a precedence decision, and the table lived in
`redox-parser`'s `binary_power`, keyed by `TokenKind` and `pub(crate)`. Correct while the
parser was the only reader.

It is now `BinaryOp::power` in `redox-ast`, and `binary_power` keeps only the half that is
genuinely the parser's — which **token** spells which operator. A formatter working from a
copy of the table would drop a needed parenthesis the moment the two drifted, and it would do
it silently, because both spellings still compile and both still pass every existing test.
`the_powers_come_from_the_operator` asserts the move was a move.

`BinaryOp::redox_text` was added beside it for the inverse reason `rust_text` exists:
`rust_text` returns `None` for `??`, `catch` and `|>` because those need a lowering rather
than a spelling, and a formatter printing Redox needs the surface spelling of all three.

### 10.14 Two defects, both found by a property and neither by the fixture

The fixture pair was green while both of these were live, which is the case for having the
properties as well.

**An ascription's inner expression is an operand, not an argument.** `1 + 2 :: u64` parses as
`Ascribe { inner: Binary { .. } }` with no parenthesis in the source — `::` binds looser than
every binary operator. Printing the inner expression as an *argument* parenthesised it, so
`1 + 2 :: u64` came out as `(1 + 2) :: u64` and the corpus property reported that the program
had changed. It had: the re-parse carried a `Paren` the original did not.

**A parenthesis the author wrote must never be wrapped in a second one.** `f (\x -> x)` grew
one parenthesis per pass. The lambda does not stand alone, so `argument` wrapped it; the inner
`Paren` survived because `can_drop_paren` declines on a lambda; and the next pass wrapped the
result again. Found by idempotence, in `examples/comprehension.sdx`, on
`([ n * 2 | n in xs ]) :: Vec<i32>`. Both arms of the fix print the **inner** expression, so a
layer can be added at most once however many times the formatter runs.

Both are now fixture rows as well, so they stay closed.

### 10.15 §4's leaf boundary is not where §4 draws it

§10.9 already corrected the leaf *order* to expressions → statements → items. Leaf 3 found
that the **boundary** is also wrong.

`if`, `case`, `do`, `async` and `try` are `Expr` variants, so §4 puts them in leaf 3. Their
bodies are `Block`s — statement sequences — so printing them is printing statements, which is
leaf 4. The real division is **single-line expressions / block-bearing constructs / items**,
and an `Expr` variant can fall on either side of it.

Those five are sliced verbatim from the source, which is criterion 7 used as the starting
point rather than as a fallback — the same discipline leaf 1 established for items, one level
down. It is what lets leaf 3 ship before leaf 4 exists.

### 10.16 Leaf 3 is not wired into `format`, deliberately

`expr_text` is exported and tested and `format` does not call it, because an expression is
only reachable through the item that holds it and items are leaf 2. That is §4's corrected
order working as intended rather than a gap: everything above expressions delegates down to
them, so the printer is built first and connected last.

The consequence for the corpus is the useful part. `every_example_is_already_canonical`
(§1.5) still passes untouched, so leaf 6's reconciliation — where an example changes or §3's
table does, one row at a time — happens when leaf 2 lands and not in pieces before it.
