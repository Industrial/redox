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

Plan 48 ported the compiler's own source to Voltaic and counted what blocked it. Two passages,
quoted exactly rather than merged:

> **One blocker is 99% of the problem.** 855 lines carry a macro invocation against 12 with a
> cast.

> **The answer to doc 02 §4.8's criterion is no, and 99% of the no is one construct.** 855 of 867
> blocking lines are macro invocations.

855 lines. Macro invocation is not one gap among seven — it is the gap, and every other
`UNREACHABLE` row in `docs/GRAMMAR.md` is rounding error beside it. `effect!`, `caps!`,
`hlist!`, `vec!`, `json!`, `bail!`, `info!`, `query!`, `html!` and every proc-macro are
unreachable in a language whose premise is *"Voltaic has crates.io"*.

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

**Voltaic's own interpolation absorbs the multi-argument format family.** `bail! \-> "no route for
#{id}"` lowers to `bail!("no route for {}", id)` — Voltaic splices the arguments itself, because
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

`html! { <div/> }`, `quote!`, `sqlx::query!` consume **tokens that are not Voltaic expressions**. No
value-passing shape can carry those, because the tokens *are* the input.

Plan 45 leaf 1 shipped `rust """…"""` — inline Rust, verbatim — and that is the answer:

```
markup = rust
  html! { <div class="x">{ name }</div> }
```

Brace-free in Voltaic; whatever braces the embedded Rust needs are the embedded Rust's business.
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

Measured 2026-11-10, against `voltaic` at `194fdf7`.

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
   and the one that proves token passthrough works for a macro Voltaic knows nothing about.
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

Amended from *"the nullary lambda itself already works (§2), so this leaf is the *argument
position*, not the lambda"*, which §11.1 found false for the **multi-line** form — which is
precisely the shape §1.3 shows and criterion 4 requires:

```text
f = \->
  a = 1
  a + 1
->  RDX0102: expected an expression
```

So this leaf **either owns a layout-delimited lambda body, or criterion 4 is wholly dependent on
plan 55 leaf 1** making an indented block an expression. The row declares the dependency on 55
and then prices this leaf as if it were free, and §11.1 settles the disposition empirically
rather than by argument: **re-run the probe above once plan 55 lands.** If a block is an
expression by then, this leaf really is just the argument position; if it is not, the
layout-delimited body is costed here. Until that probe runs the sizing is open.

### Leaf 3 — `leaf-interpolation-splice`

`"… #{e} …"` inside a macro callback lowers to a format string plus arguments, reusing plan 23's
existing interpolation lowering rather than a second one.

**Cheaper than it reads**, which §11.5 is the finding for: `format_template` already returns
exactly *(template, args)* and already has two callers — `format!` and `println!` — with a doc
comment saying it is a pair *"because two callers need it"*. A third caller is a parameter, not a
second implementation. (`split_format_spec` is in the **lexer**, not the emitter.) This leaf is
also what criterion 7's `eprintln!` waits on — §11.5 again — since `print_call` is what turns
`#{x}` into a template today.

### Leaf 4 — `leaf-rust-block`

The layout-delimited `rust` block of §1.4, beside plan 45's string form — **two things, not
one**.

Amended from §1.4's *"the mechanism exists and is tested"*, which §11.5 found overstated:
`rust """…"""` works as a **top-level item only**. In expression position it is `RDX0102`, and
there is no block form at all — `rust` over an indented block is `RDX0101`. §1.4's own sample is
an expression binding of a block, so this leaf adds both the expression position and the block
form, and plan 45's tested mechanism covers only the item-level string form.

### Leaf 5 — `leaf-retire-the-family`

Six constructs removed; six diagnostics added, each naming its replacement. One is a reserved
word (`assert`) and five are emitter special cases — §1.5, with §11.5's correction that
`leaves_ordinary_identifiers_alone` covers `print` only and the other four are true from the
enum rather than from that test. `otherwise` is untouched and stays a keyword.

