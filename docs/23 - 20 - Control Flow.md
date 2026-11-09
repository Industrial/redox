# Plan 20 — Control Flow

Plan ID: `control-flow`
Depends on: 03 (`parser-core`), 13 (`haskell-surface`), 18 (`attributes-passthrough`)
Milestone: 5 — *the language, as specified*
Status: `planned`

---

## 1. What this plan is

`if`, `while`, `until`, `for`, `loop`, `do`, and the three ways out of them.

Redox has **no control flow at all** today. A function body is a run of statements and a
guard ladder, and that is the whole of it — `if x > 0` is a parse error (§2). This is the
largest row shipped so far and the one that most changes what a Redox program can be.

### 1.1 What doc 18 deleted from this row, and what is left

Doc 04's row 20 reads: *"`if`/`unless`/`else`, `while`/`until`, `for`, postfix forms,
`cond`, `loop`, `'label` on blocks and loops, `break` with value."* Doc 18 deleted three of
those outright, so the row is smaller than it was written:

| Named by the row | Doc 18 | What does its work |
|---|---|---|
| `cond` | §7's table, deleted | Guards — the `\|` ladder, shipped |
| `unless` | §7's keyword list, retired | `if not …`, which is what `not` returning to boolean negation is *for* (§2.3) |
| postfix `if`/`unless`/`until` | §7's operator table, retired | *"Their expanded forms"* |
| `if`/`else`, `while`/`until`, `for`, `loop`, `do`, labels, `break` with value | kept | this plan |

So the deletions are already paid for and the plan builds what remains.

### 1.2 Two `if`s, and why that is Haskell's doing rather than ours

Doc 18 §7 arrives **`then`**, which exists for exactly one construct: Haskell's conditional
*expression*. Nothing else in doc 18 uses the word. So the plan reads two forms, and they
are grammatically disjoint:

```text
tier = if n > 9 then "big" else "small"    let tier = if n > 9 { "big" } else { "small" };

if n > 9                                   if n > 9 {
  print "big"                                  println!("big");
else                                       } else {
  print "small"                                println!("small");
                                           }
```

**The `then` form is an expression and the indented form is a statement**, told apart by
whether `then` follows the condition. One token of lookahead after the condition settles it,
and no backtracking is needed — the same property doc 18 §3.1 relies on for `<-`.

Both lower to Rust's `if`, which is itself an expression. Keeping two surface forms for one
Rust construct is the one place this plan adds a spelling rather than a lowering, and the
reason is that `then` is otherwise a keyword with no use, which would be worse.

### 1.3 `until` is `while` with a negation, and `not` is why it reads

`until c` is `while !(c)`. Doc 03 §7.7 keeps it from doc 01 §8, and doc 18 §2.3 freed `not`
to mean boolean negation — so `until` and `if not` are now the same word used two ways, and
both are legible. The parenthesising matters: `until a && b` is `while !(a && b)` and not
`while !a && b`, which would be a different loop.

### 1.4 `for x <- xs`, which is `<-`'s fourth position

Doc 18 §3's table lists `<-` *"after `for`, or in a comprehension"* as a generator, and the
comprehension half shipped with §3.2. This is the other half:

```text
for c <- candidates        for c in candidates {
  print c              →       println!("{}", c);
                             }
```

The four positions are *"grammatically disjoint, so no lookahead is needed to tell them
apart"* — after `for` is the simplest of them, since a `for` can hold nothing else.

### 1.5 `do` blocks, labels, and what `break` can carry

Doc 03 §7.2 keeps doc 01 §19's `do` and gives it an optional label, *"permitting early exit
with a value"*. Both forms are native Rust — a block expression, and a labelled block
expression since 1.65:

```text
found = do 'search                     let found = 'search: {
  for c <- candidates                      for c in candidates {
    if c.matches q                             if c.matches(q) {
      break 'search Some c             →           break 'search Some(c);
  None                                         }
                                           }
                                           None
                                       };
```

Labels apply to loops as well, so `break 'outer` and `continue 'outer` reach an enclosing
one. `break` takes an optional label and an optional value, in that order, and `continue`
takes an optional label — which is Rust's grammar exactly.

### 1.6 Where this leaves `return`

`return` already exists, from plan 01. It is untouched: a `return` inside an `if` inside a
`while` works because each of those is a block of statements and `return` is a statement.
Nothing about this plan changes it, which is worth saying because it is the one control-flow
construct that was already there.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| Every keyword this plan needs is reserved | `keyword.rs` | `If`, `Then`, `Else`, `While`, `Until`, `For`, `In`, `Loop`, `Do`, `Break`, `Continue` |
| `'label` is lexed | `kind.rs` | `TokenKind::Label(String)`, and `token_text` spells it `'name` |
| There is no control flow today | `redox build` on `if x > 0` | `RDX0102: expected an expression` at the `if` |
| An indented statement block is already a parser | plan 14 | `Parser::statements`, lifted out of `clause` for `test` and reused here |
| `not` is boolean negation | doc 18 §2.3, and `UnaryOp::Not` | `!` is emitted, so `if not c` needs nothing new |
| A statement already carries trivia and attributes | plans 02, 18 | `#[allow(…)]` above an `if` will work with no extra work |

