# Plan 65 — Layout Inside Brackets

Plan ID: `bracket-layout`
Depends on: 52 (`type-colon`), 55 leaf 2 (`leaf-if-expression`)
Milestone: 9 — *surface coherence*
Status: `done`

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

**Done.** `print (if c` with an indented body transpiles, compiles and runs,
and `examples/control.sdx`'s `bracketed_block` is the witness with asserted
stdout. Plan 55 leaf 7 is unblocked.

### 10.1 The mark went into `open_delims`, and `depth` did not survive

`Lexer::open_delims` was `Vec<usize>` — a byte offset per open delimiter, so an
unbalanced one reports where it opened. It is now `Vec<(usize, usize)>`, the
second field being the `indents` depth the delimiter opened at: §4's context
mark. `track_delims` pushes it on `([{`, and on `)]}` pops every level above it
with a `Dedent` each.

Both `if self.depth > 0 { return }` guards are gone, and with them **the field
itself**. §2 recorded `depth`'s doc comment as saying continuation suppression
needed it carried across lines; the comment was stale. Continuation is decided
entirely by `TokenKind::continues_line` on the previous token and
`next_line_continues` on the line ahead — `step` reads neither counter — so
with the guards removed `depth` was written and never read. `open_delims.len()`
was already the same number, which is why the mark went into that stack rather
than beside it: two parallel push/pop stacks could drift, one cannot.

**The `Dedent`s land before the closer**, because `step` calls `track_delims`
before pushing the delimiter token. That is what makes the inside of a bracket
end the way the top level does — `statements()` breaks on the `Dedent` and
`expect(Dedent)` consumes it — and it is why `print (if c …)` needed no change
in `prefix`'s `(` arm at all.

### 10.2 `skip_separators` was the wrong function, and that is the correction

§4's leaf 2 and the brief both say the bracketed constructs should call
`skip_separators`. They cannot: it skips **`Newline` only**, and the token
stream inside a bracket now carries `Indent` and `Dedent` too —
`{ Newline Indent field , … Dedent }` for `conformance/clause.sdx`'s record.

Widening `skip_separators` was not an option either. Its seven callers are
`program`, `items_until_dedent`, `function`, `statements`, `if_statement`,
`where_locals` and `guards` — every one of them a block construct that **ends**
on a `Dedent`. Teaching it to skip one would make every indented block in the
language unterminated.

So there is a second method, `Parser::skip_layout`, and the bracketed
constructs call that. Nine files:

| File | Where |
|---|---|
| `bracketed.rs` | after `[`, after the first element, around `;`, around each `,`, before `]` |
| `prefix.rs` | after `(`, after the group's expression, around each `,` of a tuple |
| `record_literal.rs` | its three `skip_separators` became `skip_layout` |
| `constructor_fields.rs` | the same three |
| `record_pattern.rs` | before the loop and at each `,` |
| `pattern_series.rs` | before the loop and at each `,` — tuple, slice and record patterns share it |
| `arguments.rs` | before the loop and at each `,` — a method call's list |
| `postfix.rs` | inside `[…]` indexing, and before a method call's `)` |
| `qualifiers.rs` | at each `,` of a comprehension |
| `deriving.rs` | at each `,` |

**`application` needed nothing**, against the brief's expectation. Its argument
list is juxtaposition, not a bracket: `starts_argument` rejects a layout token,
so the loop stops exactly where it should. Its `just_dedented()` guard is also
unharmed — the token before a bracketed group's content is an `Indent`, never a
`Dedent`.

### 10.3 What the first pass missed, and what found it

Two, and §4 was right that this half would be incomplete:

- **`bracketed.rs`** — `xs = [ 1` with `, 2` on the next line gave `RDX0101`
  *expected `]` to close the list*. Found by a hand probe before any test ran.
- **`constructor_fields.rs`** — `conformance/clause.sdx` gave `RDX0103`
  *expected a field name*, pointing at the record's first field. Found by
  running the one real affected corpus construct §1.3 names.

Neither was found by the fuzz sweep or by `redox-fmt`, which §4 nominated.
Both were green throughout: **`cargo test -p redox-fuzz` and
`cargo test -p redox-fmt` never failed once during this row.** The gates that
actually reported something were the corpus (`clause.sdx`, `examples`,
`tree_sitter`) and two lexer unit tests. That is worth recording because §4
named the wrong instrument, and §6 leaned on it — *"the fuzz sweep is doing
more of the work than usual"* is, measured, the opposite of what happened.

