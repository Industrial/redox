# Plan 55 — Expression Orientation

Plan ID: `expression-orientation`
Depends on: 52 (`type-colon`), 54 (`assignment`)
Milestone: 9 — *surface coherence*
Status: `partial` — leaves 0 and 1 done

---

## 1. What this plan is

Every body position becomes an expression. Declarations included. `test` excepted, for a reason
that is rustc's and not ours.

This row **deletes grammar**. It does not add any. The statement/expression fork leaves the
parser, four constructs lose their special cases, one keyword is retired, one is freed, and a
class of diagnostics stops being needed. The emitter gets simpler because Rust is already
expression-oriented and the two surfaces stop disagreeing about what a block is.

### 1.1 `if` is the odd one out — and the mechanism is eight lines of the emitter

The behaviour is as this row's first draft described it:

| Construct | Behaves as an expression? | Evidence |
|---|---|---|
| `loop` | **yes** | `break n` as a function tail works; `examples/async.sdx` depends on it |
| `case … of` | **yes** | emits `match` in tail position |
| `do` / `try` / `async` | **yes** | that is their whole purpose |
| indented `if` | **no** | emits `"big";` → rustc `E0308` |

**But the draft was wrong about why, and the correction makes this row much cheaper.** Three
findings from reading the code:

**`loop` and `break` are not expressions.** They are `StmtKind::Loop` and `StmtKind::Break`
variants — there is no `Expr::Loop`. `loop` behaves as one for exactly one reason:
`emit_control` writes `loop { … }` and **never appends a `;`**, so when it is a tail-position
statement rustc reads the braces as the block's value and `break v;` supplies it. Only
`case`, `do`, `try`, `async` and `unsafe` are genuine `Expr` variants.

**The `if` defect is therefore not an AST problem.** `if` and `loop` travel the *same*
`emit_stmt` → `emit_control` path. The difference is that `loop`'s value arrives via `break`,
which carries it explicitly, while an `if`'s arrives from its branches' tails — and
`emit_control` **takes no `tail` parameter at all**, hardcoding `false` at six call sites
(`emit_control.rs` lines 42, 67, 91, 109, 118, 139) plus two more in `emit_control_inline.rs`.
Those eight literals are the whole mechanical cause of `"big";`.

So §5's criterion 1 is reachable by giving `emit_control` a `tail` parameter and threading it to
the branch bodies — an emitter change of a few dozen lines, not the AST-and-parser rewrite leaf 1
was scoped as. Leaf 1 is re-scoped accordingly in §4.

**And `then` does not decide between the two `if` forms — position does.** `Parser::stmt` asks
`control_flow` first, and `control_flow` claims `Kw(If)` unconditionally with no lookahead for
`then`; `if_expression` is reachable *only* from `prefix`, and once there `then` is a hard
`expect`. A consequence worth knowing before leaf 2: **`if c then a else b` written as a bare
statement cannot parse today**, because `if_statement` reads the condition and then demands a
line break. Plan 20 §1.2's account — *"One keyword decides which of the two nodes the parser
builds"* — describes an intent the code does not implement, and `StmtKind::If`'s own doc comment
repeats it. Both are amended by leaf 8.

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

### Leaf 0 — `leaf-tail-through-control-flow`

The cheap half of §1.1, done first because it is what makes criterion 1 pass and it touches no
grammar: give `emit_control` a `tail` parameter and thread it to each branch body, replacing the
eight hardcoded `false`s. `if`, `while`, `for`, `loop` and `guard` all route through it, so this
is also where a tail-position `while` stops silently discarding a value.

Re-bless `tests/golden/control.xrs`. If this leaf alone turns criterion 1 green, leaf 1 is a
grammar simplification rather than a bug fix, and should be judged on that merit.

### Leaf 1 — `leaf-one-expression-node`

Collapse the statement/expression fork in the AST and the parser: `StmtKind`'s **14** variants
join `Expr`'s **34**, giving **46** — `StmtKind::Expr` is absorbed, because a block element *is*
an expression, and `StmtKind::If` merges with `Expr::If`.

**`Block` stays `Vec<Stmt>`, and `Stmt` survives.** This leaf's first draft said it *"gives way
to a sequence of expressions whose value is the last"*. It cannot: `Expr` carries no trivia and
`Stmt` carries `leading` and `trailing`, so a `Vec<Expr>` block would either lose comments —
which AGENTS.md makes load-bearing, doc 02 §10.1 correcting doc 01 §4.1 — or put two trivia
fields on all 46 forms. A comment attaches to a *line*, not to a subexpression. So `Stmt` becomes
the trivia carrier and nothing else: `Stmt { expr, span, leading, trailing }`.

