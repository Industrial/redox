# Plan 28 — `catch`, Optional Chaining and `try` Blocks

Plan ID: `option-result-operators`
Depends on: 22 (`is-and-guard`), 23 (`strings-and-literals`)
Milestone: 5 — *the language, as specified*
Status: `planned`

---

## 1. What this plan is

The three constructs row 28 has left: doc 03 §12.2's **`catch`**, §12.3's **`?.`**, and
§12.4's **`try` blocks**.

```text
n = parse s catch 0              let n = match parse(s) {
                                     Ok(__rdx_v) => __rdx_v,
                                     Err(_) => 0,
                                 };

n = user?.profile?.name      →   let n = (|| Some(
                                     &user.as_ref()?.profile.as_ref()?.name))();

result = try                     let result = (|| {
  a = (parse x)?                     let a = parse(x)?;
  Ok a                               Ok(a)
                                 })();
```

`??` already lowers to a `match` and `?` already passes through; doc 18 §7 deleted `??=`.
Doc 18 §8 keeps all three of these *"unchanged despite having no Haskell analogue, because
deleting them would cost reachability or ergonomics with nothing to show."* So this is doc 03
§12 as written, and all three keywords have had lexer entries since plan 02 with nothing
reading them.

### 1.1 All three lower by handing the work to rustc

That is what makes them admissible, and it is the same move three times:

| Construct | Who does the work |
|---|---|
| `catch` | A `match` — the same shape §12.1's `??` already emits |
| `?.` | **`?` inside a closure returning `Option`.** Doc 02 §6.3's lowering, which *"delegates the entire `and_then`/`map` unification to rustc"* |
| `try` | **`?` inside an immediately-invoked closure.** Rust has wanted `try` blocks for years and has them only unstably; the closure gives Voltaic them on stable today |

Two of the three are the *same* mechanism — a closure whose `?` returns from the closure
rather than the function — which is why they are one plan and not two.

### 1.2 `?.` is the one with a real design claim in it

Doc 03 §12.3 resolves doc 02 §13's question 7 outright: **the chain borrows and never
clones.**

```text
n = user?.profile?.name          (|| Some(&user.as_ref()?.profile.as_ref()?.name))()
```

**§12.3's own lowering writes `.as_ref()` on the final link too, and that does not
compile** — §10.1 records it. `.as_ref()` on a non-`Option` field means `AsRef::as_ref`,
which `String` does not implement for `String`, and choosing between the two meanings would
need a type. The final link therefore takes no method and the whole path takes a `&`.

Borrowing every link is what keeps doc 01 §25.1's no-silent-clone rule absolute. §12.3
admits the cost — *"more verbose in the common `String` case"* — and takes it, *"visible at
the call site rather than hidden in the lowering."*

And nested options are **not** flattened, satisfying doc 01 §13.1, for a reason worth
stating because it is free: `?` does not flatten. There is no flattening code to write and
none to get wrong. §10.2 records what the resulting shape actually is —
`Option<&Option<String>>`, with the borrow outside — which is not the shape the phrase
suggests.

### 1.3 `?.` is one token, which plan 15 already found out

`?.` lexes as `QuestionDot`, so `x?.await` is a parse error and `x? .await` is two
postfixes — plan 15 §10.1 records that collision and
`question_dot_is_one_token_and_not_two_postfixes` pins it. This plan inherits the token and
must not re-decide it: a chain is a run of `QuestionDot` segments, and the `?` postfix is a
different token in a different position.

### 1.4 `catch` is a terminator, and that word still means something

Doc 03 §12.2: *"`catch` is a terminator operator (§5.1.1), so the parenless argument list
ends at it and no parentheses are needed around `parse s`."*

§5.1.1's terminator *set* dissolved with doc 18 §2.1 — juxtaposition made statement position
unremarkable and plan 19 is `superseded` — but the property `catch` needs survives in the
machinery that replaced it: an application's argument list ends at a token that cannot begin
an argument, and `catch` is a keyword, so `starts_argument` already stops. **The decision is
to verify that rather than assume it**, because `*` and `>` each had to be removed from
`continues_line` for the same class of reason (plan 14 §10).

