# Plan 60 — The Bind Arrow

Plan ID: `bind-arrow`
Depends on: 54 (`assignment`), 55 (`expression-orientation`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

`<-` becomes monadic bind, read by the enclosing block modifier. Pattern tests move to `is`.
`=` goes back to meaning definition and nothing else.

```
work = async                    r = try                     if o is Some v
  n <- fetch ()     -- await      a <- parse x    -- the `?`   return v
  n + 1                           b <- parse y
                                  Ok (a + b)                guard o is Some v else
                                                              return 0
```

Three tokens, three jobs. Today `<-` means `await` and only `await`, `=` means five things, and
`is` means nothing.

### 1.1 `=` currently means five things, and this milestone has removed one

Counting what `=` does today:

| Position | Meaning |
|---|---|
| `x = 5` | binding |
| `f a b = e` | function definition |
| `Point { x = 10 }` | record literal field |
| `if Some v = o` | **pattern test** |
| `guard Some v = o else` | **pattern test**, let-else |

Plan 54 declined to add a sixth (assignment, which went to `:=`). The first three are one idea —
definition — and that is Haskell's single meaning for the token. The last two are matching, which
Haskell spells with no `=` at all because it has no `if let`.

The attribution matters, because this row reverses a specific decision. Doc 03 §5.5 **originally
specified `is`**, with scrutinee-first operands. Doc 18 §3 then retired `is` in favour of `<-`,
on §3.1's *"a name drawn out of something"* reading. Doc 30 — which supersedes doc 18 §3's table
and §3.1 — took `<-` for `await` and moved the pattern test to **`=`**, and §7 decision 3 is the
record:

> | A condition takes `=` | Restore doc 03 §5.5's `is` | No keyword, and the pattern stays on the
> left […] `is` would have reversed the operand order and cost a keyword |

So the decision being reversed is **doc 30 §7 decision 3**, and its reasons were a keyword and
the operand order. This row accepts both costs: `is` is one keyword, and reversing the operands
is the improvement rather than the price, because scrutinee-first is the order a reader thinks in
and the order Rust, C# and Kotlin all use. Doc 30 §3.1's own argument for `=` was that it keeps
plan 22's `Condition` node untouched — a cheap implementation, which is true and is not a reason
for `=` to mean two things.

### 1.2 `<-` paid for `await` with the comprehension, and now gets the better job

Doc 30's thesis is **"`<-` means `await`, and nothing else"** — and §1.1 reaches it by
*rejecting* doc 18 §3.1's *"a name drawn out of something"*, on the ground that *"that sentence
does not survive contact with the four positions."*

Doc 30 §4.2 then records the price, under the heading *"The aesthetic loss, stated rather than
hidden"*: `[ x | x <- xs ]` was the one position where doc 18's borrowing of `<-` was faithful to
Haskell, and it was given up so the token could carry await.

With plan 55 making `try` and `async` **block modifiers** rather than separate constructs, there
is a better job available: `<-` binds, and the *block* says what binding means.

```
async   ->  await the sub-effect
try     ->  `?` the Result
```

That is do-notation, earned rather than imitated — and it generalises. A future block modifier
for another effect type gets `<-` for free, which is the property that makes the token worth
having at all.

### 1.3 `?` stays, and so does the effect-only form

Two things are deliberately **not** replaced.

`?` continues to work inside a `try` block. `<-` is sugar for it, not a replacement, and a
program mixing them is legal — because `?` is also valid outside a `try` block in any function
returning `Result`, and a rule that `<-` is the only spelling would make the two positions
inconsistent.

Doc 30 §7 decision 6 keeps `<- f ()` — the effect-only form that binds nothing — distinct from
`_ <- f`. `examples/async.sdx` exercises it and the distinction survives: `<- noop ()` is a bind
whose value is discarded by position, which under plan 55's block-value rules is an ordinary
non-final expression.

### 1.4 Why `is` rather than keeping `=`

Three reasons, in order of weight.

**It removes the overload.** After plan 54 and this row, `=` means definition and `:=` means
assignment and `is` means matching. No token does two jobs.

**The operand order improves.** `if o is Some v` puts the thing being tested first.
`if Some v = o` reads as though `Some v` is being defined, which is what `=` means everywhere
else in the language — and that is the confusion, not merely an inelegance.

**Doc 03 §5.5 already specified it.** This is a restoration, not an invention; doc 18 §3 reversed
it for Haskell's sake, and the Haskell argument is weaker now that `<-` is doing Haskell's actual
job two sections up.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**`<-` is await and nothing else.** `examples/async.sdx` uses it in four positions — a plain
bind, a bind of a bind, the effect-only form, and inside an `async` block — and all four are
awaits.

**`=` is the pattern test in every position.** `examples/draw_condition.sdx` exercises
**six** shapes across **eight** sites: `if Some v = o`; the let-chain `if Some v = o && v > 2`;
`else if Some _ = o`; `while Some v = stack.pop ()`; `guard Some v = o else`; two arm guards
including the nested `y | Some (Some v) = y ->`; and the condition *expression*
`if Some _ = o then "present" else "absent"`, which the file notes is *"told apart by `then`"* —
and which plan 55 retires `then` from, so this row and that one meet here.

**`is` is not a token and not reserved.** Nothing uses it; the lexer arm is additive.

**`?` works inside and outside `try`.** Inside: `examples/fallible.sdx`. Outside:
`examples/errors.sdx`, whose `retries` has `n = raw.parse ()?` in a `where` local of a
`Result`-returning function. (`fallible.sdx`'s own out-of-`try` uses are `?.`, a different token,
which plan 57 deletes.) And `semver`-shaped code relies on a signature to pin the `try` block's
error type — the interaction plan 62 owns and this row must not disturb.

**Corpus scale.** 29 examples — the example count is **31** as of 2026-11-10 and `docs/STATUS.md` is generated and authoritative; it was 29 when this was measured; `draw_condition.sdx` is the file this row rewrites most, with
**8** pattern-test sites — lines 18, 33, 35, 47, 57, 70, 71, 77 — across the **six** shapes named
two paragraphs up.

Corrected from *"13 pattern-test sites across its five shapes"*: §11.5 found the 13 is a
`grep -c " = "` artifact counting 8 tests plus 5 equation definitions, contradicting leaf 6's own
eight, and the "five shapes" contradicts this section's own six four paragraphs earlier.

---

## 3. Concrete proof this plan targets

**A `try` block written with `<-` and a `try` block written with `?` produce byte-identical
generated Rust.** That is the strongest available statement that `<-` is sugar and not a second
mechanism, and it is checkable.

Six cases:

1. **`<-` in an `async` block** — await, unchanged behaviour from today's spelling.
2. **`<-` in a `try` block** — the `?`, with the §3 headline equality asserted.
3. **`<-` outside any block modifier** — a named Redox diagnostic. The token's meaning comes from
   the block, so no block means no meaning, and that must be a diagnostic rather than a default.
4. **The let-chain.** `if o is Some v && v > 2` — one condition with two operands where the
   second reads the first's binding. Plan 22 §1.5 verified the Rust form is stable; the spelling
   changes and the lowering does not.
5. **A `guard` with `is`.** `guard o is Some v else return 0` — let-else, with rustc enforcing
   divergence as it does today.
6. **An arm guard.** `y | y is Some (Some v) ->` — the nested-pattern case
   `draw_condition.sdx` already asserts, in the new spelling.

---

## 4. Leaves

### Leaf 1 — `leaf-is-token`

`is` as a token, and the pattern-test parse with scrutinee-first operands. Covers all **seven**
positions §11.5 counts: condition, `else if`, let-chain, `while`, `guard`, arm guard, and the
condition-expression form, whose `then` plan 55 removes, so the two rows must agree on what that
line becomes. Corrected from *"all **six** shapes §2 enumerates"*, which is the list this leaf
already wrote seven of. The `=` token is consumed at only **two** sites —
`condition_operand.rs:22` and `control_flow.rs:150` — through one scanner,
`holds_a_pattern_test`, and `condition()`'s four callers are where the surface positions come
from.

**This leaf re-reserves `is`, which is a breaking change, and a test exists to stop it.** §2's
*"not a token and not reserved … the lexer arm is additive"* is true in letter and misleading in
substance, which §11.2 is the finding for: `is` was **actively removed**. `keyword.rs`'s header
records doc 18 §7 retiring eleven words, and `leaves_ordinary_identifiers_alone` iterates that
list asserting `Keyword::from_text` is `None`, under the reason *"a retired keyword that stayed
reserved would reject a program that is now legal, and nothing else in the suite would notice"*.
`is = 5` compiles today. The precedent for restoring one is `With`, on the same retired list and
in the enum today by doc 30 §3.3. Two artifacts go stale on landing and are this leaf's to
update: that test's array, and `emit/emit_stmts.rs`'s doc comment, which still says doc 18 §3.1
*"could delete `use`, `is`, `defer` and the comprehension's `for` in favour of one token"*.

### Leaf 2 — `leaf-retire-equals-match`

`=` removed from the pattern positions, with a diagnostic naming `is`. Every pre-existing program
uses the old form, so the error message is the migration guide — plan 52's and plan 59's choice,
applied a third time.

**`until` is a rejected eighth position**, per §11.5: `control_flow.rs:50-62` raises `RDX0107`
when a negated condition carries a pattern. This leaf carries `is` into that detection **and**
into its message — `code.rs` still documents it as `until Some v <- x`, stale since doc 30 moved
the token to `=`.

### Leaf 3 — `leaf-bind-in-async`

`<-` in an `async` **function** or an `async` block is await. This is today's lowering, re-sited
onto the modified-context rule rather than onto the token. Narrowed from *"`<-` in an `async`
block is await"* per §11.1: `is_async` lives on `Signature`, not on a block, and five of
`examples/async.sdx`'s six `<-` uses sit in `async fn` bodies with no `async` block at all.

**The `<-` surface is three files, and `Expr::Draw` is not one of them** — §11.4. `Expr::Draw`
reads `Kw(With)`, doc 30 §3.3's `with conn = pool.with_connection`, and `emit_stmts`'s callback
flattening is `with`'s lowering with nothing to do with `<-`; the name is historical residue. The
real sites are `operator_table.rs`'s `("<-", DrawArrow)`, `try_await.rs` producing `Expr::Await`,
and `emit_await.rs`.

**And there is no parser context flag anywhere.** `Parser` has three fields — `toks`, `pos`,
`errors` — and `Expr::TryBlock`/`Expr::AsyncBlock` are ordinary prefix expressions the parser
never knows it is inside. The *emitter* does have the nesting (`try_block` and `emit_do` both
recurse into `emit_stmts`), so one new parameter threads it, with `check_callback_tails` as the
precedent for the post-parse alternative. Leaves 4 and 5 share that carrier.

### Leaf 4 — `leaf-bind-in-try`

`<-` in a `try` block is `?`. Shares the existing `?` lowering, which is what makes §3's
byte-identical assertion possible — two spellings, one emission path.

### Leaf 5 — `leaf-bind-outside-block`

The §3 case 3 diagnostic — `<-` in a function that is neither `async` nor inside a `try` or
`async` block, which is criterion 3's narrowing — with a fixture under
`crates/redox-cli/tests/ui/`, the path §11.6 records.

### Leaf 6 — `leaf-corpus-and-docs`

`draw_condition.sdx`'s 8 sites, plus `async.sdx`'s `if Poll::Ready v = …`, `destructuring.sdx`
and `semver`-shaped code. Doc 30 is the document most affected: **§3.1 and §7 decision 3** are
what this row reverses and must be amended with their reasoning preserved, §1.1's rejection of
doc 18 §3.1 is what §1.2 now cites, §3.2 and §4.2 describe a token economy this row changes, and
§4.2's recorded loss is **not** reversed (§6). Doc 18 §3 and doc 03 §5.5 amended;
`docs/GRAMMAR.md` regraded.

**Doc 03 §5.5's amendment is two-part, not one** — §11.5. Its operand order is the half §1.4
names, and it specifies a *second lowering* this row never mentions: *"no binders → `matches!`"*,
so `if status is Active` becomes `if matches!(status, Active)`. This row keeps the `if let`
lowering throughout, so that half has to be amended too.

**`docs/GRAMMAR.md` is already wrong in one row and missing another** — §11.6. The `let chains`
row cites *"the draw arrow in a condition (doc 30)"*, when doc 30 is what stopped `<-` being the
condition token; and there is **no `let`-`else` row at all**, though `guard` lowers to one.

---

## 5. Acceptance criteria

1. `is` works in all **seven** pattern-test positions, each exercised by a running example.

   Amended from *"all six pattern-test positions"*: §11.5 counts seven — condition, `else if`,
   let-chain, `while`, `guard`, arm guard and the condition-expression form — which is the list
   leaf 1 already wrote while calling it six.

   §11.5 also found an **eighth** position that `is` would make spellable for the first time: an
   equation-row guard cannot hold a pattern test at all today, because `Guard.condition` is an
   `Option<Expr>` rather than a `Condition`, so `| Some v = o = v` is `RDX0101`. That is an
   argument *for* this row which the row does not make, and an AST change it does not price.
   Whether it is in scope is **open and owed** rather than decided here.
2. `=` in a pattern position is a named diagnostic naming `is`, with a fixture under
   `crates/redox-cli/tests/ui/`. The negated-condition case keeps the code it already has:
   `until` with a pattern is `RDX0107` today, and leaf 2 carries `is` into both its detection and
   its message.

   **Two new diagnostic codes are owed and §5 priced none** — this criterion's retired `=` and
   criterion 3's unmodified-context `<-`, which §11.6 is the finding for. They are claimed as a
   **count rather than as numbers**: `RDX0111` is the last taken in the syntactic band and plan
   55 takes `RDX0112`.
3. `<-` inside an `async` **function** or an `async` block awaits; in a `try` block it is `?`;
   outside all three it is a diagnostic.

   Amended from *"`<-` in an `async` block awaits; in a `try` block it is `?`; outside both it is
   a diagnostic"*, which §11.1 found would reject **five of the corpus's six** `<-` uses:
   `async` is a *signature* marker, `is_async` lives on `Signature`, and five of
   `examples/async.sdx`'s six uses (lines 53, 54, 65, 72, 73, 87) sit in `async fn` bodies with
   no `async` block, only line 85 inside one. As written the criterion fails
   `examples/async.sdx`, and criterion 7 with it. §1's table, which shows only the block form,
   misframes the same thing. Measured for free: `x <- g n` in a plain non-async function emits
   `let x = g(n).await;` today with no Redox diagnostic, rustc catching it.

   **The nesting rule is still unstated**, and §11.1 names it rather than settling it: a `try`
   block inside an `async fn` has both readings available, and the row says only *"the block says
   what binding means"*. Innermost-wins would make awaiting inside a `try` inside an `async fn`
   unspellable with `<-`; the alternative is to refuse `<-` where two readings are live. Not in
   the corpus today, reachable in one line, and **open and owed**.
4. A `try` block written with `<-` and the same block written with `?` emit **byte-identical**
   Rust, asserted by a test comparing the two.

   §11.3 found this is the **easiest** criterion in the row rather than the sharpest, and §3's
   framing of it — *"the strongest available statement that `<-` is sugar"* — should be read that
   way: neither lowering has to change. `?` is emitted at exactly one place with no try-block
   awareness — `emit/expr.rs:111`, `format!("{}?", receiver(inner, indent))` — `emit/try_block.rs`
   only wraps in `(|| { … })()`, and both candidate paths funnel through the same `receiver()`,
   which unparens first, so the parenthesising coincides and so does the awkward case
   (`a = (x ?? y)?` against `a <- x ?? y`). `emit_await` gains a branch.

   **Two residual asymmetries are disclosed rather than discovered**, per the same finding:
   `Await.pattern` is an `Option<Pattern>` with no `is_mut` and no `ty`, so `mut a = (f x)?` and
   `a : T = (f x)?` have **no `<-` spelling**.
5. `?` continues to work inside and outside `try`, and mixing `?` with `<-` in one block is
   legal, asserted.
6. Doc 30 §7 decision 6's effect-only `<- f ()` still works and still differs from `_ <- f`.
7. Every example green, no rustc output, stdout unchanged.

   **No count, deliberately.** This criterion said *"All 29 examples green"*, and the corpus has been 29, then 30, then 31 during this milestone alone — `examples/local.sdx` arrived with plan 55 leaf 1. A number here is a criterion that a future example breaks. The invariant is what matters and it is already enforced: `examples.rs` runs every file and excepts exactly the ones its `NOT_PARSEABLE` constant names, today `["lexical.sdx"]`, the error-path example. `docs/STATUS.md` is generated and is where the count lives.
8. `construct_coverage` green with `is` indexed and the `=` pattern form removed — a concrete
   cost §5 left unstated, which §11.6 is the finding for: `construct_coverage.rs` hard-codes the
   token in two witnesses, the `if let` row looking for `["if", "«id»", "«id»", "="]` and `let
   chains` for `["if", "«any»", "«any»", "=", "«any»", "&&"]`.
9. Doc 30 §3.1, §3.2, §4.2 and §7 decision 3, doc 18 §3 and doc 03 §5.5 amended, each citing
   this row and preserving the original reasoning. §11.7 confirms all six targets exist, and
   *preserving the reasoning* means carrying doc 30 §7 decision 3's second half — *"`is` would
   have reversed the operand order and cost a keyword"* — which §1.1 reaches only through an
   ellipsis. Doc 03 §5.5 confirms scrutinee-first (`if msg is Move(x, y)`), so the operand order
   this row restores is the one §5.5 always specified.

   **Doc 03 §5.5's amendment is two-part**, per §11.5: the operand order, and its *second
   lowering* — *"no binders → `matches!`"*, so `if status is Active` becomes
   `if matches!(status, Active)`. This row keeps the `if let` lowering throughout, so that half
   has to be amended too and not only the operand order. Leaf 6 carries both.

---

## 6. Out of scope, and what it discloses

**The comprehension generator keeps `in`, and doc 30 §4.2's loss is not reversed.** This is the
decision a reader will most expect to go the other way, since the token that displaced `<-` from
comprehensions is now free.

It is declined because a Redox comprehension lowers to an **iterator chain**, not a monadic
bind — doc 18 §3.2's `filter_map` lowering is the evidence. Spelling an iterator generator with
the bind arrow would be a borrowed surface whose semantics did not come with it, which is the
exact defect this milestone exists to remove. Using `<-` there would be a category error that
happens to look faithful.

**`do` is reserved and unused.** Plan 55 §1.7 reserved it, and with `<-` now a bind, a future row
wanting a general monadic block has both the keyword and the arrow available. This row does not
take it, because a general bind needs a trait to bind over and Redox has no type classes of its
own — `class` is Rust's `trait`, and a `Monad` would be a library.

**No new block modifiers.** `try` and `async` are the two plan 55 delivered. `<-`'s meaning being
block-directed makes more of them cheap, and none is added here.

**The arm-guard form is the fragile one.** `y | y is Some (Some v) ->` puts a guard bar and an
`is` in one arm. Plan 22 §1.5 verified `if let` in a match guard is stable Rust; this row changes
only the surface, and criterion 1 asserts the nested case because plan 46 found an or-pattern
collision in exactly this neighbourhood.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout unchanged;
the `<-`-versus-`?` byte-identity test; `tests/ui/` fixtures for the retired `=` form and the
unmodified-block `<-`; `construct_coverage` green.

---

## 8. Emerald reuse

None. Emerald had no bind, no `await` surface and no pattern test in condition position; doc 30
exists because the token economy is Redox-specific and had to be reasoned about from scratch.

The transferable discipline is doc 30's own method, which this row follows: **write down what a
token means in every position before assigning it**, because doc 30 §4.2 is the record of a
token being spent and the bill arriving one document later. This row spends `is` and claims `<-`,
and §6 is where the bills are listed before they arrive.

---

## 9. Decision log

1. **`<-` becomes bind, read by the block modifier.** The narrower option — keep `<-` as await
   and only move pattern tests to `is` — relieves `=` and leaves `<-` meaning one special case
   of binding. Plan 55 made the general reading available and it costs nothing more.
2. **`is`, with scrutinee first.** §1.4. Doc 03 §5.5 specified it; doc 18 §3 retired it for
   §3.1's reading, and doc 30 §7 decision 3 kept `=` to avoid a keyword and an operand-order
   change. This row pays both of those costs deliberately — one keyword, and an operand order it
   judges an improvement.
3. **`<-` outside a block modifier is a diagnostic, not a default.** Defaulting it to `?` would
   make the token mean something in a position where no block declared an interpretation, which
   is how `<-` came to mean only await in the first place.
4. **`?` keeps working.** §1.3. One spelling would be tidier and would make `?` illegal in
   exactly the function bodies where it is most natural.
5. **Comprehensions keep `in`.** §6. The token is free and the semantics are not monadic.
6. **Assert byte-identity between `<-` and `?`.** §3. A sugar that emits different code is not a
   sugar, and this is the only criterion that can tell the difference.

---

## 10. Result

*Not yet executed.*

---

## 11. Corrections from orientation

Written after reading the code this row targets. The mechanism is sound. One
criterion **rejects the existing corpus**, one claim about the lexer understates
a breaking change, and the criterion §2 calls sharpest is the easiest in the
row.

### 11.1 Criterion 3 would reject five of the corpus's six `<-` uses

*"`<-` in an `async` block awaits; in a `try` block it is `?`; outside both it
is a diagnostic."* The third clause is the problem, and it is not a wording
problem.

**`async` is a *signature* marker, not a block.** `is_async` lives on
`Signature` — a separate item from the equation body, and absent entirely when
a function has no signature line. In `examples/async.sdx`, **five of six** `<-`
uses sit in `async fn` bodies with **no `async` block at all** (lines 53, 54,
65, 72, 73, 87); only line 85 is inside one. So criterion 3 as written makes
`examples/async.sdx` fail, which criterion 7 then also fails.

It must read *"inside an `async` **function** or an `async` block"*, and §1's
table — which shows only the block form — misframes it. Measured for free:
`x <- g n` in a plain non-async function emits `let x = g(n).await;` today with
no Redox diagnostic, rustc catching it.

**And the nesting rule is unstated.** A `try` block inside an `async fn` has
both readings available. The row says *"the block says what binding means"* and
never says innermost-wins, nor that awaiting inside a `try` inside an `async fn`
becomes unspellable with `<-`. Not in the corpus today; reachable in one line.

### 11.2 This row **re-reserves** `is`, and a test exists to stop it

§2 says *"`is` is not a token and not reserved. Nothing uses it; the lexer arm
is additive."* True in letter, misleading in substance: `is` was **actively
removed**. `keyword.rs`'s header records doc 18 §7 retiring eleven words —
*"`is`, `and`, `use`, `with`, `defer`, `cond`, `into`, `derive`, `mod`,
`unless`, `switch`"* — and `leaves_ordinary_identifiers_alone` iterates that
list asserting `Keyword::from_text` is `None`, under the reason *"a retired
keyword that stayed reserved would reject a program that is now legal, and
nothing else in the suite would notice"*. `is = 5` compiles today.

So this is a re-reservation and a **breaking change**, and the plan should say
so and cite the precedent it has: `With` is in the enum today despite being on
the same list, restored by doc 30 §3.3.

Two artifacts go stale on landing: that test's array, and
`emit/emit_stmts.rs`'s doc comment, which still says doc 18 §3.1 *"could delete
`use`, `is`, `defer` and the comprehension's `for` in favour of one token"*.

### 11.3 Criterion 4 is the easiest criterion in the row, not the sharpest

Byte-identity between a `try` block written with `<-` and one written with `?`
needs **neither lowering to change**. `?` is emitted at exactly one place with
no try-block awareness — `emit/expr.rs:111`, `format!("{}?", receiver(inner,
indent))` — and `emit/try_block.rs` only wraps in `(|| { … })()`. Both candidate
paths funnel through the same `receiver()`, which unparens first, so the
parenthesising coincides and so does the awkward case (`a = (x ?? y)?` against
`a <- x ?? y`). `emit_await` gains a branch.

Two residual asymmetries to disclose rather than discover: `Await.pattern` is an
`Option<Pattern>` with no `is_mut` and no `ty`, so `mut a = (f x)?` and
`a : T = (f x)?` have **no `<-` spelling**.

### 11.4 `Expr::Draw` is not a `<-` site

It reads `Kw(With)` — doc 30 §3.3's `with conn = pool.with_connection` — and
`emit_stmts`'s callback flattening is `with`'s lowering, with nothing to do with
`<-`. The name is historical residue. The real `<-` surface is three files:
`operator_table.rs`'s `("<-", DrawArrow)`, `try_await.rs` producing
`Expr::Await`, and `emit_await.rs`.

**Both names re-verified 2026-11-10, after plan 55 leaf 1 deleted
`StmtKind`.** This section originally wrote `StmtKind::Draw` and
`StmtKind::Await`; both variants moved to `Expr` and are at
`crates/redox-ast/src/expr.rs` lines 470 and 503. The substantive claim
below survived the move unchanged, which is the half worth re-checking:
`Parser` still has exactly three fields.

**There is no parser context flag anywhere.** `Parser` has three fields —
`toks`, `pos`, `errors` — and `Expr::TryBlock`/`Expr::AsyncBlock` are ordinary
prefix expressions the parser never knows it is inside. The *emitter* does have
the nesting (`try_block` and `emit_do` both recurse into `emit_stmts`), so one
new parameter threads it; and `check_callback_tails` is the precedent for the
post-parse alternative.

### 11.5 Counts, positions, and two things that argue *for* the row

- **8 corpus sites, not 13.** `examples/draw_condition.sdx` lines 18, 33, 35,
  47, 57, 70, 71, 77. §2's 13 is a `grep -c " = "` artifact — 8 tests plus 5
  equation definitions — and contradicts leaf 6's own "8 sites". §2 also says
  "five shapes" four paragraphs after saying six.
- **Seven positions, not six**, and the `=` token is consumed at only **two**
  sites (`condition_operand.rs:22`, `control_flow.rs:150`) with one scanner,
  `holds_a_pattern_test`. `condition()` has four callers, which is where the
  surface positions come from.
- **`until` is a rejected eighth position.** `control_flow.rs:50-62` raises
  `RDX0107` when a negated condition has a pattern. Leaf 2 must carry `is` into
  that detection *and* its message — `code.rs` still documents it as
  `until Some v <- x`, already stale since doc 30 moved the token to `=`.
- **An equation-row guard cannot hold a pattern test at all**, because
  `Guard.condition` is an `Option<Expr>` rather than a `Condition`:
  `| Some v = o = v` is `RDX0101`. So `is` would make that position
  **spellable for the first time** — an argument *for* this row that it does
  not make, and an AST change it does not price.
- **Doc 03 §5.5 specifies a second lowering this row never mentions.** *"No
  binders → `matches!`"*: `if status is Active` becomes
  `if matches!(status, Active)`. This row keeps the `if let` lowering
  throughout, so §5.5's `matches!` half has to be amended too, not just its
  operand order.

#### Decided 2026-11-10: `<-` is await and nothing else; the eighth position waits

**The nesting rule: neither of §11.1's options.** Both were framed as a choice
about which enclosing construct wins, and the better answer is that **`<-` does
not depend on an enclosing construct at all**. It means await. A try-bind is
spelled with `?`, which already exists.

That is decided on an asymmetry §11.1 does not use: **`?` is already a spelling
for the try operation, and await has none** — doc 30 deleted postfix `.await` to
make room for `<-`. So innermost-wins (option a) makes *await* unspellable
inside a `try` inside an `async fn`, while refusing on ambiguity (option b)
rejects a line whose intent is obvious. Reading `<-` as await costs nothing,
because the operation it displaces has a second spelling and the operation it
keeps does not.

The stronger reason is that it leaves `<-` with **one** meaning. A token whose
meaning depends on which block encloses it is the hazard class plan 52 removed
from `::` and plan 65 removed from a closer's column: a context-dependent
reading that compiles either way and surprises a reader once. Having just
removed it twice this milestone, this row should not add it back.

**The eighth position stays out**, which is §11.5's option (b). Making
`Guard.condition` an `Option<Condition>` would make an equation-row pattern test
spellable **for the first time** — which is a new feature, not the completion of
this row's surface. This row already carries a keyword restoration and a
corpus-wide breaking change; a third item whose cost is an AST type change
belongs in a row where that cost is the headline rather than a footnote. §6
should disclose that the position exists and is not reached.

### 11.6 Unpriced costs

- **Two diagnostic codes** — criterion 2's retired `=`, and criterion 3's
  unmodified-context `<-`. §5 names none. The syntactic band is at `RDX0111`
  with plan 55 taking `RDX0112`.
- **`construct_coverage.rs` hard-codes the token** in two witnesses: the
  `if let` row looks for `["if", "«id»", "«id»", "="]` and `let chains` for
  `["if", "«any»", "«any»", "=", "«any»", "&&"]`. Criterion 8's cost is
  concrete and was unstated.
- **`docs/GRAMMAR.md`'s `let chains` row is already wrong**, citing *"the draw
  arrow in a condition (doc 30)"* — doc 30 is what stopped `<-` being the
  condition token. There is also **no `let`-`else` row at all**, though `guard`
  lowers to one.
- `tests/ui/` is at `crates/redox-cli/tests/ui/`.

### 11.7 Citations

All six of criterion 9's targets exist. One is quoted short: §1.1 ellipses doc
30 §7 decision 3 and drops the half that matters — *"`is` would have reversed
the operand order and cost a keyword"* — then attributes the remainder to §3.1.
Doc 03 §5.5 confirms scrutinee-first (`if msg is Move(x, y)`), so the operand
order this row restores is the one §5.5 always specified.
