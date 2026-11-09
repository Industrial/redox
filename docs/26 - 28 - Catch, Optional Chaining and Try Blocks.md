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

n = user?.profile?.name      →   let n = (|| Some(user.as_ref()?
                                     .profile.as_ref()?.name.as_ref()))();

result = try                     let result = (|| {
  a = parse x?                       let a = parse(x)?;
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
| `try` | **`?` inside an immediately-invoked closure.** Rust has wanted `try` blocks for years and has them only unstably; the closure gives Redox them on stable today |

Two of the three are the *same* mechanism — a closure whose `?` returns from the closure
rather than the function — which is why they are one plan and not two.

### 1.2 `?.` is the one with a real design claim in it

Doc 03 §12.3 resolves doc 02 §13's question 7 outright: **the chain borrows and never
clones.**

```text
n = user?.profile?.name          (|| Some(user.as_ref()?.profile.as_ref()?.name.as_ref()))()
n = user?.profile?.name.clone!   … .name.as_ref().cloned()
```

`as_ref()` on each link is what keeps doc 01 §25.1's no-silent-clone rule absolute. §12.3
admits the cost — *"more verbose in the common `String` case"* — and takes it, *"visible at
the call site rather than hidden in the lowering."*

And nested options are **not** flattened, satisfying doc 01 §13.1, for a reason worth
stating because it is free: `?` does not flatten. There is no flattening code to write and
none to get wrong.

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

### Leaf 1 — `leaf-catch`

`Expr::Catch { subject, fallback }`, read as a left-associative infix operator below `??`
(doc 03 §4.5 places `catch` *"at `??`'s level"*), lowering to §12.2's `match`. The binder
case is §1.5's: a one-parameter lambda on the right contributes its parameter to the `Err`
pattern.

### Leaf 2 — `leaf-optional-chain`

`Expr::OptionalChain { head, segments }` — a run of `?.`-joined field accesses lowering to
§12.3's closure. `as_ref()` on each link, `Some(…)` around the body, no flattening.

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

---

## 6. Out of scope, and what it discloses

- **`??=`** is deleted by doc 18 §7 in favour of its expanded form. Not deferred.
- **`catch` on an `Option`** is not Redox: §12 keeps `Option` and `Result` *"visually
  distinct and non-interchangeable"*, `??` serves `Option` and `catch` serves `Result`.
  Writing `catch` on an `Option` emits the `match`, and rustc reports the mismatch — which is
  the division of labour, and cheaper than a check Redox would have to type-infer to make.
- **`?.` on a method call** — `user?.name()` — is not in §12.3's grammar, which chains field
  accesses. A method on the end of a chain is an ordinary postfix on the chain's result.
- **Flattening nested options** is declined by doc 01 §13.1 and costs nothing to decline,
  per §1.2.
- **`try` returning `Result` versus `Option`** is not Redox's to decide: the closure's return
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
| 1 | `?.` lowers to `?` in a closure | `and_then`/`map` chains | Doc 02 §6.3. The closure unifies every link's type through rustc; hand-written combinators differ for a field versus a method and Redox has no types to tell them apart |
| 2 | Each link takes `as_ref()` | Move, or clone | Doc 03 §12.3 and doc 01 §25.1: no silent clone, ever. Criterion 6 proves it against a non-`Clone` type rather than asserting it |
| 3 | `try` is an immediately-invoked closure | Wait for Rust's `try` blocks | They have been unstable for years. The closure is stable today and is what the feature would desugar to |
| 4 | `catch`'s binder comes from a lambda on the right | A dedicated `catch (e) ->` syntax | Doc 18 §4's lambda already has a parameter list; a second syntax for the same shape is a second thing to keep consistent |
| 5 | Verify that `catch` terminates an argument list | Assume §5.1.1's terminator property survived | §5.1.1's set dissolved with plan 19. Two tokens have already had to leave `continues_line` for this class of reason |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
