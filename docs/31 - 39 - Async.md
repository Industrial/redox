# Plan 39 — Async

Plan ID: `async`
Depends on: 26 (`generics-and-where`), doc 30 (`<-` is await)
Milestone: 7 — *tooling*, but the language half of it belongs here
Status: `planned`

---

## 1. What this plan is

Two markers and one example.

```text
fetch :: async u64 -> Result<User, Error>      async fn fetch(id: u64)
fetch id =                                 →       -> Result<User, Error> {
  user <- http::get (url id)                       let user = http::get(url(id)).await;
  Ok user                                          Ok(user)
                                               }

work = async                                   let work = async {
  <- flush ()                              →       flush().await;
                                               };
```

Doc 04's row 39 reads *"`async` functions and blocks, postfix `.await`, no executor"*.
Postfix `.await` is **gone** — doc 30 made await `x <- f`, already shipped — so what is left
is the marker on a signature, the `async` block, and the thing doc 30 §8.4 recorded as a
debt against this row: an example that **runs**.

### 1.1 The debt, stated as the reason this row is next

Doc 30 §8.4: *"`<-` is proved by eight emit tests and not by a program that runs — a weaker
proof than every other construct in this project has."* Every `<-` needs an `async` function
around it, and `async` in a signature is not implemented: `run :: async ()` emits
`fn run<ASYNC>() -> ASYNC ()`, the marker read as a type variable because doc 18 §5.3
uppercases a lowercase identifier into one.

So doc 30's conversion is under-proven until this row lands, which is why doc 30 §6 named it
step 5 and why it is the next row rather than row 34.

### 1.2 The marker is plan 16's `const fn`, one keyword over

Plan 16 put `const` between `::` and the spine and consumed it there, because *"a type in
Redox is verbatim text (doc 03 §14.1), and leaving `const` in one would make the first
parameter's type `const u32`, which is not a Rust type"*.

`async` is the identical problem with the identical answer: `Signature.is_async` beside
`is_const`, `eat_keyword(Keyword::Async)` beside the one that reads `const`, and
`emit_fn`/`member` prefixing `async ` where they prefix `const `. The keyword already exists
in the lexer.

They are **mutually exclusive in Rust** — there is no `const async fn` — and that is rustc's
to say, not Redox's. Writing both emits both and rustc reports it, which is AGENTS.md's rule.

### 1.3 No executor, and that is the whole of doc 03 §14.3

*"Redox provides no executor and no runtime (doc 01 §24), so `async` blocks and `async fn`
pass through and the runtime is the user's own dependency. A concurrent-await construct is
declined for the dependency reason of §11.3: every joining form in Rust comes from a runtime
crate."*

Which raises the question this plan has to answer and no previous row faced: **how does the
examples corpus run an async program with no runtime?** The corpus demands zero rustc output
*and* a successful run, and there is no `block_on` in `std`.

### 1.4 The answer: the example brings its own, in nine lines

Verified before this plan was written, by compiling it:

```text
block_on :: Future f => f -> f::Output
block_on fut =
  mut boxed = Box::pin fut
  waker = Waker::noop ()
  mut cx = Context::from_waker waker
  loop
    if Poll::Ready v = boxed.as_mut ().poll (&mut cx)
      break v
```

```rust
fn block_on<F>(fut: F) -> F::Output
where
    F: Future,
{
    let mut boxed = Box::pin(fut);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    loop {
        if let Poll::Ready(v) = boxed.as_mut().poll(&mut cx) {
            break v;
        }
    }
}
```

`check` exits 0 on that today, with one dead-code warning for the unused function and nothing
else. It needs **no macro**: `Box::pin` is a function where `pin!` is a macro, and
`Waker::noop` has been stable since rustc 1.85.

That it is writable at all is the proof doc 03 §14.3 actually wants. The claim is not *"async
is hard"* — it is *"the runtime is yours"*, and an example that writes a runtime in nine
lines of ordinary Redox demonstrates the claim rather than asserting it.

It also exercises six constructs from six different rows at once — a generic with a `=>`
bound (26), an associated type as verbatim text (03), `mut` bindings (20), `loop` with
`break v` (20), a path pattern in a condition (33 and doc 30), and juxtaposed calls (13).
A plan's example is usually its own row's; this one is a load test for the language.

### 1.5 `async` blocks, and where they are not

`work = async` with an indented block is `async { … }`, which is an **expression** — so it
reaches `prefix`, exactly as plan 20's `do` does, and emits the same block with `async `
in front. Plan 28's `try` block established the shape one row earlier; this is that shape
with a different word.

`async move` is **not** in this plan. Doc 03 §14.3 does not mention it, a move closure's
spelling is doc 18 §4's `\\` form with no `move` marker either, and adding one here would be
inventing surface rather than implementing it. A future that must own its captures is
written with an `async fn` taking them by value.

---

