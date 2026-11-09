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

- **a lambda** (doc 18 §4), because a `return` there returns from *that* closure, which is
  what the author wrote and expects;
- **a `try` block** (plan 28), for the same reason — it is a closure too, and a `return`
  inside one returning from it is the construct working;
- **a nested draw arrow's tail**, because that is a new closure with its own guard: the
  recursion reports it against the inner `<-`, which is the one whose closure the `return`
  would leave.

So the rule is *"a `return` in this closure's own statements"*, and the three exclusions are
all the same exclusion: another closure began.

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
that it does not mean what the source said, and only Redox knows what the source said.

That is the narrow class of check Redox is allowed to grow, and §7.6 is where this one was
specified.

---

## 3. Concrete proof this plan targets

A **UI case**, `tests/ui/return_in_callback_tail.sdx`, whose `.stderr` shows the diagnostic
pointing at the `return` — because this plan's whole deliverable is a message, and the UI
corpus is the only one that asserts rendered text.

Plus emit tests for the four shapes the walk must get right:

- a `return` directly in a tail — rejected;
- a `return` nested in an `if` in a tail — rejected, because it is still in the closure;
- a `return` inside a **lambda** in a tail — accepted, per §1.4;
- a `?` in a tail — accepted, per §1.3, and asserted as the thing that must keep working.

The example corpus cannot carry this: a corpus that demands zero rustc output and a
successful run cannot hold a program that fails to compile. Plan 16 §10.2 hit the same wall
with a lint and recorded it; the UI corpus is the answer both times.

---

## 4. Leaves

### Leaf 1 — `leaf-rdx0044`

`Code::ReturnInCallbackTail` at **44**, with `code.rs`'s reservation test corrected per §1.5.

### Leaf 2 — `leaf-tail-walk`

The walk: after a `Draw` statement, scan the remaining statements of that block for a
`return`, descending into nested blocks and stopping at lambdas, `try` blocks and nested
draw arrows.

### Leaf 3 — `leaf-proof-and-gate`

The UI case, emit tests for the four shapes, and the gate.

---

## 5. Acceptance criteria

1. A `return` directly in a draw arrow's tail is `RDX0044`, reported against the `return`.
2. A `return` nested in an `if`, `while`, `for`, `loop` or `do` inside the tail is also
   `RDX0044` — the closure is the same one.
3. A `return` inside a **lambda** in the tail is **accepted**.
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
  does with its value, which Redox cannot know. The message says what the `return` would do
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
| 2 | The walk stops at any new closure | Reject every `return` anywhere in the tail | A `return` in a lambda returns from the lambda, which is what the author wrote. Rejecting it would make lambdas unusable inside a `use` body |
| 3 | `RDX0044`, the published number | The next free code in the syntactic range | Doc 03 §17 assigned 44 to this exact diagnostic. The reserved band exists so a published number stays published — and §1.5's test had to be corrected to allow it |
| 4 | Proved by the UI corpus, not the examples | An example | A corpus requiring zero rustc output and a successful run cannot hold a program that must fail. Plan 16 §10.2 recorded the same wall |
| 5 | No suggested fix | A `did you mean` | The rewrite depends on the callback's contract, which Redox cannot see. A wrong suggestion is worse than none |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
