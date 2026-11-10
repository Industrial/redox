# Plan 55 — Expression Orientation

Plan ID: `expression-orientation`
Depends on: 52 (`type-colon`), 54 (`assignment`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

Every body position becomes an expression. Declarations included. `test` excepted, for a reason
that is rustc's and not ours.

This row **deletes grammar**. It does not add any. The statement/expression fork leaves the
parser, four constructs lose their special cases, one keyword is retired, one is freed, and a
class of diagnostics stops being needed. The emitter gets simpler because Rust is already
expression-oriented and the two surfaces stop disagreeing about what a block is.

### 1.1 `if` is the odd one out among its own siblings

Measured, not inferred:

| Construct | An expression? | Evidence |
|---|---|---|
| `loop` | **yes** | `break n` as a function tail works; `examples/async.sdx` depends on it |
| `case … of` | **yes** | emits `match` in tail position |
| `do` / `try` / `async` | **yes** | that is their whole purpose |
| indented `if` | **no** | emits `"big";` → rustc `E0308` |

`loop` and `case` are expressions. `if` is not. Nothing in doc 01, 03, 18, 20 or 22 argues for
the difference — plan 20 §1.2 explains the *two* `if`s (statement and `if … then … else`
expression) as Haskell's doing, and plan 22's result section records the consequence as a *gap
belonging to plan 20*:

> plan 20's indented `if` is a **statement**, so it is not a tail expression, and a branch
> written as a bare value emits `"big";`

That is not a design. It is an accident with a note attached, and the note is in the right place.

### 1.2 The cost, measured

The five standalone programs of the new corpus — 545 lines, 367 of them code — contain:

| | Count |
|---|---|
| `return` in non-tail position | **14** |
| `if … then … else` expressions | **2** |

(Across all twelve programs, 1,501 lines, it is 18 and 3. The five-program figure is the honest
comparison because the seven id_effect programs are written in a combinator style that reaches
for neither.)

Fourteen against two. The statement/expression split is not a neutral stylistic choice; it is
actively pushing imperative control flow into a language whose surface is declarative, because
`return` is the only way a branch can produce a value.

The expression form exists and is barely used, for a reason §2 confirms: **it is one line only**.
`if c` / `then 1` / `else 2` across three lines is `RDX0102`. So the author's real choice is
between a one-line expression and a `return`, and at any realistic branch width the `return`
wins.

### 1.3 What it deletes

Five things, and the fifth is the one a reader will care about.

| Deleted | Because |
|---|---|
| The statement/expression fork at every block tail | there is one node kind |
| `-> do` on a `case` arm | an arm body is an expression; a block is an expression |
| The `then` keyword's disambiguating job | there is nothing to disambiguate |
| `do` as a construct | a block needs no keyword announcing it |
| `RDX0101`/`RDX0102` at arm and tail positions | the positions accept expressions |

**And `then` becomes available as a method name.** That is a real recovery, though a smaller one
than it first looks: `Ordering::then` and `bool::then` exist in std and are unreachable today.
`Option::then` and `Future::then` do **not** exist — probed: rustc offers `and_then` for the
first and `FutureExt::then` from `futures` for the second — so the claim is two std methods, not
four. `Ordering::then` is the one that bites: a sort comparator must be written as a tuple
comparison because `(b.1.cmp (&a.1)).then (…)` is `RDX0103`, which is exactly what
`wordfreq`-shaped code does and says so in a comment.

### 1.4 Declarations become expressions, and Rust already allows it

The maximal reading was chosen, and the cost quoted for it at decision time was wrong in this
row's favour. Rust **does** permit items inside a block — `struct`, `enum`, `trait`, `impl`,
`use`, `const`, `static`, `fn` — so emission is *nesting*, not hoisting.

The generics limit is narrower than feared. A local item cannot capture the enclosing function's
type parameter implicitly, but it can declare its own — and doc 18 §2.2 already **synthesises the
generic list from the types**, so `data Row = Row { a : t }` inside a generic function emits
`struct Row<t> { a: t }`, which compiles. Verified.

What this buys is small and real: a helper type used by one function lives next to it, and
`docs/06 - Redox Code Architecture.md`'s one-construct-per-file rule applies to module-scope
constructs, so a local `data` is not an escape from it.

### 1.5 `test` is the one carve-out, and rustc decides it

A nested `#[test]` is **not collected**. Verified: rustc emits
`warning: cannot test inner items` under the `unnameable_test_items` lint, and the harness ran
one test where two were written.

So a local `test` would compile, warn in generated Rust, and **silently never run** — which is
precisely the failure class this milestone exists to remove. It is therefore refused at the
Redox line, with a named diagnostic pointing at module scope.

This is a carve-out from §1.4 and it is stated as one rather than smuggled in. The alternative —
hoist a local `test` to module scope at emission — was weighed and declined in §9: AGENTS.md
already puts public-API tests in `crates/<crate>/tests/` and reserves inline tests for private
constructs, so co-location inside a body is a shape the conventions discourage, and building
hoisting machinery for it would be cost against a convention.

### 1.6 A block's value, and the explicit `()`

A block's value is its last expression. Two degenerate cases need a rule:

- a block whose last element is a **binding** or a **loop** has no value, and
- a `()`-returning function's block ends in something that happens to be `()`.

The rule: **a block whose value would be unit must say `()`**. A function returning `()` ends
with `()`, written. That costs one line in `main` and in every `()`-returning function, and it
removes the case where a function falls off the end of a loop and silently type-checks as `()`
when something else was meant.

Discarding a value is unmarked, as in Rust — a non-final expression in a block is discarded.
`RDX1001`-style lints cover the shapes that bite: a discarded `Result`, a discarded `Effect`, a
discarded `must_use`.

### 1.7 `do` is retired; `try` and `async` become modifiers

If an indented block is an expression, no keyword is needed to announce one. `do` goes, and with
it `-> do`. `try` and `async` stay as **adjectives on a block**, which is what they always were:

```
value =                         r = try                     work = async
  a = f x                        Ok ((parse x)?)              n <- fetch ()
  a + 1                                                       n + 1
```

The labelled form `do 'search` is the one capability `do` carried that nothing else provides —
how a value leaves a loop that is not a function's tail. It survives as `'search` on a bare
block, which is Rust's own spelling for a labelled block.

**`do` is then reserved, not deleted.** Plan 60 takes `<-` as monadic bind interpreted by the
enclosing block modifier, and if a future row wants genuine do-notation the token should be
there for it rather than requiring a breaking change. Doc 18 named it `do` for a Haskell reason
and the Haskell meaning is the one still unclaimed.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**`loop` and `case` are already expressions.** A `loop` with `break n` is a function tail; a
`case` emits `match` in tail position. Both verified running.

**An indented `if` in tail position discards its branch values:**

```
tier n =                        fn tier(n: i32) -> &'static str {
  if n > 9                          if n > 9 {
    "big"                               "big";          // discarded
  else                              } else {
    "small"                             "small";        // discarded
                                    }
                                }
