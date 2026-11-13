# Plan 02 — Lexical Surface

Plan: 02 of 48 (`docs/04 - Voltaic Plan of Plans.md`, Milestone 1)
Slug: `lexer-complete`
Status: **`done`** — shipped 2026-11-08; see §10 for evidence per criterion
Depends on: plan 01
Discloses: plans 03, 05, 19, 23

---

## 1. What this plan is

Plan 01 built the smallest lexer that could carry one program: eight token kinds, an
off-side rule, trivia, and whitespace adjacency. This plan finishes the **lexical** surface —
everything `docs/03 - Voltaic Surface Design.md` §4 and §11 specify about how Voltaic source
text decomposes into tokens — and nothing above it.

Scope boundary, stated precisely because "complete" invites drift: this plan decides *what
tokens exist*, not *what they mean*. No grammar production, no AST node, no lowering. The
parser consumes these tokens in plan 03 and after; a token this plan emits that no parser
yet accepts is expected, not dead.

### 1.1 Why the lexer core is restructured

Plan 01's lexer iterates over physical lines (`src.split('\n')`), which was right for a
slice where every construct fitted on one line. Two of this plan's features break that
shape outright:

- **Continuation suppression** (§4.2) needs bracket depth carried *across* line boundaries,
  so the decision to emit `Newline` is no longer a property of one line.
- **`"""` strings** (§4.5) span lines, so a single token consumes several of them while
  line numbering must stay correct for everything after.

So this plan replaces the line loop with an explicit character cursor that tracks line
numbers itself. That is a real rewrite of `Lexer`'s core, and doing it now rather than
later is the point: Emerald needed a 447-line raw-source preprocessing pass
(`crates/emerald-parser/src/heredoc.rs`) to get heredocs into a lexer whose shape could not
accommodate them, and its own notes record that the splice-back technique was chosen
*because* a dynamic terminator "needs a regex backreference LALRPOP's lexer can't do". A
hand-written cursor has no such constraint.

---

## 2. Verified preconditions

None required. Everything here is internal to `voltaic-lexer`; the one external question a
lexer plan could have raised — whether rustc accepts the `.xrs` the pipeline eventually
emits — was settled by plan 01 §2.

Plan 01's three mandatory lexer properties carry forward unchanged and are re-asserted by
this plan's tests: every token carries a byte span; comments and blank runs become trivia
and are never discarded; every token records whether whitespace preceded it.

---

## 3. Concrete proof this plan targets

`examples/lexical.sdx` — a file that exercises the whole lexical surface and whose *tokens*
are asserted directly, since no parser yet accepts most of them:

```
### A fenced comment,
    spanning lines. ###

## Outer doc comment.
main = ->
  # A line comment, then every literal kind.
  count = 1_000_000
  hexed = 0xDEAD_beef
  ratio = 2.5e-3
  ch = 'x'
  byte = b'\n'
  raw = r#"no \escapes here"#
  cstr = c"nul terminated"
  words = %w[get post put]
  set = #{1, 2, 3}
  greeting = "hi #{name}, you owe #{amount:.2}"
  block = """
    indented
      relative
  """
  chain = items |> map (+1) |> filter (> 0)
  maybe = lookup id ?? "none"
  guarded = parse s catch 0
  spread = [0, ...rest, 9]
  range = 1..10
  closed = 1..=10
  label = 'outer
  field = @name
  static_call = @@create
  method_ref = &:name
```

The test asserts the exact token sequence for each line, the dedented value of the `"""`
block, the segment structure of both interpolations including the `.2` format spec, and that
every token's span indexes the original source.

---

## 4. Leaves

### Leaf 1 — `leaf-cursor`

`lexer/cursor.rs`: a `Cursor` over the source text holding a byte position and the current
1-based line number, exposing `peek`, `peek_at`, `bump`, `eat`, `eat_while`, `slice`, and
`at_line_start`. Every scanner takes `&mut Cursor` and nothing else, so each is independently
testable and `Lexer` keeps only the off-side rule and orchestration.

Line numbers advance inside `bump`, so a multi-line token cannot desynchronise them. This is
the property §1.1 names and plan 01's line loop could not offer.

### Leaf 2 — `leaf-continuation-suppression`

Newlines and indentation are **not significant** when the cursor is inside `(`, `[` or `{`,
or when the previous line ended in a binary operator or a comma. Doc 01 §4.2 gestures at this
("Newlines terminate statements unless an expression is continued") without saying when; this
leaf makes it precise:

| Condition | Effect |
|---|---|
| Bracket depth > 0 | no `Indent`, `Dedent` or `Newline` emitted |
| Previous token is a binary operator, `,`, `=`, `->` or `\|>` | no `Newline` emitted |
| Otherwise | the off-side rule of plan 01 applies unchanged |

