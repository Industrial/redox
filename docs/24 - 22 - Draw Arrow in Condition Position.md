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

Every one of those lowerings is a Rust construct of the same shape. **No new node type is
needed for any of them** — the pattern vocabulary is plan 21's, the condition is plan 20's,
and the arm guard is plan 21's. What this plan adds is a *pattern in three places that
currently take only an expression*, and the keyword that takes one at statement level.

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

*Filled in when the plan ships, including anything above that turned out wrong.*
