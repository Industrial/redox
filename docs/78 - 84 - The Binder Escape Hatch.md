# Plan 84 — The Binder Escape Hatch

Plan ID: `binder-escape-hatch`
Depends on: 62 (`ascription-completion`), 67 (`marked-receiver`)
Milestone: 11 — *expressive delta*
Status: `done` — leaves 0, 1a, 1b, 1–6 shipped 2026-11-11; §10 is the record

---

## 1. What this plan is

Doc 03 §14.1's argument, applied one level up. A **type** is carried verbatim, so every present
and future Rust type form is reachable for free. A **binder** — the generic parameter list, the
bound clause, the `where` predicates — is *synthesised* from the spine, and synthesis has a
ceiling that verbatim carriage does not.

```
join : I -> &str -> String
  given <T: Display, I: IntoIterator<Item = T>>
  where I::IntoIter: Clone
```

Both added lines are verbatim Rust text. Nothing is inferred; strictly less is inferred than
today.

### 1.1 The question this row answers, and the half that is already answered

The question asked was whether a notation can carry the **full Rust type grammar**. For types
the answer is already yes, and it has been since doc 03 §14.1: *"Type expressions are copied
through verbatim […] Every present and future Rust type form is reachable for free, and nothing
can drift."* `docs/GRAMMAR.md` §1.3 grades nine of its eleven type rows `PASSTHROUGH` on exactly
that ground — paths, generic arguments, references, raw pointers, tuples, arrays, slices,
`dyn Trait`, `impl Trait`, function pointers, `Fn` bounds and lifetimes.

So the gap is not the type grammar. It is the **binder**, which doc 18 §2.2 synthesises instead
of carrying:

> **Lowercase type names are type variables**, as in Haskell; the generic list is the order of
> their first appearance, and the bound clause is read off the `=>` constraint.

That is three inferences — *which names are variables*, *what order they are declared in*, and
*what their bounds are* — over a grammar that has more shapes than three inferences can reach.
This row does not remove them. It adds a way to decline them.

### 1.2 Measured: six gaps, two silent defects, two limitations

**Leaf 0 ran, 2026-11-11, against `redox` built from `main`.** Nine probe programs, each
transpiled and then put to rustc. The eight hypotheses this section opened with were drafted from
reading `type_vars` and `emit::generics`; what the probes found is **narrower in count and wider
in consequence**, because four of the eight turn out to share one cause, and the probes turned up
two defects that were not hypothesised at all.

**Four causes, not eight gaps:**

| Cause | What it is | Hypotheses it explains |
|---|---|---|
| **A** | The `=>` clause accepts **only `Trait var`** — a bare type identifier and a bare lowercase word | 3, 4, 7, 8, **and the unhypothesised case below** |
| **B** | `type_vars` scans lowercase words in verbatim type text, excluding only primitives, `self`, `'static`, `'_` and the four keywords `dyn`/`impl`/`mut`/`as` | 1, 2 |
| **C** | The generic list is lifetimes-then-first-appearance | 6 |
| **D** | No spelling exists | 5 |

**Cause A's largest case was not in the table, and it is the headline finding.** A bound whose
trait carries **type arguments** does not parse as a bound:

```
shout : Into<String> t => t -> String
->  fn shout<T>(x: Into<String> T FatArrow T) -> String       -- rustc: E0782
```

`T: Into<String>`, `U: From<T>`, `I: IntoIterator<Item = T>`, `F: Fn(i32) -> i32`,
`T: AsRef<str>` — **the majority of bounds in real Rust** — are all that shape and none of them
can be written. `Ord t` works and `Ord t, Clone t` works; anything with a `<` in the trait does
not. `docs/GRAMMAR.md`'s *Generic parameters and bounds* row read as unrestricted before this
measurement and now names the restriction.

**The six graded `UNREACHABLE`**, each with a fixture that still fails and rustc's answer:

| Shape | Fixture | rustc |
|---|---|---|
| Bound whose trait has type arguments | `generic_trait_bound.sdx` | `E0782` |
| Projection predicate, `where I::IntoIter: Clone` | `projection_bound.sdx` | `E0782` |
| Relaxed bound, `?Sized` | `relaxed_bound.sdx` | `E0425` |
| Lifetime bound, `'a: 'b` | `lifetime_bound.sdx` | `E0425` |
| Const parameter, `[u8; n]` | `const_generic.sdx` | `E0423` |
| Higher-ranked bound, `for<'a>` | `higher_ranked_bound.sdx` | `E0109`, `E0618` |

