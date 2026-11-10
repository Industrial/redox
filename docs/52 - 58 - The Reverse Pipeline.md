# Plan 58 — The Reverse Pipeline

Plan ID: `reverse-pipeline`
Depends on: 56 (`currying`), 57 (`methods-as-functions`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

`|>` becomes reverse application. `x |> f` is `f x`, for any `f`.

Today it is receiver-sugar: `x |> f` emits `x.f()`. That is not what the token means anywhere
else it exists, and the difference is not cosmetic — it means **you cannot pipe into your own
functions**.

```
double n = n * 2
print (5 |> double)        ->  println!("{}", 5.double());
                           ->  rustc E0599: no method named `double` found for `{integer}`
```

And composition arrives with it: `>|` forward, `|<` backward, which doc 18 §4 declined on the
ground that *"without currying there is nothing to compose"* — a ground plan 56 removed.

### 1.1 The current behaviour has a silent failure mode, not just a wrong one

The rustc error above is loud. The dangerous case is quiet: if **any trait in scope** happens to
have a method of that name, `x |> myfn` compiles and calls the method instead of the function.
Nothing reports it. Adding an import can change what an existing pipeline means.

That is a narrow hazard today because `|>` is nearly always applied to iterator methods, which
is what the corpus does with **all 23** of its live pipeline operators — §11.6 measured them and
not one is a free function. (§11.5 corrects the count: 28 occurrences, 26 in code, 23 that parse.
Two of the 24 this paragraph originally claimed are prose.) It is a wide hazard the moment anyone
writes a free function and reaches for the operator that every other language would let them use.

### 1.2 Plan 57 is what makes this affordable

Reverse application alone would break the corpus's central idiom, because `iter`, `map` and `sum`
are methods and `xs |> map (+ 1) |> sum` would stop resolving.

Plan 57 already solved that by making a method a function with its receiver last. So the idiom
survives with one character added per step, and it now means something exact:

```
xs |> .map (+ 1) |> .sum          -- methods, marked
raw |> normalise |> parse         -- functions, unmarked
raw |> .trim |> parse_version     -- and they mix
```

Each `|>` is reverse application in all three lines. There is one rule.

### 1.3 Composition, and the token collision

`>>` and `<<` are Rust's shift operators and `examples/expressions.sdx` contains
`print (1 << 1 + 1)`, which asserts shift precedence. Taking them for composition would mean
moving shifts to methods — a migration paid by every program doing bit work, for a gain to
programs doing none.

So composition takes **`>|` forward and `|<` backward**, which keeps the shifts untouched and
ties the pair visually to `|>`, the operator they are the point-free sibling of:

```
normalise = .trim >| .to_lowercase
xs |> map (.trim >| .to_lowercase)
normalise = .to_lowercase |< .trim      -- Haskell's direction
```

One family of pipe-shaped operators rather than two unrelated pairs. `.` is not considered —
doc 18 §4 calls it *"the one Haskell token that cannot be had at any price"*, and plan 57 has
just finished making `.` mean field access and nothing else.

### 1.4 What `$` is still for

`$` already exists, works correctly, and overlaps with `|>` considerably — `f $ x` and `x |> f`
are the same call. Both stay, because they read in opposite directions and the direction is the
point:

```
print $ twice $ show 7            -- right to left, innermost last
raw |> normalise |> render        -- left to right, in pipeline order
```

The overlap is disclosed in §6 rather than resolved. Removing either would be a separate
argument about one spelling too many, and this row is not the place to make it.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**`|>` is receiver-sugar**, and a free function produces a method call:

```
5 |> double     ->  5.double()     ->  rustc E0599
```

**Multi-line pipelines parse**, with the receiver on the `=` line and continuations indented
beneath it; the alternative layout is `RDX0102`. Plan 51 documents the restriction and this row
inherits it unchanged.

**`$` works** — low precedence, right-associative, respecting method chains.

**Shifts are exercised.** `examples/expressions.sdx` prints `1 << 1 + 1` and
`expressions.stdout` asserts `4`, so the precedence is a tested fact and §1.3's refusal to move it
is protecting a real assertion.

**Corpus scale, measured.** **24** `|>` operators across **5** examples — not 23 across 29 —
plus two `(|> name)` method-reference sections, which are a different construct and are counted
separately. Comments stripped before counting, because several examples discuss the operator in
prose.

The distribution matters more than the total, and no earlier count reported it:

| File | Operators |
|---|---:|
| `examples/section.sdx` | 15 |
| `examples/application.sdx` | 3 |
| `examples/generic.sdx` | 3 |
| `examples/lexical.sdx` | 2 |
| `examples/comprehension.sdx` | 1 |

**Five files, and 15 of the 24 sites are in the one example that is about sections.** So the
migration is far narrower than "the corpus" suggests — and its weight sits in the file whose
whole subject is the construct this row changes the meaning of, which is where a wrong lowering
is most likely to read as correct. `examples/lexical.sdx` is the exempt non-parsing example, so
2 of the 24 are never transpiled at all.

**Every one is a method call** —
`map`×6, `sum`×5, `into_iter`×3, `filter`×3, `count`×2, `len`, `unwrap`, `max`, `name`. Not all
are *iterator* methods (`unwrap` is `Option`'s, `len` is `str`'s), but none is a free function,
so plan 57's `.name` marking covers the whole existing corpus and no pipeline step changes
meaning.

**Not yet verified, and leaf 1 must:** that `>|` and `|<` lex without colliding with the guard
`|` in a `case` arm — `n | n > 65 ->` emits `n if n > 65 =>`, so the construct is real — or with
doc 18 §4's comparison sections (`(> 0)`, `(0 <)`; `examples/section.sdx` spells the flipped one
`(1 <)`). No `|`-prefixed operator exists, so the expectation is that they do not collide — but
doc 18 §5's *"four collisions resolved"* and plan 46's or-patterns finding are two cases where a
token looked free and was not, so this is a precondition to check rather than assume.