"A block's value is its last element" was already true before this leaf — leaf 0 threaded `tail`
through `emit_stmts`.

Note what this is **not** — leaf 0 has already fixed the observable defect, so this leaf's
justification is the grammar it deletes and the diagnostics it retires, not `"big";`. It remains
the leaf the rest depend on and the one that deletes the most code.

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

This leaf is **net-new in both the AST and the parser**, which §1.4 understated. There is no
declaration-in-body-position anywhere today: `StmtKind` has no `Item` variant, `Parser::stmt`
never calls `item()`, and a declaration keyword in a body reaches `prefix`'s default arm and
becomes `RDX0102`. The nearest existing machinery is `items_until_dedent` plus `emit/items.rs`
and `emit/module.rs`, which already nest items one indent level in — `module` is the only
construct that contains items. Rust accepting nested items (§1.4) is necessary and not
sufficient; the carrier has to be built.

### Leaf 6 — `leaf-test-carve-out`

`test` refused in a body position, with a named code and a help line. The `tests/ui/` fixture
from §3 case 5.

### Leaf 7 — `leaf-retire-do-and-then`

`do` retired as a construct and **reserved** as a token; `try` and `async` become block
modifiers; `'name` labels a bare block; `then` removed and freed. The one corpus `do` and the
`-> do` sites migrate here.

**Retiring `then` has a cost leaf 2 measured, and this leaf must price it.**
Layout is suppressed inside brackets — the lexer returns early while bracket depth is above
zero, which is what lets a record literal and a call span lines — so an indented `if` cannot be
a parenthesised argument and `print (if c then a else b)` is the only spelling there. Without
`then`, a conditional in argument position has to be hoisted to a binding first. That is an
ergonomic cost on a one-line construct, and §6 should disclose it whichever way this leaf
decides. §10.8 has the measurement; `examples/control.sdx`'s `bracketed` is the witness.

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
7. **Leaf 0 before leaf 1, then leaf 1 alone.** Leaf 0 is eight literals in the emitter and fixes
   the defect §1.1 opens with, so it ships first and independently — and once it has, criterion 1
   is green without any grammar having moved, which is the honest basis on which to judge the
   rest. Leaf 1 then deletes the fork; everything after it is a consequence. Attempting those
   consequences first means writing special cases that leaf 1 removes again.

---

## 10. Result

**Leaves 0, 1, 2, 3 and 6 done; criterion 7 withdrawn; leaves 4, 5, 7, 8, 9
outstanding.** Criterion 1 is green. The
statement/expression fork is gone: `StmtKind` is deleted, `Expr` has 46
variants, and leaf 1 moved no golden at all — `git status tests/ examples/`
empty, which is the strongest check available on a refactor across 62 files.
Leaf 2 then made the indented `if` reachable as a value.

### 10.1 Position alone cannot decide the `if` spelling, and a golden caught it

Leaf 1 was specified to merge the two `if` nodes and let **position** choose the
spelling: statement position indents, expression position stays on one line.
That is wrong, and `tests/golden/control.xrs` moved to prove it.

`Parser::inline_body` wraps whatever `parse_expr` returns in a block element, so

```text
tier n = if n > 9 then "big" else "small"
```

is a one-line `if` **standing exactly where a statement stands**. Position
cannot tell it from the indented form, and the first implementation emitted four
lines where the corpus had one.

So the merged node carries `is_inline: bool`. That is the same device
`Expr::Call`'s `is_dollar` is — a record of the spelling, nothing more — and the
same thing `item_text/same_line.rs` already recovers from spans for the
analogous clause-body ambiguity. `is_statement_form()` and `is_control_flow()`
both answer `false` for the inline form, which reproduces the old type-level
split exactly.

A second consequence in the formatter: `else_text` has to pin
`is_inline: false`, or an inline `if` inside an `else` block collapses into an
`else if` chain — a program rewrite, not a cosmetic slip.

### 10.2 The position rule needed no diagnostic, because it has no trigger

The brief for this leaf asked for `RDX0112` on a statement-only form in a nested
expression position. There is **nothing to raise it from**: a nested expression
position is only ever filled by `parse_expr` → `prefix`, and `prefix` has no
production for `let`, `guard`, `with` or `<-`. A code with no trigger has to go
into `code_coverage.rs`'s `NOT_YET_REACHABLE`, which is `&[]` precisely to make
that awkward.

