# Doc 30 — Bind

Supersedes: doc 18 §3's table and §3.1; doc 03 §14.3's postfix `.await`
Affects: plans 13, 15, 20, 22, 29, 30, 39
Status: `done` — converted 2026-11-09 in four commits; §8.4 names the one gap

---

## 1. The decision

**`<-` means `await`, and nothing else.**

```text
user <- fetch id        →    let user = fetch(id).await;
<- flush ()                  flush().await;
```

It is Haskell's monadic bind, used for the monad Redox actually has: `Future`. Everything
else `<-` currently spells gets a different spelling, and postfix `.await` is deleted.

### 1.1 What this corrects, and why doc 18 §3.1 was wrong about it

Doc 18 §3 is titled *"One arrow, one bar, one draw"*, and §3.1 — *"The draw arrow absorbs
four constructs"* — gives `<-` four jobs on the grounds that it *"reads the same way
everywhere — a name drawn out of something"*.

That sentence does not survive contact with the four positions:

| Doc 18 §3's position | What a Haskell reader sees | What Redox meant |
|---|---|---|
| `conn <- pool.with_connection` | bind in a monad | a callback whose body is the rest of the block |
| `if Some v <- msg` | — (Haskell has no such form) | a pattern test |
| `for c <- candidates` | bind in the **list** monad, inside `do` | a `for` loop |
| `[ x \| x <- xs ]` | a comprehension generator — **the one that is right** | the same |

So doc 18 borrowed Haskell's most recognisable token and kept its meaning in exactly one of
four positions. That is the failure doc 18 §8 warns about everywhere else — *"Redox is Rust
with a different surface"*, and a token whose meaning travels is worse than a token that
does not travel at all, because the reader's first guess is wrong three times in four.

`<-` as await-bind is the opposite: the meaning is exactly Haskell's. `do` notation binds
an effect's result to a name; `x <- fetch id` binds a future's result to a name. The monad
differs and the reading does not.

### 1.2 What the four positions become

| Was | Becomes | Why |
|---|---|---|
| `for c <- candidates` | `for c in candidates` | Rust's own word, which doc 18 §3's table **deleted** — `for x in xs` sits in its *Replaces* column. It comes back |
| `[ x \| x <- xs ]` | `[ x \| x in xs ]` | Consistency with `for`. §4.2 records this as the real aesthetic loss |
| `conn <- pool.with_connection` | `with conn = pool.with_connection` | Doc 03 §7.6 cites *"Koka's `with` statement as a second witness"*, and doc 18 §7 freed `with` |
| `if Some v <- x` | `if Some v = x` | Rust's `if let` with `let` dropped — §3.1 |
| `guard Ok raw <- read p else` | `guard Ok raw = read p else` | Likewise: Rust's let-else with `let` dropped |

Two keywords return — `in` and `with` — and `.await` leaves. Doc 18 §7's keyword count moves
by two in each direction, which is a wash, and the construct count is unchanged.

---

## 2. `<-` in full

### 2.1 It is a statement, and it always binds

```text
user <- fetch id               let user = fetch(id).await;
<- flush ()                    flush().await;
(name, age) <- load id         let (name, age) = load(id).await;
```

The left side is a **pattern**, because plan 33 made a binding's left side one and this is a
binding. The binder may be omitted entirely for a future awaited only for its effect, which
is the one form that binds nothing and is spelled by having nothing to bind.

### 2.2 `async` is a marker, not this document's problem

`<-` is legal only inside an `async` function or block, and **rustc enforces that** — doc 03
§14.3 provides no executor and no runtime, so `async` passes through and the error for an
await outside one is rustc's, landed on the `.sdx` by plan 07's remapping. AGENTS.md's rule,
unchanged.

Where the `async` marker goes on a signature is row 39's, and plan 16's `const fn` already
set the precedent: a marker between `::` and the spine.

### 2.3 Every await binds, which is the point and the cost