**This leaf keeps the `==` → `assert_eq!` rewrite**, which §11.2 is the finding for: retiring
`assert` into the general macro path is a **regression in diagnostic quality** without it.
`assert_call`'s own doc comment argues why — *"those macros print both values when they differ
and `assert!` prints only the expression's text. A test that fails and says `assertion failed:
add(1, 2) == 3` has told the author nothing they did not type"* — and criterion 5's example is
exactly the form that comment calls useless. §4's original text did not say the rewrite survives;
it does.

### Leaf 6 — `leaf-corpus-migration`

**29 of the 30** examples use `print` — amended from *"28 of the 29"* per §11.4; the one without
is `examples/lexical.sdx`, which §2 already names as the exception. This is the largest
single-construct migration in the milestone by site count and the most mechanical.

### Leaf 7 — `leaf-docs`

Doc 03 §12.6's panic-family table is deleted; §13.3's macro-definition decline is re-sited onto
plan 45's shipped escape hatch; doc 18 §7's `!` retirement is amended. `docs/GRAMMAR.md`
**regrades** its macro-invocation row from `UNREACHABLE` to `SUGARED` while the macro-definition
row **stays declined** (§6) — so `tests/unreachable/macro_call.sdx` goes and `macro_def.sdx`
remains, which is plan 46's rule working in both directions at once. `docs/STATUS.md` and
`docs/CONFORMANCE.md` both regenerate, and the blocking figure that document states should drop
to near zero.

Amended from *"`docs/GRAMMAR.md` loses **one** of its two macro `UNREACHABLE` rows"*, which
§11.4 found to be the wrong verb: `construct_coverage` filters `UNREACHABLE` rows out and then
requires **bidirectional** agreement between every reachable row and a `WITNESSES` entry, so a
regrade needs a new `Covers` entry *and* a corpus witness — real work, not a deletion — and
**deleting** the row would remove Rust's macro-invocation construct from the graded index
entirely, which is the thing `docs/GRAMMAR.md` exists to prevent.

Amended also from *"plan 48's 855-line figure"*: §11.4 found it wrong in both value and unit —
`docs/CONFORMANCE.md` says *"Macro invocation accounts for **885 of 897** blocking
occurrences"*, and it moves again on any commit that adds Rust. Cite the document rather than a
number.

---

## 5. Acceptance criteria

1. `\->` still emits `|| e` **in its one-line form** — a regression assertion, since §2 measured
   that much already working, and kept separate from the macro tests so a failure there is not
   mistaken for a macro failure.

   Scoped to the one-line form by §11.1: `g = \-> 7` emits `let g = || 7;`, but a `\->` over an
   indented block is `RDX0102: expected an expression`. The multi-line form is criterion 4's and
   is not a regression assertion, because it does not work today.
2. `name! \-> e` invokes a macro with one argument; `\->` repeated supplies more.
3. Interpolation inside a callback splices to a format string plus arguments, asserted in a
   golden.

   §11.5 found this cheap: `format_template` already returns exactly *(template, args)* and
   already has two callers, so a third is a parameter rather than a second implementation.
   Criterion 7 depends on it.
4. A multi-line callback is layout-delimited and runs.

   §11.1 found that the shape this requires **does not work today** — `\->` over an indented
   block is `RDX0102` — so this criterion is either leaf 2's to build or wholly dependent on
   plan 55 leaf 1 making an indented block an expression, and the row prices leaf 2 as if it were
   free. §11.1's disposition is empirical: re-run its probe once plan 55 lands. Until then the
   ownership is open.
5. `assert! \-> a == b` reports the **Rust rendering** of the expression on failure, asserted by
   a test that catches the panic and inspects the message; and the `==` → `assert_eq!` rewrite
   survives the retirement of `assert`, so a comparison still prints **both values**.

   Amended from *"`assert! \-> a == b` reports the **expression text** on failure"*, which §11.2
   found backwards: **Voltaic does not stringify the source. Rust stringifies the generated Rust.**
   `assert_call.rs` emits `format!("assert!({})", emit_expr(…))`, so under this criterion Rust's
   `assert!` would print `add(1, 2) == 3` — not the text the author wrote. §1.3's *"`assert!`
   recovers the expression text"* and §3's headline are overstated in exactly that way.

   **Whether the row must report the `.sdx` text instead is open and owed.** §11.2 raises the
   conflict with this project's standing rule that diagnostics report against the `.sdx` and does
   not resolve it, and it names two obstacles the row does not price: `Sink` exposes no
   span-to-text accessor, and **`emit()` passes `""`**, which every `voltaic-emit` test uses — so a
   source-text feature would be silently empty in unit tests and populated only under
   `voltaic build`, the worst possible failure shape. The second half of this criterion is settled:
   §11.2 found that retiring `assert` into the general macro path is a regression in diagnostic
   quality unless the `==` rewrite is kept, and leaf 5 now keeps it.
6. `effect!` from `id_effect 0.13.0` is invocable, in a program that compiles and runs.

   **Which macro, in which position, is open and owed** — §11.3 is the finding. A probe left in
   scratch from the exercise that opened this milestone records that `caps!(T)` is a
   **type-position** macro — `HCons<T, HNil>` — which `name! \-> e` cannot express in any form,
   and that `effect!` is do-notation over a **block**, so this criterion collapses onto
   criterion 4, the one thing measured not to work. §11.3 says the criterion should name which
   macro, in which position, and whether it needs the block form; it names none of the three.
   The same probe notes the family is largely avoidable in Voltaic because `|>` *is* the
   combinator API written left to right, which is an argument about how much this criterion is
   worth at all.
7. `eprintln!` works — a capability the language lacks today.

   §11.5 found it needs **no special case**: `eprint "x"` emits `eprint("x")`, a call to a
   function no program defines, so `eprintln!` arrives by the general macro path. It is only
   useful once leaf 3's splice exists, because `print_call` is what turns `#{x}` into a template
   today.