And the brief's premise — *"rustc cannot make this check"* — was already
defeated in-repo. `emit_expr`'s `BinaryOp` fallthrough emits
`compile_error!("… is parsed but not yet lowered")` for exactly this situation.
`emit/unlowerable.rs` follows that precedent: it **is** parsable Rust, so the
fuzz invariant holds; it fails loudly; and plan 07's remapping lands it on the
`.sdx`. No new code, so no merge with the concurrent agent and no renumbering.

### 10.3 The statement-only set, established by feeding rustc the output

`Binding`, `Guard`, and `Await` **only when it has a pattern** — each probed by
emitting the form in expression position and compiling it:

```text
error: expected expression, found `let` statement
```

A *parse* error, which is the class the fuzz invariant forbids. `Draw` joins
them for a different reason: it is a legal Rust expression, but its lowering's
body is **every element after it**, so a nested one silently becomes an empty
closure.

The other eight compile clean in expression position — `return 1`, `break 1`,
`continue`, `x = 1`, `x += 1`, `assert!`, `while`, `for`, `loop`,
`if … else`, `.await`. So the brief's list was right about `Binding` and `Guard`
and incomplete in two ways: `Await` splits on its `pattern` field, because the
effect-only `<- f ()` *is* `f().await` and an expression, while the binding form
is a `let`; and `Draw` is list-shaped rather than merely statement-positioned.

### 10.4 `do` is not redundant yet, and `Try` is not a block

`Expr::Try` is the **postfix `e?`**; `Expr::TryBlock` is the `try` block. After
this leaf: `Do{label: None}` → `{ … }`, `TryBlock` → `(|| { … })()`,
`AsyncBlock` → `async { … }`, `UnsafeBlock` → `unsafe { … }`. None is a synonym
for another.

In particular `do` is **not** redundant, because Redox has no bare-block
expression: a `Block` is only ever a *field* of a node, so `Expr::Do` is the
only spelling of a block expression. Leaf 7 is what makes it redundant, by
giving an indented block expression status. Leaf 1 does not.

### 10.5 The formatter needed more than the emitter

The emitter can fall back on "unreachable" for a form that cannot appear; the
formatter cannot, because it is a **writer** — all 46 forms must print, and
printing the wrong `if` spelling is a program rewrite.

`kind_text` became `code_text`, because `kind` no longer exists and the name was
a lie. `expr_text` gained 13 arms. `power_of` had to **decline** on all 12
promoted forms, which is that function's documented safety default: declining
keeps the author's parentheses.

And the three `redox-fmt` corpus-property tests each had their own duplicate
block walk. All three now call `Expr::statement_blocks()` — which deliberately
excludes the inline `if`'s branches, because including them would hand
`expression_printer` extra statements to assert over and change what the
property reads.

### 10.6 `redox-driver`'s 26-line exhaustive match became one call

`walk_stmts` listed enough `StmtKind` variants to find the five that own a
block. Listing 46 to name five is unreadable, so the no-wildcard guarantee moved
to the AST: `Expr::statement_blocks()` is where a new block-owning form must be
declared, and the three `redox-fmt` walks plus this one all read it. That is
four duplicated walks collapsed into one accessor.

### 10.7 Leaf 2 was one decision, and it exposed two things

Measured before writing anything, which turned out to be the whole of the
design. The one-line `then` form worked in all three positions. The **indented**
form worked in *tail position only* — `RDX0101` bound to a name or passed as an
argument — because `prefix`'s `If` arm reached only `if_expression`, which makes
`then` a hard `expect`. Since leaf 1 made `if_statement` return an `Expr`,
nothing had to move but the choice.

The choice is `reaches_a_then`: `then` at depth zero before the line break. The
same shape as `reaches_a_binding_eq` and `holds_a_pattern_test`, deliberately —
a depth-tracking forward scan that stops at the line break is how this parser
answers every which-production-is-this question, and a third mechanism would be
worse than the duplication. A nested conditional cannot confuse it:
`if (if a then b else c) then d else e` has its inner `then` inside
parentheses.

In *statement* position there is no choice to make and leaf 2 added none:
`stmt` asks `control_flow` before it reads an expression at all, so the
indented form wins by arriving first. That is what keeps an `else`-less `if` a
statement.

**`control_text` discarded `tail`, and leaf 2 made that observable.** Leaf 1
wrote `control_text` for a position the grammar could not yet produce — its doc
comment said so — and passed `false`, because nothing could see the choice.
Leaf 2 produced the position, and

```text
out = if n > 9
  "big"
else
  "small"
```

