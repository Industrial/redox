# Plan 69 — The Formatter in Redox

Plan ID: `fmt-in-redox`
Depends on: 68 (`ast-in-redox`), 64 (`macro-invocation`), 34 (`formatter`)
Milestone: 10 — *self-hosting*
Status: `done` 2026-11-10

---

## 1. What this plan is

**`crates/redox-fmt` is written in Redox**, by plan 68's harness and on plan
68's terms: 57 `.sdx` files, the generated Rust committed in `src/`, the
workspace compiled against it.

And it is the row that **decides whether the other ten happen**. Plan 68's
value was not self-hosting; it was nine surface gaps and three comment-loss
defects, one of which no other test in the gate could have caught. This row is
the second data point, and its §10 has to report the yield whether or not it
flatters the effort.

### 1.1 The measurement that chose this crate, and corrects a published figure

`docs/CONFORMANCE.md` reads **12 blocking occurrences, all `as` casts**, and
the figure is honest about what it counts: the five constructs
`docs/GRAMMAR.md` grades `UNREACHABLE`. It does not know about the nine gaps
plan 68 found by porting a crate, and counted across the eleven crates still
in Rust those are **six times larger**:

| | hits | crates |
|---|---:|---:|
| **graded `UNREACHABLE`** — what `CONFORMANCE.md` counts | **29** | 8 |
| `matches!` | 61 | 7 |
| inline `#[cfg(test)] mod tests` | 61 | 7 |
| `for` over a pattern | 19 | 7 |
| open-ended range | 16 | 7 |
| `Self::` as a path expression | 7 | 4 |
| a method passed as a value | 4 | 4 |
| a declaration-bodied macro | 3 | 1 |
| `vec![elem; n]` | 2 | 1 |
| **found by plan 68, counted by nothing** | **173** | |
| **total** | **202** | |

467 files and 28,316 code lines, so 202 sites is 0.7% of lines — close by that
measure and not by the one that matters, which is how many **rows** stand in
the way. §6 sorts them.

**`redox-fmt` has six of the 202.** Three `matches!`, one `for` over a
pattern, one open-ended range, one method passed as a value. No `as` casts, no
inline test modules, no declaration-bodied macros. It is the cleanest crate in
the workspace after the one already ported, which is why it is next.

### 1.2 Its shape is the opposite of `redox-ast`'s, and that is the point

A second port of the same shape would measure nothing. These two could not be
less alike:

| | `redox-ast` | `redox-fmt` |
|---|---|---|
| files | 53 | 57 |
| code lines | 1,411 | 2,910 |
| data declarations | 38 files of them | **none** |
| `impl` blocks | 6 | **none** |
| free functions | 3 | **57 — one per file** |
| `format!` calls | 2 | **125** |
| `.join(` | 1 | 24 |
| closure arguments | 9 | 27 |

`redox-ast` is *"syntax tree types, no logic"* and the port was mostly
declarations; plan 68's translator did 38 of its files mechanically and is
**useless here**, because it translates declarations and this crate has none.
Every file is one function and every one is hand-written.

So the surface under test is different: the **macro call** plan 64 shipped
three rows ago, exercised 125 times; string building; iterator chains;
closures; `if let` and `let … else`. Plan 68 exercised almost none of that.

### 1.3 The hazard that is not one, checked first

A formatter prints Redox, so its own string literals carry Redox syntax as
**data** — and `#{` in a Redox string opens an interpolation. If `redox-fmt`
held `"#{"` as a literal, the port would read the formatter's own output
spelling as a hole.

Counted before this plan was written: **all eight `#{` occurrences are in doc
comments**, where trivia is verbatim and nothing interpolates. One string
literal carries Redox syntax — `macro_call_text.rs`'s `" \\->\n"` — and `\\`
is an ordinary escape in both languages.

The brace case is already solved and already exercised: a `{` in a Redox
string is literal and the emitter doubles it, which is how plan 68's
`pattern.sdx` emits `format!("{} {{ {} }}", …)`. Confirmed there rather than
assumed here.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `32dea7a` — the commit that landed
plan 68.

**The harness exists and works.** `crates/redox-cli/tests/ast_port.rs` is the
mirror assertion, the regeneration test with its staging guard, the
escape-hatch check and leaf 4's doc-comment count. Leaf 0 of this row
generalises it to two crates rather than writing a second copy.

