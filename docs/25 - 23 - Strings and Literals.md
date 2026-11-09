# Plan 23 — Strings and Literals

Plan ID: `strings-and-literals`
Depends on: 02 (`lexical-surface`), 17 (`comments-as-trivia`)
Milestone: 5 — *the language, as specified*
Status: `planned`

---

## 1. What this plan is

Doc 03 §11: **interpolation**, the four Rust literal kinds doc 01 left unreachable, and the
`"""` block's dedent reaching the emitter.

```text
"hello #{name}"             format!("hello {}", name)
"total: #{amount:.2}"   →   format!("total: {:.2}", amount)
r"^\d+$"                    r"^\d+$"
b'x'  b"bytes"  c"ffi"      b'x'  b"bytes"  c"ffi"
"""                         "line one\n  line two"
  line one
    line two
"""
```

Doc 18 touches none of it — §5.1's table keeps `"#{expr}"` explicitly, and §11.2 lists
*"string interpolation is plan 23"* among what the conversion left to other plans. So this
is doc 03 §11 as written.

### 1.1 The lexer finished this in plan 02, and nothing consumed it

`TokenKind::Str` carries `parts: Vec<StrPart>` with every interpolation already scanned, its
body already lexed, and its format spec already separated. `StrPart`'s own doc comment says
`parts` is *"what plan 23's emitter consumes"*. `RawStr`, `ByteStr`, `CStr`, `Char` and
`Byte` are all tokens. The dedent is computed by `dedent_block`.

**None of it reaches the output.** This plan is mostly parser and emitter work on top of a
lexer that has been ready for twenty-one plans.

**That last clause turned out to be wrong, and §10.6 records it.** The lexer was ready for
the cases anyone had tried: three of the five defects this plan fixed are in the lexer, one
of them a case plan 02's own doc comment asserted worked (§10.2). The sentence is corrected
rather than deleted, because *a plan that believes an upstream stage is finished will write
its own leaves too small* is the lesson, and it is invisible if the belief is edited away.

### 1.2 The interpolation gap is silent, and that is why this row matters

Measured before writing this plan:

```text
print "hello #{name}"   →   println!("hello #{{name}}");
```

That is **valid Rust that prints `hello #{name}`**. No diagnostic, no rustc error, and the
program runs. Doc 02 §5.2 names this exact failure — output that reads as Rust and means
something else — as the worst available, and it is live in the compiler today for the single
most common thing a string does.

The doubling comes from `print_call` routing a `StrLit` through `double_braces`, which is
correct for a brace the author wrote and wrong for one the lexer already understood. The fix
is not to stop doubling: it is for an interpolated string to stop being a `StrLit`.

### 1.3 The `"""` gap is loud, and discards work already done

```text
text = """          let text = """
  line one              line one
"""                 """;
```

`Expr::StrLit` stores the literal **raw, including its delimiters**, which is right for
`"x"` and wrong for a `"""` block: Rust has no triple-quoted string, so the emitted line is
not Rust at all. rustc reports it, so nothing silent happens — but it reports against a line
the author did not write, and `dedent_block`'s result is discarded on the way.

So a `"""` block must reach the emitter as its **dedented value**, escaped into a single
Rust string. Doc 03 §11.2 adds a constraint: the form *"must preserve line numbering (doc 02
§5.1 item 5) because the source map depends on it"* — which the lexer already honours by
keeping the token's span over the whole block.

### 1.4 Why the spec text is pasted and never read

`#{amount:.2}` becomes `{:.2}`. Redox does not parse `.2`, does not validate it, and does
not know what it means. Doc 03 §11.1 is explicit: *"text after `:` up to the closing brace
is spec text, passed through unchanged. A malformed spec is reported by rustc against the
interpolation's own location through the source map."*

That is AGENTS.md's rule — every check rustc can make, rustc makes — applied to a grammar
Rust already owns. It is also what makes *"every Rust format spec reachable without a second
construct"* true rather than aspirational: there is no list of supported specs to fall behind.

### 1.5 An interpolation holds an expression, not a name

The lexer lexes the interpolation's body with the ordinary scanners and hands over a
`Vec<Token>`. So `"#{a + b}"` and `"#{user.name}"` and `"#{f x}"` are all available for
free, and the plan's job is to run the ordinary expression parser over that token slice
rather than to accept an identifier.

This needs one thing from the parser: the ability to parse an expression from a token slice
that is not the main stream. `Parser::new` already takes a slice, so it needs no new
constructor — only a synthetic `Eof` appended, because `lex_fragment` strips the one the
main stream carries. Not a second parser: writing one is how two grammars drift apart.