---

## 3. Concrete proof this plan targets

**`5 |> double` prints `10`.** The case that is `E0599` today.

Six more:

1. **A function and a method in one pipeline.** `raw |> .trim |> parse_version |> render`.
2. **A pipeline spanning five lines**, which is the shape plan 51's `examples/application.sdx`
   exists to show and the reason the operator is worth having.
3. **Forward composition in a `map`.** `xs |> map (.trim >| .to_lowercase)` — composition inside
   an adaptor, which needs plan 56's currying to produce a function at all.
4. **Backward composition**, asserted to produce the same result as the forward form with the
   operands swapped. Two operators, one tested relationship.
5. **`1 << 2` still a shift**, and `expressions.stdout` still asserting `4` for
   `1 << 1 + 1`.
6. **The §1.1 silent case made impossible.** A free function named `len` alongside a receiver
   with a `len` method: `xs |> len` now calls the function, unambiguously, and `xs |> .len` calls
   the method. Both in one example, because the whole point is that they are distinguishable.

---

## 4. Leaves

### Leaf 1 — `leaf-tokens`

`>|` and `|<` as tokens, with the §2 collision checks discharged before anything depends on
them. If either collides, the fallback pair is recorded in §9 decision 3 rather than invented at
the gate.

**The collision checks are discharged, in §11.1, before this leaf runs.** Neither spelling is
in the operator table, and the sharper question — whether maximal munch can steal a valid
program — is answered by a fact this plan did not state: Redox spells a closure `\n -> …`, not
`|n|`, so `|<` has no closure parameter list to eat. §9 decision 3's fallback pair is therefore
not needed, and this leaf is the two token arms and nothing else.

### Leaf 2 — `leaf-reverse-application`

`|>` becomes reverse application. The lowering is `f (x)` where it was `x.f()`, and plan 57's
`.name` handles everything that used to rely on the receiver reading.

The blast radius is smaller than it looks: three files. `emit/expr.rs`'s `BinaryOp::Pipeline`
arm, `emit/pipeline.rs` (whose whole lowering is two match arms), and
`emit/wraps_as_receiver.rs`'s `Pipeline` carve-out.

**`emit/pipeline.rs` is edited by plan 57 as well**, which neither row said. Plan 57's criterion
2 needs `x |> .name` to work and that file is where it lands, so by the time this leaf runs the
file has already changed shape once. The two edits do not conflict — plan 57's orientation
established that both meanings of `|>` emit identical Rust for a `.name args` right operand,
which is what makes 57-before-58 safe — but the second editor should expect the first's work
rather than the file this plan measured. Precedence needs no change —
`BinaryOp::Pipeline` is already power 1, the loosest binary rung, which is correct for reverse
application. `section.rs`'s `(|> name)` method reference becomes a different construct and is
plan 57's.

### Leaf 2a — `leaf-wire-up-the-chain-breaker`