->  rustc: E0308 mismatched types
```

**The `if` expression is single-line only.** `if c` / `then 1` / `else 2` across lines reports
`RDX0105: expected an indented \`if\` body` first and then `RDX0102`, because the parser takes the
newline as the start of a statement-form body and finds none. A block-branch spelling —
`if c then do` / `else do` — also fails, with the exact code depending on the indentation
chosen.

**A multi-line `case` arm needs `-> do`**, and a bare compound assignment is not an arm body:

```
0 ->                            RDX0102: expected an expression
  base = "zero"
  base

Exit::Success _ -> enriched += 1    RDX0101: expected a line break after a `case` arm
```

**`then` is reserved**, so `.then` is `RDX0103: expected a field name, a tuple index, or a
method name after `.``.

**Rust accepts local items**, including a local `struct` with its own type parameter
instantiated from the enclosing function's. Verified compiling and running.

**A nested `#[test]` is not collected.** Verified: `warning: cannot test inner items`, one test
run of two written.

**Corpus scale.** 703 code lines across 29 examples contain exactly **one** `do` block —
`examples/control.sdx`'s `do 'search`, which is the labelled form §1.7 keeps — so the retirement
is a zero-site migration in the examples. The `-> do` arm form is used by **five** of the new
programs across **eight** sites, and those are what leaf 3 migrates.

---

## 3. Concrete proof this plan targets

**A function whose tail is an indented `if` with bare branch values compiles, runs, and prints
the branch value** — the case that is `E0308` today.

Five more, each chosen because it fails differently:

1. **`.then` on an `Ordering`.** `(b.1.cmp (&a.1)).then (a.0.cmp (&b.0))` as a sort comparator.
   Today `RDX0103`; after this row it is the natural spelling, and
   `examples/wordfreq.sdx`-shaped code stops needing a tuple comparison.
2. **A `case` arm with a three-statement body and no `-> do`.**
3. **A `case` arm whose body is `n += 1`** — a statement in an expression position, which is
   what currently refuses.
4. **A local `data` inside a generic function**, with its field typed by the enclosing
   function's variable, so doc 18 §2.2's synthesis is exercised at local scope.
5. **A local `test`**, as a `tests/ui/` fixture carrying the refusal — because the thing being
   prevented is silent and the fixture is the only durable record of it.

And one negative: **`main` ending in a binding is a Redox diagnostic** asking for `()`, not a
silent unit.

---

## 4. Leaves

### Leaf 1 — `leaf-one-expression-node`

Collapse the statement/expression fork in the AST and the parser. A block is a sequence of
expressions; its value is the last. This is the leaf the rest depend on and the one that deletes
the most code.

### Leaf 2 — `leaf-if-expression`

`if` becomes an expression with indented branches. `then` is retained only as an optional
one-line marker for this milestone so the two existing corpus uses keep parsing, and is removed
in leaf 7. `else if` chains unchanged.

### Leaf 3 — `leaf-case-arm-blocks`

A `case` arm body is an expression, so an indented block is one. `-> do` is accepted and
deprecated for this milestone, then removed with `do`.

### Leaf 4 — `leaf-block-value`

The unit rule from §1.6: a block whose value would be unit requires `()`. The diagnostic names
the position and suggests the line. Plus the discard lints — `Result`, `must_use` — at the Redox
line rather than rustc's.

### Leaf 5 — `leaf-local-declarations`

`data`, `newtype`, `type`, `class`, `instance`, `errors`, `import`, `const`, `static` in a body
position, emitted as nested Rust items. Includes the generic-synthesis case from §1.4.

### Leaf 6 — `leaf-test-carve-out`

`test` refused in a body position, with a named code and a help line. The `tests/ui/` fixture
from §3 case 5.

### Leaf 7 — `leaf-retire-do-and-then`

`do` retired as a construct and **reserved** as a token; `try` and `async` become block
modifiers; `'name` labels a bare block; `then` removed and freed. The one corpus `do` and the
`-> do` sites migrate here.

