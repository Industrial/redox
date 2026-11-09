# Plan 22 — The Draw Arrow in Condition Position, and `guard`

Plan ID: `is-and-guard`
Depends on: 13 (`haskell-surface`), 20 (`control-flow`), 21 (`switch-and-patterns`)
Milestone: 5 — *the language, as specified*
Status: `planned`

---

## 1. What this plan is

The third and fourth of doc 18 §3's four positions for `<-`: **in a condition**, and **in a
guard**. Plus `guard`, which survives doc 18 and has had a keyword and no parser since
plan 02.

```text
if Move x _ <- msg && x > 0       if let Move(x, _) = msg && x > 0 {
while Some v <- queue.pop ()      while let Some(v) = queue.pop() {
guard Ok raw <- read p else       let Ok(raw) = read(p) else {
  return Err Error::Unreadable        return Err(Error::Unreadable)
                                  };
case x of
  y | Some v <- y -> v            y if let Some(v) = y => v,
```

Every one of those lowerings is a Rust construct of the same shape. What this plan adds is a
*pattern in three places that currently take only an expression*, and the keyword that takes
one at statement level.

**This paragraph originally claimed no new node type was needed, and that was wrong twice
over** — §10.1 and §10.3 record it. `Pattern` had no constructor-application form, so
`Some v` could not be written at all; and the condition needed a node of its own to hold a
let-chain's operands. The claim is left corrected rather than deleted, because a plan that
quietly rewrites its own premise teaches nothing.

### 1.1 The row as doc 04 wrote it, and what doc 18 did to it

Row 22 reads: *"`is` as binding-aware pattern test (`matches!` / `if-let` / let-chain);
`guard` onto let-else; derived `while let`."*

| Named by the row | Doc 18 | What does its work |
|---|---|---|
| `is` | §3's table, deleted | `<-` in a condition, *"with the operands in Haskell's order"* (§3.1) |
| `matches!` lowering | follows the deletion | — see §6 |
| `if let`, let-chain, `while let` | kept | this plan |
| `guard` onto let-else | kept, §3.1: *"`guard` survives"* | this plan |

So the row's substance is unchanged and only its spelling moved. `is` was infix with the
scrutinee on the left; `<-` puts the pattern on the left, which is the order every other
`<-` position already uses and the order Haskell reads.

### 1.2 Why this is three parser sites and not one

A pattern becomes legal in three places, and they are reached by three different functions:

1. **An `if` condition** — plan 20's `if_statement` and `if_expression`.
2. **A `while` condition** — plan 20's `control_flow`, which also owns `until`.
3. **A `case` arm guard** — plan 21's guard list, after `|`.

Each one currently calls `parse_expr`. Each one must instead try *pattern* `<-` *expression*
first and fall back to an expression. The try is cheap and unambiguous for the reason doc 18
§3.1 gives: the four `<-` positions are grammatically disjoint, and within a condition, a
pattern followed by `<-` cannot be anything else.

### 1.3 `until Some v <- x` is a trap, and this plan must not fall into it

Plan 20 lowers `until c` to `while !(c)`. A pattern test cannot be negated that way:
`while !(let Some(v) = x)` is not Rust, and `while let Some(v) = x` with the body
negated means nothing like `until`.

So **`until` with a pattern is an error**, reported with its own code, not lowered to
something plausible. Rust's equivalent of *"loop until this matches"* is
`while !matches!(x, Some(_))` — which binds nothing, and so cannot be what a binding
construct meant. Reporting it is the only honest answer, and it is doc 02 §5.2's rule
applied to a construct that does not exist rather than to one that means the wrong thing.

### 1.4 `guard`'s lowering is already written down

Doc 03 §7.1 gives it character for character, and doc 18 §3.1 says to keep it and change
only the operator:

```text
guard Ok raw <- read p else        let Ok(raw) = read(p) else {
  return Err Error::Unreadable         return Err(Error::Unreadable)
                                   };
```

The else branch must diverge, *"which is exactly `guard`'s own requirement"* — and rustc
enforces it. Redox does not check divergence: that is a check rustc can make, so rustc makes
it, and plan 07's remapping reports it against the `guard` line. This plan's job is to get
the line numbers right and nothing more.

### 1.5 The one thing that was unstable and is not

`if let` inside a **match guard** is `if_let_guard`, which was unstable for years — long
enough that doc 18 §3's *"in a condition or guard"* may have been written without checking.
It is stable as of rustc 1.97.1, verified by compiling

```rust
match x { y if let Some(Some(v)) = y => v, _ => 0 }
```

with `--edition 2024` and no feature gate. Let-chains in `if` and in `while` are stable too,
checked the same way. So all four lowerings are plain stable Rust and none of them needs a
fallback — **but this was verified before the plan was written, not assumed**, because a plan
that promises an unstable lowering is a plan that cannot ship.