8. All six deleted constructs produce diagnostics naming their replacements, with `tests/ui/`
   fixtures. `otherwise` is **not** among them and still works.
9. `test "description"` still works and is unchanged.
10. All **30** examples green, no rustc output, stdout unchanged.

    Amended from *"all 29 examples"* — the corpus is 30, and `print` appears in 29 of them, per
    §11.4.
11. `construct_coverage` green; `docs/GRAMMAR.md`'s macro **invocation** row **regraded** from
    `UNREACHABLE` to `SUGARED`, with a new `Covers` entry and a corpus witness, and its
    `tests/unreachable/macro_call.sdx` fixture retired; the macro **definition** row and fixture
    retained and still failing.

    Amended from *"macro **invocation** row deleted with its fixture"*, which §11.4 found to be
    the wrong verb: `construct_coverage` filters `UNREACHABLE` rows out and then requires
    **bidirectional** agreement between every reachable row and a `WITNESSES` entry, so a
    regrade is real work rather than a deletion — and deleting the row would remove Rust's
    macro-invocation construct from the graded index entirely, which is the thing
    `docs/GRAMMAR.md` exists to prevent.
12. `docs/CONFORMANCE.md` regenerated, and the blocking count restated **by citing that document
    rather than a number**.

    Amended from *"plan 48's blocking-line count restated"*, which §11.4 found wrong in both
    value and unit: `docs/CONFORMANCE.md` says *"Macro invocation accounts for **885 of 897**
    blocking occurrences"* where §1.1 and leaf 7 said *"855 lines"*, and the figure moves again
    on any commit that adds Rust.

---

## 6. Out of scope, and what it discloses

**`!` is disambiguated by adjacency, which this milestone otherwise removed.** `foo!` is a macro
and `! foo` is negation, told apart by whether the `!` touches the identifier — which is the rule
plan 52 struck from `::`. That makes **two** instances, not three: plan 57's selector-versus-method
rule is carried by *parentheses*, not adjacency, so it is a different mechanism with a similar
smell. The honest statement: Voltaic has a small token budget, adjacency is the cheapest
disambiguator available, and this milestone removes one use of it and adds one. A future row
consolidating what carries a distinction — adjacency, parentheses, or a sigil — would be worth
having.

