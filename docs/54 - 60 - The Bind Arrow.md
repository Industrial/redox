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

**Corpus scale.** 29 examples; `draw_condition.sdx` is the file this row rewrites most, with 13
pattern-test sites across its five shapes.

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

`is` as a token, and the pattern-test parse with scrutinee-first operands. Covers all **six**
shapes §2 enumerates: condition, `else if`, let-chain, `while`, `guard`, arm guard — plus the
condition-expression form, whose `then` plan 55 removes, so the two rows must agree on what that
line becomes.

### Leaf 2 — `leaf-retire-equals-match`

`=` removed from the pattern positions, with a diagnostic naming `is`. Every pre-existing program
uses the old form, so the error message is the migration guide — plan 52's and plan 59's choice,
applied a third time.

### Leaf 3 — `leaf-bind-in-async`

`<-` in an `async` block is await. This is today's lowering, re-sited onto the block-modifier
rule rather than onto the token.

### Leaf 4 — `leaf-bind-in-try`

`<-` in a `try` block is `?`. Shares the existing `?` lowering, which is what makes §3's
byte-identical assertion possible — two spellings, one emission path.

### Leaf 5 — `leaf-bind-outside-block`

The §3 case 3 diagnostic, with a `tests/ui/` fixture.

### Leaf 6 — `leaf-corpus-and-docs`

`draw_condition.sdx`'s 8 sites, plus `async.sdx`'s `if Poll::Ready v = …`, `destructuring.sdx`
and `semver`-shaped code. Doc 30 is the document most affected: **§3.1 and §7 decision 3** are
what this row reverses and must be amended with their reasoning preserved, §1.1's rejection of
doc 18 §3.1 is what §1.2 now cites, §3.2 and §4.2 describe a token economy this row changes, and
§4.2's recorded loss is **not** reversed (§6). Doc 18 §3 and doc 03 §5.5 amended;
`docs/GRAMMAR.md` regraded.

---

## 5. Acceptance criteria

1. `is` works in all six pattern-test positions, each exercised by a running example.
2. `=` in a pattern position is a named diagnostic naming `is`, with a `tests/ui/` fixture.
3. `<-` in an `async` block awaits; in a `try` block it is `?`; outside both it is a diagnostic.
4. A `try` block written with `<-` and the same block written with `?` emit **byte-identical**
   Rust, asserted by a test comparing the two.
5. `?` continues to work inside and outside `try`, and mixing `?` with `<-` in one block is
   legal, asserted.
6. Doc 30 §7 decision 6's effect-only `<- f ()` still works and still differs from `_ <- f`.
7. All 29 examples green, no rustc output, stdout unchanged.
8. `construct_coverage` green with `is` indexed and the `=` pattern form removed.
9. Doc 30 §3.1, §3.2, §4.2 and §7 decision 3, doc 18 §3 and doc 03 §5.5 amended, each citing
   this row and preserving the original reasoning.

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