---

## 2. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes. `P <- e` in a condition is `let P = e`; the shape is syntactic |
| Nothing outside core/alloc/std? | Yes. No lowering names a type at all |
| Preserves Rust semantics? | Yes — each form *is* the Rust form, reordered |
| Buys reachability? | Yes. Without it, `Option` and `Result` can only be taken apart by `case`, and a fallible early return has no spelling |

---

## 3. Concrete proof this plan targets

`examples/draw_condition.sdx`, which must **run** and print a sequence only a correct
lowering produces:

- an `if` pattern test whose binding is used in the body;
- a **let-chain** — `if P <- e && cond` — where `cond` reads the binding `P` made, which is
  the whole reason let-chains exist and the one case a two-statement lowering cannot express;
- a `while` pattern test that drains a collection, so a wrong lowering either loops forever
  or stops early;
- a `guard` whose else branch returns, called twice: once taking the early exit and once
  not;
- a `case` arm guard that binds.

The let-chain is the case to get right. `if Some v <- x && v > 2` must be one condition;
lowered as two nested `if`s it behaves identically here and differently as soon as an `else`
is attached, which is doc 02 §5.2's failure mode with a delay fuse on it.

---

## 4. Leaves

### Leaf 0 — `leaf-constructor-pattern`

**Added during implementation, not planned.** `Pattern::Apply { path, args }` — `Some v`,
`Ok raw`, `Event::Close d` — applied by juxtaposition, with parentheses required in an
equation's row as in Haskell. No row owned it and nothing in this plan can be written
without it; §10.1 and §10.2 record why it landed here.

### Leaf 1 — `leaf-pattern-condition`

A `condition` parser that reads *pattern* `<-` *expression* or an expression, and the node
to hold it: `Condition::Pattern { pattern, source }` beside `Condition::Expr(Expr)`. One
type used by all three sites, so a lowering fix reaches all three.

### Leaf 2 — `leaf-if-and-while`

Plan 20's `if` (both forms) and `while` take a `Condition`. `until` with a pattern is
`RDX0107`, per §1.3.

### Leaf 3 — `leaf-let-chains`

`&&` between conditions, where any operand may be a pattern test. The chain is flat and
emits as Rust's, which is why no nesting is involved.

### Leaf 4 — `leaf-guard`

`guard P <- e else <diverging>` at statement level, lowering to let-else. The keyword has
existed since plan 02 and has never been read.

### Leaf 5 — `leaf-arm-guard`

A pattern test in a `case` arm's guard list, lowering to `if let` in a match guard — stable,
per §1.5.

### Leaf 6 — `leaf-proof-and-gate`

`examples/draw_condition.sdx`, emit tests per form, a golden, a UI case for `RDX0107`, and
the gate.

---

## 5. Acceptance criteria

1. `if P <- e` emits `if let P = e`, and the binding is usable in the body.
2. `if P <- e && c` emits a **let-chain** — one `if`, two operands joined by `&&` — and `c`
   may read `P`'s bindings.
3. `while P <- e` emits `while let P = e`.
4. `while P <- e && c` emits a while-let chain.
5. `guard P <- e else <d>` emits `let P = e else { <d> };`, with the else body at the right
   indent, for `return`, `break`, `continue` and a panic form.
6. A `case` arm guard may be a pattern test, emitting `if let` in the match guard.
7. `until P <- e` is **`RDX0107`**, reached by the UI corpus, and is never lowered.
8. An ordinary expression condition still emits exactly what plan 20 emitted — asserted, so
   the new node does not change the old path.
9. Every pattern form plan 21 ships works in every one of the three positions, since the
   vocabulary is shared and sharing it is the point.
10. `examples/draw_condition.sdx` compiles with **zero** rustc output, runs, and matches its
    `.stdout`; a golden covers the text.
11. `Some v` applies by juxtaposition, `Some (Some v)` nests, a nullary constructor emits no
    parentheses, and a parenthesised application works in an equation's row.
    *Added with leaf 0.*
12. A broken let-chain is laid out as rustfmt lays it out, by the shape rule §10.3 bisected
    — not by a width. *Added with leaf 3, after the corpus caught it.*

---

## 6. Out of scope, and what it discloses

- **`matches!`** was row 22's third lowering and is now unreachable: it was what `is` emitted
  where no binding was wanted, and `<-` always binds. A pattern test that discards its
  binding is `if _ <- e`, which is `if let _ = e` — true always, and useless. Writing
  `matches!` by hand is still available, since Redox passes macro calls through. **Nothing
  in Redox emits `matches!` after this plan**, which is a deletion doc 18 caused and did not
  record; §10 should correct doc 18's §3 table if it still lists one.