**Macro *definition* stays declined.** Doc 03 §23 item 3's alternative is the inline-Rust escape
hatch, which plan 45 shipped, and plan 45 §6 recorded the decline as honest *because* leaf 1
shipped. Unchanged here.

**Argument evaluation order inside a macro is the macro's.** Voltaic lowers each callback body to
an expression and hands it over; whether the macro evaluates it once, twice or never is not
Voltaic's business. A reader seeing `\->` may expect deferral, and §1.3 is the disclosure.

**Token-stream DSLs go through `rust`, not through `!`.** §1.4. A macro whose input is not a
Voltaic expression cannot be reached by a value-passing shape, and no spelling fixes that.

**This row deletes `print`, which all but one example uses.** §11.4 counted 29 of 30; the corpus is 31 as of 2026-11-10, which is why this sentence now names the shape rather than a number. The migration is mechanical and
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

**Done 2026-11-10.** All seven leaves, every criterion met or amended in
writing, and the measurement that motivated the milestone changed: `docs/CONFORMANCE.md`
reads **12 blocking occurrences, down from 964**, all of them `as` casts.

### 10.1 The probe §11.1 owed, run

`\->` over an indented block **works**, so leaf 2 is the argument position and
nothing else:

```text
main : ()
main =
  f = \->
    a = 1
    a + 1
  print (f ())
->  fn main() { let f = || { let a = 1; a + 1 }; … }
```

§11.1 left this *"empirical, after 55 lands"* and §11.1's own addendum called
the condition met in a stronger form than it anticipated — plan 55 leaf 1
deleted `StmtKind`, and plan 65 made layout work inside brackets. Both are
right: `prefix` reads a `Newline` as a block, so a macro argument parsed with
`parse_expr` gets the multi-line form for free. **Criterion 4 cost zero.**

That is the value of writing the probe down rather than deciding by default.
The row's sizing rested on this one question, and the answer was one command.

### 10.2 The migration's own finding: an interpolation is one token

**This is the row's sharpest finding, and the gate found it rather than a
reader.** `print e` became `println! \-> "#{e}"`, and an interpolation is a
*single* `Str` token — so every expression that used to stand in a `print`'s
argument moved **inside a string**, where `construct_coverage`'s scan of token
runs cannot see it.

Three constructs had no witness anywhere else and all three were reported at
once: precedence (`+ «int» *`), a left section (`( «int» - )`), and indexing a
call's result (`) [ «int» ]`). `examples/expressions.sdx` is the precedence
row's whole witness and after the migration it witnessed **nothing**, with all
three constructs still in plain sight on the page.

The fix is a binding, which puts the tokens back on the line — and reads
better, which is why `examples/expressions.sdx` is now eight bindings rather
than eight interpolations. No change to the scanner: a construct buried in a
string is exactly what plan 51 §1 built this test to catch.

### 10.3 What `print`'s retirement actually cost, in one place

`print` was the only **built-in** that took an arbitrary expression in
argument position. `println!`'s first argument is a Rust format string, and
this row brace-doubles a string literal reaching a macro — so
`println! \-> "{}"` prints braces, exactly as `print "{}"` did, and the
two-argument `println!("{}", e)` is **deliberately unspellable**. §1.3's
*"interpolation absorbs the multi-argument format family"* taken to its
conclusion.

So an expression reaches a macro through a name or through an interpolation,
and an interpolation is one line. Two corpus sites were multi-line
`print (if c …)` — plan 65's witness in `examples/control.sdx` and the
matching tree-sitter corpus case — and neither could be rewritten
mechanically. Both pass the bracketed `if` to a **function** instead, which is
still argument position and is what the comment above each one claims.

### 10.4 Two defects found, both older than this row

**A lexical error inside an interpolation reported the wrong line.**
`lex_fragment` rebases every token's span onto the file and did not rebase its
**error's**, so `bad_quote.stderr` blessed to *line 1, column 4* for an error
on line 3. `LexError`'s own doc comment describes the identical failure one
path over, and `split_parts` applies `offset_by` to `scan_escape` and
`skip_interpolation` — the fragment path was the one call that did not. Older
than this row, unwitnessed until the migration moved two `tests/ui/` fixtures'
bad tokens inside an interpolation.