### 1.5 `catch`'s else branch takes a binder, and that is a lambda already

```text
n = parse s catch (e) ->
  log e
  0
```

The right-hand side is doc 18 §4's lambda with its own parameter list, and the lowering puts
its binder in the `Err` arm's pattern. So `catch` reads an expression and, if that expression
is a lambda of one parameter, uses the parameter as the `Err` binding instead of `_`. No new
node and no second syntax.

---

## 2. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes. Each is a syntactic rewrite; none inspects a type |
| Nothing outside core/alloc/std? | Yes. `Option`, `Result`, `as_ref`, `?` — all core's |
| Preserves Rust semantics? | Yes, and §1.1 is why: rustc does the work in all three |
| Buys reachability? | Yes. `try` is **not expressible in stable Rust at all**, which is the strongest case any row in this project has |

---

## 3. Concrete proof this plan targets

`examples/fallible.sdx`, which must **run** and print a sequence only a correct lowering
produces:

- a `catch` taking its fallback on the `Err` path **and** one taking the `Ok` value, so a
  lowering that returned the wrong arm fails;
- a `catch` with a **binder**, printing something derived from the error, so the binder
  demonstrably reaches the `Err` arm;
- a `?.` chain over a value that is `Some` all the way and one that is `None` in the middle,
  so the short-circuit is visible rather than assumed;
- a `?.` chain whose result is **used by reference**, because §12.3's whole claim is that it
  borrows — a lowering that cloned would print the same thing, so the proof is that it
  compiles against a non-`Clone` type;
- a `try` block whose `?` short-circuits **the block** and not the function, with a statement
  after it that still runs. This is the case that cannot be written in stable Rust and the
  one a golden cannot distinguish from a lowering that returns from the function.

The last one is the plan's reason to exist, and only running catches it: a `try` lowered to
bare statements compiles, and the `?` then returns from `main`.

---

## 4. Leaves

### Leaf 0 — `leaf-correct-the-chain-lowering`

**Added during implementation, not planned.** Doc 03 §12.3's published lowering does not
compile and fails doc 03 §2's first admission test — §10.1. The final link takes no method
and the whole path takes a `&`.

### Leaf 1 — `leaf-catch`

`BinaryOp::Catch` at `??`'s own precedence (doc 03 §4.5 places `catch` *"at `??`'s
level"*), lowering to §12.2's `match`. **An operator and not a node**, because it is an
infix over two expressions and `??` already proved the shape — one variant, one precedence
row, one emitter arm. The binder case is §1.5's: a one-parameter lambda on the right
contributes its parameter to the `Err` pattern.

### Leaf 2 — `leaf-optional-chain`

`Expr::OptionalChain { head, segments }` — a run of `?.`-joined field accesses lowering to
§12.3's closure. `as_ref()?` on every link **but the last**, a leading `&`, `Some(…)` around
the body, no flattening. Leaf 0 is why the last link differs.

### Leaf 3 — `leaf-try-block`

`Expr::TryBlock { body }` lowering to an immediately-invoked closure. The block's tail is
its value, and `?` inside it returns from the closure — which is the entire construct, and
costs no new machinery because plan 20's `do` already emits a block expression.

### Leaf 4 — `leaf-proof-and-gate`

`examples/fallible.sdx`, emit tests per construct, a golden, a run expectation, and the gate.

---

## 5. Acceptance criteria

1. `parse s catch 0` emits §12.2's `match` with `Err(_) => 0`.
2. `parse s catch (e) -> …` puts `e` in the `Err` pattern and the lambda's body in the arm.
3. `catch` needs no parentheses around a parenless application — §1.4, asserted rather than
   assumed.
4. `catch` sits at `??`'s precedence level, so `a catch b ?? c` groups as doc 03 §4.5 says.
5. `user?.profile?.name` emits §12.3's closure, with `as_ref()` on every link.
6. A `?.` chain **borrows**: the example compiles against a type that is not `Clone`.
7. A `?.` chain short-circuits — a `None` in the middle yields `None`, demonstrated by
   running.
8. `try` emits an immediately-invoked closure whose `?` returns from the **block**, and a
   statement after the `try` still runs.