**The emitter already implements rustfmt's `chain_width` breaking and the pipeline does not use
it.** `emit/chain.rs` exists; its only callers are `emit_comprehension` and `nest`.
`emit/pipeline.rs` builds a flat string by recursion and never consults it, which is why plan 51
found `generated_rust_is_rustfmt_clean` failing on a five-stage pipeline — the longest chain in
the corpus before that was 21 characters, so nothing had tripped it.

This row makes long chains the normal case, so it is this row that must wire `pipeline` to
`chain` and re-bless every golden whose chain exceeds 60 characters. `examples/application.sdx`
carries the finding and is deliberately under the limit until this leaf lands.

### Leaf 3 — `leaf-composition`

`>|` and `|<`, both producing a function. Shares plan 56's closure-generation path rather than
emitting its own shape — one lowering for everything that produces a closure, which is plan 56
§9 decision 5 applied again.

### Leaf 4 — `leaf-corpus-migration`

**23 live pipeline operators across 4 examples, 16 of them in `examples/section.sdx`** —
corrected by §11.5, which measured **28** occurrences in 5 files, 2 of them inside comments and
3 in `examples/lexical.sdx`, the error-path example that deliberately does not parse. The
figures this leaf was written with, *"24 across 5 examples, 15 of them in
`examples/section.sdx`"*, are within one and two of the live count, which is near enough to look
verified and is not.

Mechanically this is plan 57's migration already done — §11.6 confirmed the claim behind that
by measuring what the corpus actually pipes into: all 23 are collection or iterator methods
(`map` 6, `sum` 5, `into_iter` 3, `filter` 3, `count` 3, one each of `unwrap`, `max`, `len`) and
**not one is a free function**. So this leaf is mostly verification that nothing was missed, plus
the new examples from §3 — and §1.1's hazard has stayed theoretical for exactly the same reason.

### Leaf 5 — `leaf-docs`

Doc 18 §4's *"Why `|>` stays and `.` does not"* paragraph is rewritten: `|>` stays for a new
reason, and the composition decline is reversed with plan 56 cited. Doc 03 §5.3's receiver
semantics section is struck. `docs/GRAMMAR.md` regraded; `docs/STATUS.md` updated.

---

## 5. Acceptance criteria

1. `x |> f` emits `f(x)` for a Redox function, a curried partial application, and a
   `.name` method.
2. `5 |> double` runs and prints the right number.
3. A five-line pipeline mixing functions and methods runs, with asserted stdout.
4. `>|` and `|<` each produce a function; the swapped-operand equivalence from §3 case 4 is
   asserted.
4a. A pipeline wider than rustfmt's `chain_width` is broken across lines by the emitter, and
   `generated_rust_is_rustfmt_clean` passes on a five-stage chain —
   `examples/application.sdx`'s fifth stage is restored as the witness.
5. Composition works inside an adaptor — §3 case 3 — which exercises plan 56.
6. `<<` and `>>` are unchanged, and `expressions.sdx`'s shift assertion still passes.
7. The §1.1 hazard is unreachable: a function and a same-named method are both callable and
   distinguishable, asserted in one example.

   **§11.7 confirmed this is assertable, which was not obvious** — it reads like it needs name
   resolution Redox must never have. It does not: after leaf 2, `x |> f` is `f(x)` and
   `x |> .f` is `x.f()`, so the two differ by one character **in the source** and rustc resolves
   each independently. The criterion is a parsing and lowering assertion, which is the only form
   AGENTS.md's *"every check rustc can make, rustc makes"* leaves available to it.
8. Every example green, no rustc output, stdout unchanged from before the migration.

   **No count, deliberately.** This criterion said *"All 29 examples green"*, and the corpus has been 29, then 30, then 31 during this milestone alone — `examples/local.sdx` arrived with plan 55 leaf 1. A number here is a criterion that a future example breaks. The invariant is what matters and it is already enforced: `examples.rs` runs every file and excepts exactly the ones its `NOT_PARSEABLE` constant names, today `["lexical.sdx"]`, the error-path example. `docs/STATUS.md` is generated and is where the count lives.
   `examples/lexical.sdx` is excluded from "green" as always, and §11.5 is the reason it also
   holds 3 of the corpus's 28 `|>` occurrences without being migratable.
9. `construct_coverage` green with `>|` and `|<` indexed.
10. Doc 18 §4's composition decline is amended, citing plan 56.

---

## 6. Out of scope, and what it discloses