**Two hypotheses are struck as `UNREACHABLE` rows**, and the reason is the grading rule rather
than the measurement. `docs/GRAMMAR.md` requires an `UNREACHABLE` fixture to **fail**, and these
two produce programs that compile:

- **Hypothesis 6, declared order.** `pick : t -> u -> t` emits `fn pick<T, U>` and compiles. The
  gap is that `<U, T>` cannot be *asked for*, which is a limitation and not an unreachable
  construct. Recorded as a note.
- **Hypothesis 5, parameter defaults.** No spelling exists, so there is no program to fail.
  Recorded as a note.

### 1.3 Two silent defects, neither hypothesised

These are the measurement's real yield, and both are milestone 9's worst class — *"no construct
reads as one thing and emits another"*.

**A failed bound clause emits a `TokenKind`'s Debug name into generated Rust, with no
diagnostic.** When the clause does not match `Trait var`, the parser does not refuse it: the whole
run including `=>` is swallowed as the first parameter's verbatim type, and tokens with no source
spelling in that path render by name. Measured in four of the six fixtures:

```
Into<String> t =>   ->   x: Into<String> T FatArrow T
Clone i::IntoIter =>->   xs: Clone I::IntoIter FatArrow I
?Sized t =>         ->   x: QuestionSized T FatArrow &T
'b 'a =>            ->   a: 'b 'a FatArrow &'a str
```

`FatArrow` is `TokenKind::FatArrow`'s own name, from `operator_table.sdx`. It reaches the `.xrs`,
which doc 02 §10.1 calls a human-facing artifact. Leaf 1a is the diagnostic that should have been
there, and it is sequenced **before** the escape hatch because it is a correctness fix that stands
whether or not `given` ships.

**A bound naming a variable absent from the spine is silently dropped.** Measured:

```
hold : Default t => u -> u
->  fn hold<U>(x: U) -> U       -- the bound is gone, and this compiles clean
```

No `where T: Default`, no diagnostic, no error. The author writes a constraint and it vanishes.
This also means the **function path and the `instance` path disagree**: `emit/generics.sdx`'s own
doc comment says of a constraint naming an unused variable that it *"is emitted anyway, as a bound
on nothing — rustc reports it, and that is the right place for it: Redox has no name resolution
and must not grow one"*. That is the right rule, and the signature path does not follow it. Leaf
1b aligns them, which also closes hypothesis 7 — a bound-only parameter is reachable the moment
the bound survives, because the emitted `where T: Default` makes rustc ask for `T`'s declaration,
and `given` supplies it.

### 1.4 It may close `blanket_impl`, which is a listed gap

`docs/STATUS.md` carries nine gaps. One of them is **blanket impls**, and
`docs/GRAMMAR.md` §1.2 states its cause in binder terms:

> `instance Display t => Describe t` parses and emits `impl Describe for t`, with a lowercase
> target rustc rejects: `TypeApp::vars` reads the *arguments* and a bare variable is the head.

That is a synthesis failure, not a type failure — the head is a variable where the synthesiser
expects a name. A verbatim binder on an `instance` head writes `impl<T: Display> Describe for T`
directly and the question does not arise. Leaf 4 tests it; criterion 7 asserts it; §6 is honest
that it may not be the whole of that gap.

### 1.5 Why a second spelling is a hatch and not a duplicate

`docs/DECLINED.md` declines the turbofish with a principle this row has to answer:
**"One way to pin a type is better than two."**

The distinction is what the second way reaches. The turbofish was declined because an ascription
already reaches everywhere it would have — *"the declared type is sufficient wherever it is
written down"* — so it was a second mechanism for an already-reachable thing. `given` is not:
every shape in §1.2 is one the first mechanism cannot express at all.

That makes its relationship to synthesis the same one **inline Rust** has to the surface, which
doc 01 §23.1 admitted on exactly this reasoning. A hatch that reaches further is not a duplicate
of the thing it reaches past.

### 1.6 Synthesis is all or nothing per signature