---

## 2. Admission tests (doc 03 §2)

| Test | Verdict |
|---|---|
| Lowering fixed by source text alone? | Yes. `"a#{e}b"` is `format!("a{}b", e)` — the shape is syntactic and the spec is copied |
| Nothing outside core/alloc/std? | Yes. `format!` is `alloc`'s; the four literal kinds are rustc's own |
| Preserves Rust semantics? | Yes — each literal kind *is* the Rust literal, passed through |
| Buys reachability? | Yes, twice. `char`, byte strings and C-strings *"had no spelling at all"* (§11.2), and §11.3 declines regex literals **on condition** that `r"…"` exists |

---

## 3. Concrete proof this plan targets

`examples/strings.sdx`, which must **run** and print a sequence only a correct lowering
produces:

- an interpolation whose value is an **expression**, not a name, so a lowering that accepted
  only identifiers fails;
- an interpolation with a **format spec**, printed so the spec demonstrably took effect —
  `#{amount:.2}` of `1.5` prints `1.50` and a dropped spec prints `1.5`;
- a brace the author wrote **beside** an interpolation, so escaping and interpolation are
  exercised in one template and the fix for one cannot break the other;
- a `"""` block printed, so the dedent is visible in the output;
- `r"…"`, `'c'`, `b'x'`, `b"…"` and `c"…"` each used for something, since a literal kind
  that parses and emits wrongly is caught by rustc and one that parses and emits *nothing*
  is caught only by running.

The brace-beside-interpolation case is the one the current compiler cannot express at all.

---

## 4. Leaves

### Leaf 0 — `leaf-lexer-repairs`

**Added during implementation, not planned.** Three defects in plan 02's scanner, all
reachable from doc 03 §11.2's own table and all found by writing the example: a string
literal inside an interpolation ended the outer string (§10.2), `\x` scanned greedily and
rejected `b"\x7fELF"` (§10.3), and a decoded control character was re-emitted literally
(§10.3).

### Leaf 1 — `leaf-interp-node`

`Expr::StrInterp { parts, span }` with `StrInterpPart::{Text, Interp { expr, spec }}`, and
the parser that builds it: `interpolated_string` over the lexer's parts, and
`expr_from_tokens` over each `StrPart::Interp`'s slice.

A string with **no** interpolation stays an `Expr::StrLit`, so every existing path is
untouched — a plain literal is still passed through raw, which is what plan 02 designed
`raw` for.

### Leaf 2 — `leaf-interp-emit`

`format!` with the parts assembled: literal text escaped and brace-doubled, each
interpolation contributing `{spec}` to the template and its expression to the argument list.
`print` of an interpolation is `println!` with the same template — one function, two macros.

### Leaf 3 — `leaf-literal-kinds`

`r"…"`, `r#"…"#`, `b'x'`, `b"…"` and `c"…"` as expressions. Pure passthrough, including the
`#` count on a raw string, which is why the token carries `hashes`.

### Leaf 4 — `leaf-dedent-block`

A `"""` block reaches the emitter as its dedented value, escaped into one Rust string, with
the token's span preserved for the source map.

### Leaf 5 — `leaf-proof-and-gate`

`examples/strings.sdx`, emit tests per form, a golden, a run expectation, and the gate.

---

## 5. Acceptance criteria

1. `"hello #{name}"` emits `format!("hello {}", name)`.
2. `"#{a + b}"` and `"#{user.name}"` work — an interpolation holds an **expression**.
3. `"total: #{amount:.2}"` emits `format!("total: {:.2}", amount)`, the spec copied verbatim
   and uninterpreted.
4. A brace the author wrote is doubled, **and** an interpolation in the same string is not —
   asserted together, because the current defect is exactly the two being confused.
5. `print "hi #{name}"` emits `println!("hi {}", name)`, not `format!` inside a `println!`.
6. A string with no interpolation emits exactly what it emits today — asserted, so the new
   node does not change the old path.
7. `r"…"`, `r#"…"#`, `b'x'`, `b"…"` and `c"…"` each parse and emit verbatim.
8. A `"""` block emits one Rust string holding the dedented text, and the token's span still
   covers the block.
9. Several interpolations in one string, and an interpolation holding a string literal of its
   own, both work.
10. `examples/strings.sdx` compiles with **zero** rustc output, runs, and matches its
    `.stdout`; a golden covers the text.
