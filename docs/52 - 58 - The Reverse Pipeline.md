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
is what the corpus does with all 23 of its pipeline steps. It is a wide hazard the moment anyone
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

**Corpus scale.** 23 `|>` steps across 29 examples, and **every one is a method call** —
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

### Leaf 2 — `leaf-reverse-application`

`|>` becomes reverse application. The lowering is `f (x)` where it was `x.f()`, and plan 57's
`.name` handles everything that used to rely on the receiver reading.

### Leaf 3 — `leaf-composition`

`>|` and `|<`, both producing a function. Shares plan 56's closure-generation path rather than
emitting its own shape — one lowering for everything that produces a closure, which is plan 56
§9 decision 5 applied again.

### Leaf 4 — `leaf-corpus-migration`

23 pipeline steps across 29 examples. Mechanically this is plan 57's migration already done —
every corpus step is a method and plan 57 marked them — so this leaf is mostly verification that
nothing was missed, plus the new examples from §3.

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
5. Composition works inside an adaptor — §3 case 3 — which exercises plan 56.
6. `<<` and `>>` are unchanged, and `expressions.sdx`'s shift assertion still passes.
7. The §1.1 hazard is unreachable: a function and a same-named method are both callable and
   distinguishable, asserted in one example.
8. All 29 examples green, no rustc output, stdout unchanged from before the migration.
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