**Plan 07 §2.1's whole-statement fallback lost its witness, correctly.**
`check/restructured.sdx` asserted that a line the emitter *restructures*
underlines the whole statement, with `print (add "x" 2)` as the witness. The
replacement is **longer** in the `.sdx` than the generated Rust is, so the
shifted column lands inside the statement's span and `narrow` returns a
precise caret — right, but then nothing exercised the fallback. A construct
that still grows under lowering does; the fixture is a comprehension now.

### 10.5 The register's own numbers, corrected a third time

§11.4 found criterion 12's figure wrong in both value and unit — *885 of 897
occurrences*, not *855 lines* — and said to cite the document. By the time the
row ran it was **952 of 964**, which is the argument for citing it made
concrete twice over. The generator no longer counts macro invocation at all:
the document's own sentence under **Blockers** reads *"each is graded
`UNREACHABLE` in `docs/GRAMMAR.md`"*, and a `SUGARED` row there would make the
document wrong about itself.

### 10.6 Six sweeps, one mistake

Leaf 6 touched 110 `.sdx` and corpus files and 38 Rust files, and the Rust
half was reverted and re-run **five times**. Every failure was the same
mistake in different clothes — *a segment does not carry its own context* —
and `.tmp/p64rssweep.py`'s header records all six in order: prose rewritten as
code, a bare `"` closing a Rust literal, `\->` as an invalid Rust escape, a
file list whose boundary missed `\n`, a segment carrying the Rust that closes
its literal, and a statement on the same line as its opening quote.

Two things made that survivable. The sweep **asserted a unique anchor** and
failed rather than guessing; and the diff was read every time. The one cost
not caught by reading was that `git checkout` over the sweep's file list took
the compiler-side edits with it — those files are candidates *because their
doc comments quote `print`* — which is why they now live in one script,
`.tmp/p64src.py`, rather than in five.

**No `.stdout` file changed.** Thirty programs, every one printing exactly
what it printed before, which is the strongest correctness signal this corpus
offers for a migration of this size.

### 10.7 What the readers cost

All five, and the grammar needed three rounds. `macro_call` is
`prec.right(1, …)` over `token.immediate("!")` — the adjacency rule, stated in
the one place tree-sitter can state it — and its argument had to name
`block_expression` explicitly for the same reason `binding` does: `$._expression`
excludes a bare block, because including it gives GLR two readings of every
indented body. `inline_rust` gained the layout body and a `rust_hatch`
expression rule at `prec(-1)` so the item form wins a tie.

`assert` left `highlights.scm`, the vim syntax and the TextMate grammar, and
`editors.rs` caught both highlighters in one assertion — the cross-check plan
66 §10 found the gap in from the operator side.

---

## 11. Corrections from orientation

Written after reading the code and probing it. §2's measured preconditions are
mostly true — and the **one** the leaf sizing rests on is false for exactly the
shape the row needs. Two numbers are stale, and criterion 5 has its mechanism
backwards.

### 11.1 The multi-line `\->` does not work, which is the shape the row needs

Criterion 1 is true for the one-line form: `g = \-> 7` emits `let g = || 7;`.

```text
f = \->
  a = 1
  a + 1
->  RDX0102: expected an expression
```

So §2's *"`\->` already works"* and leaf 2's *"the nullary lambda itself already
works … this leaf is the argument position, not the lambda"* are **false for the
multi-line form** — which is precisely what §1.3 shows (`r = catch_unwind! \->`
over an indented block) and what criterion 4 requires.

Leaf 2 therefore either owns a layout-delimited lambda body, or criterion 4 is
**wholly dependent on plan 55 leaf 1** making an indented block an expression.
The row declares the dependency on 55 and then prices leaf 2 as if it were free.
Re-run the probe above once plan 55 lands: if a block is an expression by then,
leaf 2 really is just the argument position.