Doc 01 §24 had **prefix** `await`. Doc 03 §14.3 corrected it to **postfix** `.await`,
because *"postfix composes with method chains, with `?`, and with the receiver pipeline of
§5.3; prefix requires parenthesising to do any of them."*

This document makes it a **statement-level bind**, which is the third position in three
documents and the first that cannot compose at all. §14.3's argument is therefore live
against this choice, and the cost is exactly what §14.3 predicted. Its own example:

```text
-- doc 03 §14.3, as published
fetch id =
  http::get "/users/#{id}"
    .await?
    .json ()
    .await?
```

```text
-- under this document
fetch id =
  response <- http::get "/users/#{id}"
  checked = response?
  body <- checked.json ()
  body?
```

**The same four lines**, two names instead of none, and no chain. That is the trade, and it
is not free: a reader who wanted one expression now reads four statements. What is bought: one spelling for await, a name on every
awaited value, and the sequence of effects readable down the left margin — which is the
property Haskell's `do` notation exists to provide and is why the user asked for this.

### 2.4 `?` on an awaited value wants its own line, because `?.` is one token

`response?.json ()` does **not** mean "unwrap, then call". `?.` lexes as `QuestionDot`, doc
03 §12.3's optional chain — plan 15 §10.1 recorded that collision and
`question_dot_is_one_token_and_not_two_postfixes` pins it.

Deleting `.await` makes `?` adjacent to an awaited value far more often, so the collision
gets more prominent rather than less. Two spellings work:

```text
checked = response?            -- bind it, which this document's style wants anyway
body <- checked.json ()

body <- (response?).json ()    -- or parenthesise
```

The first is idiomatic here and is what §2.3's example uses. This is a **disclosed
consequence**, not a defect: the lexer is right, and the shape that avoids it is the shape
this document is for.

---

## 3. The displaced positions, in detail

### 3.1 A condition takes `=`

```text
if Some v = x                      if let Some(v) = x {
if Some v = x && v > 2         →   if let Some(v) = x && v > 2 {
while Some v = stack.pop ()        while let Some(v) = stack.pop() {
guard Ok raw = read p else         let Ok(raw) = read(p) else { … };
case o of
  y | Some v = y -> v              y if let Some(v) = y => v,
```

**Rust's `if let` with `let` dropped.** The pattern stays on the left, so plan 22's
`Condition` node, its `ConditionOperand`, `emit_condition` and every lowering are untouched:
only the token the parser looks for changes, from `DrawArrow` to `Eq`.

It costs no keyword, which is why it beats restoring doc 03 §5.5's `is`. `is` would also
work and would read better as prose, at the price of a keyword and of reversing the operand
order — and the reversal would mean rewriting plan 22's parser rather than retargeting one
token.

**Two near-misses, both benign:**

- `if x = 1` where `x == 1` was meant becomes `if let 1 = x`, which **means the same thing**.
  The typo is harmless.
- `if a = b` where both are names is `if let a = b`, an irrefutable pattern — always true,
  and rustc warns `irrefutable_let_patterns`. The typo surfaces as a warning on the author's
  own line.

A condition's `=` cannot be confused with a binding's, because the two are in disjoint
positions: a binding is a statement and a condition follows `if`, `while` or `|`. The scan
plan 22 built already stops at `&&`, `||`, `then`, `->` and a line break, and already counts
brackets so a record literal's `=` inside braces is not seen — so it retargets without
changing shape.

### 3.2 `for` and the comprehension take `in`

```text
for c in candidates           for c in candidates {
[ x | x in xs, x != 0 ]   →   xs.into_iter().filter_map(|x| (x != 0).then(|| x))…
```

`for` is Rust's own word, deleted by doc 18 §3 and restored here, and it needs no argument
beyond that. The comprehension is the loss: `|` with `<-`
is the single most recognisable thing in Haskell's syntax, and `[ x | x in xs ]` reads as
neither Haskell nor Rust. §4.2 states the case and takes it anyway, because one token with
one meaning is worth more than one position that reads perfectly.

### 3.3 The callback bind takes `with`