**`$` and `|>` overlap and both stay.** §1.4. Three ways to apply a function — juxtaposition,
`$`, `|>` — is one more than strictly necessary, and the third is justified by reading direction
rather than by capability. A future row may reasonably argue for removing `$`; this row does not,
because the argument is about style and this milestone has been about meaning.

**Composition needs currying and therefore inherits its limits.** Plan 56 §1.4 bounds partial
application to signatures visible in the file and its loaded modules, so `f >| g` across a module
boundary without a visible signature is plan 56's diagnostic, not a composition failure. Stated
because the error will name arity and the reader will be looking at `>|`.

**Shifts keep the good tokens and composition gets the second-best pair.** `>|` is less
immediately legible than `>>`. That is the price of not migrating every program doing bit work,
and §1.3 is the argument.

**No composition for methods without plan 57's marking.** `.trim >| .to_lowercase` works because
plan 57 made those functions. `trim >| to_lowercase` is two unresolved names and a plan 56
diagnostic.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout **unchanged
from before the migration**; goldens re-blessed and reviewed; `construct_coverage` green.

---

## 8. Emerald reuse

Nothing. Emerald had no pipeline operator, so the reading-direction argument in §1.4 has no prior
evidence behind it and is a design judgement stated as one. (Doc 02 §4.3 is about `nil` →
`Option[T]`, not Emerald's call syntax; the nearest relevant section is §4.6 on parenless-call
ambiguity, which concerns application rather than piping.)

The transferable item is the one plan 46 and 47 both landed on: **count the same thing a second
way**. This row's §3 case 6 is that applied to a hazard — rather than asserting the silent case
is gone, it builds the collision deliberately and asserts both spellings resolve differently.

---

## 9. Decision log

1. **Reverse application, not receiver-sugar with a diagnostic.** Keeping the receiver reading
   and merely refusing non-methods would preserve a token that means something unique to Redox,
   and would still not let anyone pipe into their own function.
2. **`>|` and `|<`, not `>>`/`<<`.** §1.3. The shifts are exercised by a corpus assertion and
   composition is more common than bit work in the code Redox is for, but not so much more that
   it is worth a migration priced in every bit-twiddling program.
3. **If `>|` collides, the fallback is `>>>`/`<<<`.** Recorded now so leaf 1's collision check
   has an answer rather than becoming a design discussion at implementation time.
4. **Keep `$`.** §1.4, §6. The overlap is real and the reading direction is a real difference.
5. **One closure-generation path shared with plan 56.** Two mechanisms producing closures would
   drift, and plan 44's IR-equality harness would need two pairs where one will do.
6. **Depend on plan 57 rather than ship together.** They could be one row. Separating them means
   the corpus migrates once for method syntax and once for pipeline semantics, each with its own
   unchanged-stdout assertion — and if the second goes wrong, the first is already green.

---

## 10. Result

*Not yet executed.*

---

## 11. Corrections from orientation

Probed against the tree at commit `59706fe`, before any leaf ran. Three of
this plan's claims are exactly right, two of its numbers are not, and the
collision check §2 defers to leaf 1 can be discharged here instead.

### 11.1 Leaf 1's tokens are free, and the closure spelling is why

`>|` and `|<` appear nowhere in the lexer's operator table, which holds
`>`, `>=`, `>>`, `|`, `|>` and `||`. So neither is taken.

The question worth asking is not whether the spelling is taken but whether
**maximal munch can steal a valid program**, and the answer turns on something
this plan does not mention: **Redox spells a closure `\n -> …`, not `|n|`.**
`examples/application.sdx` lines 58–59 are `filter (\n -> *n % 2 == 0)` and
`map (\n -> n * 2)`.

That is what makes `|<` safe. In a Rust-shaped surface, `|<` would sit directly
on top of a closure parameter list — `f(|x| < y)` is lexed `|`, `x`, `|`, `<`,
and a greedy `|<` would eat the closing bar. Redox has no such sequence to
break.

`|` does already carry two meanings, and a third is being added:

| Spelling | Meaning | Where |
|---|---|---|
| `\|` | `BinaryOp::BitOr` | `binary_power.rs`, `TokenKind::Or` |
| `\|` at line start | the `data` variant separator | `examples/data.sdx` line 22, `\| Close` |
| `\|>` | `BinaryOp::Pipeline` | `binary_power.rs`, `TokenKind::Pipe` |

Note the naming trap for whoever implements this: **`TokenKind::Pipe` is `|>`,
and bare `|` is `TokenKind::Or`.** Reading the table quickly gets this backwards.