#### Decided 2026-11-10: neither option — the probe is runnable now

§11.1 left this to be resolved *"empirically after 55 lands"*, and that
condition is met in a stronger form than it anticipated. **Plan 55 leaf 1 is
done** — `StmtKind` is deleted and every body position holds an `Expr` — and
**plan 65 then made layout work inside brackets**, which is precisely the
construct in question: a multi-line body inside `(…)` that the lexer used to
emit no layout token for at all.

So the question is no longer which of two costs to pay. It is a measurement,
and the measurement is cheap: write `name! -> e` with a layout-delimited body
and see what comes out. If it works, criterion 4 needs nothing and leaf 2's
contingent sizing collapses to zero; if it does not, leaf 2 owns the body and
the cost is priced against a real failure rather than an anticipated one.

**Owed: run the probe and record the result here before leaf 2 starts.** An
"empirically, later" that is never run becomes a decision made by default, and
that is what this block exists to prevent.

### 11.2 Criterion 5 has the stringification backwards, and deleting `assert` costs a feature

**Voltaic does not stringify the source. Rust stringifies the generated Rust.**
`assert_call.rs` emits `format!("assert!({})", emit_expr(…))`, so the failure
message carries the *Rust* rendering: `assert add 1 2 == 4` becomes
`assert_eq!(add(1, 2), 4)`. Under criterion 5's `assert! \-> a == b`, Rust's
`assert!` would print `add(1, 2) == 3` — not the text the author wrote, against
this project's standing rule that diagnostics report against the `.sdx`.

The source *is* reachable — `emit_with_map` takes it and `Sink` holds it — but
with two obstacles the row does not price: `Sink` exposes no span-to-text
accessor, and **`emit()` passes `""`**, which every `voltaic-emit` test uses. A
source-text feature would be silently empty in unit tests and populated only
under `voltaic build`, which is the worst possible failure shape.

**And `assert` is not merely a special case to delete.** `assert_call`'s own
doc comment argues why it exists: `==` becomes `assert_eq!`, and *"those macros
print both values when they differ and `assert!` prints only the expression's
text. A test that fails and says `assertion failed: add(1, 2) == 3` has told the
author nothing they did not type."* Criterion 5's example is exactly the form
that comment calls useless. Retiring `assert` into the general macro path is a
**regression in diagnostic quality** unless the row keeps the `==` → `assert_eq!`
rewrite, and §4 does not say it does.

#### Decided 2026-11-10: accept Rust's rendering, and the tension is apparent only

Rust's rendering of the generated Rust, option (a). The `.sdx` source-text
alternative needs a span-to-text accessor on `Sink` *and* a fix for `emit()`
passing `""`, and §11.2's own observation settles it: such a feature would be
silently empty in every `voltaic-emit` test and populated only under
`voltaic build`, which is a feature whose tests cannot see it.

**The conflict with *"diagnostics report against the `.sdx`"* is a category
error, not a trade.** That rule governs **Voltaic diagnostics** — the `RDX` codes
this compiler issues about source it has read. A failed `assert!` is a **Rust
runtime panic** emitted by the generated program at execution time; Voltaic is
not the one reporting it, and the `.xrs` is a human-facing artifact precisely so
that what Rust says about it is readable. The bridge already exists and is the
right one: the source map, which plan 50's `voltaic_source_map` serves.

So no rule is being bent. A panic message is not a diagnostic, and the two were
only in tension while they were described in the same words.

### 11.3 Criterion 6's `id_effect` macros may not fit the callback shape at all

§7 correctly discloses that `id_effect` is a dependency of nothing. But a
probe left in scratch from the exercise that opened this milestone records
something criterion 6 needs to answer: `caps!(T)` is a **type-position** macro
— `HCons<T, HNil>` — which `name! \-> e` cannot express in any form; and
`effect!` is do-notation over a **block**, so criterion 6 collapses onto
criterion 4, the one thing measured not to work.

Criterion 6 should name *which* `id_effect` macro, in which position, and
whether it needs the block form. The same probe notes that the macro family is
largely avoidable in Voltaic because `|>` *is* the combinator API written left to
right — which is an argument about how much criterion 6 is worth.