9. `?.` is still one token and `x? .await` still parses as two postfixes — plan 15's test
   still passes, asserted here because this plan is the one that could break it.
10. `examples/fallible.sdx` compiles with **zero** rustc output, runs, and matches its
    `.stdout`; a golden covers the text.
11. Any new diagnostic has an `RDX` code reached by the UI corpus.
12. A `?.` chain's **final** link takes no method, so the lowering is fixed by the source
    text alone — asserted as the thing that must *not* appear, because §12.3's published
    form compiles for an `Option` field and not for any other. *Added with leaf 0.*

---

## 6. Out of scope, and what it discloses

- **`??=`** is deleted by doc 18 §7 in favour of its expanded form. Not deferred.
- **`catch` on an `Option`** is not Voltaic: §12 keeps `Option` and `Result` *"visually
  distinct and non-interchangeable"*, `??` serves `Option` and `catch` serves `Result`.
  Writing `catch` on an `Option` emits the `match`, and rustc reports the mismatch — which is
  the division of labour, and cheaper than a check Voltaic would have to type-infer to make.
- **`?.` on a method call** — `user?.name()` — is not in §12.3's grammar, which chains field
  accesses. A method on the end of a chain is an ordinary postfix on the chain's result.
- **Flattening nested options** is declined by doc 01 §13.1 and costs nothing to decline,
  per §1.2.
- **`try` returning `Result` versus `Option`** is not Voltaic's to decide: the closure's return
  type is inferred from its tail and its `?`s, so both work and neither is annotated. A
  `try` whose arms disagree is a rustc error against the block.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald had no `?` operator to delegate to and no closure form that could carry
one, so all three of these were interpreter-level constructs there.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | `?.` lowers to `?` in a closure | `and_then`/`map` chains | Doc 02 §6.3. The closure unifies every link's type through rustc; hand-written combinators differ for a field versus a method and Voltaic has no types to tell them apart |
| 2 | Each link takes `as_ref()` | Move, or clone | Doc 03 §12.3 and doc 01 §25.1: no silent clone, ever. Criterion 6 proves it against a non-`Clone` type rather than asserting it |
| 3 | `try` is an immediately-invoked closure | Wait for Rust's `try` blocks | They have been unstable for years. The closure is stable today and is what the feature would desugar to |
| 4 | `catch`'s binder comes from a lambda on the right | A dedicated `catch (e) ->` syntax | Doc 18 §4's lambda already has a parameter list; a second syntax for the same shape is a second thing to keep consistent |
| 5 | Verify that `catch` terminates an argument list | Assume §5.1.1's terminator property survived | §5.1.1's set dissolved with plan 19. Two tokens have already had to leave `continues_line` for this class of reason |

---

## 10. Result

