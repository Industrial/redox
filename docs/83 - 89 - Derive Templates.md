# Plan 89 — Derive Templates

Plan ID: `derive-templates`
Depends on: 64 (`macro-invocation`), 87 (`pattern-abstraction`)
Milestone: 11 — *expressive delta*
Status: `done` — scalar substitution and leaf 3's repetitions both shipped 2026-11-11; §10.3 records why the first withdrawal was wrong

---

## 1. What this plan is

`deriving` becomes extensible by a **template** declared in the file, instantiated textually
against a declaration's own text.

```
deriving template Forward
  instance Display $Type
    fmt : &self -> &mut Formatter -> fmt::Result
    fmt self f = Display::fmt &self.$field0 f

newtype Meters = Meters f64
  deriving (Debug, Forward)
```

Rust cannot do this without a **separate proc-macro crate**, which is the capability gap: a
derive needs `syn`, `quote`, its own `Cargo.toml`, its own compilation unit, and cannot be local
to the file that uses it. A template needs none of those because it is substitution, not a
program.

### 1.1 The ambitious version of this row is already declined, and the decline is correct

The first form this row took was *derive rules written in Voltaic and executed at transpile time*.
That is `comptime`, and `docs/DECLINED.md` refuses it:

| Construct | From | Reason |
|---|---|---|
| Transpile-time evaluation (`comptime`) | Zig, Emerald | requires an interpreter in the compiler (doc 03 §13.3) |

A user-written derive rule that *runs* is an interpreter in the compiler, exactly as stated, and
nothing about calling it a derive changes that. So the ambitious version is refused, not deferred,
and this row is the narrower construct that survives: a template with a **closed substitution
vocabulary** and no evaluation.

The line between them is sharp and worth stating, because it is what makes this admissible:

| | Reads | Computes | Admissible |
|---|---|---|---|
| A template | the declaration's text | nothing | yes |
| A `comptime` rule | the declaration's text | arbitrarily | no — doc 03 §13.3 |

### 1.2 The substitution vocabulary is closed, and that is the whole design

Five substitutions, fixed by the compiler, with no way to add a sixth from source:

| Form | Expands to | Available for |
|---|---|---|
| `$Type` | the declaration's type name | `data`, `newtype` |
| `$field0` … `$fieldN` | the *n*th field's name, or its index for a tuple struct | single-constructor declarations |
| `$fields` | a repetition over field names | single-constructor declarations |
| `$variants` | a repetition over variant names | `data` with more than one constructor |
| `$Self` | `Self`, for use inside the generated `instance` | all |

A repetition is a loop with no body language — the template text inside it is emitted once per
item with the substitutions resolved. That is `macro_rules!`'s `$(…)*` and nothing more. There is
no condition, no arithmetic, no recursion and no way to ask a question about a field, because each
of those is the interpreter §1.1 refuses.

A template that wants a sixth substitution is a request for a new row, not a thing the author can
work around — and that is the property that keeps this construct from becoming `comptime` by
accretion.

### 1.3 Admission, clause by clause

- **Type-free.** Every substitution reads the declaration's **source text**: its name, its field
  names, its variant names. No type is resolved. This is the same information doc 03 §9.5's
  `deriving` already uses to emit `#[derive(Debug)]` and that doc 03 §9.2's field punning already
  reads.
- **No implied dependency.** The expansion names only what the template's author wrote. If a
  template names `Display`, the user's file must import it — the template does not.
- **Semantics-preserving.** The expansion is an `instance` declaration, which lowers through the
  existing `instance` path to an ordinary `impl`. Nothing is inserted; no clone, no reborrow.
- **Reachability-positive.** New expressiveness, and it closes nothing it did not open.

### 1.4 It fabricates API, and §8.7's objection has to be answered

Doc 03 §8.7 declines default arguments because *"a synthesised wrapper fabricates API surface the
user never wrote, which breaks trait implementations, rustdoc and go-to-definition."* A derive
template fabricates an `impl`. The objection has to be met, not sidestepped.

**It is met by `deriving` already doing this.** `deriving (Debug)` fabricates an entire `Debug`
impl today, and so does Rust's `#[derive]`; the surface anyone writes in either language is full
of API nobody typed. The objection in §8.7 is specifically about *fabricating a signature that
shadows the one the author wrote* — a wrapper standing in front of the real function, so that
rustdoc and go-to-definition point at the wrapper. A derive adds an impl beside the declaration
and shadows nothing.