Bracket depth is `Lexer` state, incremented on an opening delimiter and decremented on its
match; an unbalanced delimiter at `Eof` is a named error rather than a silent dedent cascade.

### Leaf 3 — `leaf-hash-table`

Doc 03 §4.1's six `#` forms, resolved by **longest match** in exactly the documented order:

| Order | Token | Becomes |
|---|---|---|
| 1 | `###` | fence; opens or closes a multi-line comment trivium |
| 2 | `##!` | inner doc comment trivium |
| 3 | `##` | outer doc comment trivium |
| 4 | `#[` | attribute; the bracketed text is passed through verbatim |
| 5 | `#{` | set literal open |
| 6 | `#` | line comment trivium |

Doc comments become `TriviaKind::DocComment { inner: bool }` rather than a token: they are
attached to the item they precede, exactly as plan 01 attaches ordinary comments, so the
emitter can turn them into `///` and `//!` in plan 23 without a grammar change.

### Leaf 4 — `leaf-string-interpolation`

`lexer/scan_string.rs`. A `"…"` literal becomes a `TokenKind::Str(Vec<StrPart>)`, where a
part is either literal text or an interpolation:

```rust
pub enum StrPart {
  Text(String),
  Interp { tokens: Vec<Token>, spec: Option<String> },
}
```

Interpolation bodies are lexed **recursively** with the same scanners, so `"#{f(x, "y")}"`
works and nothing needs a second tokenizer. The body ends at the `}` that returns brace
depth to zero.

**Format specs** (doc 03 §11.1) are split at the **first depth-zero `:` that is not part of
`::`**. The depth test is what makes `"#{Point{x: 1}}"` keep its struct literal, and the
`::` test is what keeps `"#{HashMap::new()}"` intact. The spec text itself is not
interpreted — it is carried verbatim for plan 23's emitter to paste into the format string.

Escapes are decoded here: `\n \r \t \\ \" \0 \xNN \u{…}`. An unknown escape is a named
error, not a silent passthrough.

### Leaf 5 — `leaf-multiline-and-raw-strings`

- **`"""…"""`** (doc 01 §7.2): common leading whitespace is stripped from every line, and
  the opening and closing delimiter lines are dropped. Interpolation works inside it for
  free, because the dedented text is handed to leaf 4's scanner. **Line numbers after the
  block are preserved**, which is the property Emerald's heredoc pass had to work for and
  this plan's cursor gives without effort.
- **Raw strings**: `r"…"` and `r#"…"#` with any number of hashes, no escape processing and
  no interpolation. More necessary in Voltaic than in Rust, because `#{` is live inside
  ordinary strings.
- **Byte and C strings**: `b'x'`, `b"…"`, `c"…"`, each its own token kind so plan 23 can
  pass them through verbatim.

### Leaf 6 — `leaf-numeric-literals`

`lexer/scan_number.rs`: decimal, `0x`, `0o`, `0b`, with `_` separators anywhere after the
first digit; floats with `.`, `e`/`E` exponents and signs; and Rust's type suffixes
(`42u8`, `1.5f64`) carried verbatim. A malformed literal — `0x` with no digits, two decimal
points — is a named error.

### Leaf 7 — `leaf-operators-and-sigils`

Every token in doc 03 §4.5, by longest match: `|>`, `??=`, `??`, `?.`, `?`, `...`, `..=`,
`..`, `.(`, `.+ .- .* ./ .%`, `&:`, `@@`, `@`, `::`, `->`, `=>`, `==`, `!=`, `<=`, `>=`,
`&&`, `||`, `+= -= *= /= %=`, `<<`, `>>`, and the single-character set.

**`'` disambiguation**, which doc 03 §4.3 fixes as Rust's own rule: `'x'` is a char literal
when a closing quote follows one character (or one escape); otherwise `'name` is a lifetime
or a label. Decided by lookahead, never by context.

### Leaf 8 — `leaf-keywords`

A `Keyword` enum covering doc 03 §4.5's reserved set plus doc 01's. `scan_ident` returns
`TokenKind::Kw(k)` for a reserved word and `TokenKind::Ident(s)` otherwise, so the parser
matches on a variant rather than a string and `if = 5` fails in the lexer.

`and`, `or` and `not` are keywords, and doc 03 §4.4 fixes them as **pattern combinators
only** — the lexer emits them as keywords and the parser decides they are legal only in
pattern position. Boolean logic uses `&&`, `||`, `!`.

### Leaf 9 — `leaf-tests-and-gate`