### Leaf 8 — `leaf-corpus-and-docs`

29 examples; the `return`-heavy shapes rewritten as expressions where that is what they meant.
Docs 01 §8, 03 §7, 18 §1.1 and plan 20 §1.2 amended; plan 22's recorded gap marked resolved.
`docs/GRAMMAR.md` regraded for the retired and freed tokens.

---

## 5. Acceptance criteria

1. An indented `if` is an expression; a function may end in one with bare branch values.
2. `.then` is callable as a method, exercised on an `Ordering` in a running example.
3. A `case` arm takes an indented block with no marker; a compound assignment is a legal arm
   body.
4. A local `data`, `class`, `instance`, `errors` and `import` each compile, in a running example,
   including the generic case.
5. A local `test` is a named Redox diagnostic with a `tests/ui/` fixture.
6. A block whose value would be unit requires an explicit `()`, diagnosed with a suggestion.
7. A discarded `Result` is a Redox lint, not only a rustc warning.
8. `do` is not a construct and **is** a reserved word; using it as an identifier is diagnosed.
9. `try` and `async` work as block modifiers; `'name` labels a bare block and `break 'name v`
   still carries a value out of a loop.
10. All 29 examples plus the two new-program corpora transpile, compile with no rustc output,
    run, and have asserted stdout.