What is genuinely worse here than with `#[derive]` is **discoverability**: `Debug` is standard and
`Forward` is one file's invention. §6 discloses it and criterion 6 is the mitigation — the
generated `impl` carries a provenance comment naming the template, and comments reach the `.xrs`
(doc 02 §10.1), so the `.xrs` says where the impl came from.

### 1.5 Scope is the file

A template is usable where its declaration is visible in the file being compiled, which is plan 56
§1.4's boundary and plan 14 decision 4's refusal: *"Voltaic has no name resolution and must not grow
one."*

An unknown name in a `deriving` list is already something — doc 03 §9.5 passes it to
`#[derive(…)]` — so an undeclared template is **not** an error at the Voltaic level. It becomes
`#[derive(Forward)]` and then rustc's `E0433`. That is a worse message than a Voltaic diagnostic
would be, and criterion 5 pins it rather than assuming it is acceptable; §9 decision 4 records why
a diagnostic is not available.

---

## 2. Preconditions to verify

**Nothing in this section is measured yet.** The row is authored, not executed.

```bash
cargo run -p voltaic-cli -- build /tmp/probe.sdx && cat /tmp/probe.xrs
cargo run -p voltaic-cli -- check /tmp/probe.sdx
```

1. **How `deriving` is parsed and emitted today**, and whether its list is a set of names passed
   straight to `#[derive(…)]`. §1.5 depends on the answer.
2. **What the AST carries for a `data` or `newtype` declaration** — specifically whether field
   names and variant names are available at emit time, since every substitution in §1.2 reads
   them. Doc 03 §9.2's field punning suggests they are; measured rather than inferred.
3. **Where `instance` is emitted**, because a template's expansion must enter that path and not a
   parallel one.
4. **Whether `voltaic-fmt` round-trips a template declaration.** Plan 56 §11.3's rule: the expansion
   must be on the emit side, or `voltaic fmt` rewrites the user's `deriving (Forward)` into the
   expanded impl.
5. **That `$` is free in this position.** `$` is the application operator (doc 18 §4, plan 56 §2)
   and is hardcoded in the Pratt loop at binding power 1. Inside a template body it would have to
   mean substitution instead, which is a second meaning for a live operator — the exact shape
   plan 60 §1.1 counted five of for `=`. Precondition 5 is whether position separates them, and
   §9 decision 3 is the fallback if it does not.

---

## 3. Concrete proof this plan targets

**`Meters` from §1 compiles with a working `Display` impl that no one wrote, runs, and prints.**

Six cases, each failing differently:

1. **A single-field `newtype`**, using `$Type` and `$field0`.
2. **A multi-field `data`**, using `$fields` — the repetition, which is where a substitution-only
   implementation passes case 1 and fails.
3. **An enum**, using `$variants`.
4. **Two templates on one declaration**, plus a built-in: `deriving (Debug, Forward, Tally)`.
5. **A template whose body names something not imported**, asserted as a **rustc** error, per
   §1.3's no-implied-dependency claim.
6. **An undeclared name in a `deriving` list**, asserted as rustc's `E0433` in `tests/ui/`, per
   §1.5.

And one that must **not** change: `deriving (Debug)` emits byte-identical Rust.

---

## 4. Leaves

### Leaf 0 — `leaf-substitution-probe`

Measure preconditions 2 and 5. If field and variant names are not at emit time, or if `$` cannot
be separated, this leaf resizes or stops the row before leaf 1 designs around a wrong answer.

### Leaf 1 — `leaf-template-declaration`

`deriving template Name` and its indented body, stored verbatim with its substitution points
located. Declaration only — no expansion.

### Leaf 2 — `leaf-scalar-substitution`

`$Type`, `$field0`…`$fieldN`, `$Self`. Cases 1 and 4 of §3.

### Leaf 3 — `leaf-repetitions`

`$fields` and `$variants`. Separate leaf because a repetition is the only part of §1.2 that is not
a direct replacement, and case 2 is its proof.

### Leaf 4 — `leaf-provenance-comment`

The generated `impl` carries a comment naming its template. §1.4's mitigation, criterion 6's
assertion.

### Leaf 5 — `leaf-fmt-round-trip`

`voltaic fmt` prints the template and the `deriving` list, not the expansion. Precondition 4's
hazard.

### Leaf 6 — `leaf-corpus-and-docs`