emitted `"big";` and earned `E0308`. The same defect leaf 0 fixed for tail
position, in the one path leaf 0 could not reach. `true` is safe for the other
three forms `control_text` serves, because `emit_control` forwards `tail` only
to an `if`'s branches and only when there is an `else` — a `while`, a `for`, a
`loop` and an `else`-less `if` all ignore it, which is right, since every one is
`()` in Rust.

### 10.8 Layout is suppressed inside brackets, which is an argument against leaf 7

An indented `if` **cannot** be a parenthesised argument:
`print (if c` + indented branches is `RDX0101`, and not because of anything
leaf 2 did. The lexer suppresses `Indent` and `Newline` while bracket depth is
above zero — the same mechanism that lets a record literal and a call span
lines — so there is no layout inside brackets for the off-side rule to read.

The spelling there is `print (if c then a else b)`, and that is the strongest
argument for keeping `then` that this row has found. **Leaf 7 proposes retiring
it.** After leaf 7, a conditional in argument position would have to be hoisted
to a binding first:

```text
v = if c
  a
else
  b
print v
```

That is not unspellable, and it is a real ergonomic cost on a construct that is
one line today. Leaf 7 should price it, and §6 should disclose it either way.
`examples/control.sdx`'s `bracketed` is the witness, written in the interpolated
form so the example also proves `#{…}` carries a conditional.

### 10.9 The second parser, again

`tree_sitter.rs` and `editors.rs` both went red the moment the Rust parser
accepted the new position — doc 04's fifth ordering commitment, earned by plan
52 and paid again here. `conditional_expression` reached only the `then` form,
so the grammar needed `if_statement` as an alternative, and reusing that rule
rather than copying it is the point: the two spellings are one AST node since
leaf 1, and two grammar rules for one node is how a highlighter drifts from its
compiler.

GLR then produced two conflicts the Rust parser does not have, both resolved by
saying out loud what the Rust parser gets for free:

- **`prec(-1, $.if_statement)`** inside `conditional_expression`, so the
  *statement* reading wins where both are possible. The Rust parser gets that
  by asking `control_flow` first.
- **`prec.right` on `if_statement`**, for the dangling `else`:
  `x = if c then if d …block… else …` became ambiguous about which `if` owns
  the `else`. Right-associative binds it to the nearest one, which is what the
  Rust parser does by greedy recursion — `if_statement` consumes its own `else`
  before returning, so the outer `if_expression` then has none and says so.

### 10.10 Leaf 3 — compound assignment was not free, and `-> do` needs no diagnostic

The expectation going in was that `n += 1` as a bare arm body would already
work, `Assign` being an `Expr` variant since leaf 1. **It did not**, and the
reason is worth keeping: `parse_expr` never reads an assignment operator —
`Parser::stmt` read it *after* calling `parse_expr`. So
`Exit::Success _ -> enriched += 1` stopped at the `+=` and died on `RDX0101`,
exactly as §2 recorded. Leaf 1 changed where an assignment can be **stored**,
not where it can be **parsed**.

The fix is `assign_tail`, extracted from `stmt` so the arm body and the block
element read the same six operators from one place rather than two.

**An arm body's shape is unchanged: still `body: Expr`.** An indented block
becomes `Expr::Do { label: None }`, which §10.4 establishes as Redox's only
spelling of a block expression. Three things fall out of that, all of them
good: `-> do` keeps working with nothing written for it, both spellings emit
byte-identical Rust, and **leaf 7 removes a keyword rather than a form** — once
`prefix` parses a bare indented block, `arm_body`'s special case simply goes.

**`-> do` gets no deprecation diagnostic, and should not.** A diagnostic needs
something to distinguish, and after this leaf there is nothing: the two
spellings produce the same node, so a warning would have to fire on a token the
AST no longer records. Recovering it would mean an `is_marked: bool` beside
`is_inline` — a spelling flag kept alive solely to complain about itself, one
milestone before the keyword goes.

The deprecation is carried where it costs nothing instead: **`redox fmt`
rewrites `-> do` into the marker-free form.** That is doc 02 §4.5's migration
tool doing the migration, it needs no new code, and it avoids a second
deprecation code beside plan 53's `RDX1002`.

Two emitter decisions, both driven by rustfmt rather than by taste: a block
body whose block is one plain expression with no trivia collapses to
`pat => expr,`, and a block body takes **no trailing comma**, matching
rustfmt's `arm_comma`. The second closed a latent bug — `emit_case` wrote
`},` after a `-> do` body, which rustfmt rewrites, and **no corpus site used
`-> do`**, so it had never been exercised. Leaf 3 would have hit it on its
first block arm.