---

## 3. Concrete proof this plan targets

`examples/control.sdx`, which must **run** and print a sequence only a correct lowering
produces: a loop that counts, a `break` with a value, a labelled exit from a nested loop,
and an `until` whose condition is a conjunction — because `while !a && b` and
`while !(a && b)` differ on the second iteration and nowhere else.

That last case is the one a golden could not catch and a run can.

---

## 4. Leaves

### Leaf 1 — `leaf-if`

`StmtKind::If { condition, then_block, else_branch }` and `Expr::If { condition, then, otherwise }`,
told apart by `then`. `else if` is an `else` whose branch is another `if`, which is how Rust
reads it too.

### Leaf 2 — `leaf-loops`

`StmtKind::While { condition, negated, body, label }` — one node for `while` and `until`,
with a flag, for the reason plan 16 gave for `const` and `static`. `StmtKind::For { binder,
source, body, label }` and `StmtKind::Loop { body, label }`.

### Leaf 3 — `leaf-do-and-labels`

`Expr::Do { label, body }`, emitted as a block or a labelled block. The label on a loop is
part of leaf 2's nodes, so this leaf is the block form and the `'name:` prefix.

### Leaf 4 — `leaf-break-continue`

`StmtKind::Break { label, value }` and `StmtKind::Continue { label }`.

### Leaf 5 — `leaf-assignment`

**Added during implementation, not planned.** `StmtKind::Assign { target, op, value }` for
doc 01 §4.3's five compound operators. No row owned statement-level assignment, and §3's
counting loop does not terminate without it — §10.1 and §10.2 record why it landed here and
what it deliberately does not cover.

### Leaf 6 — `leaf-proof-and-gate`

`examples/control.sdx`, emit tests per construct, a golden, and the gate.

---

## 5. Acceptance criteria

1. `if c then a else b` is an **expression** and emits `if c { a } else { b }`.
2. `if c` with an indented block is a **statement**; `else`, and `else if` chains, emit as
   Rust's.
3. `while c` emits `while c`; `until c` emits `while !(c)`, **parenthesised** — so
   `until a && b` is `while !(a && b)`.
4. `for x <- xs` emits `for x in xs`.
5. `loop` emits `loop`.
6. `do` emits a block expression; `do 'name` emits `'name: { … }`.
7. `break`, `break v`, `break 'l`, `break 'l v`, `continue` and `continue 'l` each emit
   their Rust.
8. A label on a `while`, `until`, `for` or `loop` emits `'name: ` before the keyword.
9. A `return` inside any of these still works — asserted, because §1.6 claims it for free.
10. `examples/control.sdx` compiles with **zero** rustc output, runs, and matches its
    `.stdout`; a golden covers the text.
11. Any new diagnostic has an `RDX` code reached by the UI corpus.
12. `n += 1` and the other four compound operators emit their Rust, and a compound
    assignment to a field — doc 18 §2.5's `self.value += 1` — works, because the target is
    an expression and not a name. *Added with leaf 5.*
13. A plain `=` is still a **binding**, asserted, so §10.2's disclosed limit is a tested
    fact rather than an accident. *Added with leaf 5.*

---

## 6. Out of scope, and what it discloses

- **`cond`, `unless` and the postfix forms** are deleted by doc 18, not deferred. Writing
  `unless` gets "expected a name to begin a declaration", because it is no longer a keyword
  — which is the correct outcome and an unhelpful message. Row 38's lint pass is where a
  "did you mean `if not`?" would belong.
- **`if let` and `while let`** need `<-` in *condition* position, which is row 22. So
  `while Some v <- x` is not this plan, and the `while` it would use is.
- **`for` over a range needs no special case**, because `1 .. 4` is an expression and the
  receiver rule parenthesises it — the same machinery plan 13's comprehension needed.
- **Plain reassignment** — `counter = 2` — is not deferred, it is **unspecified**: no
  surface doc gives it a spelling, and telling it from a binding needs scope tracking
  AGENTS.md forbids. §10.2 states the consequence: a loop can advance a numeric accumulator
  and cannot rebind anything else.
- **Loop labels on a `do` block that is not a loop** are legal Rust (`'a: { break 'a v }`)
  and this plan emits them, but `continue 'a` on a block label is not. Redox will emit it
  and rustc will reject it, which is the division of labour AGENTS.md states.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald's control flow lowers to its own IR, not to another language's statements.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Two `if` forms, told apart by `then` | One form, either expression or statement | Doc 18 §7 arrives `then` for exactly this, and a keyword with no use is worse than a second spelling. The two are grammatically disjoint, so neither costs lookahead |