11. `construct_coverage` green; the retired constructs removed from the index and the new ones
    added.
12. Plan 22's disclosed gap and plan 20 §1.2's two-`if` rationale are both marked resolved in
    doc 04.

---

## 6. Out of scope, and what it discloses

**Layout now carries the discard decision alone.** Today statement-ness tells the emitter where
values are dropped; after this row, indentation does. That is Haskell's layout-algorithm problem
and it is the real work in leaf 1. The off-side rule is already in place (`RDX0002` proves it),
but its interaction with juxtaposed application across lines is where this row will find its
surprises. Disclosed rather than estimated.

**`for` and `while` are expressions of type `()`.** "Everything is an expression" is partly
vacuous for them, and deliberately so — Rust's answer is the same, and a loop that accumulates a
value is a comprehension, which doc 18 §3.2 already provides.

**Declarations are expressions; they are not values.** A `data` declaration evaluates to `()`
like any other non-value element. It cannot be bound, passed or returned, and `let t = data …`
is nonsense that the grammar permits shaping and the emitter refuses. Recorded because §1.4's
framing invites the question.

**`test` remains the only carve-out**, and it is rustc's call, not a design preference (§1.5).
If `unnameable_test_items` ever becomes collectable, the carve-out can go.

**`then` is freed but no construct claims it.** A future row wanting it for something is
unobstructed; this row only stops it being reserved for a job that no longer exists.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under real rustc with **no output**, runs, stdout
asserted; goldens re-blessed and the diff reviewed; `tests/ui/` fixtures for the local `test`
refusal and the unit-block diagnostic; `construct_coverage` green;
`formatting_preserves_meaning` green — plan 48 found `redox fmt` deleting doc comments inside a
continuation, and this row changes what a continuation is.

---

## 8. Emerald reuse

Nothing on the statement/expression question. Doc 02's §4 reviews Emerald for specific
transferable lessons and none of its eight subsections addresses statement orientation — so the
argument in §1.2 is this row's own measurement rather than a borrowed lesson, and is stated as
such.

The transfer is plan 34's formatter again, as the migration tool (doc 02 §4.5) — and doc 04's
fourth ordering commitment, which this row leans on harder than any before it: the corpus has no
string-literal fixtures to miss, because `no_literal_expectations.rs` forbids them.

---

## 9. Decision log

1. **Everything, including declarations.** The narrower readings — `if` only, or control flow
   only — each leave a statement/expression fork in the parser for a smaller set of positions.
   The fork is the defect; narrowing it preserves it.
2. **`test` is refused in a body, not hoisted.** §1.5. Hoisting is buildable and would serve a
   shape AGENTS.md discourages.
3. **An explicit `()` for a unit block.** The alternative is positional silence, which is how a
   function that falls off a loop type-checks as `()` by accident. The cost is one line per
   `()`-returning function and it is paid visibly.
4. **`do` reserved rather than deleted.** §1.7. Plan 60 is about to give `<-` a monadic reading,
   and the token whose Haskell meaning is still unclaimed should not be spent on an identifier.
5. **`then` removed rather than kept as an optional marker.** Keeping it would leave two
   spellings for one construct, which is the redundancy this milestone removes — and would keep
   `.then` unreachable, which is the recovery §1.3 is for.
6. **Discarding stays unmarked.** Requiring `_ = e` for every discard would add a token to every
   `print` line in a `main`. The lints cover the shapes that actually bite.
7. **Leaf 1 first and alone.** It deletes the fork; everything else is a consequence. Attempting
   the consequences first means writing special cases that leaf 1 then removes.

---

## 10. Result

*Not yet executed.*