If `given` is present, the generic list is **not** synthesised and the `=>` clause is **not**
read for that signature. Lowercase spine atoms are then verbatim Rust types like any other, and
a signature's binder has one author.

The alternative — synthesise what `given` omits — was rejected in §9. A half-synthesised binder
is a thing a reader has to simulate the compiler to predict, and it would make the four causes of
§1.2 four interaction questions rather than one escape.

---

## 2. Preconditions

**Measured 2026-11-11 by leaf 0**, against `redox` built from `main`. Probe sources are not kept
— the six that fail became `tests/unreachable/` fixtures, which is the durable form.

```bash
cargo run -q -p redox-cli -- build probe.sdx     # the emitted Rust
cargo run -q -p redox-cli -- check probe.sdx     # rustc's answer
```

| # | Precondition | Result |
|---|---|---|
| 1 | Each §1.2 shape, graded | **Done.** Six `UNREACHABLE` with fixtures, two struck as compiling, two silent defects found. §1.2 and §1.3 |
| 2 | Where lowercase scanning descends | **Confirmed, and it descends further than hypothesised.** `type_vars` reads any lowercase word in the type's text, including an array's length position (`[u8; n]` → `<N>`) and including words that are Rust keywords in a type (`for`, `fn` → `<FOR, FN>`). Its exclusion list is primitives, `self`, `'static`, `'_`, `dyn`, `impl`, `mut`, `as` and path segments |
| 3 | Whether a signature-position `where` exists | **It does not.** Doc 03 §14.2 declares the role and doc 18 §2.2 took the job with `=>`; nothing parses `where` before the body. So leaf 3 is an implementation, not a wiring change |
| 4 | That `given` collides with nothing | **Free.** `operator_table.sdx` holds no such spelling and it is not in the lexer's keyword set |
| 5 | The formatter exemption | **Not yet measured.** Plan 61 §11.6's signature-line passthrough is assumed by leaf 5 and asserted by criterion 6; it is the one precondition leaf 0 did not reach, and leaf 5 is sized on the assumption that it holds |

Two results outside the list, both recorded in §1.3 and both of which change the leaf set:
the `FatArrow` leak, and the silently-dropped bound. Neither was hypothesised, and both are
defects rather than gaps — they are reached by programs the author believes are correct.

---

## 3. Concrete proof this plan targets

**`join` from §1 compiles, runs, and prints the joined string** — a signature no synthesis can
reach, written and working.

Five cases, each failing differently:

1. **A const parameter.** `pad : [u8; N] -> [u8; N]` with `given <const N: usize>`.
2. **A projection predicate.** §1's `where I::IntoIter: Clone`, which has no `=>` form.
3. **A bound-only parameter.** `<T, U: From<T>>` taking only `U` — the case where first-appearance
   ordering has nothing to appear in.
4. **A blanket impl**, per §1.4, asserted by `tests/unreachable/blanket_impl.sdx` **starting to
   compile** and the gate therefore breaking. That breakage is the proof, per `docs/GRAMMAR.md`'s
   rule that a fixture fails when the compiler gets better.
5. **A signature with `given` and a lowercase spine atom**, asserting §1.6: the atom is a
   verbatim type and *not* a synthesised variable.
6. **A malformed bound clause**, `Into<String> t =>`, refused by a named Redox diagnostic rather
   than swallowed — §1.3, leaf 1a. A `tests/ui/` fixture, asserted also **not** to emit the text
   `FatArrow`.
7. **A bound naming a variable absent from the spine**, emitted rather than dropped, so rustc
   reports the undeclared parameter — §1.3, leaf 1b. The fixture captures rustc's answer, because
   the point is that there now *is* one.

And one that must **not** change: every signature in `examples/generic.sdx` emits byte-identical
Rust. Synthesis is the common case and does not pay for this row.

---

## 4. Leaves

### Leaf 0 — `leaf-binder-gap-inventory` — **done 2026-11-11**

Measured §1.2. Six fixtures under `tests/unreachable/`, six `UNREACHABLE` rows in
`docs/GRAMMAR.md`, the *Generic parameters and bounds* row amended to name the `Trait var`
restriction it did not previously disclose, and `docs/STATUS.md` regenerated — nine gaps to
fifteen.