`crates/voltaic-lexer/tests/` gains one file per surface area — `string.rs`, `number.rs`,
`operator.rs`, `comment.rs`, `continuation.rs`, `keyword.rs` — plus `examples/lexical.sdx`
and its token-sequence assertion. Plan 01's `tests/lex.rs` keeps passing unchanged, which is
the regression bar: this plan may add tokens, never change the eight plan 01 defined.

Run the full gate, and re-verify independently of any single agent's report.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | `examples/lexical.sdx` lexes with no error, and its token sequence matches the asserted list exactly |
| 2 | Every behavioural assertion in plan 01's `tests/lex.rs` still holds. **Amended during execution:** the criterion originally said "passes unchanged", which was wrong — finishing the string surface necessarily changes the string token from raw text to a decoded structure, so that one token's spelling in the test changes while every assertion it makes does not. The token carries `raw` alongside `parts` so the parser and emitter, which this plan has no business touching, are unaffected |
| 3 | Every token carries a byte span that indexes the original source, including tokens inside an interpolation |
| 4 | A `"""` block's value is correctly dedented, and the line number of the token after it is correct |
| 5 | `"#{HashMap::new()}"` keeps its `::`; `"#{Point{x: 1}}"` keeps its struct literal; `"#{a:>8}"` yields spec `>8` |
| 6 | Bracket-spanning expressions emit no `Newline`, `Indent` or `Dedent` |
| 7 | An unbalanced delimiter at `Eof` is a named error, not a dedent cascade |
| 8 | `'x'` lexes as a char and `'outer` as a label, by lookahead alone |
| 9 | A reserved word lexes as `Kw`, never `Ident` |
| 10 | Comments, doc comments and fences are all trivia; none is discarded |
| 11 | Every new `.rs` file obeys `docs/06 - Voltaic Code Architecture.md` §§1–2 |
| 12 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| Any grammar production for these tokens | **plan 03** | This plan emits tokens; the parser accepts them later |
| `miette` rendering of lexer errors | **plan 05** | Errors carry spans already; rendering is separate |
| Emitting `///`/`//!` from doc-comment trivia | **plan 23** | The lexer classifies; the emitter translates |
| Deciding `and`/`or`/`not` are pattern-only | **plan 21** | A lexer cannot know pattern position |
| The call-grammar meaning of adjacency | **plan 19** | `ws_before` is collected, not interpreted |
| Interpreting format-spec text | **plan 23** | Carried verbatim; rustc validates it |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

Plus: every example `.sdx` still transpiles, compiles under rustc, runs, and matches its
asserted stdout.

---

## 8. Emerald reuse

**Consulted, no code taken:** `crates/emerald-parser/src/heredoc.rs`. Two properties are
worth copying as *requirements* rather than implementation — that a multi-line string's body
can be handed to the ordinary string scanner so interpolation works for free, and that line
numbers after the block must be preserved. Its actual approach, a raw-source preprocessing
pass before lexing, exists because LALRPOP's lexer could not express a dynamic terminator;
Voltaic's hand-written cursor has no such limitation, so the technique does not transfer.

**Deliberately not taken:** `crates/emerald-lexer` (116 lines, `logos`) — the reason from
plan 01 stands and strengthens here. Continuation suppression is lexer *state* carried
across lines, and no regex-dispatch table expresses it.

---

## 9. Decision log

- **The cursor rewrite happens now, not when something forces it.** §1.1 gives the two
  features that break the line loop. Doing it as a side effect of a later plan would mean
  rewriting the off-side rule while also adding grammar, which is how a pipeline stage
  acquires two bugs at once.

- **Interpolation is lexed recursively, not re-lexed later.** The alternative — storing the
  raw body and lexing it in the parser — splits one responsibility across two crates and
  gives interpolated expressions worse spans. Recursion costs a function parameter.

- **Format specs are split at the first depth-zero non-`::` colon.** Both guards are
  load-bearing and both have a worked example in criterion 5. Taking the *last* colon
  instead would break `"#{a:>8}"` the moment a spec contained one.

- **Doc comments are trivia, not tokens.** They attach to the item they precede exactly as
  ordinary comments do, so plan 23 translates them without the grammar knowing they exist.
  Emerald made the opposite choice — a `doc: Option<String>` field on six AST node kinds —
  and its formatter can round-trip only those six, losing every other comment. Trivia is the
  general mechanism; a field on a node is the special case.

- **Keywords are distinct tokens.** The alternative, matching identifier text in the parser,
  defers `if = 5` to a confusing parse error and spreads the reserved-word list across every
  production that cares.