**`redox-fmt` is 57 of 57 files blocker-free** by the five graded constructs,
and carries six of the 202 sites §1.1 counts.

**Plan 68's nine gaps and their workarounds are known**, which is the
difference between this row and that one: an open-ended range binds its end
first, a `for` over a pattern derefs in the body, `matches!` becomes a `case`
ladder, a method passed as a value becomes a lambda, `Self::name` becomes the
type's own name. None has to be rediscovered.

**Two clippy allows will be needed**, and plan 68 §10.3 is why:
`match_like_matches_macro` because `matches!` is unreachable, and `no_effect`
if any `()`-returning function needs plan 55 §1.6's written unit tail. Both
are statements of fact, not suppressions, and the second has a successor owed.

**`format!` is reachable and exercised.** Plan 68's port used it, and
`format! \-> "#{a}-#{b}"` emits `format!("{}-{}", a, b)`. Interpolation
splices, so a 125-call crate needs nothing new.

---

## 3. Concrete proof this plan targets

**`cargo check --workspace` compiles `redox-fmt` from Rust generated by Redox,
and every test passes** — including `redox-fmt`'s own 1,463 lines of tests,
which assert the formatter's output byte for byte. A printer that prints one
character differently fails its own suite, so this is a sharper oracle than
`redox-ast` had.

Four more:

1. **`redox fmt --check` passes on the whole `.sdx` corpus**, formatted by a
   formatter compiled from Redox. The row's one genuinely circular assertion
   and the one worth having: the formatter formats its own source.
2. **125 `format!` calls survive**, which is plan 64's macro invocation
   exercised at a scale no example reaches.
3. **Every doc comment survives as a doc comment**, by the count leaf 4
   already applies — now over a crate with no declarations to hang them on,
   so they sit on functions instead.
4. **The yield is reported**, whether it is nine findings or none. §10.

---

## 4. Leaves

### Leaf 0 — `leaf-two-crate-harness`

`ast_port.rs` becomes a harness over a **list of ported crates** rather than
one hard-coded path, and `crates/redox-fmt/sdx/` joins
`crates/redox-ast/sdx/`. The staging guard checks each crate it is given.

One test file, not two: the four assertions are the same four, and a second
copy would be the one-predicate-two-positions defect plan 34 §10.31 found five
times and plan 61 found again.

### Leaf 1 — `leaf-port-the-printers`

The 57 files. Every one is a single function, so the leaf is bounded by count
rather than by shape, and the order is smallest first — `indent_text` is three
lines, `expr_text` is 305.

### Leaf 2 — `leaf-the-six-gaps`

The six sites §1.1 counts, each with the workaround plan 68 established, and
each **named in the `.sdx`** so a reader meets the reason where the oddity is
rather than in a plan document.

### Leaf 3 — `leaf-docs-and-register`

`docs/CONFORMANCE.md`'s worked-port section names two crates;
`docs/GRAMMAR.md` keeps its grades; milestone 10's table carries the **202**
rather than the 29, because the 29 is what made the distance look shorter than
it is. `docs/STATUS.md` regenerates.

---

## 5. Acceptance criteria

1. `crates/redox-fmt/src/**/*.rs` is generated from
   `crates/redox-fmt/sdx/**/*.sdx`, one file for one file.
2. `cargo check --workspace --all-targets` passes with no change to any other
   crate.
3. Every test passes, unchanged — **including `redox-fmt`'s own**, which
   assert the formatter's output byte for byte.
4. `redox fmt --check` passes on the whole `.sdx` corpus, run by the
   Redox-compiled formatter.
5. Regenerating reproduces what is committed, byte for byte, and the staging
   guard refuses a generation that does not compile.
6. Every `-- |` and `-- !` reaches the generated Rust as `///` and `//!`.
7. No escape hatch anywhere in the source.
8. One harness for both crates, not two.
9. `docs/CONFORMANCE.md` and `docs/STATUS.md` regenerated; milestone 10's
   table carries the 202.
10. **§10 reports the yield** — the defects and gaps this port found, or the
    fact that it found none. The row is a measurement and the measurement is
    the deliverable.

---

## 6. Out of scope, and what it discloses

