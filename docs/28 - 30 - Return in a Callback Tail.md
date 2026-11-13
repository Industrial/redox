# Plan 30 — `return` in a Callback Tail

Plan ID: `with-and-use`
Depends on: 20 (`control-flow`), 29 (`comprehensions-and-collections`)
Milestone: 5 — *the language, as specified*
Status: `planned`

---

## 1. What this plan is

One diagnostic: **`RDX0044`**, for a `return` inside a draw arrow's tail.

```text
run :: Result<(), Error>                 error[RDX0044]: `return` here returns
run =                                        from the generated closure, not
  conn <- pool.with_connection               from `run`
  if bad conn
    return Err Error::Bad             ←  --> run.sdx:5:5
  Ok ()
```

Doc 03 §7.6 calls it a *"disclosed trap, rejected rather than documented away"*: a `return`
in a `use` tail *"would return from the generated **closure**, not from the enclosing
function — a silent change of meaning."*

The flattening itself shipped with plan 13, and doc 18 §3.1 then deleted `use`, `is`,
`defer` and the comprehension's `for` in favour of `<-` — *"the flattening was already the
hard part, and it was already written."* What was never written is the guard.

### 1.1 Why this is the whole row

Doc 04's row 30 names two things. `with` blocks are **deleted** by doc 18 §7 — *"an
ordinary binding — no construct needed"* — and the flattening is done. So the row is one
diagnostic, and after it the row is closed.

### 1.2 The trap is real, and silent in the worst way

```text
conn <- pool.with_connection      pool.with_connection(|conn| {
return Err Error::Bad                 return Err(Error::Bad);
Ok ()                                 Ok(())
                                  })
```

That is **valid Rust**. The `return` leaves the closure, `with_connection` receives an
`Err`, and whatever it does with one happens — which is almost certainly not what the author
meant by `return`. Doc 02 §5.2's failure mode: output that reads as Rust and means something
else.

And it is worse than the usual case, because the author's mental model is *"`return` returns
from the function"*, which is true everywhere else in the language. Nothing in the source
hints that this block became a closure.

### 1.3 `?` is unaffected, and that is the asymmetry to get right

§7.6 is explicit: *"`?` is unaffected, because it propagates out of the closure, which is
the intended behaviour and is why the example above type-checks."*

So the check must reject `return` and leave `?` alone. They look alike — both leave the
block early — and they differ because `?` propagates through the closure's `Result` to the
caller, which is exactly the flattening's purpose, while `return` stops at the closure
boundary.

A check that rejected both would break the one idiom `use` exists for.

### 1.4 Where the walk stops, which is the only subtle part

A `return` *lexically inside* the generated closure returns from it. So the walk descends
into every block the tail contains — `if`, `else`, `while`, `for`, `loop`, `do`, a guard's
else — because all of those become statements inside the same closure.

It must **not** descend into:

- **a `try` block** (plan 28), because that is a closure too, and a `return` inside one
  returning from it is the construct working;
- **a nested draw arrow's tail**, because that is a new closure with its own guard: the
  recursion reports it against the inner `<-`, which is the one whose closure the `return`
  would leave.

So the rule is *"a `return` in this closure's own statements"*, and both exclusions are the
same exclusion: another closure began.

**A lambda was the third item on this list and is not reachable** — §10.2 records it. Doc 18
§4's lambda body is an *expression* and `return` is a *statement*, so `\x -> return x` is
`expected an expression` and no lambda can contain a `return` to descend past. The exclusion
is therefore structural rather than written: the statement kinds that own blocks are exactly
the ones that do not open a closure, so the walk needs no deny-list at all.

### 1.5 A test currently forbids the code this plan needs

`code.rs`'s `no_code_collides_with_the_published_reservation` asserts that **no** `Code` has
a number in `40..=99`:

```rust
const RESERVED: std::ops::RangeInclusive<u16> = 40..=99;
for code in Code::all() {
  assert!(!RESERVED.contains(&code.number()), …);
}
```

Its doc comment explains the intent correctly — *"doc 03 published `RDX0042`-`RDX0063` for
surface features belonging to plans 19-28. A code in that band here would ship two meanings
for one number"* — and then enforces the opposite of it. The band is reserved **for** those
plans; using `RDX0044` for the thing doc 03 §17 assigned `RDX0044` to is the reservation
working, not a collision.