| 2 | `until c` is `while !(c)`, parenthesised | `while !c` | `until a && b` must be `while !(a && b)`; the unparenthesised form is a different loop and would be a silent one — doc 02 §5.2's worst failure mode |
| 3 | One node for `while` and `until`, with a flag | Two nodes | They differ by a negation; plan 16 made the same call for `const` and `static` |
| 4 | `for x <- xs` and not `for x in xs` | Keep Rust's `in` | Doc 18 §3's table lists `<-` after `for` as one of its four positions, and `in` is left to Rust |
| 5 | A `do` block is an `Expr` | A statement | §7.2's own example assigns one to a binding, so it has to be an expression; a statement form would need a second node for no gain |

---

## 10. Result

Shipped. 563 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :cargo-check` are green. Every criterion is met as written, and the
plan grew one leaf it did not have.

### 10.1 No row owned assignment, and loops do not work without it

§3 asked for *"a loop that counts"*. The first attempt at one emitted this:

```rust
while n < 3 {
    let n = n + 1;      // a new binding, shadowing the old
}
```

— an infinite loop that compiles. Redox had no assignment statement at all: at statement
level `x = v` is a **binding**, and nothing else accepted a `=`.

Doc 01 §4.3 answers it in two lines — `mut counter = 0` then `counter += 1` — and doc 18
§2.5's `increment self = self.value += 1` uses the same form on a field. So the surface was
specified; no plan had claimed it. Doc 04 row 16 was module-scope bindings, row 20 is
control flow, and compound assignment fell between them. It is implemented here, because a
`while` whose body cannot change its condition is not a loop, and shipping row 20 without
it would have shipped a construct that cannot be used correctly.

This is the fourth time a row's scope was smaller than the thing it claimed to deliver —
after plan 14's doc 03 §13.5, plan 16's `const fn`, and plan 18's `use T::*`.

### 10.2 Plain reassignment has no spelling, in any specification

The five compound operators are what docs 01, 03 and 18 define, and **plain reassignment
appears in none of them**. `counter = 2` is a binding wherever it is written.

This is not an implementation gap that a later plan closes by trying harder. Telling a
rebinding from a binding means knowing whether the name is already bound, which is scope
tracking — and AGENTS.md's rule is that every check rustc can make, rustc makes. Redox has
no scope table and must not grow one for this.

So the consequence is disclosed rather than designed around: **a loop can advance a numeric
accumulator and cannot rebind anything else.** `examples/control.sdx` is written within that
limit — every mutating loop in it uses `+=` — and `a_plain_equals_is_still_a_binding`
asserts the boundary so the limit is a tested fact and not an accident waiting to be
"fixed".

If the limit ever has to go, the spelling is the decision, not the implementation: a
distinct token (`:=`) costs one lexer arm and no analysis, which is the shape of answer this
project can afford. That belongs in a surface-design doc, not in a plan.

### 10.3 The parenthesisation is the only thing here a golden cannot catch

Criterion 3 is the plan's reason for existing, and §9's decision 2 called it correctly.
`until n > 5 && n % 2 == 0` lowers to:

```rust
while !(n > 5 && n % 2 == 0) { n += 1 }      // settles at n = 6
while !(n > 5) && n % 2 == 0 { n += 1 }      // stops at n = 1
```

Both are legal Rust, both read correctly at a glance, and they agree on the first iteration.
`examples/control.sdx` prints `6`, so the wrong lowering fails the run corpus with a
one-character diff — doc 02 §5.2's worst failure mode, caught by the only corpus that can
catch it.

`until_parenthesises_a_single_term_as_well` pins the no-special-case decision too:
`until done` is `while !(done)`, not `while !done`. The tighter form is correct there and
would be a special case, so there is none.

### 10.4 What criterion 9 was right to ask for

§1.6 claimed `return` inside a loop *"for free"*, and it was free — a loop body is a
statement list, and `StmtKind::Return` predates this plan. Criterion 9 asserted it anyway,
which is the practice plan 14 established after §1.1's identical free-of-charge claim turned
out to be false. Free claims are cheap to assert and the assertion is what makes them facts.

### 10.5 The architecture tests caught a fifth thing

`emit/do_block.rs` declared `emit_do`, and doc 06 §2.3 requires the path to name the
construct — `emit_do.rs` by the stem, or `emit/do.rs` by parent-plus-stem, and the second is
unavailable because `do` is a keyword and the `mod` declaration would not parse. Renamed to
`emit/emit_do.rs`, the same resolution plan 16 reached for `emit_const.rs`.

Fifth time these tests found something that reading the file did not. Both are worth the
twenty seconds they cost.

### 10.6 The nullary call, which is a surface rule and not a defect

`print settle` emitted `println!("{}", settle)` — the function item, which rustc rejects
with *"use parentheses to call this function"*. Doc 18 §7 replaced doc 03 §5.2's `!`
zero-argument suffix with `f ()`, so the call is spelled `settle ()`: juxtaposition needs
something to juxtapose, and a lone `()` is the empty argument list. The example was wrong,
not the emitter — and rustc said so, which is the division of labour working.