**The other ten crates are not decided by this row**, they are *informed* by
it. §10's yield is the input to that decision and §1 is the argument for
taking it seriously in both directions.

**The 202 sort into four kinds, and only one is porting.** Written here
because the next reader of `docs/CONFORMANCE.md` will otherwise price
self-hosting at 12 occurrences:

- **Reachable today, awkwardly — about 109 sites.** All 61 inline test
  modules, because item-level `test "…"` already emits
  `#[cfg(test)] mod __rdx_tests { use super::*; }` and sees the file's private
  items; plus the ranges, `for` patterns, `Self::` paths, `vec![x; n]` and
  method-as-value workarounds. Reachable is not the same as fine: this is 109
  deliberate uglifications.
- **Three parser rows the language wants anyway** — `for` over a pattern,
  open-ended ranges, `Self::` as a path expression. Each is small, and the
  range one closes a **live defect**: `r = 2..` silently swallows the
  statement below it, which is a fifth deferred case found after milestone 9
  declared that class empty.
- **Two surface decisions, the expensive kind.** A macro form that can take a
  **pattern**, for `matches!`'s 61 sites — otherwise it is `case` ladders and
  a `match_like_matches_macro` allow in seven crates. And `as`, 29 sites,
  where the nearest working thing is `try_from` and that is a *different
  program*, since `as` truncates.
- **One genuinely hard case.** `thread_local!` — a macro whose body is a
  **declaration**. Plan 64 §6 disclosed that `name! \-> e` cannot carry it. It
  is in exactly three files, all in `redox-emit`, all scope guards from plans
  59 and 60. The escape hatch is the answer and criterion 7 bans it in a port,
  so that crate needs the macro form or a stated exemption.

**100% cannot mean deleting the Rust.** Doc 02 §4.8 says the Rust
implementation stays *permanently*, correcting doc 01 §34. The coherent target
is what plan 68 did once: every crate has a `.sdx` source and the committed
`.rs` is generated. Both representations, with a regeneration test — and that
is a cost, paid forever, which §10's yield has to be weighed against.

**`redox-fmt`'s tests are not ported.** 1,463 lines of them, and they are what
makes criterion 3 sharp — a Rust test asserting the output of a
Redox-generated printer is a better oracle than a ported test asserting it.
Plan 68 left `redox-ast`'s tests alone for the same reason.

---

## 7. Gate

```bash
moon run :check          # the whole workspace, against both generated crates
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # every test, including redox-fmt's own
moon run :format         # treefmt
```

Plus: the regeneration test for both crates; the doc-comment count; no `rust`
hatch; the mirror assertion; `redox fmt --check` over the corpus;
`docs/CONFORMANCE.md` and `docs/STATUS.md` re-blessed and the diffs reviewed.

---

## 8. Emerald reuse

None, and for the reason plan 68 gives: Emerald never attempted this. What
transfers is **plan 68's harness and its nine gaps**, which is the first time
in this project a self-hosting row has had a predecessor to inherit from
rather than a measurement to make.

---

## 9. Decision log

1. **`redox-fmt`, because its shape is the opposite of `redox-ast`'s.** §1.2.
   A second port of the same shape would measure nothing; this one exercises
   the macro call 125 times where the first exercised it twice.
2. **One harness for two crates.** A second copy of four assertions is the
   defect plan 34 §10.31 found five times.
3. **The yield is a deliverable**, criterion 10. A row whose purpose is to
   inform a decision has to report what it found even if that is nothing,
   because "nothing" is the answer that would stop the effort.
4. **The 202 goes in the register.** `docs/CONFORMANCE.md`'s 12 is honest
   about its own scope and misleading about the distance, and a reader pricing
   self-hosting from it would be out by a factor of six.
5. **The tests stay in Rust.** A Rust test asserting a Redox-generated
   printer's output byte for byte is a sharper oracle than a ported one.

---

## 10. Result

**Done 2026-11-10.** `crates/redox-fmt` is written in Redox: 57 `.sdx` files
under `crates/redox-fmt/sdx/`, 2,355 lines of Redox generating **2,907 lines
of Rust** committed in `src/`, with the whole workspace compiled against it and
**851 tests passing**. 1,045 `-- |` in and 1,045 `///` out, 37 `-- !` in and 37
`//!` out, no escape hatch anywhere, and `redox fmt --check` clean over all 57.
51 of 57 generated files differ from the hand-written Rust by **545 insertions
and 548 deletions** — a crate a person could have written, which is the same
reading plan 68 got.

