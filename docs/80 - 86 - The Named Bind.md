# Plan 86 — The Named Bind

Plan ID: `named-bind`
Depends on: 55 (`expression-orientation`), 60 (`bind-arrow`)
Milestone: 11 — *expressive delta*
Status: `done` — leaves 1–5 shipped 2026-11-11; §10 is the record

---

## 1. What this plan is

`do .method` becomes a block modifier, and `<-` inside it binds through that method. The decline
on general monadic comprehension is lifted, because the condition it carries is met.

```
users = do .and_then                 pairs = do .flat_map
  u <- find id                         x <- 1..3
  a <- u.address                       y <- 1..3
  Ok a                                 iter::once (x, y)
```

```rust
let users = find(id).and_then(|u| u.address.and_then(|a| Ok(a)));
let pairs = (1..3).flat_map(move |x| (1..3).flat_map(move |y| std::iter::once((x, y))));
```

### 1.1 The decline, and the exact condition it carries

`docs/DECLINED.md` declines this construct under *"needs type information"*, and it is one of
only two entries in the whole document with a venue rather than a permanent refusal:

| Construct | From | Reason |
|---|---|---|
| General monadic comprehension | Scala | must choose `flat_map` versus `and_then`. **Venue**: revisit only if a type-free discrimination is found |

Doc 03 §22 states the same venue: *"revisit only if a type-free discrimination is found."*

This row is that venue, and the discrimination is not new machinery. **It is the block modifier
plan 60 already built.**

### 1.2 Plan 60 already made the modifier a discrimination

Plan 60's thesis is in its own first line: *"`<-` becomes monadic bind, read by the enclosing
block modifier."* Its table is three blocks and three meanings for one token:

```
work = async                    r = try
  n <- fetch ()     -- await      a <- parse x    -- the `?`
  n + 1                           b <- parse y
                                  Ok (a + b)
```

So Redox already has monadic bind, already resolves it against the enclosing block, and already
resolves it **from the source text and nothing else**. What it does not have is a modifier that
names an arbitrary method — the set is closed at `async`, `try` and `unsafe`, which plan 55 leaf 7
made *"one arm reading an adjective and then the block above"*.

Opening that set is the whole of this row. `do .and_then` reads an adjective that happens to be a
method name.

### 1.3 Why this satisfies the admission test, clause by clause

Doc 03 §2's first clause is the one the decline rests on, and it is worth quoting exactly, because
its second sentence is the permission:

> **Type-free.** Its lowering is determined by the source text alone. Where two Rust shapes are
> possible, the user's own syntax selects between them — **never an inference**.

`flat_map` versus `and_then` is precisely *"two Rust shapes are possible"*, and `do .flat_map`
versus `do .and_then` is precisely *"the user's own syntax selects between them"*. The decline's
reason — *"must choose"* — described a construct that chose for the author. This one does not
choose at all; it transcribes.

The other three clauses:

- **No implied dependency.** The lowering names only the method the author wrote and the
  expressions the author wrote. It introduces no path, not even `Ok` — see §1.4.
- **Semantics-preserving.** A nest of method calls with closures, which is what the author would
  write by hand. No clone, no reborrow, no reordering: row *n* is evaluated inside row *n−1*'s
  closure, which is the order the source reads in.
- **Reachability-positive.** It adds expressiveness. It also closes something: a `?` chain works
  only where `Try` does, and `and_then` over a user type has no block form today.

### 1.4 Nothing is fabricated, including the final row

Haskell's `do` ends in an expression *of the monad*, and a naive port inserts `pure`. This row
does not, and must not: choosing between `Ok`, `Some` and `iter::once` is an inference, which
clause 1 forbids, and `iter::once` is also a path the author did not write, which clause 2
forbids.

**So the last row is emitted as written.** `Ok a`, `Some a`, `iter::once (x, y)` — the author's
expression, in the innermost position. That is one more line of source than Haskell needs and it
is the line that keeps the construct admissible.

It is also consistent with how the existing modifiers behave: plan 60's `try` block ends in
`Ok (a + b)`, written by the author, not synthesised.

### 1.5 An intermediate non-binding row sequences, and that needs `move`

A row that is neither the last nor a `<-` binding is Haskell's `>>`: it lowers to the same method
with a discarded binder, `e.METHOD(|_| rest)`.