11. Any new diagnostic has an `RDX` code reached by the UI corpus.
12. A string literal inside an interpolation works — `"outer #{String::from("inner")} done"`
    — which plan 02's `lex_fragment` claimed and did not do. *Added with leaf 0.*
13. `b"\x7fELF"` parses, and a decoded control character is re-escaped rather than written
    as itself. *Added with leaf 0.*

---

## 6. Out of scope, and what it discloses

- **Validating a format spec** is rustc's, per §1.4. `"#{x:nonsense}"` emits
  `format!("{:nonsense}", x)` and rustc reports it — against the interpolation's own
  location, because plan 06's map puts it there.
- **Rust's positional and named arguments inside a spec** — `{0}`, `{name}` — are not
  Redox's. An interpolation carries its own expression, so there is nothing to refer to by
  position. A literal `{0}` the author writes is doubled like any other brace.
- **`write!`, `eprintln!` and the rest of the format family** are ordinary macro calls and
  stay that way. This plan lowers interpolation; it does not grow a macro vocabulary.
- **Sigils beyond `%w`** are declined by doc 03 §11.3 and stay declined. This plan is what
  makes that decline honest: a regex is `Regex::new(r"…")`, which needs leaf 3.
- **Doc comments** are the fourth item in doc 04's row 23 and are **already done** — plan 17
  shipped all four comment forms as trivia, and doc 18 §5.1 changed their markers to `--`,
  `{- -}`, `-- |` and `-- !`. The row's cell was written before either. Nothing to do but
  say so, and correct the stale marker text in `TriviaKind`'s doc comments, which still
  name `#` and `##`.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None usable.** Emerald had interpolation, but it lowered to its own string builder rather
than to a host language's format macro, so the mechanism does not transfer. Its one
transferable lesson is negative and already paid for: Emerald's interpolation accepted only
an identifier for eleven plans, and widening it later meant revisiting every call site.
§1.5 takes the expression form from the start.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | A non-interpolated string stays `StrLit` | One node for every string | The raw passthrough is what plan 02 built `raw` for, and criterion 6 asserts the old path is untouched. One node would make every existing golden a diff |
| 2 | The interpolation's body is parsed by the **ordinary** expression parser over a token slice | A small dedicated grammar | Two grammars drift. The lexer already hands over tokens, so the slice is the natural seam |
| 3 | Spec text is pasted uninterpreted | Validate it | AGENTS.md: every check rustc can make, rustc makes. It is also what makes "every Rust format spec" true with no list to fall behind |
| 4 | `print` of an interpolation emits `println!` with that template | `println!("{}", format!(…))` | One allocation and one macro instead of two, and it is what a person would write |
| 5 | A `"""` block carries its **dedented** value | Keep the raw text and dedent in the emitter | The lexer already computed it, and doc 03 §11.2 ties the form to line numbering the lexer owns |

---

## 10. Result

Shipped. 603 tests pass; `fmt`, `clippy -D warnings` and
`moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit` are green. Every
criterion is met. §1.1 was right that the lexer had finished the scanning — and wrong that
the scanning was finished, which is a different claim and the one that cost the most.

### 10.1 The silent defect, and what it looked like

```text
print "hi #{name}"      →    println!("hi #{{name}}");      -- before
print "hi #{name}"      →    println!("hi {}", name);       -- after
```

The first compiles, runs, and prints `hi #{name}`. Doc 02 §5.2's worst failure mode, live in
the most common thing a string does, for twenty-one plans.

It was not a missing feature so much as two correct things colliding. `print_call` routed a
`StrLit` through `double_braces` — right, because a brace the author wrote must print as a
brace — and the `StrLit` still held *raw source* in which the interpolation had never been
replaced. Neither function was wrong. The fix is that an interpolated string stops being a
`StrLit` at all, which is why §9's decision 1 keeps the two nodes separate and criterion 4
asserts the brace and the hole in **one** template: fixing either alone would have passed a
test that checked only the other.

### 10.2 Plan 02 claimed a case that did not work, and the claim was in a doc comment

`lex_fragment`'s own doc comment has said since plan 02 that *"`"#{f(x, "y")}"` works and
nothing needs a second tokenizer."* It did not work. The **outer** string's scanner stopped
at the first unescaped `"` — the one opening `"y"` — so the string ended mid-interpolation
and the error was `unterminated interpolation; expected }`.

The mechanism was right and incomplete: `split_parts` counted braces to find the
interpolation's end, and `scan_string` counted nothing to find the string's. **Two scanners
walking the same text have to agree about where it ends**, and they now agree by both
calling `skip_interpolation`, which counts braces outside string literals and skips string
literals whole.