All 125 `format!` calls survive. (A count of the generated source says 126; the
extra is the word `format!` inside a comment this port added.)

### 10.1 The yield: seven findings, two of them defects in the crate being ported

Criterion 10 exists because a row whose purpose is to measure has to report the
measurement even when it is zero. It is not zero.

| # | Finding | Kind | Sites |
|---|---|---|---|
| 1 | Plan 68's keyword survey covered **fields and module names** and missed every other kind of name | method defect | 5 |
| 2 | A binding whose **value is a block** swallows a following `.name` statement | surface | 4 |
| 3 | A **lambda parameter is a name, not a pattern**, so `.enumerate()` and `.zip(…)` have no idiomatic consumer | surface | 2 |
| 4 | A **comment between two pipeline stages** breaks the pipeline | **defect** | 2 |
| 5 | A prefix `*` in **argument** position reads as binary multiplication, silently | surface | 1 |
| 6 | `redox fmt` **corrupts** a lambda with a block body | **defect, fixed** | 5 files |
| 7 | `redox fmt` has **no fixed point** for a nested unary — a two-cycle | **defect, fixed** | 1 |

Findings 6 and 7 are what §1.3 was hoping for and did not expect to get: the
formatter, reading its own ported source, mis-printed it. Both are fixed **in
Redox** — the edits are to `crates/redox-fmt/sdx/expr_text/lambda_text.sdx`
and `crates/redox-fmt/sdx/expr_text.sdx`, and the `.rs` is regenerated. They
are the first defects in this project fixed in Redox source rather than Rust.

### 10.2 Finding 1 — the survey, not the language

Plan 68 surveyed Redox's keywords against `redox-fmt`'s identifiers and
reported three names colliding across four sites. It counted **fields and
module names**. It did not look at parameters, locals, lambda parameters or
pattern binders, and four of the five collisions this port actually hit are in
those four categories:

| Name | Kind of name | Where |
|---|---|---|
| `otherwise` | function parameter | `stmt_text/else_text.sdx` |
| `guard` | lambda parameter | `item_text/clause_text.sdx` |
| `test` | plain local | `item_text/clause_text.sdx` |
| `from` | pattern binder | `stmt_text/code_text.sdx` |
| `otherwise` | record-pattern **field name** | `stmt_text/code_text.sdx` |

`r#name` handles all five, and `r#foo` and `foo` are the same identifier in
Rust, so the escape costs nothing downstream. The last row needed a probe to
establish that `r#name` is accepted in a record pattern's *field-name* position
as well as its binder position.

The lesson is about measurement rather than syntax, and it generalises past
keywords: **a survey of one syntactic category reads as a survey of the
language.** Plan 68's number was not wrong about fields; it was quoted as a
number about names.

One diagnostic is worth a row of its own. `Expr::Guard { pattern, source =
from, otherwise, .. }` reports `RDX0103` with its caret on **`from`**, the
renamed binder *before* the offending field. Moving the rename later moved the
caret onto `otherwise` and revealed it. Three attempts went into a two-keyword
pattern because the span pointed at the wrong one.

### 10.3 Finding 2 — a block-valued binding eats the next statement

```text
arrow = if hangs
  " \\->\n"
else
  " \\-> "
.push_str arrow out
->  RDX0115: method-call syntax is retired
```

The `.push_str` is read as a **postfix on the `if`**, so plan 57's
`RDX0115` refuses `(if hangs … else …).push_str(…)`. Two narrower statements of
this were written and both were wrong, which is why the probes exist:

* *"a statement beginning with `.name` is read as a postfix on the preceding
  expression"* — too broad. `target_text.sdx` has `.push ' ' out` then
  `.push_str (&arg.text) out` and parses; a `Newline` terminates the first.
* *"after a block-bearing expression, because a `Dedent` does not terminate the
  statement"* — wrong, and `expr_text/macro_call_text.sdx` holds the
  counterexample three lines below the comment that claimed it: an `if`/`else`
  **statement** followed by `.push_str (…) out`, which parses.