The inner closures need `move`, and doc 03 §10.1's comprehension lowering is the precedent — it
already emits `move` on exactly these closures:

```rust
(1..3).flat_map(|x| (1..3).filter(move |y| x != *y).map(move |y| (x, y)))
```

This is **the opposite** of plan 56 §11.2's decision, which dropped `move` from partial
application, and the difference is real rather than an inconsistency. A partial application
captures an argument that may be borrowed, and `move` there turns rustc's borrow error into a
worse move error about the same mistake. A bind's inner closure captures an *outer binder* and is
returned past its frame, so it has to own what it captures or it does not compile at all. §9
decision 3 records this and criterion 5 asserts both shapes.

### 1.6 It overlaps doc 03 §10.1's comprehension, and the overlap is disclosed

Doc 03 §10.1's bracket comprehension already lowers to nested `flat_map`:

```
pairs = [ (x, y) for x in 1..3 for y in 1..3 when x != y ]
```

So `do .flat_map` is a second way to write that particular shape. Both exist in Haskell for the
same reason — a comprehension is an expression over a source, a `do` block is a sequence of rows
— and §9 decision 5 keeps both. But `docs/DECLINED.md` declines cascades and scope functions for
*"duplicates something Redox already has"*, so the overlap must be argued rather than ignored,
and §6 does.

---

## 2. Preconditions to verify

**Nothing in this section is measured yet.** The row is authored, not executed.

```bash
cargo run -p redox-cli -- build /tmp/probe.sdx && cat /tmp/probe.xrs
cargo run -p redox-cli -- check /tmp/probe.sdx
```

1. **That `do` is reserved and unspent.** `docs/GRAMMAR.md` §1.4 states it twice — *"`do` stays
   reserved for plan 60's do-notation (`RDX0114`)"* and, in the MoonScript table, the same
   sentence. This row spends that reservation; the measurement is that `RDX0114` is what `do`
   produces today and that no construct has quietly taken it.
2. **How the modifier arm reads its adjective.** Plan 55 leaf 7 made `async`, `unsafe` and `try`
   *"one arm reading an adjective and then the block above"*. Whether that arm can read a
   **method reference** rather than a keyword is the size of leaf 1.
3. **What `<-` lowers through today.** Plan 60's two existing meanings are an operator (`?`) and a
   postfix (`.await`), neither of which is a method call with a closure argument. The nesting
   lowering is therefore new code, not a parameterisation of existing code, and leaf 2 is sized
   accordingly.
4. **That `(.and_then)` is already a method reference.** Plan 57 made a method a function and
   `docs/GRAMMAR.md` records that *"`(.age)` is the selector because a `)` follows the name"*. The
   modifier position needs `.and_then` as a **name**, not as a call, and whether the existing
   selector form reaches it decides whether this row spends new syntax.
5. **Interaction with `try` and `async` in the same block.** A `do .and_then` block containing a
   `<-` that the author meant as `await` has no spelling, and precondition 5 is to confirm that
   nesting a block solves it rather than needing a second modifier on one block.

---

## 3. Concrete proof this plan targets

**`users` from §1 compiles, runs, and prints the address** — an `and_then` chain written as a
block, which is the construct `docs/DECLINED.md` says Redox does not have.

Six cases, each failing differently:

1. **`and_then` over `Option`.** Two binds, a written final row.
2. **`flat_map` over a range.** §1's `pairs`, which is the case that needs `move` on the inner
   closure and fails to compile without it.
3. **A user type.** A `data` declaration with an inherent `and_then`, bound in a `do` block —
   the case that proves the construct is not a `Try` special case in disguise.
4. **An intermediate non-binding row**, per §1.5.
5. **A single-row block.** `do .and_then` with one `<-` and one final row — the degenerate case,
   which must emit one method call and no nesting.
6. **A block whose method does not exist on the value.** `do .and_then` over a `Vec` — asserted as
   a **rustc** error against the generated closure, in `tests/ui/`, because §1.3's claim is that
   rustc judges the choice and an untested claim about where an error appears is the kind this
   repository has been bitten by.

And two that must **not** change: every existing `try` and `async` block emits byte-identical
Rust, and `examples/comprehension.sdx` is untouched.

---

## 4. Leaves

### Leaf 1 — `leaf-method-modifier`