Writing the case into the example is what found it, because every simpler interpolation
works. A doc comment is not a test — this is the second time this run that a claim in prose
turned out false (plan 14's §1.1 was the first), and both were found by exercising them
rather than reading them.

### 10.3 Two escape defects, both in literals rustc accepts

- **`\x` scanned greedily.** `eat_while(is_ascii_hexdigit)` on `\x7fELF` consumed `7fE` as
  three digits and rejected the literal. Rust reads exactly two characters and stops, so
  `b"\x7fELF"` is `0x7f`, `E`, `L`, `F` — a four-byte ELF magic, which is the only reason
  anyone writes a hex escape followed by a letter. Fixed to take two.
- **A decoded control character was re-emitted literally.** The lexer decodes escapes, so
  `\x7f` became a `DEL` byte, and re-encoding passed it through as itself. Valid Rust, but
  it puts an unprintable byte in a human-facing artifact (doc 02 §10.1) — one that nothing
  in a diff or a terminal shows. Now every ASCII control character is re-escaped as `\xNN`,
  while `\n`, `\r` and `\t` keep their named forms because those three are what a person
  reads.

Both were in plan 02's scanner, both were reachable from doc 03 §11.2's own table, and both
needed an example that used a byte string for something real rather than for the word
`bytes`.

### 10.4 The diagnostic got better by accident, and the corpus noticed

`RDX0005`'s span used to cover the string's **whole body**; it now covers from the `#{` that
was never closed. That is more useful and was not the point of the change — `skip_interpolation`
simply owns the region it scans.

The first version of it entered *after* the `#{` and the span lost those two characters,
moving four columns right of the thing the message names. The UI corpus caught that in one
run, which is the argument for a corpus that asserts rendered text rather than codes.

### 10.5 Row 23's fourth item was already done

Doc 04's row 23 names *"`##`/`##!` doc comments"*. Plan 17 shipped all four comment forms as
trivia, and doc 18 §5.1 then changed their markers to `--`, `{- -}`, `-- |` and `-- !`. The
row's cell predates both.

What was **not** done is the documentation: `TriviaKind`'s four doc comments still named `#`,
`##`, `##!` and `### … ###`. The scanners were converted and their prose was not, which is
the same drift as §10.2 one layer down. Corrected here, with a line saying when the markers
changed and why — so the next reader learns the history rather than the stale spelling.

### 10.6 What §1.1's framing got wrong

§1.1 said *"this plan is almost entirely parser and emitter work on top of a lexer that has
been ready for twenty-one plans."* Three of the five defects above are **in the lexer**. The
lexer was ready for the cases anyone had tried, which is not the same thing, and a plan that
believes an upstream stage is finished will write its own leaves too small.

The leaf list was right anyway, because leaf 5 is an example that runs — and an example that
runs does not care which crate the defect is in.

### 10.7 The architecture tests, an eighth time

`interpolated_string.rs` declared two constructs; `expr_from_tokens` became a sibling under
`parse/parser/` rather than a child, which is where the depth cap put plan 22's helpers for
the same reason and where the crate's other helpers already were.

### 10.8 One unexplained thing, recorded as unexplained

The first full-gate run after a batch of edits exited 1 with no task reporting a failure;
an immediate re-run with no changes on disk passed. It happened three times across plans 20,
22 and 23. Re-running `moon run :format` and the gate twice in a row afterwards reproduced
nothing — both runs passed and the working tree was byte-identical between them.

So the cause is **not established**, and it is deliberately not written into AGENTS.md's
gotcha list, which is for traps with a verified cause. The mitigation is to re-run and read
the second result; if it recurs with a diagnosable failure, that is when it earns an entry.

> **Cause established 2026-11-09, during plan 34 leaf 4**, and it has earned its entry.
> blaze's worker cache lives on a 6.2 GB tmpfs under `/run/user/1000` — RAM, not the 1.9 TB
> root that `target/` sits on — and a broad enough build writes past it. The worker dies with
> `No space left on device` and blaze reports `could not build <every crate>`, which is why no
> task names a failure, why a re-run passes once the cache is reclaimed, and why every target
> passes when run alone: one target builds less and fits.
>
> The paragraph above held the entry back for the right reason and reached the wrong
> conclusion, because *"re-running reproduced nothing"* is what this failure looks like when it
> is working as described. The fact that would have settled it was in blaze's own output on all
> three sightings. Plan 43 §10.3 records what went wrong with the method: the greps looked for
> `FAILED` and `panicked`, a test runner's words, and blaze's worker does not fail a test.