The pair that settles it is four lines each — the `if` as a statement, which
is clean, and the same `if` as a binding's value, which is `RDX0115`. The
second is committed as
`tests/ui/block_valued_binding_eats_the_next_statement`; the first needs no
fixture, because every passing file in the corpus is one. It is
the **binding** path: a value expression goes through the Pratt loop, which
tries `postfix` once the value ends, and a block-bearing value ends on a
`Dedent` that does not stop the loop. A block in statement position never
reaches `postfix`.

Three spellings work, and the port uses all three: push inside the branches,
parenthesise the statement, or `recv |> .name args` — which is what `RDX0115`'s
own help text offers, and which reads better anyway when two consecutive lines
are about the same receiver.

### 10.4 Finding 3 — a lambda parameter is a name

```text
|> .map (\(i, constructor) -> …)
->  RDX0103: expected a lambda parameter name
```

This is the finding with the widest reach, and the one least about printers.
Rust closures take patterns, and `|(i, x)|` is how `.enumerate()` and
`.zip(…)` are consumed — two of the most common adapters there are. The port
takes the pair whole and splits it with `.0` / `.1`, twice.
`tests/ui/lambda_parameter_is_not_a_pattern` is the fixture.

### 10.5 Finding 4 — a comment between pipeline stages, which is a defect

```text
  xs
    |> .iter
    -- A comment between two stages.
    |> .map (\n -> n * 2)
    |> .collect
->  RDX0102: expected an expression, on the `|>` after the comment
```

Four lines, committed as `tests/ui/comment_between_pipeline_stages` — the one
fixture of the three that records a **defect**, so fixing it must fail there and
be reviewed. Remove the comment and the same five lines parse, and a
multi-statement lambda in a pipeline stage is fine on its own, so the comment
is the whole of it.

Unlike findings 2, 3 and 5 this is not a surface decision. *"Comments are
trivia, never discarded"* is a load-bearing rule in `AGENTS.md`, and a trivium
that makes its neighbour fail to parse is the strongest form of discarding it.
Both sites moved their comment above the binding, which reads acceptably and is
not the point.

### 10.6 Finding 5 — a prefix operator in argument position, decided silently

```text
lambda_text params body *is_move indent source
->  lambda_text(params, body) * is_move(indent, source)
```

Both readings parse, so **nothing is diagnosed**. rustc caught it, through the
staging guard plan 68 built after breaking the bootstrap: the generated
`redox-fmt` failed to compile, nothing was written, and `crates/redox-fmt/src/`
was untouched. That is the first defect the guard has stopped, and it was not
written for this one.

The ambiguity is not about `*`. **Juxtaposition application and infix operators
collide wherever an operator is also a prefix**, and Redox resolves it towards
the infix reading. `&` and `-` have the same shape. A parenthesis is the fix —
`(*is_move)` — and `if *inclusive` elsewhere in this crate is safe only because
nothing precedes the `*`.

### 10.7 Findings 6 and 7 — the formatter, read through itself

§1.3 asked whether running `redox fmt --check` over the formatter's own Redox
source would find anything. It found two defects, and neither could have been
found any other way, because **no corpus file had the shapes the port wrote**.

**A lambda with a block body was corrupted.** `redox fmt` turned

```text
    |> .map (\stmt ->
      text = stmt_text stmt indent source
      …)
```

into `|> .map (\stmt ->     text = stmt_text stmt indent source`, which does
not parse. Five of 57 files — every one that maps a multi-statement lambda over
a collection. The cause is that `lambda_text` writes `" -> "` and then the body,
but an unlabelled `Expr::Block` has **nothing on its own first line**:
`expr_text` returns `block_text(body, indent + 1, …)`, whose text begins with
the block's first statement already padded. `binding_text` knows this and
chooses between `"= "` and `"=\n"` on exactly that pattern, with a comment
saying why. `lambda_text` never got the same choice, because until this port no
corpus file had a lambda with a block body.

**And a nested unary has no fixed point.** Measured over
`crates/redox-fmt/sdx/expr_text.sdx`:

```text
pass 1:  **inner    ->  *(*inner)
pass 2:  *(*inner)  ->  **inner
pass 3:  **inner    ->  *(*inner)
pass 4:  *(*inner)  ->  **inner
```