It did its job in both directions. It **shrank** the plan by collapsing four hypotheses into one
cause, and it **grew** it by two leaves: §1.3's defects are reached by programs an author believes
are correct, which is a worse failure than a gap and is why 1a and 1b come first.

### Leaf 1a — `leaf-malformed-bound-diagnostic`

A bound clause that does not match `Trait var` is a **named Redox diagnostic**, not a verbatim
type. §1.3. This stands whether or not the rest of the row ships, and it is the leaf that stops a
`TokenKind` Debug name reaching a human-facing artifact.

The diagnostic must name the restriction rather than the token: an author writing
`Into<String> t =>` has made a reasonable attempt at a real Rust bound, and *"a bound's trait
cannot take type arguments"* is actionable where *"expected a type"* is not.

### Leaf 1b — `leaf-bound-survives-the-spine`

A bound naming a variable absent from the spine is **emitted anyway**, as `emit/generics.sdx`
already does for an `instance`, so rustc reports the undeclared parameter instead of the bound
vanishing. §1.3. Closes hypothesis 7 once `given` can declare the parameter.

### Leaf 1 — `leaf-given-clause`

Parse `given` followed by verbatim text to the end of its layout run, and store it on the
signature. The text is not inspected — the same contract a type atom has.

### Leaf 2 — `leaf-synthesis-off`

When `given` is present, suppress generic-list synthesis and `=>` reading for that signature.
§1.6 is the rule; the test is case 5 of §3.

### Leaf 3 — `leaf-verbatim-where`

The signature-position `where` carries verbatim predicates and emits as Rust's `where`. Sized by
precondition 3. Doc 03 §14.2's two-role table is the specification and this leaf is the half
that may not exist yet.

### Leaf 4 — `leaf-class-and-instance-heads`

The same binder on a `class` and an `instance` head. §1.4 is the motivation and case 4 of §3 is
the test. If it closes `blanket_impl`, `docs/GRAMMAR.md`, `docs/STATUS.md` and the fixture all
move in this leaf.

### Leaf 5 — `leaf-formatter`

`redox fmt` round-trips a `given` line. Per precondition 5 this is expected to be an assertion
rather than an implementation, and plan 61 §11.6 is why — but plan 56 §11.3's rule stands: *a
pass that rewrites `Program` has two consumers and will surprise one of them*, and an untested
exemption is an assumption.

### Leaf 6 — `leaf-corpus-and-docs`

`examples/binder.sdx` exercising each §3 case. Doc 03 §14.1 gains the one-level-up statement;
doc 18 §2.2 is **amended** to say synthesis is the default rather than the only mechanism;
`docs/GRAMMAR.md` regrades the binder rows; `docs/DECLINED.md`'s *"Bracket type arguments"* row
gains a cross-reference, because its reason — *"requires translating every type expression"* — is
the argument this row reuses rather than contradicts.

---

## 5. Acceptance criteria

1. Each §1.2 row is either struck as already working, or graded in `docs/GRAMMAR.md` with a
   fixture, before leaf 1 begins.
2. §1's `join` transpiles, compiles with no rustc output, runs, and its stdout is asserted.
3. A const parameter, a projection predicate and a bound-only parameter each have a running
   example and a golden.
4. `given` suppresses synthesis entirely for its signature, asserted by a lowercase spine atom
   emitting as a type and not as a parameter.
5. Every existing signature emits byte-identical Rust — `tests/golden/` re-runs unchanged, with
   **no re-blessing**. A golden that needs blessing in this row is a defect, not a diff.
6. `redox fmt` round-trips `given` and the signature-position `where`.
7. `tests/unreachable/blanket_impl.sdx` either compiles — breaking the gate deliberately, with
   `docs/GRAMMAR.md` and `docs/STATUS.md` updated in the same commit — or is recorded in §6 as
   having a second cause, with that cause named.
8. All examples green, no rustc output, stdout asserted; `construct_coverage` green with no new
   `ALLOW` entries.
9. Doc 18 §2.2's synthesis sentence is amended and the amendment cites this row.
10. **No generated `.xrs` in any corpus contains a `TokenKind` name.** Asserted by a scan for
    `FatArrow` and `QuestionSized` across `tests/golden/` and every example's output, not only by
    leaf 1a's fixture — §1.3 found two such names and a scan is what finds the third.