### 10.11 Leaf 6 — and the `help` field the parser did not have

`RDX0112` refuses a `test` in a body position. The carve-out is §1.5's: a
nested `#[test]` is **not collected**, so rustc warns `cannot test inner
items` and the test silently never runs. A construct that compiles and then
does nothing is worse than one that is refused.

The check lives in `Parser::stmt` rather than in `prefix`, because `stmt` is
where a body position begins — one comparison covers clause bodies, guard-row
bodies, `where` lists and every statement-owned block. It **consumes** the
declaration before raising, and that is not tidiness: a local `test` used to
produce **two** `RDX0102`s, the second from recovery landing inside a construct
that was never read.

`ParseError` gained `help: Option<String>` to carry the help line. It had none,
while `LexError` has carried one since plan 05, and `diagnose` hard-coded
`help: None` for the whole syntactic half — so this is the matching half rather
than a new idea. `expected` and `with` set `None`, so no existing diagnostic
moved.

### 10.12 Criterion 7 is withdrawn, and the position it wanted is already there

*"A discarded `Result` is a Redox lint, not only a rustc warning."* Measured
before building anything, which is what retired it.

**The position is already correct.** Plan 07's remapping puts
`unused_must_use` on the author's own line and column, for a `std` call and a
locally-declared one alike, one warning per site, exit 0 — under `redox check`,
which is where rustc runs at all. Leaf 4 worded this criterion's value *as*
position; there was nothing to move.

**And the breadth it asks for needs a type checker.** Exactly one discard shape
is decidable from the source: a call to a function whose return type is written
in the same file. `u8::try_from 300` discards a `Result` and **nothing in the
`.sdx` says so** — knowing it means reading the standard library's signatures,
which is the checker AGENTS.md forbids. Since the decidable shape is a strict
subset of where rustc already fires, a lint would have had *worse* coverage than
the warning it was meant to improve on, plus a false positive rustc does not
have: a `where` local shadowing a module-level `Result`-returning name, which
only name resolution could tell apart.

A hole was looked for and not found: a `Result` in the tail of a `()` function
is `E0308`, in a `case` arm against `()` arms is `E0308`, and everywhere else is
`unused_must_use`.

So criterion 7 joins plan 56 leaf 3 and plan 59 criterion 5 — withdrawn on
measurement rather than built on a heuristic. The argument lives in
`crates/redox-cli/tests/check/discarded_result.sdx` and its test, so it is in
the tree rather than only in a plan.

**One real gap surfaced, and it is plan 07's.** `remap` keeps only positioned
`level: message` lines, so rustc's `note: this Result may be an Err variant,
which should be handled` and `help: use let _ = … to ignore the resulting
value` are **dropped**. The author gets the position and never the remedy. That
is true of every remapped rustc diagnostic, not just this one, and fixing it
would move `tests/check/` expectations across the board.

### 10.13 Open

- **Leaves 2-9.** Criterion 1 is green from leaf 0 and the fork is gone; the
  remaining criteria — `.then` as a method, a `case` arm's indented block, local
  declarations, the `test` carve-out, the explicit `()`, the discarded-`Result`
  lint, retiring `do` and `then`, the labelled bare block — are untouched.
- **`StmtKind` is named in documents that no longer describe the code**:
  `docs/06 - Redox Code Architecture.md`'s two rows for `block/stmt/kind.rs`,
  plan 20 §§6-7, plan 30 §4, plan 34 §§10.15/11, plan 54 §§10/11.3, plan 60
  §11.4, and this row's own §4. None is machine-checked. Leaf 8 owns them.
- `crates/redox-fmt/tests/fmt/statements.sdx` line 1 still says *"every
  `StmtKind`"*. It is a fixture whose formatted output is compared, so editing
  the comment means editing an expectation pair for a historical note.
- `emit_expr`'s cognitive complexity rose from 13 to 21. The twelve new arms are
  mostly one-liners, and splitting the match would cost the exhaustive
  no-wildcard property its doc comment argues for. Worth a look if it grows
  again.
- **A history muddle, recorded rather than rewritten.** Leaf 1 used `git rm`,
  which stages; commit `2739b2b` — a docs commit — picked three file deletions
  out of the index as a result. `block/stmt/kind.rs`,
  `stmt_text/kind_text.rs` and `parser/blocks_in_the_same_closure.rs` are
  therefore deleted under a message about orienting plans 59-64. The code is
  correct and complete; only the attribution is wrong, and fixing it means
  rewriting a commit that already has four descendants.