- **`if let … else`** chains where the `else` binds — Rust has no such form, so neither does
  this.
- **A pattern in a comprehension's `|` guard** is already a generator (`<-` position four)
  and so is plan 29's, not this plan's. The two read alike and are told apart by being inside
  `[ … ]`.
- **`guard` with no `else`** is not Redox. Doc 03 §7.1's form has one, let-else requires one,
  and a `guard` that cannot exit is a `let`.
- **Divergence checking** of the else branch stays rustc's, per §1.4 and AGENTS.md.
- **rustfmt's layout of a broken chain in a value-producing conditional, and in a match
  guard**, is not replicated — §10.6 shows both and says why. The emitted Rust is valid and
  rustfmt would reflow it; row 34's formatter is where that belongs.
- **Making plan 20's indented `if` a tail expression** is plan 20's row, not this one's.
  §10.7 records the gap the example ran into and where the fix goes.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald had no let-else and no pattern-in-condition form; its `guard` analogue
lowered to its own IR.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | One `Condition` type used by `if`, `while` and arm guards | A pattern flag on each site | Three sites with the same two cases is one type; three flags is three chances to fix a lowering in two places |
| 2 | `until` with a pattern is an error | Lower to `while !matches!(…)` | That form binds nothing, so it cannot be what a binding construct meant — §1.3 |
| 3 | Let-chains are flat | Nest an `if` per operand | Nesting is observably different the moment an `else` is attached, and Rust's own form is flat |
| 4 | Verify `if_let_guard`'s stability before planning the leaf | Trust doc 18 §3's table | A plan that promises an unstable lowering cannot ship. It is stable; the check cost one file — §1.5 |
| 5 | `guard` keeps rustc's divergence check | Check it in Redox | AGENTS.md: every check rustc can make, rustc makes |

---

## 10. Result

Shipped. 585 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit` are green. Every
criterion is met, and §1's opening claim — *"no new node type is needed for any of them"* —
was wrong twice over.

### 10.1 The pattern every `Option` needs had no owner

§1 said the pattern vocabulary was plan 21's. It was not: `Pattern` held literal, range,
alternation, wildcard and **nullary** constructor forms, and nothing for `Some v`. The
`data` example works only because it was written with payload-free constructors —
`describe Close` — and `examples/case.sdx`'s own header says constructor alternation *"needs
§2.4's `data` to have constructors to alternate"*, which is a different gap recorded in the
same place.

So `if Some v <- x` could not be written at all. Row 21's remainder is named *"record, slice
and as-patterns"*, and this is the **tuple-variant** form, which is none of those and is the
one pattern every use of `Option` and `Result` needs. `Pattern::Apply` is implemented here
for the same reason plan 20 implemented compound assignment: the row that introduces the
position has to introduce the pattern, or it ships a position nothing can be written in.

Fifth row whose scope was smaller than the thing it claimed to deliver — after plan 14's doc
03 §13.5, plan 16's `const fn`, plan 18's `use T::*`, and plan 20's assignment. The pattern
across all five is the same: **a surface the specification shows in an example but no row's
sentence names.**

It arrived with three things the plan did not ask for, because the vocabulary is shared:
`case Some 0 or None ->` alternates applications, `depth (Some n) = n` takes one in an
equation's row, and `Event::Close d` takes a path. All three are asserted.

### 10.2 Juxtaposition in a pattern needs Haskell's parentheses, in two places

`Some v` reads greedily, so a *row* of patterns is ambiguous — `f Some v` could be
`f (Some v)` or the three patterns `f`, `Some`, `v`. Haskell requires parentheses in a row
and so does Redox, which is why `pattern_app` is called only from positions holding exactly
one pattern and `pattern` gained a parenthesised-group arm for the row.

The same rule bites one level down: `Some Some v` is `Some` applied to two arguments, and
`Some (Some v)` is the nesting. rustc rejects the first on arity, so the division of labour
holds, but the parentheses are the spelling and the example says so.

A **tuple** pattern — `(a, b)` — is deliberately not this. It needs a comma, and it is plan
33's along with the record and slice forms.

### 10.3 rustfmt breaks a let-chain on shape, not on width

The example failed `generated_rust_is_rustfmt_clean` on one line, and the cause is a rule
with no width in it. Bisected by running rustfmt over seven shapes:

| Condition | rustfmt |
|---|---|
| `if let Some(v) = o` | one line |
| `if a && let Some(v) = o` | one line |
| `if let Some(v) = o && v > 2` | **broken** |
| `if let Some(v) = o && let Some(w) = p` | **broken** |
| `if a && b && let Some(v) = o` | **broken** |
| `if a && b` | one line — no `let`, so not a chain |
| `if a && let Some(verylongbindingnamehere) = o` | one line, over width |

So: **one line iff there are at most two operands and no `let` before the last.** The last
row is what settles it as a shape rule — a chain rustfmt could have broken for width it
leaves alone, and a chain that fits in thirty columns it breaks anyway.

This is the sixth rustfmt behaviour pinned by experiment rather than reasoning, after plan
13's five width constants, and for the reason plan 14 §10.2 records: *reasoning from half of
rustfmt's behaviour* is what produced the import-grouping defect.

### 10.4 Two scans, and the bug that needed both

The first implementation asked one question — *does a `<-` appear before this operand ends?*
— and used it for the condition as a whole. That made `if n > 0 && Some v <- x` a parse
error: the scan stopped at `&&` and reported no pattern, so the whole condition went down the
expression path and the `<-` had nowhere to go.

The two questions are genuinely different. *Does the **condition** hold a pattern test?* runs
past `&&`. *Is **this operand** a pattern test?* stops at it. Criterion 2's own test caught
it, which is the argument for writing the three-operand case down rather than assuming two
would generalise.

The expression path is kept deliberately: a condition with no `<-` is parsed by one
`parse_expr(p, 0)` call, byte for byte what plan 20 did. That is not an optimisation —
parsing operand-wise changes what the precedence climber sees, and `a && b || c` read as two
`&&` operands leaves `|| c` unread. Criterion 8 asserts five such conditions.

### 10.5 `if_let_guard` was stable, and checking was still right

§1.5 verified it rather than trusting doc 18 §3's table, and the verification came back
*stable* — rustc 1.97.1, `--edition 2024`, no feature gate, along with let-chains in `if` and
in `while`. The check cost one file and five minutes, and the alternative was discovering at
the gate that a whole leaf could not ship. Decision 4 was right for a reason that did not
materialise, which is the only kind of right a precaution gets to be.

### 10.6 Two formatting divergences, disclosed

A **broken** chain is formatted as rustfmt would in an `if` statement, an `else if` and a
`while`. It is **not** in two places, both verified rather than assumed:

```rust
// Redox emits                               // rustfmt writes
if let Some(v) = o && v > 2 { "a" }          if let Some(v) = o
  else { "b" }                                   && v > 2
                                             { "a" } else { "b" }