As written the test would have blocked every plan 19–28 surface code. Plan 30 is the first
to need one, which is why it has gone unnoticed. The test becomes what its comment says: a
code in the band must be one §17 published, at the number §17 gave it.

---

## 2. Admission tests (doc 03 §2)

Not applicable — this plan adds no construct. It adds a **refusal**, and the admission
tests are for admissions. The relevant rule is AGENTS.md's: *every check rustc can make,
rustc makes.* This is not one rustc can make. The generated Rust is valid; what is wrong is
that it does not mean what the source said, and only Voltaic knows what the source said.

That is the narrow class of check Voltaic is allowed to grow, and §7.6 is where this one was
specified.

---

## 3. Concrete proof this plan targets

A **UI case**, `tests/ui/return_in_callback_tail.sdx`, whose `.stderr` shows the diagnostic
pointing at the `return` — because this plan's whole deliverable is a message, and the UI
corpus is the only one that asserts rendered text.

Plus emit tests for the shapes the walk must get right:

- a `return` directly in a tail — rejected;
- a `return` nested in each block a statement can own — rejected, because it is still in the
  same closure;
- a `return` inside a `try` block in a tail — accepted, per §1.4;
- a `?` in a tail — accepted, per §1.3, and asserted as the thing that must keep working;
- a `return` **before** the arrow, and in a function with no arrow at all — accepted, which
  is every function written before this plan.

The example corpus cannot carry this: a corpus that demands zero rustc output and a
successful run cannot hold a program that fails to compile. Plan 16 §10.2 hit the same wall
with a lint and recorded it; the UI corpus is the answer both times.

---

## 4. Leaves

### Leaf 1 — `leaf-rdx0044`

`Code::ReturnInCallbackTail` at **44**, with `code.rs`'s reservation test corrected per §1.5.

### Leaf 2 — `leaf-tail-walk`

The walk: after a `Draw` statement, scan the remaining statements of that block for a
`return`, descending into the blocks a statement owns and stopping where another closure
begins. Run once per function body as a whole-program pass, because the walk recurses and a
per-block check would report a nested tail's `return` twice.

### Leaf 3 — `leaf-proof-and-gate`

The UI case, emit tests for the four shapes, and the gate.

---

## 5. Acceptance criteria

1. A `return` directly in a draw arrow's tail is `RDX0044`, reported against the `return`.
2. A `return` nested in an `if`, `while`, `for`, `loop` or `do` inside the tail is also
   `RDX0044` — the closure is the same one.
3. A `return` cannot appear inside a lambda at all — asserted as
   `expected an expression`, because a test for an unreachable exclusion would pass
   whatever the code did. *Corrected from "is accepted" — §10.2.*
4. A `return` inside a `try` block in the tail is **accepted**.
5. A `?` in a tail is accepted and still propagates — asserted, because §7.6's own example
   depends on it and a check that rejected both would break the idiom `use` exists for.
6. A nested draw arrow's tail gets its own check, reported against the inner arrow's closure.
7. A `return` **before** the draw arrow in the same block is accepted: it is not in the
   closure.
8. `RDX0044` is reached by the UI corpus, and its number is 44 — doc 03 §17's own
   assignment.
9. Every existing test still passes, in particular plan 13's draw-arrow goldens: this plan
   adds a refusal and must change no lowering.

---

## 6. Out of scope, and what it discloses

- **`with` blocks** are deleted by doc 18 §7, not deferred.
- **`break` and `continue` in a tail** are *not* this diagnostic, and they are not legal
  either: there is no loop in the generated closure, so rustc rejects them — which is a
  check rustc can make, so rustc makes it.
- **A `return` inside a nested function or `class` member in a tail** is not reachable: a
  declaration cannot appear in a statement block.
- **Suggesting a fix** is declined for now. The honest rewrite depends on what the callback
  does with its value, which Voltaic cannot know. The message says what the `return` would do
  instead, which is the information the author lacks.
- **A lint for a `return` that is the tail's last statement** — where the closure's value
  and the function's would coincide for a `Result`-returning callback — is deliberately not
  carved out. The meaning still differs for any other callback, and an exception the author
  has to reason about is worse than a rule they can.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald had no callback-flattening construct, so it never had the trap.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Reject `return`, accept `?` | Reject both, or accept both | §7.6 and §1.3. `?` propagating out of the closure is the whole point of the construct; `return` stopping at it is the trap |