`examples/deriving.sdx`. Doc 03 §9.5 gains the template specification; `docs/GRAMMAR.md` §1.2's
attributes row gains a note, since `deriving` is the one place a Voltaic declaration emits an
attribute; `docs/STATUS.md` regenerated. **`docs/DECLINED.md`'s `comptime` row gains a
cross-reference to §1.1**, because this row is the nearest admissible neighbour of a declined
construct and a reader who finds one should find the other.

---

## 5. Acceptance criteria

1. Each §3 case transpiles, compiles with no rustc output, runs, and its stdout is asserted.
2. The repetition cases (2 and 3) work, not only the scalar ones.
3. A golden shows the template in the `.sdx` and the expanded `impl` in the `.xrs`, so the
   substitution is visible in the corpus rather than asserted in prose.
4. Case 5 is a **rustc** error, captured in `tests/ui/` — the template imports nothing.
5. Case 6 is rustc's `E0433`, captured in `tests/ui/` and asserted to be rustc's rather than
   Voltaic's, per §1.5.
6. Every generated `impl` carries a provenance comment naming its template, asserted in the
   golden.
7. `voltaic fmt` round-trips a template declaration and a `deriving` list unexpanded.
8. `deriving (Debug)` and every existing `deriving` emits byte-identical Rust; `tests/golden/`
   re-runs with **no re-blessing** of existing files.
9. A template using a substitution that does not exist — `$fieldX`, `$whatever` — is a named Voltaic
   diagnostic with a `tests/ui/` fixture. The vocabulary is closed (§1.2) and the closure must be
   enforced, or it will be extended by accident.
10. All examples green; `construct_coverage` green with no new `ALLOW` entries.

---

## 6. Out of scope, and what it discloses

**This is the weakest row in milestone 11, and the register should say so.** Its capability gain
over `#[derive]` is real but narrow — a derive without a proc-macro crate — and its gain over
simply writing the `instance` by hand is only repetition. Four declarations that need the same
forwarding impl is where it pays; one is not.

**No whole-module generation.** The pitch that motivated this row was generating a `Visitor` from
*every* `data` declaration in a file — the thing no proc macro can do, because a macro sees one
item. That requires iterating declarations and choosing shapes, which is the interpreter doc 03
§13.3 declines, and §1.1's table is the line. It is named here because it is the version worth
wanting and a future row may find a template-shaped way to reach it.

**No conditions, no arithmetic, no recursion** (§1.2). A template cannot ask whether a field is
`Copy`, cannot count, and cannot call another template.

**Discoverability is worse than `#[derive]`'s** (§1.4). The provenance comment is a mitigation,
not a fix; go-to-definition on a template-generated method lands in the `.xrs`.

**An undeclared template is rustc's error, not Voltaic's** (§1.5), and criterion 5 pins how bad the
message is.

**`$` carries a second meaning if precondition 5 allows it**, and a third thing for a reader to
hold. §9 decision 3 is the alternative if it does not.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixtures for criteria 4, 5 and 9; goldens asserted unchanged per criterion 8;
`construct_coverage` green.

---

## 8. Emerald reuse

**Negative, and it is the reason this row is as small as it is.** `docs/DECLINED.md` attributes
`comptime` to *"Zig, Emerald"* — Emerald had transpile-time evaluation and Voltaic declined to
repeat it, on the ground that it *"requires an interpreter in the compiler"*.

Emerald's other measurement bears on the same decision from the other side: 112 of 203 plans spent
on a standard library. A template language with conditions and arithmetic is how a compiler grows
a second language inside itself, and the cost shows up the same way — as plans. §1.2's closed
vocabulary is the cheap defence: there is nothing to extend without a row, so growth is visible in
the register rather than in the template corpus.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | Declarative templates | Derive rules written in Voltaic and executed at transpile time | The executed form is `comptime`, declined by `docs/DECLINED.md` for requiring an interpreter in the compiler. §1.1 |
| 2 | A closed substitution vocabulary | An open one, extensible from source | An open vocabulary becomes a language, and a language becomes an interpreter. Closed means growth needs a register row |
| 3 | `$` for substitution if position allows | A distinct sigil such as `%` or `@` | `$` is the familiar spelling, but it is live as the application operator at Pratt binding power 1, and plan 60 §1.1 counted the cost of one token meaning five things. If precondition 5 finds no positional separation, this reverses to a distinct sigil, which is cheap because it is one character in one position |
| 4 | No diagnostic for an undeclared template | A named Voltaic diagnostic | Doc 03 §9.5 passes an unknown `deriving` name through to `#[derive(…)]`, so Voltaic cannot distinguish a misspelled template from a real Rust derive macro without name resolution — plan 14 decision 4's refusal. The same undecidability plan 56 §11.1 found for arity |
| 5 | A provenance comment | Nothing, or a `#[doc]` attribute | Comments are trivia that reach the `.xrs` by doc 02 §10.1, so the comment costs nothing and survives to the artifact a person reads. An attribute would be API the author did not write |