Maximal munch is safe for both new tokens because neither `<` nor `>` can begin
an expression or a variant name, so no existing program contains `| <` or
`> |` as adjacent tokens with the second one starting something.

### 11.2 `BinaryOp::Pipeline` is power 1, as claimed

Confirmed at `crates/redox-ast/src/expr/binary_op.rs` line 122. Leaf 2's
*"precedence needs no change"* holds.

### 11.3 A fourth site leaf 2 does not name, and it correctly stays

Leaf 2 says the blast radius is three files. There is a fourth place that
mentions `Pipeline`, and it is worth naming precisely because a careful
implementer will find it and wonder: `BinaryOp::rust_text` returns `None` for
`Coalesce`, `Catch` and `Pipeline` (`binary_op.rs` line 74), on the stated
ground that those *"need"* a lowering rather than an operator.

That stays true after this row. Reverse application still has no Rust operator
— `f(x)` is a call, not an infix form — so the `None` is as correct for
reverse application as it was for receiver sugar. **It is a site to read and
leave alone**, which is a different instruction from a site not to look at.

### 11.4 Leaf 2a's callers are right

`emit/chain.rs`'s only callers are `nest.rs` line 43 and `comprehension.rs`
line 78. `optional_chain.rs` line 71 is a *definition* that looks like a call
in a grep, and `expr.rs` line 150 calls `optional_chain` rather than `chain`.
The claim survives the check that would most easily have broken it.

### 11.5 The corpus count is 28, not 24 — and the migratable count is 23

Leaf 4 says *"24 pipeline operators across 5 examples, 15 of them in
`examples/section.sdx`"*. Measured:

| File | `\|>` | In a comment | Code |
|---|---:|---:|---:|
| `examples/section.sdx` | 17 | 1 | 16 |
| `examples/application.sdx` | 4 | 1 | 3 |
| `examples/generic.sdx` | 3 | 0 | 3 |
| `examples/lexical.sdx` | 3 | 0 | 3 |
| `examples/comprehension.sdx` | 1 | 0 | 1 |
| **total** | **28** | **2** | **26** |

So **28** occurrences, 26 of them code, and **23 that a migration can touch** —
because `examples/lexical.sdx` is the error-path example that deliberately does
not parse, and its three are there to be lexed and rejected.

The plan's 24-and-15 is within one and two of the migratable figures, which is
near enough to look verified and is not. The number that matters to leaf 4 is
**23 in four files**, and the one that matters to whoever reads §1.1's *"all 24
of its pipeline operators"* is that two of them are prose.

### 11.6 What the live corpus actually pipes into

All 23 are collection or iterator methods — `map` 6, `sum` 5, `into_iter` 3,
`filter` 3, `count` 3, and one each of `unwrap`, `max` and `len`. **Not one is
a free function**, which is both why §1.1's hazard has stayed theoretical and
why leaf 4 is as cheap as it claims: every live step is exactly the kind plan
57 marks with a leading dot.

The remaining two occurrences are the `(\|> name)` method-reference section,
in `examples/lexical.sdx` line 41 and in a comment in `examples/section.sdx`
line 35. Leaf 2 already assigns that construct to plan 57.

### 11.7 Criterion 7 is assertable, which was not obvious

*"The §1.1 hazard is unreachable: a function and a same-named method are both
callable and distinguishable"* reads like it needs a check Redox must never
have. It does not. After leaf 2, `x \|> f` is `f(x)` and `x \|> .f` is
`x.f()`, so the two are distinguished by one character **in the source**, and
rustc resolves each independently. The criterion is a parsing and lowering
assertion, not a name-resolution one — which is the only form it could take
here, since AGENTS.md forbids the analysis that any other form would need.

### 11.8 The method-reference section has an owner now, and it is not this row

Leaf 2 says the `(\|> name)` method reference *"becomes a different construct
and is plan 57's"*, while plan 57's §11.3 read its own leaf 7 as regrading only
a doc table. Each row pointed at the other.

**Settled 2026-11-10 in plan 57 §1.3: plan 57 retires the construct**, because
plan 57 builds the replacement and `Section::Method`'s emitted text is
byte-identical to what a bare `.name` will lower to. The rule applied is plan
65's: whoever builds the replacement retires what it replaces, so no window
exists in which neither spelling works.

What that means here is that **this row inherits nothing**. By the time `\|>`
becomes reverse application there is no `(\|> name)` left to reinterpret —
which matters, because under reverse application `(\|> f)` would mean
`\|x\| f x`, and that is just `f`: a section that sections nothing.