```text
with conn = pool.with_connection      pool.with_connection(|conn| {
with tx = conn.transaction        →       conn.transaction(|tx| {
tx.execute sql ?                              tx.execute(sql)?;
Ok ()                                         Ok(())
                                          })
                                      })
```

Doc 03 §7.6's lowering, unchanged — *"rewrites the remainder of the enclosing block into a
closure passed as the final argument"*. Only the spelling moves, from `use x <- f` (§7.6) via
`x <- f` (doc 18 §3.1) to `with x = f`.

`with` is the right word and was free: doc 03 §7.6 cites Koka's `with` statement as its
second witness, and doc 18 §7 deleted doc 03 §7.5's unrelated `with` *blocks* — *"an ordinary
binding — no construct needed"* — leaving the keyword unused.

`RDX0044` survives untouched. Plan 30's diagnostic is about a `return` in the generated
closure's tail, which is a property of the lowering and not of the keyword that selects it.
Its message names the construct, so the word in it changes and nothing else does.

---

## 4. What this costs, measured

### 4.1 The conversion

Counted rather than estimated, as doc 18 §11.4 did:

| Surface | Count |
|---|---|
| `<-` occurrences in `.sdx` across the corpora | 28, in 6 files |
| Rust sources naming `TokenKind::DrawArrow` | 7 |
| Emit-test files with `<-` in a source literal | 7 |
| Plans whose spelling changes | 13, 20, 22, 29, 30 |
| Plans whose construct is **deleted** | 15 (postfix `.await`) |

Every `.sdx` holding a `<-` is one of six files, and five of the seven Rust sites are a
single token in a single `check` call.

Plan 15 is cheaper to undo than expected, because it never had a corpus of its own: there is
no `examples/await.sdx` and no golden. What goes is `Expr::Await`, one emitter arm, one
postfix arm in the parser, and six of the seven tests in
`crates/redox-emit/tests/await_postfix.rs`. Row 15 becomes `superseded` — the fourth row in
this project superseded by a later surface decision rather than by being wrong.

**The seventh test moves rather than goes.**
`question_dot_is_one_token_and_not_two_postfixes` pins the `?.` collision, and §2.4 is why
that collision matters *more* after this change, not less: optional chaining is plan 28's
and still live. The test belongs beside it, in `fallible.rs`.

### 4.2 The aesthetic loss, stated rather than hidden

`[ x | x <- xs ]` is Haskell's comprehension, and doc 18 §3.2 adopted it exactly. Replacing
`<-` with `in` there gives up the one position where doc 18's borrowing was faithful, in
order to free the token for the position where it is *more* faithful.

That is a real trade and it should be recorded as one. The argument for taking it: a reader
meeting `<-` in Redox will meet it as await far more often than in a comprehension, because
async code is common and comprehensions are a line here and there — so the token's one
meaning should be the one it carries most.

### 4.3 What this document does **not** change

- **Every lowering.** Doc 18 §1's governing rule applies to this document too: the Rust
  side of every table above is what it already was. `with` emits plan 13's closure, `=` in a
  condition emits plan 22's `if let`, `in` emits Rust's `for`, and `<-` emits `.await`.
- **`RDX0044`**, per §3.3.
- **`|` as the guard bar**, which doc 18 §5.4 protects by keeping `or` for alternation. `in`
  arriving inside `[ … | … ]` does not touch it.
- **Doc 03 §11.3's refusal of a concurrent-await construct.** Every joining form in Rust
  comes from a runtime crate, so `join` and friends stay out for §2's dependency reason, and
  `<-` sequences effects exactly as `do` does.
- **The `::` adjacency rule, the signature line, equations, guards, `case … of`, `data`,
  `class`** — doc 18's five moves are untouched. This corrects one table in §3 and nothing
  in §2.

---

## 5. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes. `p <- e` is `let p = e.await`; the shape is syntactic |
| Nothing outside core/alloc/std? | Yes. `.await` is a language feature and the executor stays the user's |
| Preserves Rust semantics? | Yes — it *is* `.await`, bound |
| Buys reachability? | **No, and it does not need to.** This is a spelling decision, not an admission. It removes a spelling (`.await`) and moves three others. Doc 18 §1's second rule governs instead: *change the spelling, keep the lowering* |