- **`'` is disambiguated by lookahead, as Rust does.** Context-sensitivity here would make
  `'a'` mean different things in different positions, which is the class of ambiguity doc 02
  §5.2 names as the worst available failure mode.

- **Scanners are free functions over `&mut Cursor`.** Keeping them as `Lexer` methods would
  grow one file toward the line cap and make each scanner untestable without constructing a
  whole `Lexer`. Doc 06 §1.1 requires an `impl` to travel with its type, so the choice is
  between a large `Lexer` and small independent scanners; the scanners win.

---

## 10. Shipped — evidence per criterion

Executed 2026-11-08. Full gate green: all six moon tasks exit 0. **117 tests pass**, up from
63.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Fixture lexes; tokens match | `lexical_surface.rs`, 12 tests over `examples/lexical.sdx` |
| 2 | Plan 01's assertions hold | `tests/lex.rs`, amended for the string token only |
| 3 | Spans index the source, incl. interpolations | `every_token_span_indexes_the_source`, `interpolated_tokens_carry_original_file_spans` |
| 4 | `"""` dedents; line numbers stay correct | `dedents_a_multiline_block_and_keeps_line_numbers` |
| 5 | `::`, struct literal and spec all handled | `split_format_spec`'s 5 tests plus `lexes_interpolation_with_and_without_a_format_spec` |
| 6 | Bracketed expressions emit no structural tokens | `continuation.rs`, 4 tests |
| 7 | Unbalanced delimiter is a named error | `an_unbalanced_delimiter_is_a_named_error` |
| 8 | `'x'` char vs `'outer` label by lookahead | `distinguishes_a_char_literal_from_a_label` |
| 9 | Reserved words never lex as identifiers | `reserved_words_never_lex_as_identifiers` |
| 10 | Every comment form survives as trivia | `every_comment_form_survives_as_trivia` |
| 11 | Doc 06 §§1–2 obeyed | `one_construct_per_file.rs`, `directory_structure.rs` |
| 12 | Full gate green | six moon tasks, exit 0 |

### Four defects found by running it, not by inspection

1. **`->` and `=>` were suppressing the newline a function header needs.** The first version
   of `continues_line` included every token that could plausibly end a continued line, which
   swallowed the `Indent` opening a body. `main = ->` proved it. They are now excluded, with
   the reason written into the code so it is not re-added.
2. **Doc 03 §5.3 puts `|>` at the *start* of a continuation line, not the end** — so
   continuation is also a property of the line ahead. Added a one-token lookahead. Only `|>`
   qualifies: a leading `.` would be ambiguous with §7.5's `with` block, which uses it to
   introduce statements.
3. **Suppressing the `Newline` was not enough** — `start_of_line` still ran and opened a
   block. A continuation now consumes the next line's indentation so the off-side rule never
   sees it.
4. **A comment line's own terminating newline was recorded as a blank line**, inserting a
   spurious blank between two adjacent comments; and the rewrite had dropped plan 01's rule
   attaching a trailing comment to the closing `Newline`, so inline comments migrated onto
   the following statement. Both were caught by plan 01's golden file, which is exactly what
   a golden file is for.

### Deviations from the plan as written

- **Criterion 2 was amended during execution**, from "passes unchanged" to "every
  behavioural assertion still holds". Finishing the string surface necessarily changes the
  string token; claiming otherwise would have meant contorting the design or quietly editing
  the test. The token carries `raw` alongside `parts` specifically so the parser and emitter
  — which this plan has no business touching — are unaffected.
- **`emit_trivia` gained three arms.** The new doc-comment and fence kinds made its match
  non-exhaustive. Handling them (`///`, `//!`, `/* */`) rather than adding `_ => {}` is
  deliberate: a wildcard would silently drop a doc comment, the exact defect doc 02 §10.1
  exists to prevent. The *parser* side of doc comments remains plan 23.
- **`Keyword::from_str` was renamed `from_text`**, because clippy flags a non-trait
  `from_str` as confusable with `std::str::FromStr`.
- **`Cursor::src` was deleted rather than suppressed.** Written speculatively and unused; an
  `#[allow(dead_code)]` would have been worse than removing it.
- **Leaf 9's test file list changed.** The plan named six files by surface area; the work
  produced `lexical_surface.rs` and `continuation.rs` plus unit tests living with the
  scanner each one covers, which doc 06 §7 prefers for private constructs.

### What plan 03 inherits

`examples/lexical.sdx` lexes cleanly but is **rejected by the parser**, as expected — the
verification run confirms it fails at `lexical.sdx:7:9`, the first binding plan 01's grammar
cannot express. That file is plan 03's work list: every line in it is a token sequence with
no production yet.