11. A bound survives to the emitted Rust whether or not the spine mentions its variable, asserted
    in a golden. Leaf 1b.

---

## 6. Out of scope, and what it discloses

**Two ways to write a binder, permanently.** §1.5 argues it is a hatch; it is still a second
spelling, and a reader will meet both. The mitigation is that `given` is unreachable by accident
— it is a keyword, not a shape you fall into.

**`given` is a new keyword.** The inventory is doc 03 §4.5 and this row spends one. `forall` was
the alternative and §9 records why it lost.

**It does not close `unsafe trait` or `unsafe impl`.** Those are the other `class`/`instance`
gaps in `docs/STATUS.md` and their cause is a missing modifier, not a missing binder.

**Synthesis is not fixed, only escapable.** Causes C and D of §1.2 remain surprising in a signature
that does *not* use `given`, and nothing here warns about them. A diagnostic would need to know
what order the author wanted, which it cannot.

**Two of the eight hypotheses are limitations rather than gaps, and stay unfixtured.** Declared
parameter order (cause C) and parameter defaults (cause D) both produce programs that *compile*,
so `docs/GRAMMAR.md`'s rule — an `UNREACHABLE` fixture must fail — excludes them. `given` reaches
both, and neither can be asserted by the fixture mechanism; §3 cases 3 and 5 carry them instead,
which is weaker, and this is the honest place to say so.

**Verbatim text cannot be formatted.** `redox fmt` will not normalise spacing inside a `given`
clause, for the same reason it does not inside a type. That is doc 03 §14.1's cost, inherited.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/golden/` unchanged per criterion 5; `grammar_inventory` green; `construct_coverage` green;
the `blanket_impl` fixture resolved either way per criterion 7.

---

## 8. Emerald reuse

**Negative, and it is the most useful kind here.** Emerald's bracket type arguments (`Array[T]`)
are declined in `docs/DECLINED.md` under *scope*, with the reason *"requires translating every
type expression, where doc 03 §14.1 carries a type verbatim"*.

That decline is this row's argument. Emerald translated its type grammar and paid for it in drift
against the host language; Redox carries it and pays nothing. The only correction this row makes
is that §14.1's reasoning was applied to the type and **not** to the binder, which is why the
binder is where the gaps are. Taking the same decision twice is the whole of the design.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | Carry the binder verbatim | A Haskell-shaped type language with a total mapping to Rust's | Redox does not reason about types; a bijective type language buys **no capability** and costs a permanent tax, since every Rust type-system addition becomes a surface-grammar change. Doc 03 §14.1 already settled this one level down |
| 2 | `given` | `forall` | `forall` reads as a quantifier and would then carry `<const N: usize>`, which is not a quantification. `given` reads as *"given these parameters"* and makes no claim about what is inside |
| 3 | `given` | A leading `<…>` run on the signature, with no keyword | `<` already opens a qualified path (`<T as Trait>::Item`) in type position, so a leading `<` is genuinely ambiguous and would need lookahead |
| 4 | All-or-nothing per signature (§1.6) | Synthesise what `given` omits | A half-synthesised binder turns §1.2's four causes into four interaction questions and cannot be predicted without simulating the compiler |
| 5 | Signature-position `where` for predicates | Fold predicates into `given` | Doc 03 §14.2 already assigns that position this role, and a predicate list is where rustc puts it too, so the emitted Rust reads as hand-written |
| 6 | Measure first (leaf 0) | Design from §1.2 as written | Plan 46 §1.3: eleven of sixteen apparent gaps were the probe's own bug, and a false `UNREACHABLE` is worse than a missing row |

---

## 10. Result

Shipped 2026-11-11. `moon run :format :lint :test` green; the full gate run four
times across the row, each time after the generated documents were regenerated
**last** — §10.7 is why that ordering is written down.

### 10.1 What shipped

`given <…>` after the `:` on a signature line, a signature-position `where` for
the predicates, and the same `given` on an `instance` head. All three carry
**verbatim** text, which is doc 03 §14.1's contract for a type applied one level
up to the binder.

Measured against the six shapes leaf 0 graded:

| Shape | Spelling | Emitted |
|---|---|---|
| Const parameter | `given <const N: usize> [u8; N] -> u8` | `fn first_of<const N: usize>(xs: [u8; N]) -> u8` |
| Trait with type arguments | `given <T: Into<String>> T -> String` | `fn shout<T: Into<String>>(x: T) -> String` |
| Projection predicate | `… I -> u64 where I::IntoIter: Clone` | `where I::IntoIter: Clone,` |
| Relaxed bound | `given <T: ?Sized> &T -> usize` | `fn width<T: ?Sized>(…)` |
| Lifetime bound | `given <'a, 'b: 'a> …` | `fn longer<'a, 'b: 'a>(…)` |
| Higher-ranked bound | `… where for<'a> F: Fn(&'a str) -> &'a str` | the same, verbatim |

Plus the bound-only parameter and the chosen order, which cause C and cause D
blocked together: `given <U: From<T>, T> T -> U` emits `fn widen<U: From<T>, T>`.
`examples/binder.sdx` is all of it, compiled, run, and its stdout asserted.

**`docs/STATUS.md`'s gap count went 9 → 8.** The six rows leaf 0 graded
`UNREACHABLE` are now `SUGARED` with their fixtures deleted, and **`blanket_impl`
closed** — leaf 4, `instance given <T: Display> Describe T` emitting
`impl<T: Display> Describe for T` with rustc clean. The row added no gaps of its
own.

### 10.2 The two silent defects were the row's real yield

§1.3 found them and neither was hypothesised. They are worth more than the
feature, because a gap is a program an author cannot write and a silent defect is
a program they believe they *have* written.

**`RDX0120`** now refuses a bound clause that is not `Trait var`, where the
parser used to rewind silently and let the clause become the first parameter's
verbatim type. And a bound naming a variable the spine does not mention is
**emitted** rather than dropped, so rustc reports the undeclared parameter.

The second of those is `generalise`'s own doc comment finally being true. It had
said since it was written that such a constraint *"is kept anyway, because rustc
reports it and Redox has no name resolution to do better"*, and the body filtered
it out — and additionally returned before assigning `bounds` at all when the
spine had no variables. AGENTS.md's *"a doc comment is not an oracle for its own
code"* in its other direction: here the comment was right and the code was wrong,
twice over.

### 10.3 The leaked names had one root cause, and it was not the bound clause

`FatArrow` and `QuestionSized` looked like two defects in the clause. They were
one defect in **`token_text`**, whose case ladder ended in
`other -> format! "#{other:?}"` — a `Debug` name for any token with no arm, which
is identifier-shaped and compiles in some positions.

So the fix was not in `constraints` at all. `token_text` now spells **every**
operator token, and `respells_every_operator_the_lexer_produces` is the standing
proof. That test asks the **lexer** rather than a copy of `OPERATOR_TABLE`: it
lexes a source of bare operator spellings and compares each respelling against
its own span, so a missing arm fails for free and no second table exists to
drift. Fifty-odd tokens checked, and the `?` that this row needed is one of them.

`no_generated_rust_contains_a_token_kind_name` in `examples.rs` is the wider
backstop, over every golden. Both were needed: the scan found the symptom, the
ladder was the cause.

### 10.4 The formatter dropped the clause from an `instance` and not from a
signature, exactly as plan 56 §11.3 predicted

Plan 56 §11.3's rule is *a pass that rewrites `Program` has two consumers and
will surprise one of them*, with one structural exemption plan 61 §11.6 records:
signature lines, which `redox-fmt` passes through from the source.

Both halves held, and the split is the whole lesson. A signature's `given`
round-tripped from the first run because the line is passed through verbatim. An
`instance` head is **re-printed from the tree** — `target_text`'s own doc comment
records that distinction — so `redox fmt` turned
`instance given <T: Display> Describe T` into `instance Describe T`, a different
program, caught by `every_example_is_already_canonical`.

**And the first fix was wrong in an instructive way**: printing the stored text
produced `instance <T: Display> Describe T`, which is valid *Rust* and not a
Redox program. The stored form is the Rust one, `<…>` with no keyword, because
that is what `emit::instance` pastes; the printer has to put `given` back. One
field, two spellings, and the only place in the row where they differ.

### 10.5 The fuzzer found the cost of carrying text verbatim