---

## 6. Order of conversion

One construct per commit, as doc 18 §11 did, and in this order so the suite is green at
every step:

1. **`for x in xs`** and the comprehension's `in`. Independent of everything else; `<-` keeps
   working meanwhile.
2. **`with x = f`** for the callback bind, with plan 30's `RDX0044` message reworded.
3. **`=` in a condition**, retargeting plan 22's scan from `DrawArrow` to `Eq`, and `guard`
   with it.
4. **`<-` becomes await**, and postfix `.await` is deleted — last, because only now is the
   token free.
5. **Row 39's `async`**, which is where `async fn` and `async` blocks were always going.

Step 4 is the only one that cannot be done first, and steps 1–3 are each a token change plus
a corpus rewrite. Doing 4 before them would leave `<-` with five meanings for one commit,
which is the state this document exists to end.

---

## 7. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | `<-` means await and nothing else | Doc 18 §3.1's four positions | A token whose meaning travels is worse than one that does not, because the reader's first guess is wrong three times in four — §1.1 |
| 2 | Every await binds; postfix `.await` is deleted | Keep `.await` for expression position and `<-` for statements | One spelling. The cost is §2.3's: a two-await chain becomes two bindings, which is what doc 03 §14.3 warned prefix would cost and is the shape `do` notation is for |
| 3 | A condition takes `=` | Restore doc 03 §5.5's `is` | No keyword, and the pattern stays on the left — so plan 22's node, emitter and lowerings are untouched and only the token moves. `is` would have reversed the operand order and cost a keyword |
| 4 | `for` and the comprehension take `in` | A third token for the generator | Rust's own word, which doc 18 §3's table deleted and which costs nothing to restore. §4.2 records the comprehension's loss as the price |
| 5 | The callback bind takes `with` | Restore `use`, or `from` | Doc 03 §7.6 names Koka's `with` as its second witness, and doc 18 §7 freed the keyword. `use` is Rust's import word and was always a poor fit |
| 6 | The binder may be omitted — `<- flush ()` | Require a binder, or `_ <- flush ()` | A future awaited for its effect binds nothing, and `_ <- f` is a pattern that discards where no pattern is wanted. Rust writes `f().await;` and so does this |

---

## 8. Result

Shipped in four commits, one per construct, as §6 fixed the order. 657 tests pass; `fmt`,
`clippy -D warnings` and every gate target are green. **`<-` means `await` and nothing
else.**

| Step | Commit | What moved |
|---|---|---|
| 1 | `for x in xs`, and the comprehension's generator | Two parse sites, 28 corpus lines, no lowering |
| 2 | The callback bind is `with x = f` | One keyword back, `try_draw` reads a keyword instead of scanning |
| 3 | A condition and a `guard` take `=` | Three `expect` calls and one scan |
| 4 | `<-` becomes await; postfix `.await` is deleted | `StmtKind::Await` in, `Expr::Await` out |

### 8.1 The conversion cost what §4.1 measured, and less

Not one golden moved in steps 1 or 2, because neither changed a lowering: `for c <- 1 .. 4`
and `for c in 1 .. 4` both emitted `for c in 1..4`, and `with conn = f` emits plan 13's
closure unchanged. Step 3 moved one UI expectation, and only the **quoted source line** in
it — the diagnostic's text was already right.

Step 2 made the parser strictly simpler rather than merely different. Doc 18 §3.1's `x <- f`
needed a token of lookahead *and* a hard-won lexer fix: `x <- f` once lexed as `<` then `-`,
parsed as `x < -f`, and emitted valid Rust comparing `x` against negated `f` with no
diagnostic at all. A keyword cannot do that, so the construct that caused this project's
worst shipped defect is now the one that cannot recur.

### 8.2 §3.1's claim held exactly