The block modifier arm accepts `do .name` beside the three adjectives. Sized by preconditions 2
and 4.

### Leaf 2 — `leaf-bind-nesting`

The lowering: rows fold rightwards into nested `method(|binder| …)` calls, the last row innermost
and verbatim. `move` on inner closures per §1.5.

### Leaf 3 — `leaf-sequencing-rows`

The non-binding intermediate row, `e.METHOD(|_| rest)`. Separate from leaf 2 because it is the
one row shape with no binder and therefore no `move` obligation of its own.

### Leaf 4 — `leaf-diagnostics`

A `do` block with no `<-` and no rows; a `<-` outside any modifier block; a modifier naming
something that is not a method reference. Three named diagnostics with `tests/ui/` fixtures. Case
6 of §3 is deliberately **not** here — it is rustc's.

### Leaf 5 — `leaf-corpus-and-docs`

`examples/bind.sdx` exercising each §3 case. Then the document work, which is the larger half of
this leaf:

- `docs/DECLINED.md`'s *"General monadic comprehension"* row **moves out of the declines** and
  into a record of a lifted decline, naming this row and the condition that lifted it. Its
  *Reopenable* table loses one of its two entries.
- Doc 03 §22's identical row and its venue sentence are amended the same way.
- `crates/redox-cli/tests/declined.rs` asserts every §22 construct appears in `DECLINED.md`, so
  the two documents must move in one commit or the gate breaks. That is the test working.
- `docs/GRAMMAR.md` §1.4 regrades the block-modifier row and `docs/STATUS.md` is regenerated.

---

## 5. Acceptance criteria

1. Each §3 case transpiles, compiles with no rustc output, runs, and its stdout is asserted.
2. The user-type case (3) works with a method defined in the file, proving the construct is not
   tied to `std`.
3. Nothing is fabricated: a golden asserts the innermost expression is the author's final row
   verbatim, with no inserted `Ok`, `Some` or `iter::once`.
4. Case 6 is a **rustc** error, captured in `tests/ui/`, and is *not* a Redox diagnostic.
5. Inner closures carry `move` and plan 56's partial applications still do not — both asserted in
   goldens, so §1.5's distinction is visible rather than argued.
6. Every existing `try` and `async` block emits byte-identical Rust; `tests/golden/` re-runs with
   **no re-blessing** of those files.
7. `examples/comprehension.sdx` is unchanged, per §1.6.
8. `docs/DECLINED.md`, doc 03 §22 and `declined.rs` move in one commit and the gate is green
   after it.
9. All examples green; `construct_coverage` green with no new `ALLOW` entries.
10. `RDX0114` no longer fires for `do` in modifier position, and still fires everywhere else the
    word appears, asserted both ways.

---

## 6. Out of scope, and what it discloses

**The author now writes the method, and can write the wrong one.** That is the price of clause 1
and it is the right price, but it is a real one: `do .and_then` over an iterator is a rustc error
rather than a Redox diagnostic, and the message names a generated closure. Criterion 4 pins
exactly how bad that message is instead of assuming it is tolerable.

**It overlaps the comprehension (§1.6).** Argued rather than resolved, on three grounds. The
comprehension is an expression over a source and produces a collection; a `do` block is a
sequence of rows and produces whatever its method produces. The comprehension cannot bind a
non-iterator, which is this row's main case. And `docs/DECLINED.md`'s duplicate-declines —
cascades, scope functions — were refused for having *"no Rust lowering of its own"*, which is the
opposite of this row's situation: the lowering is the author's named method.

**No `pure`, ever.** §1.4. Haskell programmers will find the final row verbose, and the
verbosity is load-bearing.

**One block, one bind method.** A block that needs both `await` and `and_then` nests two blocks.
Precondition 5 confirms this works; a combined modifier is not in scope and §9 decision 4 is why.

**`<-` now has three meanings and the block says which.** That was already true at plan 60 — this
row makes the third meaning open-ended rather than adding a fourth position.

**Nothing here is a monad.** There is no `Monad` class, no law, and no check that the named method
behaves like a bind. Redox cannot check it, rustc cannot check it, and this document should not
imply otherwise.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixtures for leaf 4's three diagnostics and for case 6's rustc error; existing
`try`/`async` goldens asserted unchanged per criterion 6; `declined.rs` green after the document
move; `construct_coverage` green.