A two-cycle, so `redox fmt --check` can never pass on a file containing one,
whichever spelling is committed. Two rules in `expr_text` disagree: the `Unary`
arm parenthesises a `Unary` operand to stop `-(-x)` printing as `--x`, and
`can_drop_paren` then judges that parenthesis redundant because its content
binds at `u8::MAX`. Each is right alone.

The arm is what is wrong, and **this crate had already learned why once**.
`Expr::Field`'s comment says *"the question is about the printed text and not
the tree: a `Paren` around the `?` is dropped by `can_drop_paren`, so asking
the node would miss `(x?).name`."* The `Unary` arm asks the node. Looking
through a parenthesis makes `*(*x)` a fixed point and `**x` converge to it in
one pass.

That arm's own comment read *"No corpus case reaches it, so it is guarded here
rather than found later."* The port is the corpus case and this is later — which
is the general argument for this row, stated by the code it was porting.

### 10.8 What did not happen

**No new defect in the emitter, parser or lexer beyond finding 4.** Findings 2,
3 and 5 are surface gaps — spellings the language does not have, or resolves
one way without saying so — and each has a workaround in the port with the
reason beside it.

**Plan 68's nine gaps were re-sighted, not rediscovered.** The open-ended range
(`item_text/signature_line.sdx`), the multi-line `or` alternation
(`expr_text/argument/stands_alone.sdx`, `power_of.sdx`, `expr_text.sdx`) and
`matches!` (three sites) all cost a comment rather than an investigation, which
is what §2 predicted and the first measurable return on plan 68 having written
them down.

The multi-line `or` was the most expensive: `power_of`'s `match` listed
nineteen alternatives one per line with comments between them, and an `or`
alternation cannot be broken across lines. Flattening it would have been a
380-column line with the comments lost, so **one Rust arm became five**, split
along the reasons the comments already gave. `expr_text`'s thirteen-alternative
fallthrough split into two the same way.

**`clippy` needed one allow, not two.** `match_like_matches_macro`, for plan 64
§6's reason and with the same wording `redox-ast` carries. `no_effect` was not
needed: no `()`-returning function in this crate wanted plan 55 §1.6's unit
tail, so the rule contradiction plan 68 found did not recur here. It is still
owed a fix.

Six `needless_borrow` warnings were the partial translator being literal —
copying the Rust's own `&source[a..b]` into argument position where the method
would borrow for itself. Not a port finding.

One stale `-- TODO plan 68` survived above `PIPELINE_WIDTH` and only `clippy`
noticed it, as `empty line after doc comment`. The TODO sweep searched for
`TODO plan 69`.

### 10.9 Does the effort continue?

§1 said this row decides, because plan 68's value was its gaps rather than
self-hosting itself and a second port of the same shape would measure nothing.
The shapes were as different as claimed — 57 one-function files, no data
declarations, 125 `format!` calls against two — and the yield was **seven
findings against plan 68's twelve**, with two of them defects in the crate under
port and two of them fixed in Redox.

So the answer is yes, and the shape of the argument has changed. Plan 68's case
was *porting finds defects*. This row's case is narrower and stronger: **porting
a crate makes that crate's own corpus include shapes its authors never wrote**,
and findings 6 and 7 were unreachable without it. The next row should be chosen
for a shape neither of these two had — `redox-emit`, which is 98 files of
`match` over the same AST, or `redox-lexer`, which is the only crate with a
mutable state machine at its centre.

### 10.10 Owed to successors

| Owed | Why it is not here |
|---|---|
| A diagnostic for finding 2 — `.name` after a block-valued binding | `RDX0115` is correct about what it saw; the fix is a *parse* decision about whether a `Dedent` terminates a binding, which wants a row |
| A spelling for finding 3 — a pattern as a lambda parameter | Grammar, `redox-fmt`, tree-sitter and five editor clients, by plan 52's four-readers rule |
| Finding 4, the pipeline comment | A genuine defect, small, and in the lexer's continuation logic rather than here |
| A diagnostic for finding 5 — prefix operator in argument position | Silent today. An ambiguity resolved without a word is the class `AGENTS.md` keeps a section about |
| The `no_effect` / plan 55 §1.6 contradiction | Plan 68 found it; this row did not reach it, and it is still true |