### 10.4 The tree-sitter grammar needed the scanner, not the grammar

Doc 04's fifth ordering commitment is that a surface change has four readers,
and this is the fourth. `grammar.js` is **unchanged**: `parenthesized` already
holds `$._expression`, which already reaches `conditional_expression`, which
already reaches `if_statement`. The external scanner is where it broke, and
`examples/control.sdx` came back as one 16-line `ERROR`.

The reason is structural. `tree_sitter_redox_external_scanner_scan` measures
indentation **at a line break**, and returns false for mid-line whitespace. The
closer in `    "small")` is on the body's own line, so the line break the
measurement needs never comes and `statement_block` never got its `_dedent`.
The Rust lexer has no such problem because it holds a mark per open delimiter;
the scanner cannot, because `(` is an ordinary grammar token it is never asked
to lex and so has no way to see one open.

It gets the same answer from the grammar instead: **the only reason `DEDENT` is
valid with a closer ahead is that a `statement_block` inside the bracket is
still open.** Twelve lines in the `!crossed_a_line` branch, emitting a
zero-width `NEWLINE` then one `DEDENT` per level.

The `NEWLINE` is not decoration. `expression_statement` is
`seq($._expression, $._newline)`, so emitting the `DEDENT` alone left a
`(MISSING _newline)` in the tree. **`tree-sitter parse` does not print it and
exits 0** — `_newline` is a hidden rule — so the only thing that saw it was
`tree-sitter test`, which is in no gate. The framing that fixed it is that a
closer **ends the line** as far as layout is concerned, and so closes a block
in the order a line break does.

`test/corpus/blocks.txt` gains *an indented block inside parentheses* and
`tree-sitter test` is 6/6. Two notes for whoever is next: the corpus harness
strips the input's trailing newline, so a case ending in `()` reads as a
pattern and errors for an unrelated reason; and the first `tree_sitter` run
after editing `scanner.c` can fail one test while the grammar's shared library
is rebuilt under three parallel invocations — it passes on the re-run.

### 10.5 The corpus moved its tokens and not its output

Criterion 5: `conformance/clause.xrs` is committed and tracked, and `git
status` reports it unmodified after rebuilding. Its token stream *did* change —
a `Newline` after the `{`, an `Indent` before the first field, a `Dedent`
before the `}` — and the emitted Rust did not, which is the whole claim.

Criterion 4: all 68 `tests/ui` fixtures pass unchanged, `unterminated_fence`,
`unterminated_attribute` and `unbalanced_delimiter` among them. An unbalanced
delimiter is now a context that never closes, and `finish` still reports
`RDX0012` at the opener's offset because the levels it opened are simply never
popped — the error path did not move.

Criterion 3 gains `tests/ui/inconsistent_dedent_in_brackets.sdx`: a list whose
third item sits one column left of its siblings, reported as `RDX0002` with the
same words as the top-level case, which is §6's decision.

### 10.6 Two tests asserted the rule this row removes

`continuation.rs`'s `a_bracketed_expression_emits_no_structural_tokens` was
plan 02 leaf 2's criterion 6 stated directly — *a bracketed expression emits no
structural tokens* — and `nested_delimiters_track_depth` with it. They are
**replaced, not deleted**, by the property criterion 6 was protecting:
`a_bracket_closes_every_level_its_interior_opened` and
`nested_delimiters_each_close_their_own_context` assert that `Indent`s and
`Dedent`s balance and that the stack after a closer is the one before its
opener. Three new ones carry criteria 1, 2 and 3 at the lexical level.

### 10.7 The depth-equals-width property has a limit now, and it is written down

`indentation_round_trip.rs`'s `token_depth_tracks_the_texts_indentation` failed
on `control.sdx`. A `Dedent` used to arrive only at a line boundary, so *"the
first token after a layout token"* and *"the first token on its line"* were one
statement; the `)` after the `Dedent`s its bracket owed is the first and not the
second. A new `opens_its_line` predicate tells them apart.

**The remaining limitation is real and is disclosed rather than papered over.**
A line aligned under its opener pushes a level whose width is the opener's
column, which has nothing to do with the file's other levels at that token
depth — so `aligned = [ 1` with `, 2` under the bracket *would* fail this
property. It is not a disagreement between stack and text; it is a different
context's base, and §1.2 is the argument for allowing it.