---

## 8. Emerald reuse

None available, and the relevant prior art is Scala's, which is what `docs/DECLINED.md` cites as
the source of the declined construct.

The transferable discipline is doc 02 §5.2's, and it is the sharpest case for it in the register:
*"a Redox expression that reads like Rust and means something different is the worst possible
failure mode for a transpiler."* A `do` block whose rows nest in the wrong order type-checks
whenever the binders happen to be compatible, and prints the wrong thing. So doc 02 §8.3 is the
only check that catches it — *"compiled by rustc, run, and its stdout asserted"* — which is why
every §3 case is a running program and why case 2's `move` is asserted in a golden rather than
reasoned about.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | The modifier names the method | Infer `flat_map` versus `and_then` | The inference is what `docs/DECLINED.md` declined, and doc 03 §2 clause 1 forbids it in the same words. Naming it is the condition the venue asks for |
| 2 | `do` | A new keyword such as `bind` | `do` is already reserved for exactly this — `docs/GRAMMAR.md` says so twice — and discharging a standing reservation costs nothing where a new keyword costs one |
| 3 | `move` on inner closures, none on partial applications | One rule for both | They capture different things for different reasons. A bind's closure is returned past its frame and must own; a partial application may capture a borrow, where `move` converts rustc's borrow error into a worse move error (plan 56 §11.2). Doc 03 §10.1's comprehension already emits `move` here |
| 4 | One bind method per block, nest for two | A combined modifier | A combined modifier needs an order between the two binds that nothing in the source states, which is an inference. Nesting makes the order the author's |
| 5 | Keep the comprehension (§1.6) | Retire it in favour of `do .flat_map` | It is specified in doc 03 §10.1, shipped, has an example, and reads better for its own shape. Retiring a working construct to avoid an overlap is a worse trade than the overlap |
| 6 | The final row is verbatim | Insert `pure` | Choosing what `pure` is requires knowing the type. §1.4 |

---

## 10. Result

Shipped 2026-11-11. `moon run :format :lint :test` green.

### 10.1 What shipped, and the decline it lifted

`do .method` is a block modifier and `<-` inside it binds through the named
method. From `tests/golden/bind.xrs`:

| Source | Emitted |
|---|---|
| `do .and_then` over `Option`, two binds | `Some(s.trim()).and_then(move \|t\| t.split_whitespace().next().and_then(move \|w\| Some(w)))` |
| `do .flat_map` over two ranges | `(1u64..3).flat_map(move \|x\| (1u64..3).flat_map(move \|y\| once((x, y))))` |
| `do .and_then` over a **user type** | `(Tally { value: 5 }).and_then(move \|n\| Tally { value: n * 2 })` |
| an intermediate row with no binder | `Some(1).and_then(move \|_\| …)` |
| a single-row block | the row, with no nesting at all |

**`docs/DECLINED.md` is down one row, and it is the only decline this project
has ever lifted.** The venue asked for *"a type-free discrimination"* and plan
60 had already built one — `<-` read by the enclosing block modifier — so the
work was opening a closed set rather than finding a new mechanism. Doc 03 §22's
row and its venue sentence moved in the same commit, which `declined.rs`
requires.

The user-type case is the one that matters most: `Tally` and its `and_then` are
both declared in `examples/bind.sdx`, so nothing in `std` is involved and the
construct is demonstrably not a `Try` special case in disguise.

### 10.2 It is a node, not a third `BindMode`, and that was the right call

`try` and `async` read `<-` as a **postfix**: `emit_bind` writes
`let a = parse(x)?;` and the rows stay a sequence of statements. A named bind
cannot work that way — each row *contains* the rest of the block — so the fold
is rightwards and the whole block is one expression.

That made `BindMode` the wrong place and `Expr::BindBlock` the right one. The
cost is one more `Expr` variant, which §10.3 is the bill for.

`move` on the inner closures is the opposite of plan 56 §11.2's decision and the
two do not conflict: a partial application may capture a *borrowed argument*,
where `move` turns rustc's borrow error into a worse move error, while a bind's
closure captures an **outer binder** and is returned past its frame, so it has
to own what it captures. Doc 03 §10.1's comprehension lowering already emits
`move` on exactly these closures.