Shipped. 616 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit` are green. Every
criterion is met, and all thirteen emit tests passed on their first run — the first time in
this run of plans, and entirely because §1.1's framing was right: three constructs that hand
their work to rustc have very little of their own to get wrong.

Except one, which had been wrong in the specification since it was published.

### 10.1 Doc 03 §12.3's lowering does not compile, and fails its own admission test

§12.3 publishes this, twice, as the answer to doc 02 §13's question 7:

```rust
let n = (|| Some(user.as_ref()?.profile.as_ref()?.name.as_ref()))();
```

Compiling it against a `String` field gives:

```text
the trait bound `String: AsRef<String>` is not satisfied
```

`.as_ref()` on the **final** link means two different methods depending on the field's type —
`Option::as_ref` when the field is an `Option`, `AsRef::as_ref` when it is not — and the
second does not exist for `String`. So the lowering is **type-dependent**, which doc 03 §2's
first admission test forbids outright: a lowering must be fixed by the source text alone.

That is not a technicality here. §12.1 rejected `unwrap_or_else` for `??` on exactly this
ground and called the reason *"decisive rather than merely persuasive"*. §12.3 then published
a lowering that fails the same test, three sections later in the same document.

The correction is one character's worth of design: the final link takes **no** method, and
the whole path takes a leading `&`.

```rust
let n = (|| Some(&user.as_ref()?.profile.as_ref()?.name))();
```

`&` borrows whatever is there, whatever its type. It is what §12.3 wanted and says — *"the
chain borrows and never clones"* — and it keeps doc 01 §13.1's no-flattening property, which
`as_ref` only kept by accident when the last field happened to be an `Option`.

**Found by compiling the specification's own example**, which is the third time this run that
a published lowering was wrong in a way reading could not reveal — after doc 18 §2.4's
missing `use T::*;` and plan 23's `lex_fragment` claim.

### 10.2 What no-flattening actually looks like

Doc 01 §13.1 requires that nested options are not flattened, and §1.2 said that costs
nothing to honour because `?` does not flatten. True, but the shape is not the one the
phrase suggests: a final field of type `Option<String>` yields

```rust
Option<&Option<String>>
```

— nested once, with the borrow on the outside. Not `Option<Option<&String>>`, which is what
`as_ref` on the last link would have produced and what the signature in the first draft of
the example asserted. The `&` form is the consistent one: the chain borrows its result and
never looks inside it.

### 10.3 §1.4's verification was worth doing and came back clean

Doc 03 §12.2 claims `catch` needs no parentheses around `parse s` *because* §5.1.1's
terminator set says so — and that set dissolved when doc 18 §2.1 made juxtaposition the call
form and plan 19 became `superseded`. The property survives for a different reason:
`starts_argument` is an allow-list holding no keyword but `true` and `false`, so a keyword
ends an argument list by construction.

Checked rather than assumed, because `*` and `>` each had to be removed from
`continues_line` for this class of reason (plan 14 §10). The check cost one test and
confirmed the claim; decision 5 was right for a reason that did not materialise, which is
the only kind of right a precaution gets to be — the same note plan 22 §10.5 made about
`if_let_guard`.

### 10.4 The `try` proof came out stronger than §3 asked for

§3 wanted a `try` whose `?` short-circuits the block with a statement after it that still
runs. The example's `after_a_failed_try` returns `&'static str`, which makes the case
**doubly** caught:

- lowered to a bare block, the `?` would be a `?` in a function returning `&'static str`,
  and rustc rejects that outright — so the wrong lowering does not even compile;
- and at runtime the statement after the `try` still runs, printing `reached`, which a
  lowering where `?` escaped the block would skip.

One case, caught by compiling *and* by running. That was not planned; it fell out of giving
the function a non-`Result` return type, which is worth remembering as a technique: **put
the construct in a context where the wrong lowering is ill-typed**, and the compiler becomes
the second assertion.

### 10.5 Two surface limitations found by writing the example, neither this row's

- **An ascribed binding needs its value on the same line.** `r :: Result<i32, String> = try`
  followed by an indented block is `expected a line break after the statement`. Doc 18 §5.2's
  ascribed binding (`n :: u64 = 0`) was built for a one-line value, and an indented
  block-valued form was never considered. The example works around it with a helper whose
  parameter pins the type — which reads better anyway — but the limitation is real and
  belongs to doc 18 §6's row.
- **`?` binds to the last argument, not to the application.** `parse_it x?` is
  `parse_it(x?)`, not `(parse_it x)?`, because `?` is a postfix on an atom and juxtaposition
  is the looser form. The correct spelling is `(parse_it x)?`, which is what doc 03 §12.4's
  own example writes — `parse(x)?` — and which was invisible before juxtaposition made the
  parentheses optional elsewhere. It produces a type error rather than wrong behaviour, so
  it is a sharp edge and not a defect; it belongs to row 13's juxtaposition if anywhere.

### 10.6 `catch` needed no new node, and that is the plan's shape

`BinaryOp::Catch` at `??`'s precedence, one arm in the emitter beside `coalesce`'s, and the
binder read out of a lambda the language already had. Doc 03 §12.2's second form — `catch
(e) -> …` — needed no syntax at all, because doc 18 §4's lambda already carries a parameter
list.

`?.` and `try` needed nodes, because neither is an operator over two expressions. But both
emit through machinery plan 20 built: `try_block` uses the nested `Sink` that `emit_do`
introduced, for the reason `emit_do`'s comment already records, and inherits the same coarse
source-map attribution and the same note about what closing it would take.
