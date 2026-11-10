# Plan 57 — Methods as Functions

Plan ID: `methods-as-functions`
Depends on: 55 (`expression-orientation`), 56 (`currying`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

A method becomes a function. `.trim` is its name, the receiver is its last argument, and
`x.trim ()` is retired.

```
raw |> .trim |> .to_lowercase          -- instead of raw.trim ().to_lowercase ()
.write_str "ok" s                      -- s.write_str("ok")
map (.cmp y) xs                        -- partial application, over a method
```

`.` keeps exactly one job: field access. This is the largest migration in the milestone and the
row that makes plan 58's reverse-application `|>` possible at all.

### 1.1 Four separate defects share one cause

Measured while writing against the current surface. Each of these is a different symptom of
*a lone `()` is the empty argument list* — plan 20 §10.6's rule, correct in itself, colliding
with juxtaposition four ways:

| Written | Emitted | Caught by |
|---|---|---|
| `m.insert (k) (v)` | `m.insert(k)(v)` — curried | rustc `E0061` |
| `drain ().len ()` | `drain(().len())` | rustc `E0061` |
| `succeed ()` | `succeed()`, not `succeed(())` | rustc `E0061` |
| `(xs.get i)?.clone ()` | optional chaining, not `?` then a call | rustc, two types away |

Four rustc errors in generated Rust, from one surface rule. A method call that takes its
receiver as an ordinary last argument has none of them, because there is no `()` in the
spelling to be ambiguous about.

### 1.2 Receiver last, because that is the only order that works

`|>` feeds its left operand as the **last** argument (plan 58), and partial application supplies
arguments from the left (plan 56). Receiver-last is what makes both work on the same spelling:

```
x |> .cmp y          -- .cmp y x   -- x.cmp(y)
map (.cmp y) xs      -- a function awaiting a receiver
```

This is Haskell's data-last convention and it is the reason `map (.cmp y)` means *compare each
against `y`* rather than something that needs a lambda. Read bare, `.cmp y x` puts the receiver
in a surprising place; read in a pipeline, which is where it will almost always be read, it is
in the only place it could be.

### 1.3 The selector/method distinction, and the honest objection to it

`(.name)` is already doc 18 §4's field selector and `map (.age)` already works. If bare `.name`
is a method, the two have to be told apart:

```
map (.age) users        -- field selector
raw |> .trim            -- method call
map (.cmp y) xs         -- a method, parenthesised
```

**The discriminator is a `)` immediately after the name, not a parenthesis.** This row's first
draft said "the two spellings differ by parentheses", and §11.3 is the correction: criterion 3's
`map (.cmp y)` is parenthesised *and* a method, so parenthesisation cannot be the test. It is a
one-token lookahead — `(.age)` closes, `(.cmp y)` does not.

Today `(.cmp 1)` is a hard `RDX0101`, because `section.rs:44` uses `p.expect(...)?` and
propagates where its siblings at `:83`, `:89` and `:105` rewind. Leaf 2 has to convert that to a
rewind, which is a smaller change than the rule's first statement implied and a different one.

**It remains a one-token semantic difference, which is the hazard class plan 52 removed from
`::`.** It is adopted anyway, and the trade should be stated rather than discovered:

- the alternatives were a new sigil for methods (`..trim`), or losing `(.age)` and writing
  `\u -> u.age`;
- `(.age)` is specified, exercised, and good, and doc 18 §4 retired the `&:name` sigil to get it;
- Redox has no type checker, so it cannot tell a field from a method, and *something* must carry
  the distinction syntactically.

The mitigation is a diagnostic rather than a hope: a parenthesised `(.name)` used where a method
is required, or a bare `.name` used as a selector, fails in rustc today and gets a named Redox
code in leaf 5 instead. §6 records this as the row's one internal inconsistency.

### 1.4 `?.` is deleted

Doc 18 §8 keeps `?.` among constructs retained *"because deleting **them** would cost
reachability or ergonomics with nothing to show"*. That reasoning no longer holds: with `|>` as
reverse application and methods callable as functions, the chain is expressible — and doc 02
§6.3's lowering is already a `?`-closure that lets **rustc** perform the `and_then`/`map`
unification, so the chain was always there and `?.` was the spelling.

**"In the ordinary combinator vocabulary" is what this said, and §11.4 falsifies it for exactly
the case criterion 6 names.** A field selector *moves*: `(.name)` lowers to
`|__rdx_1| __rdx_1.name` with no `&`, so

```text
ps |> into_iter |> map (.tag)
->  ps.into_iter().map(|__rdx_1| __rdx_1.tag)
->  E0507: cannot move out of `__rdx_1.tag` which is behind a shared reference
```

and `examples/fallible.sdx`'s chain is over a `newtype Tag = Tag String` chosen precisely
because it is not `Clone`. Two replacements do compile, and **both need an explicit `&` written
by hand**:

```text
u.as_ref () |> and_then (\x -> x.profile.as_ref ()) |> map (\p -> &p.tag)
try / Some (&u.as_ref ()? .profile.as_ref ()? .tag)
```

Leaf 4 owes a choice between them. The chain is expressible; it is not expressible *without
thinking about borrows*, which is a different and smaller claim than this paragraph made.

(Amending that paragraph will also fix a count: §8 says *"Four Redox constructs are kept
unchanged"* and then lists seven. Independent of `?.`, and leaf 7 should not leave it.)

Against keeping it: `?.` is one token, so `(e)?.m ()` silently means optional chaining rather
than `?` followed by a method call. In the sixteen-item survey that opened this milestone, that
case sits in the **deferred** class — Redox accepts it and rustc rejects the result two types
away — which is four of the sixteen; one further case is caught by nothing at all. Five of
sixteen reach no Redox diagnostic at the point of the mistake, and this is one of them.

Deleting a construct is the strongest possible fix for a construct's ambiguity, and it is
available here because the replacement is not a workaround — it is the lowering.

### 1.5 What the migration costs, measured

| Thing | Count |
|---|---|
| Method calls in `examples/` | **52** |
| `.sdx` files touched | **17** of 29 — 14 hold all 52 call sites, 3 more are pipeline-only |
| Goldens to re-bless | 28 (plus 24 `examples/*.xrs` and their `.map` siblings) |
| `?.` sites | 6, all in `examples/fallible.sdx` |

Unlike plan 52, **the generated Rust does change** — the receiver moves from a `.` chain into an
argument position and back out again — so goldens are not comment-only here. The compiled
behaviour must not change, which is what the run corpus asserts.

Hand-migrated, per §9. 52 sites is small enough that a rewriter would cost more than it saves,
and doc 04's fourth ordering commitment plus `no_literal_expectations.rs` mean there are no
fixtures hiding in string literals.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**All four §1.1 defects reproduce.** Each was hit writing real programs, not constructed:

```
m.insert ("endpoint".to_string ()) ("https://…".to_string ())
->  m.insert("endpoint".to_string())("https://…".to_string())
->  E0061: this method takes 2 arguments but 1 argument was supplied

drain ().len ()     ->  drain(().len())
succeed ()          ->  succeed()      // E0061: argument #1 of type `()` is missing
(self.tokens.get (self.at))?.clone ()
                    ->  (|| Some(&self.tokens.get(self.at).as_ref()?.clone))()()
```

**Unparenthesised juxtaposed method arguments *do* group correctly.** `RATE_CALLS.fetch_add 1
Ordering::SeqCst` emits `RATE_CALLS.fetch_add(1, Ordering::SeqCst)`. So the defect is specific
to parenthesised arguments, which are separate applications.

**A field access cannot be a parenless argument.** `self.tokens.get self.at` is `RDX0101`,
because `f a.b` is ambiguous. An index can: `parts[0]` works.

**`(.name)` and `(|> name)` both work**, per doc 18 §4 and `examples/section.sdx` — the field
selector and the method reference, which is the distinction this row generalises.

**Corpus scale.** 52 method calls across 703 code lines in 29 examples.

---

## 3. Concrete proof this plan targets

**All four of §1.1's defects become unspellable**, and the programs that worked around them are
rewritten without the workarounds.

Six cases:

1. **A two-argument method.** `.insert k v m`, and `m |> .insert k v` — the case that curries
   today.
2. **A method as a partially applied value.** `map (.cmp y) xs`, which needs plan 56.
3. **A field selector beside a method, in one expression.** `users |> map (.name) |> map .len` —
   the §1.3 distinction, exercised where a reader will meet it.
4. **A chain four stages long**, spanning lines, which is the shape the whole row is for.
5. **`?.` deleted**, with `examples/fallible.sdx`'s chain rewritten and its *borrowing* property
   preserved — plan 28 made that file's `Tag` deliberately non-`Clone` so a cloning lowering
   fails to compile, and that assertion must survive the rewrite.
6. **A unit argument.** `succeed (())` is no longer needed because the construct that required it
   is gone; the replacement spelling is asserted.

---

## 4. Leaves

### Leaf 1 — `leaf-method-as-function`

`.name` in head position is a function; the receiver is the last argument. Emission is
`receiver.name(args…)`, which is purely syntactic — no resolution, no types.

**`emit/pipeline.rs` is part of this leaf**, which this list omitted. `x |> .trim` is `RDX0102`
today, so criterion 2 cannot be met without it, and §11.1 records the trap: that file's
`other =>` arm appends `()`, so a zero-argument `x |> .name` must be matched **before** it or
the lowering yields `x.(|__rdx_1| __rdx_1.name())()`. Plan 58 leaf 2 also claims the file, so it
is edited twice across the two rows.

### Leaf 2 — `leaf-selector-vs-method`

The rule from §1.3: a `)` immediately after the name selects a field, anything else calls a
method. One parser decision, made on one token of lookahead — and the mechanical part of it is
converting `section.rs`'s propagating `expect` into a rewind, so `(.cmp y)` stops being
`RDX0101`.

### Leaf 3 — `leaf-retire-method-call`

`x.f ()` and `x.f (a)` removed. A diagnostic recognises the old form and names the new one —
every program written before this row is full of them, and the error message is the migration
guide.

**There are two AST shapes for a method call and this leaf only retires one**, which §11.2
measured. `x.f (a)` is `Expr::MethodCall`; `x.f a` is `Expr::Call { callee: Expr::Field }`,
because `postfix.rs:19-27` deliberately removed the paren-call step. Deleting the `MethodCall`
branch therefore does **not** retire `x.f (a)` — it re-parses as the second shape and emits the
same `x.f(a)` silently, and `x.f ()` degrades to `x.f(())` rather than to an error. The
diagnostic has to recognise *application over a `Field`*, not a branch in `postfix`.

### Leaf 4 — `leaf-delete-optional-chaining`

`?.` removed from the lexer and the parser; its lowering deleted. `examples/fallible.sdx`
rewritten with the borrowing assertion intact (§3 case 5).

### Leaf 5 — `leaf-misuse-diagnostics`

The §1.3 mitigation: a parenthesised selector where a method is wanted, and a bare method where a
selector is wanted, each get a named code rather than a rustc type error.

### Leaf 6 — `leaf-corpus-migration`

All 29 examples, 52 sites, hand-migrated and re-read. Goldens re-blessed; the **run** corpus is
the real assertion here, because the generated Rust changes and the behaviour must not.

### Leaf 7 — `leaf-docs`

Doc 18 §4's table regraded — `(.name)` and `(|> name)` become one rule with this row's
generalisation. Doc 18 §8's `?.` justification struck with the reason (§1.4). Doc 03 §12.3's
borrow claim re-sited onto the new spelling. `docs/GRAMMAR.md` and `docs/STATUS.md` updated.

---

## 5. Acceptance criteria

1. `.name args receiver` emits `receiver.name(args)`, for zero, one and two arguments.
2. `x |> .name args` works, and reads left to right across lines.
3. `map (.cmp y) xs` works — a method partially applied, which requires plan 56.
4. `(.name)` still selects a field; bare `.name` calls a method; both in one running example.
5. `x.f ()` is a named Redox diagnostic naming the replacement.
6. `?.` is not a token; `examples/fallible.sdx` keeps its non-`Clone` assertion and still
   compiles, which is what proves the chain still borrows.
7. Misuse of selector-versus-method is a Redox diagnostic, with `tests/ui/` fixtures both ways.
8. All 29 examples transpile, compile with no rustc output, run, stdout asserted — **and the
   stdout is unchanged from before the migration**, which is this row's definition of
   behaviour-preserving.
9. `construct_coverage` green with `.name` indexed. (`?.` is not a `docs/GRAMMAR.md` row today —
   that inventory grades *Rust* constructs — so there is nothing to remove; plan 51's index is
   the thing that gains the entry.)
10. Doc 18 §8's `?.` reasoning is amended rather than left standing.

---

## 6. Out of scope, and what it discloses

**The §1.3 parenthesis distinction is this row's one internal inconsistency.** Plan 52 removed a
whitespace-carried semantic difference and this row adds a parenthesis-carried one. The
justification is that Redox has no types and something must carry it; the mitigation is leaf 5's
diagnostics; and the honest statement is that this is a worse rule than the rest of the milestone
produces. Recorded here so a future row proposing `..name` has the argument available rather than
having to reconstruct it.

**Rust's method-call syntax is gone from the surface and remains in the output.** A reader of a
`.xrs` sees `s.trim()`; a reader of the `.sdx` sees `s |> .trim`. That widens the
source-to-generated gap, which doc 04's second ordering commitment cares about — but the source
map is per-span and the receiver's span is preserved through the move, which is what leaf 1 must
assert.

**`?.` is deleted, not deprecated.** Unlike plan 53's comment pragma, there is no window. The
construct is ambiguous and its replacement is exact; keeping both would mean keeping the
ambiguity for the sake of code that can be mechanically identified.

**No UFCS.** `.name` resolves to a method on the receiver, and nothing here makes a free function
callable in method position or vice versa. The rejected alternative — resolve `f x` to `x.f()`
when no function `f` exists — would make meaning depend on what is in scope, so that adding an
import could silently change a call. That is why `.name` is marked.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with **no output**, runs, and its stdout is
**byte-identical to the pre-migration stdout**; goldens re-blessed and reviewed; `tests/ui/`
fixtures for the retired call form and both misuse directions; `construct_coverage` green;
`formatting_preserves_meaning` green.

---

## 8. Emerald reuse

Doc 02 §4.5's *"ship the migration as a tool, not as instructions"* is the sentence this row
most wants, and **it does not get it**: §1 and §9 decision 5 both say hand-migrate, having
declined a rewriter. §8 claimed the formatter as the migration tool "for the third time in this
milestone" and §11.6 caught the contradiction; decision 5 wins, because it is the decision.

What the formatter *is* here is the check rather than the tool.
`crates/redox-fmt/tests/properties.rs`'s `formatting_preserves_meaning` and
`every_example_is_already_canonical` run over the whole corpus, so a hand-migration that
changes a program fails before its golden does — which is how plan 51 found that `redox fmt`
was deleting `$`. Emerald's plan 71 cutover missed 113 fixtures embedded as string literals in Rust
test modules, and doc 04's fourth ordering commitment plus `no_literal_expectations.rs` are
Redox's standing answer.

The nearest prior art is doc 02 **§4.6**, *"The parenless-call ambiguity — Emerald already did
Redox's homework"*, which records that Emerald's `puts x` worked *"only because `puts` was a
hardcoded grammar keyword"*. That is about application, not method syntax, so there is no
transferable decision here — and §3's insistence on asserting unchanged stdout rather than
reasoning about equivalence is the compensation.

---

## 9. Decision log

1. **Retire `x.f ()` rather than keep both.** Two call syntaxes for one operation is the
   redundancy this milestone removes, and keeping the method form keeps all four §1.1 defects.
2. **Receiver last.** §1.2. Receiver-first would require `|>` to special-case `.name`, which
   breaks the uniformity that is the reason for having the operator.
3. **Parentheses carry the selector/method distinction.** §1.3, and §6 records the cost. The
   alternatives lose `(.age)` or add a sigil; this one is the least bad, not good.
4. **Delete `?.` in this row rather than its own.** It is the same subject — what `.` means — and
   separating them would leave `?.` ambiguous for a milestone with no benefit.
5. **Hand-migrate 52 sites.** A rewriter is justified at plan 52's 184 and was declined there
   too; the corpus exists to be read.
6. **Assert unchanged stdout, not equivalent behaviour.** The generated Rust changes here, so the
   golden is no longer a proof of behaviour. The run corpus is, and criterion 8 says so in the
   strongest available form.

---

## 10. Result

*Not yet executed.*

---

## 11. Corrections from orientation

Written after reading the code this row targets. The mechanism is sound; the
*rule* §1.3 states is wrong, the counts are low, two preconditions are false,
and criterion 6 is much harder than it reads. The single most useful finding
answers the sequencing question this row and plan 58 share.

### 11.1 Plans 57 and 58 are independent, in that order

**57 before 58, and 57 need not assume either meaning of `|>`** — because the
two meanings emit identical Rust for a `.name args` right operand:

| | today's rule (`emit/pipeline.rs:26-46`) | plan 58's reverse application |
|---|---|---|
| `x \|> .cmp y` | RHS dismantled: head → method, LHS → receiver ⇒ `x.cmp(y)` | `.cmp y` awaits its last argument ⇒ `x.cmp(y)` |

The reverse order is impossible, which is plan 58 §1.2's own argument: every
live pipeline step is a method.

Two things follow that neither plan says. **`emit/pipeline.rs` is missing from
§4's leaves** and criterion 2 cannot be met without it — `x |> .trim` is
`RDX0102` today. Plan 58 leaf 2 claims that same file as part of its
three-file radius, so it is edited twice and this row under-declares. And the
implementation trap: `emit/pipeline.rs:44`'s `other =>` arm appends `()`, so a
zero-argument `x |> .name` must be matched **before** it or the lowering yields
`x.(|__rdx_1| __rdx_1.name())()`.

There is also a **transient redundancy nothing discloses**. After 57 and before
58, `xs |> map f` (22 live corpus sites) and `xs |> .map f` both work and mean
the same thing. This milestone's anti-redundancy rule is suspended for exactly
one plan; §6 should say so rather than leave it to be discovered.

### 11.2 There are two AST shapes for a method call, and leaf 3 only retires one

- `x.f (a)` → `Expr::MethodCall { recv, name, args }` (`postfix.rs:93-100`).
  The receiver **is** distinguished here.
- `x.f a` → `Expr::Call { callee: Expr::Field, args }`, because
  `postfix.rs:19-27` deliberately removed the paren-call step. Verified:
  `s.find "h"` emits `s.find("h")`; `h.ts.get h.at` emits `h.ts.get(h.at)`.

So deleting the `MethodCall` branch does **not** retire `x.f (a)` — it
re-parses as `Call{callee: Field}` and emits the same `x.f(a)` silently, and
`x.f ()` degrades to `x.f(())` rather than to an error. Criterion 5's
diagnostic has to live on the application-over-`Field` shape, not in `postfix`.

The discriminator for a method, verified, is `p.check(&TokenKind::LParen)` at
`postfix.rs:93` and nothing else — it is **not** adjacency-aware despite the
comment above it saying "immediately after", so `x.f ()`, `x.f()` and `x.f  ()`
are all `MethodCall`. `x.f` with no parens is `Expr::Field`, emitted verbatim.

### 11.3 §1.3's rule is wrong, and §2 has a false measurement

**The rule.** §1.3 says the two spellings "differ by parentheses". They cannot:
criterion 3's `map (.cmp y)` is parenthesised *and* a method. The real
discriminator is **`)` immediately after the name**. Today `(.cmp 1)` is a hard
`RDX0101` — *"expected `)` to close the section"* — because `section.rs:44`
uses `p.expect(...)?` and propagates where `:83`, `:89` and `:105` rewind. Leaf
2 must convert that to a rewind. §6's "one internal inconsistency" is
understated: it is a one-token-lookahead difference, not a parenthesis one.

**The measurement.** §2 claims *"A field access cannot be a parenless argument.
`self.tokens.get self.at` is `RDX0101`, because `f a.b` is ambiguous."*
Probed: `h.ts.get h.at` builds clean and emits `h.ts.get(h.at)`; `id h.at`
emits `id(h.at)`. **False for plain receivers.** It may hold inside a
`class`/`instance` body where `self`/`@` sugar applies — if that is where the
measurement came from, say so rather than generalising. Unsettled.

**And `(.name)` and `(|> name)` are both fully implemented.** `section.rs:37-50`
and `:53-65`; `emit/section.rs:27-28` emits `|__rdx_1| __rdx_1.name` and
`|__rdx_1| __rdx_1.name()`; `examples/section.sdx` exercises both with a
`.stdout`, and `benchmarks/pairs/section.sdx` pairs them. In particular
`examples/lexical.sdx:41`'s `method_ref = (|> name)` **parses cleanly** — that
file's exemption comes from lines 25, 28 and 40 only. Any claim that it is
unparsed is false.

What happens to `(|> name)` is unassigned: plan 58 leaf 2 says it "becomes a
different construct and is plan 57's", and this row's leaf 7 only regrades a
doc table. Since `(|> len)` and a bare `.len`-as-value would have the same
lowering, this row makes one of them redundant and should say which goes.

### 11.4 Criterion 6 is the hardest criterion, and §1.4's reasoning is false

The risk is not evaluation order. It is that **a field selector moves**.
`(.name)` lowers to `|__rdx_1| __rdx_1.name` with no `&`. Probed:

```text
ps |> into_iter |> map (.tag)
->  ps.into_iter().map(|__rdx_1| __rdx_1.tag)
->  E0507: cannot move out of `__rdx_1.tag` which is behind a shared reference
```

So §1.4's claim that the `?.` chain "is expressible in the ordinary combinator
vocabulary" is false for exactly the case criterion 6 names. Two replacements
do work, both compiled clean, and both need an **explicit `&` written by
hand**:

```text
u.as_ref () |> and_then (\x -> x.profile.as_ref ()) |> map (\p -> &p.tag)
try / Some (&u.as_ref ()? .profile.as_ref ()? .tag)
```

Leaf 4 should name which, and §1.4's sentence should be struck. Also missed by
leaf 4: `benchmarks/pairs/optional_chain.sdx` + `.rs` is a plan-44 IR-equality
pair built on `?.`, so deleting the construct deletes a zero-cost assertion.
And the `?.` deletion radius is wider than "the lexer and the parser" — eleven
source files reference `QuestionDot`/`OptionalChain`, four of them in
`redox-fmt`, plus the benchmark pair.

### 11.5 The corpus is half again as large as §1 says

Counted by regex over comment-stripped sources, with every match in the top
five files hand-checked:

- **`examples/`: 55 sites across 15 files**, not 52 across 14 — `fallible` 15,
  `comprehension` 6, `arrays` 6, `class` 5, `errors` 4, `application` 3,
  `module` 3, then nine files with one or two.
- **Plus 12 sites in 8 files §1 does not count at all**, including
  `crates/redox-fmt/tests/fmt/expressions.sdx` and its
  `expressions.canonical.sdx` (3 each) — those are *canonical-form*
  assertions, so the formatter's own output changes — two lint fixtures, a
  `tests/ui/` case, `tests/unreachable/macro_call.sdx`, and two benchmark
  pairs.
- **Real total: 67 sites across 23 files.**
- `?.`: **6, all in `examples/fallible.sdx`** — §1 is right.
- `|>`: 26 occurrences in examples minus two `(|> name)` sections =
  **24 operators**, 22 in the run corpus. Plan 58's "23" is off either way.
- `tests/golden/` holds **no `.sdx`** — goldens are `.xrs` paired against
  `examples/*.sdx`, so §1's "28 goldens plus 24 `examples/*.xrs`" needs
  rechecking against that.
- There are **30** `examples/*.sdx` and 29 `.stdout`; "29 examples" is right
  only as the run corpus.

The `UnwrapUsed` lint is lexical, so it survives the respelling, but the two
lint fixtures still change.

### 11.6 Smaller corrections

- **§8 contradicts §9 decision 5.** §8 says "plan 34's formatter as the
  migration tool, for the third time in this milestone"; §1 and decision 5 say
  hand-migrate and that a rewriter was declined. Pick one.
- **Criterion 3's dependency on plan 56 is over-claimed.** Receiver-last makes
  the missing-argument count always exactly one, so `map (.cmp y)` is
  `emit/section.rs:28`'s shape with arguments threaded — no signature
  visibility, no arity resolution. The dependency is for sharing one
  closure-generation path, not for capability, and should say so.
- **§1.1's four defects do not share one cause.** `s.find ("x") ("y")` is
  `postfix.rs:93` taking the first paren group; `mk ().len ()` is postfix
  binding tighter than application; `succeed ()` is a lone `()` reading as the
  empty argument list; the `?.` case is `"?."` being one token
  (`operator_table.rs:27`). Three distinct causes, and the fourth is worse than
  described — it silently nests a closure and a spurious `as_ref()` per link.
- **Criterion 9 needs a new witness, not just an index entry.**
  `docs/GRAMMAR.md:100` does carry the method-call row, and its note
  ("adjacency decides a call from a section") becomes false under this row. But
  `construct_coverage.rs`'s witness for it is `Toks(["«id»", ".", "«id»"])`
  plus a left section — **field access only** — so it would stay green through
  the entire migration without ever witnessing a method call.
- **Diagnostics: next free is `RDX0110`**, and this row needs three
  (criterion 5, plus criterion 7 both ways) → 0110/0111/0112. The fixture path
  is **`crates/redox-cli/tests/ui/`**, not `tests/ui/` as criterion 7 says.
  The 0040–0099 band is reserved for doc 03 §17 and gated by a hard-coded
  `PUBLISHED` list, so the syntactic band is correct.
- **Doc 18 §8's passage is at `:659-663`**, and §1.4's miscount complaint is
  right: it says "Four" and lists seven. §4's table rows for `(.name)` and
  `(|> name)` are at `:409-410`; the paragraph leaf 7 rewrites is `:443-447`.
- **tree-sitter and the editors are unlisted work.**
  `crates/redox-cli/tests/tree_sitter.rs:70` asserts every example parses with
  no `ERROR` node — `lexical.sdx` included — and `tree-sitter-redox/grammar.js:711-715`
  is where `.`-sections live. Five editor clients' `highlights.scm` are
  cross-checked by `crates/redox-cli/tests/editors.rs`. None of this appears in
  §4 or §5; its size is unmeasured.
- **Unsettled: the formatter's canonical form for `.name args receiver`.**
  `MethodCall` is referenced in four `redox-fmt` files. If the receiver is a
  bare name, `.name a b` and `(.name a) b` both re-print plausibly, and
  `can_drop_paren.rs` has to decide. `formatting_preserves_meaning` is the gate
  that will catch a wrong choice.
