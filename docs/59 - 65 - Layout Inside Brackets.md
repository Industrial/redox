# Plan 65 — Layout Inside Brackets

Plan ID: `bracket-layout`
Depends on: 52 (`type-colon`), 55 leaf 2 (`leaf-if-expression`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

A bracketed expression gets its own layout context, so an indented construct
can be written inside `(…)`, `[…]` and `{…}`.

This row exists because plan 55 leaf 7 cannot retire `then` without it, and
because the measurement that established *that* also established the rule is
cheaper than it looks.

### 1.1 The one thing that cannot be written

Measured at plan 55 leaf 2, which made the indented `if` an expression:

```text
v = if c        -- works: a binding's value
  a
else
  b

print (if c     -- RDX0101
  a
else
  b)
```

The second is not a grammar gap. The lexer **suppresses layout entirely while
bracket depth is above zero** — `end_of_line` and `start_of_line` both return
early on `self.depth > 0` — so there is no `Newline`, no `Indent` and no
`Dedent` inside brackets for the off-side rule to read, and
`statement_block`'s `expect(Indent)` has nothing to find.

So `print (if c then a else b)` is the **only** way to write a conditional in
argument position. That is what makes plan 55 leaf 7's retirement of `then`
expensive: without it, every conditional used as an argument has to be hoisted
to a binding first.

### 1.2 Why the suppression exists, and what it is actually protecting

It is not arbitrary. It is what lets a construct span lines inside brackets
without the off-side rule firing on cosmetic alignment:

```text
m = f (a,
       b)
```

That continuation is indented seven columns to line up under the paren. With
one global indent stack, seven matches no enclosing level and `RDX0002` —
*"an indentation width matching no enclosing level"* — rejects it. The
suppression is a blunt instrument aimed at a real target.

**The target survives a sharper instrument.** What breaks cosmetic alignment is
sharing *one* stack between the file's layout and a bracket's interior. A
bracket that opens a layout context of its own has nothing to disagree with:
every line inside it is either at the context's base or deeper, and deeper is
one consistent level however many columns it happens to be.

### 1.3 The corpus is almost silent on this, and that is the finding

Every line in the corpus that *ends* inside brackets is a line whose successor
currently gets no layout token, and so a line this row changes. There are
**24**, in 6 files, and the breakdown is what makes this row small:

| File | Lines | What it is |
|---|---:|---|
| `conformance/clause.sdx` | 15 | a multi-line `data … = … { … }` record declaration |
| `crates/redox-cli/tests/ui/unterminated_attribute.sdx` | 4 | a **deliberately** unterminated delimiter |
| `examples/reachability.sdx` | 2 | inside a `rust """…"""` verbatim block — the lexer's verbatim path, not bracket depth |
| `examples/lexical.sdx` | 1 | the error-path example, which does not parse |
| `tests/ui/unterminated_fence.sdx`, `unbalanced_delimiter.sdx` | 1 each | deliberately unterminated |

So **one** real construct is affected: a brace-delimited record declaration
spanning lines, whose field lines sit at a consistent four-space indent.

That is a measurement about the corpus rather than about the language, and §6
says so: real programs written against this surface — the `id_effect` ones that
opened this milestone — are full of multi-line bracketed constructs. The corpus
being silent means the corpus is the weak part of this row's evidence, not that
the change is free.

---

## 2. Verified preconditions

Measured at HEAD, after plan 55 leaf 5.

| Claim | Verdict |
|---|---|
| `end_of_line` and `start_of_line` both return early while `depth > 0` | **true** — `crates/redox-lexer/src/lexer.rs` |
| `depth` is maintained by one counter over `([{` and `)]}` | **true** — and its doc says continuation suppression needs it carried across lines |
| an indented `if` inside `(…)` is `RDX0101` | **true**, probed |
| the same `if` bound to a name works | **true**, probed — plan 55 leaf 2 |
| `RDX0002` is the diagnostic for an indent matching no level | **true** — `inconsistent_indentation_is_rejected` asserts it |
| a `rust """…"""` body is lexed by the verbatim path, not by bracket depth | **true** — `examples/reachability.sdx` is not a bracket case |

---

## 3. Concrete proof this plan targets

1. `print (if c` + indented branches + `)` transpiles, compiles and runs.
2. `m = f (a,` + a continuation aligned under the paren still parses, and does
   **not** raise `RDX0002`. This is the case §1.2 says the suppression exists
   for, and it is the one a careless fix breaks.
3. `conformance/clause.sdx`'s record declaration is byte-identical in its
   `.xrs` — the one real corpus construct affected.
4. A genuinely inconsistent dedent **inside** brackets is still `RDX0002`, so
   the fresh context is a context and not an amnesty.
5. `redox fmt` round-trips a bracketed indented construct, which is the
   property that catches a lexer change the parser tolerates.

---

## 4. Leaves

### Leaf 1 — `leaf-bracket-layout-stack`

A bracket pushes a layout context; its closer pops it. Inside a context, the
off-side rule runs against *that* context's base rather than the file's, so a
continuation aligned under the opener is one consistent deeper level and
`RDX0002` cannot fire on it.

The base is the indentation of the line the opener sits on, not the opener's
column — aligning under the opener and indenting by two are then the same
thing as far as the rule is concerned, which is the point.

### Leaf 2 — `leaf-skip-layout-in-bracketed-constructs`

Every construct that reads a comma-separated list inside brackets skips layout
tokens at its item boundaries: `expr_list`, `record`, `list`, the tuple arm,
`comprehension`, `record_pattern`, `constructor_fields`. `skip_separators`
already exists and is what they call.

This is the half that is mechanical and the half that will be incomplete on
the first attempt — the fuzz sweep and `redox fmt`'s corpus properties are the
two gates that find a missed one.

### Leaf 3 — `leaf-corpus-and-docs`

`conformance/clause.sdx` re-blessed if it moves at all; the three deliberately
unterminated `tests/ui/` fixtures confirmed unchanged, because an unbalanced
delimiter is now a context that never closes and its diagnostic must not
change. Doc 03 §3's off-side rule gains the bracket context; plan 02's lexer
plan gains a pointer.

---

## 5. Acceptance criteria

1. An indented `if` is writable inside `(…)`, exercised in a running example
   with asserted stdout.
2. A continuation aligned under its opener parses and raises no `RDX0002`.
3. An inconsistent dedent **inside** brackets is still `RDX0002`, with a
   `tests/ui/` fixture.
4. The three unterminated-delimiter fixtures produce their existing
   diagnostics, byte-identical.
5. `conformance/clause.sdx` compiles and its `.xrs` is unchanged.
6. All examples green, no rustc output, stdout unchanged.
7. `redox fmt` round-trips every example, and `formatting_is_idempotent` holds.
8. `cargo test -p redox-fuzz` green — the sweep is what finds a construct whose
   item loop forgot to skip layout.
9. `construct_coverage` green.

---

## 6. Out of scope, and what it discloses

**The corpus is the weak evidence here.** §1.3 found one real affected
construct, and that is a fact about a 31-file corpus rather than about the
language. The programs that opened this milestone — 1,501 lines against the
shipped surface — are full of multi-line bracketed constructs, and none of them
is in this repository. So criteria 6 and 7 are weaker than they look, and the
fuzz sweep is doing more of the work than usual.

**No change to what a bracket means.** A bracket is still grouping, a call's
argument list, a literal. This row changes only whether the lines inside one
carry layout.

**`RDX0002`'s message is unchanged**, and that is a decision: a reader who hits
it inside brackets gets the same words as one who hits it at the top level,
because it is the same rule applied to a different base.

---

## 7. Gate

Every plan clears, before it is `done`:

```bash
moon run :check          # per-crate `cargo check` — rustc's verdict
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus `cargo test -p redox-fuzz`, which this row leans on more than most, and
the three blessable corpora.

---

## 8. Emerald reuse

None. Emerald had no off-side rule — doc 02 §4 records that its surface was
brace-delimited — so there is no prior art to inherit and no mistake of its to
avoid. The relevant prior art is Haskell's own layout rule, which does exactly
this: a bracket opens an implicit context and the closer ends it.

---

## 9. Decision log

1. **A fresh context per bracket, not an exemption.** The alternative — keep
   suppressing layout and have the parser accept a block without `Indent`
   inside brackets — pushes a lexical question into the parser, which would
   then need bracket depth it does not have. It also leaves §1.1's example
   depending on the parser's mood rather than on a rule.
2. **The base is the opener's *line* indentation, not the opener's column.**
   So aligning under the opener and indenting by two are the same level. The
   alternative makes a cosmetic choice semantic, which is the hazard class plan
   52 removed from `::` and plan 55 §10.9 declined to reintroduce.
3. **`RDX0002` still fires inside a context.** A fresh base is not an amnesty;
   an indent matching no level of *this* context is as wrong as one matching no
   level of the file, and silence there would be a worse trade than the
   suppression this row removes.
4. **This is its own row rather than a leaf of plan 55.** It changes the lexer
   for every bracketed construct in the language, not just for `if`, and plan
   55 leaf 7's dependency on it is one consequence rather than its purpose.

---

## 10. Result

*Not yet executed.*
