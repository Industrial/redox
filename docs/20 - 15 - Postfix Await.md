# Plan 15 — Postfix Await

Plan ID: `await-postfix`
Depends on: 03 (`parser-core`)
Milestone: 4 — *specification-correct*
Status: `planned`

---

## 1. What this plan is

`.await`, postfix.

Doc 03 §14.3 **corrects doc 01 §24** from prefix `await` to Rust's own postfix form, and
gives the reason in one sentence: *"Postfix composes with method chains, with `?`, and with
the receiver pipeline of §5.3; prefix requires parenthesising to do any of them."*

```text
http::get url |> await |> json |> await     no — `await` is not a method
(fetch id).await?.json ().await?            yes
```

### 1.1 Why this is a correction and not a feature

Doc 01 §24 specified `await f ()`, which is prefix. Rust has had postfix `.await` since
1.39, and doc 03 §21's table lists it at that version — so the prefix form was never
reachable, and a `.await` written today is a parse error (§2). Milestone 4's rows are
*"corrections to a published specification"*, and this is one: the specification named a
syntax Rust does not have.

### 1.2 Why this plan is one leaf and not four

`.await` is **verbatim passthrough** (§14.3), and the postfix machinery it needs already
exists for `?`, for a field, for a tuple index and for a method call. Doc 18 §2.1 names
`(get u).await` as one of its two surviving required-parens cases, so even the grouping
rule is written. What is missing is one arm.

Doc 04's row 39 (`async`) owns `async fn` and `async` blocks. **This plan deliberately does
not**: a `.await` inside a non-`async` function is rustc's error to give, and giving it is
better than Redox refusing to parse the construct that provokes it. Doc 01 §24 provides no
executor and no runtime, so there is nothing else here to build.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `await` is reserved | `keyword.rs` | `Keyword::Await`, from doc 03's set |
| `.await` does not parse today | `redox build` on a one-line fixture | `RDX0103: expected a field name, a tuple index, or a method name after \`.\`` |
| The postfix loop already composes | plan 03, and `?` under plan 17 | `postfix` returns `Ok(Ok(grown))` and loops, so `.await?` needs no sequencing work |
| Doc 03 §14.3 wants no lowering | §14.3 | *"It is also verbatim passthrough."* |

---

## 3. Concrete proof this plan targets

`.await` in the three positions §14.3 says prefix could not reach without parentheses: a
method chain, before `?`, and as a pipeline's receiver.

The example corpus **cannot run this one**, and that is worth stating rather than
discovering: an `async fn` needs an executor to drive it, doc 01 §24 provides none, and
plan 10's rule is that a dependency needs a written reason — a runtime crate for one
example is not one. So the proof is the golden corpus and the emit tests, and the
`examples/` corpus gets nothing. §6 records what that costs.

---

## 4. Leaves

### Leaf 1 — `leaf-await-postfix`

`Expr::Await { inner, span }`, one arm in `postfix` after `.`, one arm in `emit_expr`
emitting `{receiver}.await`. The receiver goes through `receiver`, so a `.await` on an
operator expression gets the parentheses `.` would otherwise take from it.

### Leaf 2 — `leaf-proof-and-gate`

Emit tests for the three positions, a golden, and the gate.

---

## 5. Acceptance criteria

1. `(fetch id).await` emits `fetch(id).await`.
2. `.await?` emits `.await?` — the two postfixes compose in source order.
3. `.await` chains: `(fetch id).await.json ()` emits `fetch(id).await.json()`.
4. A `.await` on an operator expression is parenthesised, as every other receiver is.
5. `xs |> await` is **not** special-cased into `.await` — `await` is not a method, and a
   pipeline whose right operand is `await` must emit whatever a pipeline emits. The
   construct is `(…).await`.
6. A golden covers all of the above; the gate is green.

---

## 6. Out of scope, and what it discloses

- **`async fn` and `async` blocks** are row 39. Until then `.await` is writable and the
  function containing it cannot be declared `async`, so **every use of this construct is a
  rustc error today** — "`await` is only allowed inside `async` functions and blocks". That
  is a real limitation and it is the right one: the alternative is refusing to parse a
  correction that Milestone 4 says lands first.
- **No executor.** Doc 01 §24 is explicit, and §14.3 declines a concurrent-await construct
  for doc 03 §11.3's dependency reason — *"every joining form in Rust comes from a runtime
  crate"*.
- **No example.** §3 says why. The cost is that `.await`'s *runtime* behaviour is unproven
  by this plan, which is unavoidable while there is no executor; its *text* is proven by the
  golden, which is the whole of what a passthrough can be asked for.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald has no async surface.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | `.await` is postfix | Doc 01 §24's prefix `await` | Rust has only the postfix form; the prefix one was never reachable. Doc 03 §14.3's correction |
| 2 | `Expr::Await` rather than reusing `Expr::Field` with the name `await` | A field named `await` | `await` is a reserved word, so it is not an `Ident`, and a node that pretends otherwise would let the formatter and the span map both read it wrongly |
| 3 | Ship it before row 39's `async` | Wait, so the construct is usable | Milestone 4's rows land first by doc 04's rule. §6 discloses the cost |
| 4 | No `examples/` case | Add a runtime crate for one example | Doc 02 §5.5 requires a written reason per dependency, and "one example needs an executor" is not one |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