y if let Some(v) = y && v > 0 => v,          y if let Some(v) = y
                                                 && v > 0 =>
                                             { v }
```

The arm-guard form is the awkward one: rustfmt converts the arm body to a block, so the
divergence is structural and not a line break. Both are rare shapes — a let-chain in a
value-producing conditional, and a let-chain in a match guard — and neither appears in the
corpus, so `generated_rust_is_rustfmt_clean` does not catch them. **That is the point of
writing them down**: a gate that cannot see a divergence is not evidence there is none. Row
34's formatter is where they belong, and until then the emitted `.xrs` is valid Rust that
rustfmt would reflow.

### 10.7 A gap in plan 20, found by using it

The example's `describe` first read:

```text
if Some v <- o && v > 2
  "big"
else
  "small"
```

and emitted `return "big";`'s predecessor — `"big";`, a statement — because plan 20's
indented `if` is a **statement** and therefore never a tail expression. rustc caught it
(*mismatched types*), so nothing silent happened, but the natural spelling of a
value-producing conditional does not work: Rust's `if` is an expression and Redox's indented
form does not use that.

Not fixed here, because it is plan 20's row and not this one's; `emit_stmts` already carries
a `tail` flag, so the fix is small and belongs where it can be tested against plan 20's own
corpus. The example uses `return` in each branch and says why.

### 10.8 The architecture tests caught two things, in one plan

- `condition.rs` and `pattern_app.rs` each declared three or four constructs. Split.
- The split then put them **four** levels below `src/`, over doc 06 §2.4's cap of three. The
  helpers became siblings under `parse/parser/` instead — which is where `pattern_alt.rs` and
  `range_or_literal.rs` already sat, so the layout the cap forced is the layout the crate
  already used.

Sixth and seventh time these tests found something reading did not. The second one is the
more useful: it rejected a module tree that compiled, passed every behavioural test, and was
shaped unlike every sibling of it.

### 10.9 One true error reported once

`RDX0107` initially returned before reading the loop body, so recovery resumed *inside* a
block it had not consumed and emitted a second, cascading *"expected an expression"* against
the loop's own next line. One true error reported twice reads as two defects. The decline now
reads the body first and reports after — the UI expectation is a single diagnostic, which is
what the corpus is for.

### 10.10 `matches!` is gone, as §6 predicted

Nothing in Redox emits `matches!` after this plan. It was what `is` produced where no binding
was wanted, and `<-` always binds. Doc 18 §3's table should not be read as implying one.