## 2. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes. The marker is a marker and the block is a block |
| Nothing outside core/alloc/std? | Yes — and §1.4 is the test of it: the example's executor uses `core::future`, `core::task` and `Box::pin` and nothing else |
| Preserves Rust semantics? | Trivially: both forms *are* the Rust forms |
| Buys reachability? | Yes. Without the marker, **no `<-` in the language can be written**, so doc 30's whole conversion is unreachable |

---

## 3. Concrete proof this plan targets

`examples/async.sdx`, which must **run** and print a sequence only a correct lowering
produces:

- a nine-line `block_on`, because a corpus with no runtime has to bring one, and because its
  compiling is §1.3's claim made good;
- an `async fn` whose body uses `<-` **twice**, so the sequence of effects is visible and a
  lowering that dropped one prints the wrong value;
- an `async fn` that awaits **another** `async fn`, which is where a missing `async` on the
  inner one would be a rustc error against a line the author wrote;
- an `async` **block** bound to a name and awaited separately, so the expression form is
  exercised apart from the marker;
- an effect-only `<- f ()` inside an async body, since doc 30's binderless form has never
  been executed;
- a value that is **computed** across two awaits and printed, so the program's output
  depends on both having happened in order.

The last one is what the run catches and no golden can: a lowering that emitted the awaits
in the wrong order, or dropped one, compiles and prints a different number.

---

## 4. Leaves

### Leaf 1 — `leaf-async-marker`

`Signature.is_async`, read where `is_const` is read and emitted where it is emitted.

### Leaf 2 — `leaf-async-block`

`Expr::AsyncBlock { body }`, reached from `prefix` as plan 20's `do` and plan 28's `try`
are, emitting `async { … }`.

### Leaf 3 — `leaf-proof-and-gate`

`examples/async.sdx` with §1.4's executor, emit tests for both forms, a golden, a run
expectation, and the gate.

---

## 5. Acceptance criteria

1. `f :: async T -> U` emits `async fn f(…) -> U`, and the marker does not leak into the
   first parameter's type — plan 16's defect, asserted here because the mechanism is shared.
2. `async` composes with everything a signature already carries: a generic list, a `=>`
   bound, `export`, and a `where` ladder.
3. `x = async` with an indented block emits `let x = async { … };`.
4. An `async` block's statements may include `<-`, and an `async fn`'s may too.
5. A `<-` in a **non**-`async` function is a rustc error reported against the `.sdx` — doc
   03 §14.3's division of labour, asserted through the UI or by inspection rather than
   worked around.
6. `const` and `async` on one signature emit both and let rustc object — §1.2.
7. `examples/async.sdx` compiles with **zero** rustc output, runs, and matches its
   `.stdout`; a golden covers the text.
8. The example's `block_on` uses no macro and no dependency, so the corpus gains no
   `Cargo.toml` entry — which is doc 03 §2's second admission test made visible.
9. Any new diagnostic has an `RDX` code reached by the UI corpus.

---

## 6. Out of scope, and what it discloses

- **An executor.** Doc 03 §14.3 and doc 01 §24: the runtime is the user's dependency. The
  example writes one to prove it can be written, not to ship one.
- **A concurrent-await construct** — `join`, `select`, `race`. Declined by doc 03 §14.3 for
  §11.3's reason: every joining form in Rust comes from a runtime crate, and Redox may not
  inject a dependency the author never declared.
- **`async move`**, per §1.5.
- **Async traits**, `async` closures, and `impl Future` return types. None is in doc 03
  §14.3's two sentences, and `async` closures stabilised only recently; a form with no
  specification has no agreed meaning here.
- **Checking that `<-` appears only inside an `async`.** rustc does it, so Redox does not —
  AGENTS.md. Criterion 5 asserts the error lands on the right line, which is plan 07's job
  and the only part Redox owns.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald was synchronous throughout; it had no futures, no polling and no runtime
to decline providing.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | `async` is a marker between `::` and the spine | A separate keyword before the signature line, or `async` inside the type | Plan 16 settled this for `const` and gave the reason: a type is verbatim text, so a marker left inside one produces `async u64`, which is not a Rust type |
| 2 | The example brings its own `block_on` | Add a runtime dependency, or declare the row unprovable by running | Doc 03 §14.3's claim *is* that the runtime is the user's. Writing one in nine lines demonstrates the claim; adding `tokio` would contradict §2's second admission test |
| 3 | `async` and `const` may both be written | Reject the pair in Redox | rustc rejects it, and AGENTS.md says every check rustc can make, rustc makes |
| 4 | `async` blocks reach `prefix` like `do` and `try` | A statement form | §7.2's `do` and §12.4's `try` are both expressions and this is the third of the same shape. A statement form would need a second node for no gain |
| 5 | No `async move` | Add it alongside | Doc 03 §14.3 does not have it, and doc 18 §4's lambda has no `move` marker either. Inventing one here is surface design, not implementation |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