---
## 10. Result

**Shipped partially**, 2026-11-11. The scalar substitution works end to end;
**leaf 3's repetitions are withdrawn**, so §3 cases 2 and 3 are not met and
criterion 2 fails on purpose. Status is `partial`, which is the same verdict
plans 87 and 88 carry and for the same structural reason: the leaf that was cut
is the one that needed a construct the row does not have.

`examples/template.sdx` declares two templates and derives them across two
`newtype`s, and `tests/golden/template.xrs` holds the three generated `impl`s
beside the `#[derive(Debug, Clone, Copy, PartialEq)]` the same `deriving` list
produced. `examples/template.stdout` asserts what it prints.

### 10.1 §10.1's blocker was real, and the fix is the one it named

The version of this section written before implementation said the row was not
shipped because *"a placeholder has to be spellable, and `$Type` is not"*. That
diagnosis was right and is kept below in full, because **it is also the design**:
its own table says a parseable body is *"clearly better"* and names the form —
reserved upper-case names, `Target` and `Field0`.

What it got wrong is the **cost** of that form. It concluded:

> What that costs is a **general identifier rewrite over an arbitrary `Item`
> tree** — every declaration form, every expression inside it. … `Expr` has
> forty-six and `Item` thirteen. That is the real size of the row.

It is not, because the substitution does not have to happen on the tree. The
body is emitted through `emit_instance` **first**, exactly as a hand-written
`instance` is, and the placeholders are replaced in the **resulting string**.
`crates/voltaic-emit/sdx/pattern.sdx` is 70 lines, most of them
comment, and does the whole job in two `.replace` calls.

So the row's two halves were assessed independently and both assessments were
wrong in the same direction:

| Claim | Verdict |
|---|---|
| `$Type` is unspellable, so the vocabulary must change | **Right.** `$` is the application operator; `@` and `%` are taken |
| A parseable body costs an `Item`-wide tree rewrite | **Wrong.** Emit first, substitute on the text — nothing walks a tree |

The lesson generalises past this plan: **"the body must be parseable" and "the
substitution must be structural" are separate requirements, and conflating them
inflated the estimate by an order of magnitude.** A parseable body gets the
whole existing `instance` pipeline — `generalise`, `given_clause`,
`class_body`, `member_text` — for free; what is done *after* that pipeline runs
is a free choice, and text is the cheapest thing it can be.

### 10.2 What shipped

| Leaf | State | Where |
|---|---|---|
| 0 — substitution probe | folded into leaf 1 | the preconditions were answered by writing the row |
| 1 — template declaration | **done** | `Item::DeriveTemplate`, `derive_template.sdx`, `RDX0122` |
| 2 — scalar substitution | **done** | `expanded_template.sdx`, `expanded_template/field_names.sdx` |
| 3 — repetitions | **done** | §10.3, after the first version withdrew them |
| 4 — provenance comment | **done** | `emit_templates.sdx` |
| 5 — fmt round-trip | **done** | `item_text.sdx`'s `Item::DeriveTemplate` arm |
| 6 — corpus and docs | **done** | `examples/template.sdx`, the corpus case, `docs/GRAMMAR.md` |

Thirteen files of `.sdx`, four of them new, across five crates:

- `voltaic-ast` — `DeriveTemplate`, the `Item` variant, the span and `leading` arms
- `voltaic-parser` — `derive_template.sdx`, the one-token dispatch in `Parser::item`
- `voltaic-diag` — `Code::DeriveTemplateShape` → `RDX0122`
- `voltaic-emit` — `TemplateTable`, `TemplateScope`, `expanded_template`,
  `emit_templates`, and `emit_data`'s split of the `deriving` list
- `voltaic-fmt`, `voltaic-lsp` — the printer arm and the outline entry

### 10.3 Repetitions were withdrawn, then shipped — the withdrawal was design-bound

The section that stood here withdrew leaf 3, and its argument was:

> `$fields` and `$variants` are a **loop**, and a loop cannot be expressed by
> replacing a name in a finished string. The textual substitution that made
> leaf 2 cheap is exactly what makes leaf 3 impossible, so the two leaves are
> not two sizes of the same work — they are different constructs sharing a
> declaration.

**The first clause is true and the second does not follow.** The loop does not
have to be *in* the string. It runs over the declaration's **own field list** and
produces the text that gets substituted — which is precisely what `emit_data`
already does to write one line per field, and what `emit_templates` does to
write one `impl` per `deriving` entry. The emitter is full of loops over
declaration structure, and none of them is an evaluator.

So leaf 3 shipped the same day, as two more placeholders:

| Placeholder | Becomes |
|---|---|
| `Fields` | every field as a `self` access, comma-separated — `self.x, self.y` |
| `FieldNames` | every field's name as a string literal — `"x", "y"` |

```text
deriving template Labelled
  instance Labelled Target
    labelled : &self -> String
    labelled self = zip_pairs (&[FieldNames]) (&[Fields])

data Point  = Point { x : u32, y : u32 }          deriving (Labelled)
data Triple = Triple { a : u32, b : u32, c : u32 } deriving (Labelled)
```
```rust
impl Labelled for Point {
    fn labelled(&self) -> String { zip_pairs(&["x", "y"], &[self.x, self.y]) }
}
impl Labelled for Triple {
    fn labelled(&self) -> String { zip_pairs(&["a", "b", "c"], &[self.a, self.b, self.c]) }
}
```

`examples/template.stdout` asserts `x=1, y=2` and `a=3, b=4, c=5` — **one
template, two declarations of different widths**, which is §3 case 2 and the
case the section above said a substitution-only implementation could not reach.
The three-field declaration is what makes it a loop rather than a pair: a
`Field0`-only implementation passes the two-field case.

#### Doc 03 §13.3 is untouched, and the distinction is worth stating

§1.1's floor is that *"the executed form is `comptime` and `docs/DECLINED.md`
refuses it. Whatever is built must compute nothing."* This computes nothing
**the author wrote**: it joins a list the parser produced. An interpreter runs
the program's own expressions; this reads the declaration's shape, which the
emitter does on every line it writes.

That is the line, and it is the same one plan 90 found from the other side: a
cost estimate inherits the assumptions of the design it was made against. Here
the assumption was *substitution happens on the finished string and nowhere
else* — true of leaf 2's two placeholders, and never a requirement.

#### What is still not available, and this one is real

A **per-field fragment**: a template cannot say *"emit this member once for each
field"*. `Fields` and `FieldNames` hand the body two lists it can zip, which
covers a template that visits every field — but a template that wants a whole
`fn` per field, or a `match` arm per variant, would need the fragment to be
something the template language can name and repeat.

`$variants` is the same gap seen from the enum side, and it is **not** shipped
for an additional reason: `field_names` answers for a single-constructor
declaration and returns nothing for a `data` with alternatives, because there is
no one field list. Choosing a constructor would be an inference, and choosing
the first would be an inference that usually looks right.

Both are a successor's, and both now need a *construct* rather than a table
entry — which is the honest size of what remains, where the section above had
the honest size of nothing.

### 10.4 Criteria, honestly

| # | Criterion | Verdict |
|---|---|---|
| 1 | each §3 case transpiles, compiles, runs, stdout asserted | **met for 1, 2, 4** and the no-change case; case 3 (an enum, `$variants`) is not built — §10.3 |
| 2 | the repetition cases work | **met** — §10.3. `Fields` and `FieldNames` visit a declaration of any width |
| 3 | a golden shows template and expansion | **met** — `tests/golden/template.xrs` |
| 4 | case 5 is a rustc error | **not built** — §10.5 |
| 5 | case 6 is rustc's `E0433` | **met in behaviour, not in a fixture** — §10.5 |
| 6 | every generated `impl` carries a provenance comment | **met** — `emit_templates.sdx`, visible in the golden |
| 7 | `voltaic fmt` round-trips a template unexpanded | **met** — `every_example_is_already_canonical` covers it |
| 8 | existing `deriving` emits byte-identical Rust | **met** — every other golden is unchanged |
| 9 | an unknown substitution is a named Voltaic error | **fails** — §10.5 |
| 10 | examples green, no new `construct_coverage` `ALLOW` | **met** |

### 10.5 The two diagnostics that are rustc's instead of Voltaic's