`redox-fuzz`'s `the_generated_rust_always_parses` truncated
`examples/binder.sdx` mid-predicate — `where for<'a> F: Fn(&'a str) ->` — the
parser accepted it, and the emitter wrote that arrow into the `.xrs`. Verbatim
carriage means a *truncated* clause is carried truncated.

The fix had to be syntactic or it would have been a type checker. `can_end_a_type`
is the same predicate `capture_type` uses to find a type's last token, so
requiring a predicate to end where a type can end inspects the clause no more
deeply than `given` inspects its angle brackets. Every real predicate ends in a
trait name or a closing bracket.

Worth recording that the examples sweep, the goldens, the UI corpus and rustc
were all green when the fuzzer failed. It is the only gate that generates the
input.

### 10.6 Two false greens, both self-inflicted, both caught by the next gate

Written down because each one *read* as evidence.

**The tree-sitter parse that parsed nothing.** `tree-sitter parse
examples/binder.sdx`, run from inside `tree-sitter-redox/`, resolves a path that
does not exist: the command failed, a `grep -c ERROR` over its output counted
zero, and it was reported here as a clean parse. The real parse had `ERROR` nodes
at the `:`, the `=` and the `?` inside the binder. AGENTS.md says to run the
input; it does not say the run cannot lie about *which* input it ran, and a
grep for the absence of a word cannot tell silence from success.

**The keyword set that grew as a side effect.** `_angle_group`'s first draft
listed `"dyn"` and `"impl"` among its punctuation, which made both *grammar
keywords* — and `every_grammar_keyword_is_coloured` then demanded them of all
three highlighters. Neither needed to be there; both reach the group as
`$.identifier`. A keyword set that grows out of a punctuation list is precisely
the drift that test exists to catch.

### 10.7 Generated documents must be regenerated last, and that is not obvious

`docs/STATUS.md` and `docs/CONFORMANCE.md` are generated from the working tree,
so every source edit drifts them — including edits to *test* files, since
`STATUS.md` counts test lines. The gate failed on `status_is_current` three
separate times in this row for no reason except ordering.

The rule is one line and belongs in anyone's muscle memory: **source, then
goldens, then the generated documents, then the gate.** `CONFORMANCE.md` moved
once here, 532 → 533 files, for the one new parser module.

### 10.8 What this row exposed and did not fix

**`vec!(…)` is not rustfmt-clean.** `examples/binder.sdx`'s first draft used
`vec! \-> 1 \-> 2 \-> 3`, which emits `vec!(1, 2, 3)` where rustfmt writes
`vec![1, 2, 3]` — so `generated_rust_is_rustfmt_clean` failed. **No example in
the corpus used `vec!` before this row**, which is why a defect in the most
common macro in Rust had never been seen. The example uses an array instead and
the defect is owed to a row that owns macro emission; it is not plan 84's.

**`class` heads take no binder.** Leaf 4 shipped `instance` only, because that is
what closed a listed gap. A `class`'s generic list is doc 18 §5.3's own
synthesis from the head (`class Scale t` → `trait Scale<T>`) and nothing measured
says it falls short.

**`unsafe impl` and `unsafe trait` are still gaps**, and §6 said they would be:
their cause is a missing modifier, not a missing binder.

**The tree-sitter grammar cannot know a `given` clause's shape in a signature.**
`signature_type` is a flat `repeat1` and `given` is one more literal in it, which
is enough to parse and not enough to distinguish the binder from the spine. The
`instance` head needed the delimited `given_binder` precisely because a target
follows it. `test/corpus/binder.txt` pins both.

### 10.9 Corrections to this document

- §1.2's eight hypotheses are **four causes**, and the largest case under cause A
  was not among the eight: a bound whose trait carries type arguments. §1.2 is
  rewritten and the strike is recorded there.
- Hypotheses 5 and 6 are **not** `UNREACHABLE` rows, because their programs
  compile. §6 records them as limitations instead, which is weaker than a fixture
  and is said so.
- §2 was *"Preconditions to verify"* and is now measured. Precondition 5, the
  formatter exemption, was the one leaf 0 did not reach — and §10.4 is what that
  cost.
- Leaves **1a** and **1b** did not exist when this document was written. They
  come from §1.3 and they are sequenced first, because a defect an author can
  reach beats a gap they cannot.