That is also **§1.3's finding turned the other way round, and it is a harder
limit than §1.3 suggests.** §1.3 says the corpus is almost silent on this row.
It cannot be anything else: `redox fmt` normalises a spread list onto one line,
so `every_example_is_already_canonical` rejects the aligned layout before any
other test sees it. Measured — adding one to `examples/arrays.sdx` failed that
test, the depth property and the tree-sitter grammar together, and the addition
was withdrawn. So the witness for criterion 2 is a direct `lex` call in
`continuation.rs` with four alignments, and **`examples/` cannot hold one at
all**. §6 says the corpus is the weak evidence here; the stronger statement is
that for *this* construct the corpus is structurally unable to be evidence.

### 10.8 The design's one rough edge, and why it was closed rather than left

§4 says a continuation aligned under the opener *"is one consistent deeper
level"* and that the closer unwinds past it. True. What is **not** true is the
stronger claim that the closer's own indentation is therefore never reconciled:
`start_of_line` measures a line before any token on it is scanned, so a closer
that opens its own line at an *intermediate* width still meets `sync_indent`.

```text
foo (bar
       baz
     )        -- RDX0002: 5 matches neither 7 nor 0
```

That was first written down as a rough edge to leave unspelled — Haskell's
`parse-error(t)` rule in miniature — on the grounds that every shape the corpus
and `redox fmt` produce closes at the opener's line or on the content's. Both
halves of that were right. The conclusion was wrong, for a reason that only
shows up when the case is probed rather than reasoned about.

**Re-probed, with the operator trailing instead of leading.** The first
reproduction used `(1` then `+ 2` on the next line, and a *leading* `+` is not
a continuation operator — only `|>` is — so it also produced an `RDX0101` from
a spurious `Newline`, which obscured what was being measured. With the `+`
trailing, the intermediate closer **parses**: a continuation line is suppressed
before `sync_indent` runs at all, so no level is ever pushed and there is
nothing for the closer to disagree with.

So the edge is **not** about continuations, which were the only thing brackets
could hold before this row. It is about the levels this row newly lets a
bracket's interior push — and that puts it squarely inside criterion 2 rather
than beside it. The rejected thing is a `)`, and what decides the rejection is
nothing but which column the author put it in. That is the same hazard class
§1.2 describes and the same one plan 52 removed from `::`: a cosmetic choice
with a semantic consequence.

**The rule, in `sync_indent`, eight lines:** a line that *opens* with a closer
is not content at that indentation. `track_delims` already owes the `Dedent`s
that unwind the context, so the closer's column is free and no column is
privileged. `a_closers_own_column_is_never_an_inconsistent_dedent` asserts all
four placements — base, short, under the opener, and column zero.

It stays narrow by construction, and criterion 3 is what shows it:
`an_inconsistent_dedent_inside_brackets_is_still_rejected` fires on a *content*
line and still fails. The guard only ever sees a line whose first character
closes a bracket.

### 10.9 Gate

`moon run :check`, `:lint`, `:test`, `:format` green; `examples`, `ui`, `run`
green; `redox-fuzz`, `redox-fmt`, `tree_sitter`, `editors`,
`construct_coverage` green; `tree-sitter test` 6/6.

Two expectations were re-blessed and reviewed: `tests/golden/control.xrs` for
the new function, and `tests/ui/inconsistent_dedent_in_brackets.stderr` as a
new fixture. `docs/STATUS.md` is left red deliberately — its per-crate line
counts moved and it is blessed once at the end of the milestone, with
`docs/CONFORMANCE.md`.

One caution for the next reader of that list: `CONFORMANCE.md` counts **lines
containing a macro invocation**, so a single `matches!` added anywhere in
`crates/` moves it. `skip_layout` was first written with one and failed
`conformance_is_current` for that reason alone; it now uses the three `check`
calls `at_statement_end` already uses, which is the better match for the file's
style regardless.


---

## The witness moved, at plan 70

This row cites `conformance/clause.sdx` seven times — the 15-line multi-line
`data … = … { … }` record declaration was its evidence that a bracket opens a
layout context, and criterion 5 asserts the file compiles with an unchanged
`.xrs`.

That file is **gone**, retired with plan 48's probe when plan 70 removed the
`conformance/` directory. The witness is not: the same module, ported for real,
is `crates/redox-ast/sdx/program/item/clause.sdx`, and its record declaration
is the same shape. It is now graded better than it was — `tests/port.rs`
compares it byte for byte against the committed Rust, and rustc compiles that
Rust as part of the workspace, where the retired fixture was only checked by
`syn::parse_file`.

So read criterion 5 as naming the ported file. Nothing about the behaviour this
row landed has changed.