Criterion 9 wanted `Field7` on a one-field newtype to be a Voltaic error. It is
`E0609` from rustc, and the reason it stays there is the one doc 03 §2's
corollary gives: answering it needs to know **how many fields a template
intends to use**, which is a question about the body rather than about the use,
and a body is an arbitrary `instance`. A check that scanned the emitted text
for a surviving `FieldN` would also fire on a program that legitimately names a
field called `Field0`.

Criterion 4's fixture — a template whose body names something not imported —
was not built, for the same reason case 6's was not: both are rustc errors
whose message is about the *generated* `impl`, and `tests/ui/` captures Voltaic's
diagnostics. `RDX0122` is the one Voltaic diagnostic the row adds, and it has a
fixture: `crates/voltaic-cli/tests/ui/derive_template_shape.sdx`, for an
`instance` whose target is a concrete type rather than `Target`. That is caught
at the declaration, where the mistake is — not at a `deriving` list, which
would expand to an impl for the wrong type and compile the first time.

### 10.6 Two findings worth keeping

**`template` is contextual by choice, not after a revert.** Plan 87 reserved
`pattern`, found that `voltaic-ast` stopped transpiling because `pattern` is a
field name in four of its own declarations, and reverted. This row read that
record first and never reserved `template`: `deriving` is already a keyword and
at item position nothing else can follow it, so one token settles the dispatch
and the second word rides in as an ordinary identifier. The grammar still
spells it as a literal, so it is coloured — the third entry in `editors.rs`'s
`COLOURED_BUT_NOT_A_KEYWORD`, after `from` and `pattern`.

**`tree-sitter parse` run from the repository root is a false green, and
AGENTS.md already said so.** The command was run as
`tree-sitter parse examples/template.sdx`, exited 1 with *"Failed to load
language … No language found"*, and `grep -c ERROR` counted zero. Plan 84 §10.6
recorded the identical mistake. The habit that catches it is reading the exit
code, not the match count — which is the same rule as *"do not grep compiler
output for `^error`"*, and that one bit twice in this session too: `cargo
build`'s errors are ANSI-prefixed, so `grep -E "^error"` reported a clean build
while `voltaic-parser` was failing with two `E0599`s.

### 10.7 The §10.1 that was written before implementation

Kept verbatim, because its table is the design that shipped and its cost
estimate is the mistake §10.1 above corrects.

> ### The blocker: a placeholder has to be spellable, and `$Type` is not
>
> §1.2 fixes the substitution vocabulary as `$Type`, `$field0`, `$fields`,
> `$variants` and `$Self`. **`$` is already the application operator** — doc 18
> §4, hardcoded in the Pratt loop at binding power 1 (plan 56 §11.6) — so
> `$Type` lexes as `Dollar` then `Ident` and a template body containing it does
> not parse as Voltaic at all.
>
> §9 decision 3 anticipated a collision and offered "a distinct sigil such as
> `%` or `@`" as the fallback. Both are taken too: `@` is the `@`-binding token
> and `%` is `Percent`.
>
> | If the body is | Then substitution is | And |
> |---|---|---|
> | **unparseable** Voltaic (`$Type`) | textual, on source | the result must be **re-parsed at emit time**, so `voltaic-emit` gains a dependency on `voltaic-parser` |
> | **parseable** Voltaic | a rewrite over `Vec<Item>` | the placeholder must be an ordinary name, so the vocabulary is not `$Type` at all |
>
> The second is clearly better and it is a different construct from the one
> §1.2 specifies. The natural form is **reserved upper-case names**: a body
> that is an ordinary `instance` declaration over a type called `Target`, with
> fields as `Field0`, `Field1`, …

### 10.8 Corrections to this document

- §1.2's vocabulary is not spellable, and is replaced by `Target` / `Field0`…
  §10.1.
- §9 decision 3's fallback sigils are both taken. §10.1.
- §10.1's *"general identifier rewrite over an arbitrary `Item` tree"* is not
  what the parseable form costs. Emitting first and substituting on the text
  costs neither.
- §2 has no precondition for *when the body is parsed*, which is the decision
  the whole row rested on — and once answered, the decision that made it cheap.
- §6's self-assessment — *"the weakest row in milestone 11"* — stands.
  Forwarding a trait for a newtype is the narrowest capability milestone 11
  adds, and it is still the one Rust makes most expensive.
- §3 cases 2 and 3 describe a construct this row does not build. §10.3.