#### Decided 2026-11-10: a simpler crates.io proc-macro, not an `id_effect` one

Option (b), and against the reading that the witness should come from the crate
that motivated the row. The deciding fact is in **plan 59 §11.5**, which
measured it: `id_effect` appears in **documents only** — zero hits under
`crates/`, under `examples/`, or in any `Cargo.toml`, and the workspace manifest
has no `[workspace.dependencies]` section at all. The 1,501-line corpus that
motivated this milestone is not in this repository.

So option (a) does not merely name a macro. It requires the
`examples/cargo-project/` shape — `Cargo.toml`, `build.rs`, `include!`, driven
by `compile_dir` — because `voltaic check` shells out to bare rustc with **no
`--extern`**. Plan 59 §11.5 flagged that same infrastructure as an open cost
nobody has funded. Making criterion 6 depend on it funds it by accident, inside
a row about macro invocation.

And the criterion's job is narrower than its example suggests: it proves **token
passthrough**, which any value-position proc-macro proves. `caps!` and `effect!`
move to §6 as disclosed non-fits — §11.3 is right that one is type-position and
the other block-position — and the witness becomes a proc-macro already
reachable from this workspace.

If the `id_effect` corpus is ever brought in-tree, promoting the witness is a
one-line change against a passing test. That is the right order.

### 11.4 Two numbers, and a verb

- **Criterion 12's figure is wrong in both value and unit.**
  `docs/CONFORMANCE.md` says *"Macro invocation accounts for **885 of 897**
  blocking occurrences"*; §1.1 and leaf 7 say *"plan 48's 855-line figure"*. It
  is **885 of 897 occurrences**, not 855 lines — and it moves again on any
  commit that adds Rust, so cite the document rather than a number.
- **Criterion 11's *"deleted"* is the wrong verb.** `construct_coverage`
  filters `UNREACHABLE` rows out and then requires **bidirectional** agreement
  between every reachable row and a `WITNESSES` entry. Regrading macro
  invocation to `SUGARED` needs a new `Covers` entry *and* a corpus witness —
  real work, not a deletion. **Deleting** the row instead would remove Rust's
  macro-invocation construct from the graded index entirely, which is the thing
  `docs/GRAMMAR.md` exists to prevent. Regrade it.
- *"`print` appears in 28 of the 29 examples"* is 29 of 30; the one without is
  `examples/lexical.sdx`, which §2 already names as the exception.

### 11.5 Verified true, and two small imprecisions

Verified: `!` is `TokenKind::Bang`, consumed **only** for negation;
`vec![1, 2, 3]`, `format! "hi"` and `s.len!` are all `RDX0101`; `assert! true`
emits `assert!(!true)`; `otherwise` is reserved and works; `test "…"` works;
`eprint "x"` emits `eprint("x")`, a call to a function no program defines, so
`eprintln!` needs **no** special case and arrives by the general path — though
it is only useful once leaf 3's splice exists, since `print_call` is what turns
`#{x}` into a template today.

**Criterion 3 is cheap.** `format_template` already returns exactly
*(template, args)* and already has two callers — `format!` and `println!` — with
a doc comment saying it is a pair *"because two callers need it"*. A third
caller is a parameter, not a second implementation. (`split_format_spec` is in
the **lexer**, not the emitter.)

Two imprecisions worth fixing in place: `s.len!` reports *"expected a line break
after the statement"*, not the quoted *"expected `)`"* — that needs a
parenthesised context. And §1.5's appeal to `leaves_ordinary_identifiers_alone`
covers `print` only; the other four special cases are true from the enum but not
from that test.

**§1.4's escape hatch is overstated.** `rust """…"""` works as a **top-level
item only**: in expression position it is `RDX0102`, and there is no block form
— `rust` over an indented block is `RDX0101`. §1.4's own sample is an expression
binding of a block, so leaf 4 adds **two** things, and *"the mechanism exists
and is tested"* applies only to the item-level string form.
