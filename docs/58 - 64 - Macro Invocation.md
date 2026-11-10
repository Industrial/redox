# Plan 64 — Macro Invocation

Plan ID: `macro-invocation`
Depends on: 45 (`reachability-residue`), 55 (`expression-orientation`), 59 (`value-bindings`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

Macros become callable. The spelling takes a callback and no brackets: `name! \-> e`.

And then the keyword family that existed only because macros were unreachable is **deleted** —
`assert`, `panic`, `todo`, `unimplemented`, `unreachable`, `print` — because Rust's own are
better and now reachable.

```
println! \-> "#{a} #{b}"
assert!  \-> add 1 2 == 3
bail!    \-> "no route for #{id}"
write!   \-> f \-> "#{n} rows"

r = catch_unwind! \->
  a = f x
  a + 1
```

This closes the largest reachability gap in the language. Plan 45 §1.5 found it and explicitly
left it for a row that would take the spelling decision; plan 48 then **priced** it.

### 1.1 Plan 48 measured the cost, and it is 99% of one construct

Plan 48 ported the compiler's own source to Redox and counted what blocked it. Two passages,
quoted exactly rather than merged:

> **One blocker is 99% of the problem.** 855 lines carry a macro invocation against 12 with a
> cast.

> **The answer to doc 02 §4.8's criterion is no, and 99% of the no is one construct.** 855 of 867
> blocking lines are macro invocations.

855 lines. Macro invocation is not one gap among seven — it is the gap, and every other
`UNREACHABLE` row in `docs/GRAMMAR.md` is rounding error beside it. `effect!`, `caps!`,
`hlist!`, `vec!`, `json!`, `bail!`, `info!`, `query!`, `html!` and every proc-macro are
unreachable in a language whose premise is *"Redox has crates.io"*.

### 1.2 Why `!` is free, and what plan 45 found about it

Doc 18 §7 retired `!` as the zero-argument-call suffix, leaving it meaning boolean negation. Plan
45 §1.5 measured the consequence and found something worse than unreachability:

> `assert! true` appears to work and is a misparse — it reads as the `assert` keyword applied to
> `!true` and emits `assert!(!true)`.

A silent inversion. That is the strongest possible argument for taking `!` back for macros, and
it also explains why the keyword family must go in the same row: as long as `assert` is a
keyword, `assert!` is ambiguous between the keyword-plus-negation reading and the macro reading.

Verified independently: the bare `ident!` zero-argument call from doc 01 §6.3 (`user.save!`,
`config.load!`, `Vec.new!`) is **also unimplemented** — `s.len!` is `RDX0101`. So nothing holds
`!` except the negation reading, and plan 59 removes the need for a zero-argument-call suffix —
at the cost §59 §6 records, that a value and a nullary function become indistinguishable at the
use site.

### 1.3 The callback shape, and what it costs

No brackets, no braces, no parentheses. `!` marks the name, `\->` opens the argument, layout
closes it:

```
print! \-> "#{a}"               -- one argument
write! \-> f \-> "#{n}"          -- two, one arrow each

r = catch_unwind! \->            -- multi-line, layout-delimited
  a = f x
  a + 1
```

Three things make this work better than it first appears.

**Redox's own interpolation absorbs the multi-argument format family.** `bail! \-> "no route for
#{id}"` lowers to `bail!("no route for {}", id)` — Redox splices the arguments itself, because
`#{}` already does. So the commonest macros take exactly one argument.

**`[1, 2, 3] : Vec<i32>` already replaces `vec!`** (doc 18 §7, plan 29), so the bracketed-literal
family needs nothing.

**`assert!` recovers the expression text.** A function cannot: `assert(1 == 2)` can only print
*"assertion failed"*, where `assert!(1 == 2)` prints the source. Verified by building both, in a
scratch crate rather than in this tree — so unlike this row's other measurements it is not
reproducible from the repository, and §6 says so.

The cost is that `\->` on a macro that does not defer — `println!` consumes its arguments
immediately — says something slightly untrue about evaluation. Accepted in §9 decision 2: the
uniformity and the bracket-freedom are worth one reading inaccuracy, and the alternative shapes
each had a worse one.

### 1.4 Token-stream DSLs need the escape hatch, and plan 45 already shipped it

`html! { <div/> }`, `quote!`, `sqlx::query!` consume **tokens that are not Redox expressions**. No
value-passing shape can carry those, because the tokens *are* the input.

Plan 45 leaf 1 shipped `rust """…"""` — inline Rust, verbatim — and that is the answer:

```
markup = rust
  html! { <div class="x">{ name }</div> }
```

Brace-free in Redox; whatever braces the embedded Rust needs are the embedded Rust's business.
This row adds the layout-delimited block form beside plan 45's string form and nothing else —
the mechanism exists and is tested.

### 1.5 The family deletes itself — and only one of it is a keyword

Six constructs exist because macros did not. With macros reachable they are redundant, and
`assert` in particular is actively harmful (§1.2).

**"Keyword family" is the wrong noun for five of the six.** `keyword.rs` reserves `assert` (and
`otherwise`, `test`, `rust`) and **not** `print`, `panic`, `todo`, `unimplemented` or
`unreachable` — a test named `leaves_ordinary_identifiers_alone` asserts exactly that, with the
comment *"The words doc 18 §7 retires are ordinary identifiers again."* So five are emitter
special cases on an ordinary call, and only `assert` is a reserved word. That makes the deletion
cheaper than it sounds for five of them and the lexer change real for one.

| Deleted | Replacement |
|---|---|
| `assert cond`, `assert a == b` | `assert! \-> cond`, `assert_eq! \-> a \-> b` |
| `panic "m"` | `panic! \-> "m"` |
| `todo`, `unimplemented`, `unreachable` | `todo!`, `unimplemented!`, `unreachable!` |
| `print x` | `println! \-> "#{x}"` |

**`otherwise` is not in the family and stays a keyword.** It is a guard-ladder catch-all with no
macro equivalent, so deleting it would mean writing `| true =`. The option this row was chosen
from proposed making it a prelude `const` — that is withdrawn, because a `const` needs a home and
AGENTS.md's rule is *"no plan implements a library function"*. A keyword costs one lexer arm and
no library, which is the right trade for a word that appears in every guard ladder.

**`print` gains a destination it never had.** It is *not* single-argument — `print a b` emits
`println!("{} {}", a, b)`, measured — so the gain is not arity. It is stderr: `eprint "x"` emits
`eprint("x")`, a call to a function no program defines, which rustc rejects. There is no way to
write to stderr at all, so a diagnostic-emitting program cannot exist. That is a real capability,
not a tidy-up.

`test "description"` is **not** deleted. It declares a `#[test]` item and cannot be a macro call,
and plan 55 §1.5 already made it the milestone's one carve-out from expression orientation.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**Every macro spelling is rejected.** `vec![1, 2, 3]`, `vec! (1)`, `format! "hi"`,
`(vec![1, 2, 3])`, `vec![1, 2] : Vec<i32>`, and `println! "{} {}" a b` — all `RDX0101`, while
`std::vec::Vec::new ()` is accepted. Paths work; macros do not. Plan 45 §1.5's measurement,
re-confirmed.

**`assert! true` misparses to `assert!(!true)`.** Plan 45 §1.5.

**The bare `ident!` call form is unimplemented.** `s.len!` is `RDX0101: expected ')' to close the
group`. So `!` carries only negation.

**`#[track_caller]` puts a failure at the caller's line**, and a function returning `!` flows into
any position — both verified by building the function-based alternative. That alternative was
weighed and declined in §9 decision 1; the measurement is why the decline is informed.

**`rust """…"""` works.** Plan 45 leaf 1 shipped it, and plan 45 §5 criterion 6 asserts the old
broken behaviour is gone.

**Scale.** Plan 48: 855 of 867 blocking lines. Corpus: `print` appears in **28** of the 29
examples — `examples/lexical.sdx` is the exception, being a pure literal-and-trivia fixture that
deliberately does not parse.

**`\->` already works.** `g = \-> 7` emits `let g = || 7;` today. So the nullary lambda is **not**
a prerequisite this row must build, which its first draft assumed; criterion 1 becomes a
regression assertion rather than a deliverable.

---

## 3. Concrete proof this plan targets

**`assert! \-> add 1 2 == 3` fails with the expression text in the message**, which is the thing
no function-based `assert` can do and the thing `assert! true` currently inverts.

Six more:

1. **A one-argument macro with interpolation.** `bail! \-> "no route for #{id}"` →
   `bail!("no route for {}", id)`, asserted in a golden so the splice is visible.
2. **A two-argument macro.** `write! \-> f \-> "#{n} rows"`.
3. **A multi-line callback.** The `catch_unwind!` shape from §1.3, where layout does the
   delimiting.
4. **A proc-macro from crates.io.** `id_effect`'s `effect!` — the case the whole row exists for,
   and the one that proves token passthrough works for a macro Redox knows nothing about.
5. **A token-stream DSL.** §1.4's `rust` block form.
6. **`println!` and `eprintln!`**, the latter being a capability the language does not currently
   have.

And one negative: **every deleted construct is a diagnostic naming its replacement.** 28
examples use `print`, so the error message is the migration guide — which plans 52, 57, 59 and 60
each also chose, making this the fifth and establishing it as the milestone's standing answer to
a retired spelling.

---

## 4. Leaves

### Leaf 1 — `leaf-macro-token`

`ident!` as a macro head, with `!` disambiguated from negation by position: `!` immediately
following an identifier with no space is a macro marker; elsewhere it is negation. The spacing
rule is the one thing plan 52 removed from `::`, so §6 records the inconsistency.

### Leaf 2 — `leaf-callback-argument`

`\-> e` as the argument form, one arrow per argument, layout-delimited for the multi-line case.
The nullary lambda itself already works (§2), so this leaf is the *argument position*, not the
lambda.

### Leaf 3 — `leaf-interpolation-splice`

`"… #{e} …"` inside a macro callback lowers to a format string plus arguments, reusing plan 23's
existing interpolation lowering rather than a second one.

### Leaf 4 — `leaf-rust-block`

The layout-delimited `rust` block of §1.4, beside plan 45's string form.

### Leaf 5 — `leaf-retire-the-family`

Six constructs removed; six diagnostics added, each naming its replacement. One is a reserved
word (`assert`) and five are emitter special cases — §1.5. `otherwise` is untouched and stays a
keyword.

### Leaf 6 — `leaf-corpus-migration`

28 of the 29 examples use `print`. This is the largest single-construct migration in the
milestone by site count and the most mechanical.

### Leaf 7 — `leaf-docs`

Doc 03 §12.6's panic-family table is deleted; §13.3's macro-definition decline is re-sited onto
plan 45's shipped escape hatch; doc 18 §7's `!` retirement is amended. `docs/GRAMMAR.md` loses
**one** of its two macro `UNREACHABLE` rows — invocation closes, **definition stays declined**
(§6) — so `tests/unreachable/macro_call.sdx` goes and `macro_def.sdx` remains, which is plan 46's
rule working in both directions at once. `docs/STATUS.md` and `docs/CONFORMANCE.md` both
regenerate, and plan 48's 855-line figure should drop to near zero.

---

## 5. Acceptance criteria

1. `\->` still emits `|| e` — a regression assertion, since §2 measured it already working,
   and kept separate from the macro tests so a failure there is not mistaken for a macro
   failure.
2. `name! \-> e` invokes a macro with one argument; `\->` repeated supplies more.
3. Interpolation inside a callback splices to a format string plus arguments, asserted in a
   golden.
4. A multi-line callback is layout-delimited and runs.
5. `assert! \-> a == b` reports the **expression text** on failure, asserted by a test that
   catches the panic and inspects the message.
6. `effect!` from `id_effect 0.13.0` is invocable, in a program that compiles and runs.
7. `eprintln!` works — a capability the language lacks today.
8. All six deleted constructs produce diagnostics naming their replacements, with `tests/ui/`
   fixtures. `otherwise` is **not** among them and still works.
9. `test "description"` still works and is unchanged.
10. All 29 examples green, no rustc output, stdout unchanged.
11. `construct_coverage` green; `docs/GRAMMAR.md`'s macro **invocation** row deleted with its
    fixture, and the macro **definition** row and fixture retained and still failing.
12. `docs/CONFORMANCE.md` regenerated, and plan 48's blocking-line count restated.

---

## 6. Out of scope, and what it discloses

**`!` is disambiguated by adjacency, which this milestone otherwise removed.** `foo!` is a macro
and `! foo` is negation, told apart by whether the `!` touches the identifier — which is the rule
plan 52 struck from `::`. That makes **two** instances, not three: plan 57's selector-versus-method
rule is carried by *parentheses*, not adjacency, so it is a different mechanism with a similar
smell. The honest statement: Redox has a small token budget, adjacency is the cheapest
disambiguator available, and this milestone removes one use of it and adds one. A future row
consolidating what carries a distinction — adjacency, parentheses, or a sigil — would be worth
having.

**Macro *definition* stays declined.** Doc 03 §23 item 3's alternative is the inline-Rust escape
hatch, which plan 45 shipped, and plan 45 §6 recorded the decline as honest *because* leaf 1
shipped. Unchanged here.

**Argument evaluation order inside a macro is the macro's.** Redox lowers each callback body to
an expression and hands it over; whether the macro evaluates it once, twice or never is not
Redox's business. A reader seeing `\->` may expect deferral, and §1.3 is the disclosure.

**Token-stream DSLs go through `rust`, not through `!`.** §1.4. A macro whose input is not a
Redox expression cannot be reached by a value-passing shape, and no spelling fixes that.

**This row deletes `print`, which 28 of 29 examples use.** The migration is mechanical and
large, and it is last in the milestone precisely so that nothing else is in flight while the
corpus changes its most common line.

**Two of this row's measurements are not reproducible from the repository.** The
`#[track_caller]` function-alternative experiment (§2, §9 decision 1) and the
`assert` message comparison (§1.3) were built in a scratch crate. Plan 62 committed its
turbofish measurement as a fixture and this row should do the same, so decision 1 rests on
something a reader can re-run.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout unchanged;
six `tests/ui/` fixtures for the deleted constructs; the `effect!` program compiling against
`id_effect 0.13.0` — which is **not** a dependency of any crate today, so this leaf adds the
fixture crate as well as the program; `grammar_inventory` green with the macro-invocation row
deleted and the definition row still failing; `construct_coverage` green; `docs/STATUS.md` and
`docs/CONFORMANCE.md` re-blessed and the diffs reviewed.

---

## 8. Emerald reuse

Doc 01 §23's macro-invocation syntax is retained in spirit and not in spelling — the retention
was the specification's, and the spelling it assumed (`!` as a suffix) was retired by doc 18 §7
before any row implemented it. That sequence is the transferable lesson: **a specification
retained a construct whose enabling token a later document spent**, and three plans (29, 39, 45)
found the consequence before one took it.

Plan 51's `construct_coverage` is the standing fix for that class of drift, and this row is the
largest test of it.

---

## 9. Decision log

1. **Macros, not a function family.** The alternative — make `assert`, `panic`, `todo` into
   `#[track_caller]` functions — was built and measured: location is preserved, `!` return types
   flow, and `assert_eq` is fully equivalent. It fails on one point, bare `assert cond`, which
   can only print *"assertion failed"* where the macro prints the source. And it leaves 855
   blocking lines untouched. The measurement is what makes this decline informed rather than
   assumed.
2. **The callback shape, accepting its one inaccuracy.** §1.3. The bracketed forms were the
   alternative and `{` fights the off-side rule; the juxtaposed form reintroduces the
   `f a b`-versus-`f (a) (b)` ambiguity plan 57 spent a row removing.
3. **One arrow per argument.** An argument-list-per-line block form was the alternative and makes
   a one-argument `println!` two lines. Repetition keeps the common case on one line.
4. **Delete the keyword family in the same row.** §1.2: while `assert` is a keyword, `assert!` is
   ambiguous. They cannot be separated.
5. **`\->` is not a prerequisite — it already works.** The row's first draft assumed otherwise,
   from the true but different observation that `\_ -> 7` emits `|_| 7` with arity one. `g = \-> 7`
   emits `let g = || 7;` today. Criterion 1 is therefore a regression assertion, and the reason
   the new corpus passes named functions to `Effect::retry` is that a bare name is already a
   function item (plan 59 §1.2), not that `\->` was missing.
6. **Last in the milestone.** §6. It touches every example's most common line, and nothing else
   should be moving when it does.

---

## 10. Result

*Not yet executed.*