| 2 | The walk stops at any new closure | Reject every `return` anywhere in the tail | A `return` in a `try` block returns from that block, which is what the author wrote. The lambda case turned out unreachable (§10.2), so the rule is carried by *which statement kinds own blocks* rather than by a deny-list |
| 3 | `RDX0044`, the published number | The next free code in the syntactic range | Doc 03 §17 assigned 44 to this exact diagnostic. The reserved band exists so a published number stays published — and §1.5's test had to be corrected to allow it |
| 4 | Proved by the UI corpus, not the examples | An example | A corpus requiring zero rustc output and a successful run cannot hold a program that must fail. Plan 16 §10.2 recorded the same wall |
| 5 | No suggested fix | A `did you mean` | The rewrite depends on the callback's contract, which Voltaic cannot see. A wrong suggestion is worse than none |

---

## 10. Result

Shipped. 636 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit` are green. Every
criterion is met, one of them after being corrected. **Row 30 is closed.**

### 10.1 A test enforced a reservation by forbidding its use

`code.rs` held `no_code_collides_with_the_published_reservation`, asserting that no `Code`
has a number in `40..=99`. Its own doc comment states the intent correctly — doc 03 §17
published that band for plans 19–28, and a code there *"would ship two meanings for one
number"* — and the assertion says the opposite: not *"must be the published one"* but
*"must not exist"*.

As written it would have failed on **every** plan 19–28 surface code. Plan 30 is the first
plan to need one, twenty-nine plans after the band was reserved, which is why it went
unnoticed for so long: a reservation nothing draws on is indistinguishable from a
prohibition.

It now asserts what its comment says, against a transcribed list of §17's assignments: a
code in the band must be the diagnostic §17 gave that number to. `RDX0044` is the first
entry, and the list is the place the next one goes.

The range check needed widening too — it allowed `1..=39` and `100..=199` and nothing
between, so the published band was unreachable twice over.

### 10.2 One of the three exclusions is unreachable, and the test says so

§1.4 listed three places the walk must not descend: a lambda, a `try` block, and a nested
arrow's tail. **A lambda cannot contain a `return` at all.** Doc 18 §4's lambda body is an
*expression* and `return` is a *statement*, so `\x -> return x` is
`RDX0102: expected an expression` and there is nothing to descend past.

So the exclusion is structural rather than written, and the shape of the code says why: the
statement kinds that own blocks — `if`, `while`, `for`, `loop`, `guard` — are exactly the
ones that do not open a closure. `blocks_in_the_same_closure` therefore needs no deny-list,
which is a stronger property than a correct deny-list because it cannot fall out of date
when a construct is added.

Criterion 3 was corrected from *"a `return` inside a lambda is accepted"* to the fact: it
does not parse. A test for an unreachable branch passes whatever the branch does, and would
have been worse than no test — it would have read as coverage.

### 10.3 Why the check is a whole-program pass

`return_in_tail` recurses into the blocks a statement owns, and every block also gets its
own `statements()` call during parsing. Running the check there would report a nested tail's
`return` twice — once from the inner block's own call, once from the outer walk descending
into it. `a_nested_return_is_reported_once` asserts it is reported once.

The pass also runs **only when parsing otherwise succeeded**, so a file with a syntax error
reports the syntax error rather than a second diagnostic about a tail the parser recovered
into. That is a judgement about message quality, not correctness, and it is written down
here because the alternative is defensible and someone will wonder.

### 10.4 The asymmetry held, and the test for it is the important one

`?` in a tail is accepted and still propagates. §7.6's own example depends on it, and a
check that rejected both `return` and `?` would have broken the single idiom the construct
exists for while passing a test suite that only checked `return`.

That is why `a_question_mark_in_a_tail_is_accepted` sits next to the rejection tests rather
than in a corner: the two assertions are one assertion about where the closure boundary
matters.

### 10.5 The architecture tests, a ninth time

`check_callback_tails.rs` declared three constructs and `return_in_tail.rs` three. Split
into six files, all siblings under `parse/parser/` because doc 06 §2.4 caps depth at three
levels below `src/` — the same resolution plans 22 and 23 reached, and by now the expected
one rather than a surprise.

Clippy then caught a `collapsible_if` in the extracted `scan_run`, which the let-chain form
fixes. Worth noting only because AGENTS.md records a false "clippy clean" report from
grepping output for `^error`: the exit code was 101 and the grep for the message found
nothing, exactly as that note predicts.