*"Only the token the parser looks for changes, from `DrawArrow` to `Eq`."* It was three
`expect` calls and one scan. `Condition`, `ConditionOperand`, `emit_condition`,
`condition_head`, the let-chain breaking and `RDX0107` were all untouched, and plan 22's
suite passed without an assertion changing except the two that quote source.

`holds_a_draw_arrow` is now `holds_a_pattern_test`, because its name said which token it
looked for. Its doc comment records that depth matters *more* than it did: a record literal's
`=` lives inside its braces, so `if p { x = 1 }.valid` must stay an ordinary condition, and
the bracket count is what keeps the two apart. `==`, `+=` and the other four compound
operators are different tokens and are never seen.

### 8.3 The receiver rule carried step 4 for free

`.await` is a postfix and binds tighter than every operator, so an operator-valued future
needs parenthesising:

```text
n <- count () ?? 0    →    let n = (match count() { … }).await;
```

`emit_await` reaches `receiver` for that rather than answering the question a second time,
and the answer was already right — the first probe of the new form produced the parentheses
with no work. This is the fourth construct to inherit that rule unchanged.

### 8.4 The runnable example waits for row 39, and that is the one gap

Every `<-` needs an `async` function around it, and `async` in a signature is **not
implemented**: `run :: async ()` emits `fn run<ASYNC>() -> ASYNC ()`, reading the marker as a
type variable. §2.2 assigned the marker to row 39 before the conversion started, and that
assignment stands.

So step 4 is proved by `crates/redox-emit/tests/await_bind.rs` — eight tests over the three
forms, the receiver rule, and the two things `<-` is *not* — and **not** by a program that
runs. That is a weaker proof than every other construct in this project has, and it is
stated here rather than papered over: `examples/async.sdx` is row 39's first deliverable, and
until it exists the await-bind's lowering is asserted and not executed.

The alternative was to implement row 39's marker inside this conversion, which would have
made doc 30 a plan for a row it does not own.

> **Paid 2026-11-09 by plan 39** (`docs/31 - 39 - Async.md`). `examples/async.sdx` runs,
> with its own nine-line executor, and exercises the await-bind in an `async fn`, in an
> `async` block, across two awaits whose results are combined, and in the binderless form.
> The await-bind is now proved like every other construct here.

### 8.5 Two things deleted, and one test moved

`Expr::Await` is gone, with its emitter arm, its postfix parse arm, and
`await_postfix.rs`'s seven tests. `wraps_as_receiver` lost its `Await` case: an await is a
statement now, so it never appears as a receiver.

`question_dot_is_one_token_and_not_two_postfixes` **moved** rather than went, as §4.1 said it
should. It also stopped using `.await` to demonstrate itself, which is fitting: doc 30 §2.4
is precisely that the `?.` collision matters more after this change, so the test now shows it
with `user?.profile` against `(parse x)? .inner`.

### 8.6 The architecture tests, a tenth time

`try_await.rs` declared two constructs; `reaches_a_draw_arrow` became a sibling. Tenth time
in this run, and by now the expected tax on any new parser function rather than a surprise.

### 8.7 One thing still unexplained, with more evidence

Plan 23 §10.8 recorded a full-gate run that exits 1 with no task reporting a failure, where
an immediate re-run passes. It happened again during step 2, and this time **twice
consecutively** before passing — while every target passed when run individually in the same
shell.

Still no identified task, so still not in AGENTS.md's gotcha list. The new fact is that one
re-run is not always enough.

> **Explained 2026-11-09, during plan 34 leaf 4.** blaze's worker cache lives on a 6.2 GB
> tmpfs under `/run/user/1000`, and a broad enough build writes past it: the worker dies with
> `No space left on device` and blaze reports `could not build <every crate>`. Every symptom
> follows — no named task, a re-run that passes once the cache is reclaimed, and each target
> passing alone because one target builds less and fits. The observation above, that one
> re-run is not always enough, is the reclaim not being instant. It is in AGENTS.md's gotcha
> list now, and the diagnosis was in blaze's output all along: three plans grepped it away
> while recording that there was none.