### 10.3 Adding an `Expr` variant deadlocks the port, and the way out is worth
writing down

This is the row's most transferable finding and it cost the most time.

`port.rs` **compiles the generated tree before writing it** and refuses a
generation that does not compile — which is exactly right, and which turns a new
`Expr` variant into a deadlock. The sequence:

1. `redox-ast`'s generated `expr.rs` is written, so `Expr::BindBlock` exists;
2. every crate that matches on `Expr` is now non-exhaustive — `E0004`;
3. so the port cannot write *them*, because the workspace does not compile;
4. and it cannot rebuild the compiler that would write them, for the same
   reason.

The tree sits half-updated and `cargo run -p redox-cli` is unavailable. Four
match sites were involved: `redox-emit`'s `emit_expr`, `redox-fmt`'s `expr_text`
and its `power_of`, and the module declaration for the new emitter file.

**The way out is a hand-patch of the generated `src/`, and then a port run that
overwrites it.** Add the arms by hand, add the `mod` line, copy the new file's
own `.xrs` into place, `cargo build`, then `REDOX_UPDATE=1 … --test port` and
the hand edits are replaced by generated output. `the_port_regenerates_the_committed_rust`
is the proof that nothing hand-written survived.

AGENTS.md's *"Edit the `.sdx`, never the `.rs`"* is not suspended by this — the
`.sdx` is where the change was made, and the `.rs` edit is a bootstrap bridge
that the next port run demolishes. A successor adding a variant should expect to
cross it, and should expect the first `E0004` to name only one site: they arrive
one at a time as each crate reaches the front of the build.

### 10.4 The grammar reuses the reserved-word node, which saved three
highlighters

`do` is **globally reserved** in `grammar.js` — `reserved: { global: … }` — so no
other rule may produce the literal. `bind_block` therefore opens with
`$.reserved_word`, the node the reservation already points at.

That was forced, and it paid twice. The word stays reserved everywhere else,
matching the compiler exactly; and because `do` was already coloured through
`(reserved_word) @keyword`, **no highlighter needed a change** —
`every_grammar_keyword_is_coloured` stayed green where plan 84's one new keyword
had required four files.

### 10.5 Leaf 4 needed no new diagnostic

The plan asked for three named codes. All three shapes were already covered:
`do` without a `.` is `RDX0114` (plan 55 leaf 7's reservation, untouched),
`do .method` with no body is `RDX0105`, and `do .` with no name is `RDX0103`.
`tests/ui/bind_block_shape.sdx` pins the third and says why there is no fourth
code — a construct that reuses its neighbours' diagnostics is one fewer code to
learn, and the help line carries the construct-specific part.

### 10.6 `<-`'s node is still called `Await`, and this row is the second to read
past the name

Plan 60 renamed `emit_await` to `emit_bind` on the ground that *"a function
called `emit_await` that sometimes writes `?` is the stale-name hazard this
project keeps recording"* — and left the **node** called `Expr::Await`. This row
gives the token a third meaning and matches on `Expr::Await` to find a bind,
which is the same hazard one level up.

Not renamed here, deliberately: it touches `redox-ast`, `redox-parser`,
`redox-emit` and `redox-fmt`, and §10.3 is what an `Expr` change costs. Owed to
a row that wants it.

### 10.7 What this row exposed and did not fix

Two **pre-existing grammar gaps**, both found because `examples/bind.sdx` is the
first corpus file to come near them, and both isolated with `tree-sitter parse`
on a four-line file:

- **a parenthesised `(impl Fn(u64) -> T)` in a signature** reports `ERROR` on the
  `)` before the arrow, while the unparenthesised form parses;
- **`export` on a member whose signature contains an `Fn(…)` type** reports
  `ERROR`, while the same member without `export` parses.

The compiler accepts all four shapes. The example uses the two the grammar
agrees with, and says so inline. Neither is plan 86's to fix.

### 10.8 Corrections to this document

- §1 and §4 treat the lowering as a generalisation of plan 60's modifier. It is
  not a generalisation — it is a different lowering that shares a modifier
  position. §10.2.
- §4 leaf 4's three diagnostics are withdrawn. §10.5.
- §2 has no precondition for what adding an `Expr` variant costs, which was the
  largest single piece of work in the row. §10.3 is owed to §2.
