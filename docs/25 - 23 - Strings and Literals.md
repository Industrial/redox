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

**None of it reaches the output.** This plan is almost entirely parser and emitter work on
top of a lexer that has been ready for twenty-one plans.

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

This needs one new thing from the parser: the ability to parse an expression from a token
slice that is not the main stream. It is a constructor on `Parser`, not a second parser —
writing a second one is how two grammars drift apart.

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

### Leaf 1 — `leaf-interp-node`

`Expr::StrInterp { parts, span }` with `StrLitPart::{Text, Interp { expr, spec }}`, and the
parser that builds it: `Parser::expr_from_tokens` over `StrPart::Interp`'s slice.

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

*Filled in when the plan ships, including anything above that turned out wrong.*
