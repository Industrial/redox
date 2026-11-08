# Redox — COMPILER.md

Status: toolchain decisions. Authored by plan 01 (`docs/05 - 01 - Vertical Slice.md`),
leaf 9. Extended in place by later plans as further decisions are measured.

Format follows `/data/Code/emerald/spec/COMPILER.md`: for each decision, the axes compared,
the decision, the rationale, the evidence paths, and a **named revisit trigger**. Doc 02
§3.5 adopts that format because Emerald's own history shows the value of writing the
revisit condition down — its Cranelift-versus-LLVM trigger fired for real, and the record
of what the trigger was is what made the second decision fast.

---

## 1. Parser: hand-written recursive descent, over LALRPOP

**Chosen: hand-written recursive descent with a Pratt expression loop.**

Doc 02 §5.2 recommended this on five arguments; doc 03 §23 item 6 recorded honestly that
the recommendation **stood on argument rather than measurement**. Plan 01 leaf 4 is that
measurement. Both arms were built against the identical slice —
`main = ->` / `print "hello"` — and both parse it correctly.

The six axes were **fixed before either arm was built**, so the result could not be
rationalised afterwards. This mirrors Emerald tuning Cranelift's optimisation level
*before* measuring it against LLVM, specifically so its own bake-off could not be rigged.

| # | Axis | Hand-written | LALRPOP 0.23.2 |
|---|---|---|---|
| 1 | Expresses the off-side rule without lexer/parser feedback? | Yes — the lexer emits `Indent`/`Dedent` and the parser consumes them as ordinary tokens | **No.** LALRPOP's built-in tokenizer cannot produce `Indent`/`Dedent` at all, so the arm needs an `extern` token enum plus a hand-written pre-lexer — a 64-line `prelex` in the arm's own `main.rs` |
| 2 | Can carry trivia onto AST nodes? | Yes — `leading`/`trailing` are read from the token at each node's start, in the one place that node is built | Possible but invasive: every production's action code must thread it by hand, and the generated parser offers no hook for "attach this token's trivia to this node" |
| 3 | One pass reports more than one error? | **Yes.** Recovery at item boundaries is ~15 lines. Measured: two malformed items in one file produce two errors | **No.** Measured: the same input reports one error and stops. Emerald needed a dedicated plan (26) threading `errors: &mut Vec<_>` through its grammar to get this |
| 4 | Diagnostic for a missing `->` | `expected `->` after `=`` with a byte span, resolved to `file:2:7` by the CLI | `UnrecognizedToken { token: (6, Newline, 6), expected: ["\"->\""] }` — names the expected token, but carries byte offsets only, with no line, column or snippet |
| 5 | Setup cost | **Zero third-party dependencies.** `Cargo.lock` holds 8 packages, all first-party. No build script | **46 crates compiled** on a cold build, plus a `build.rs` codegen step and a generated 1,002-line parser module |
| 6 | Lines for this slice | 241 non-comment lines (`redox-parser/src/lib.rs`, excluding tests) | 26 non-comment grammar lines — **plus** the 64-line pre-lexer axis 1 forces, so 90 hand-written lines either way, against a 1,002-line generated module |

### Rationale

Axis 1 is the decisive one, and it is stronger than doc 02 §5.2 predicted. The argument was
that LALR(1) "fits indentation-sensitive grammars badly"; the measurement is that it cannot
express them at all without a hand-written lexer feeding an `extern` token enum. **The
generated parser therefore replaces only the smaller half of Redox's front end** — the half
that axis 6 shows is 26 lines of grammar against 1,002 lines of generated code, while the
lexer, which is where this language's real complexity lives (off-side rule, trivia,
whitespace adjacency, and later doc 03 §4.1's longest-match sigil table), stays
hand-written regardless.

Axis 3 settles the rest. Doc 01 §29 wants good diagnostics and doc 02 §11 puts them in
Milestone 2, so multi-error recovery is near-term work, not speculative. It is ~15 lines in
recursive descent and was a whole plan in Emerald.

Axis 2 matters more for Redox than for most languages, because comments must reach the
generated `.xrs` (doc 02 §10.1) and the formatter must round-trip them (plan 34). Emerald's
formatter loses every non-doc comment precisely because that information was discarded
early; a design where trivia threading is invasive is a design that invites the same defect.

Axis 4 is closer to a wash than expected, and is recorded as such. LALRPOP's
`UnrecognizedToken` does name the expected token, which is more than doc 02 §3.5's summary
of Emerald's experience suggested. Neither arm produces publication-quality diagnostics
without the `miette` layer plan 05 adds.

**Not decided on the line count.** Axis 6 superficially favours LALRPOP, 26 lines against
241. That comparison is misleading in both directions: the grammar file excludes the
pre-lexer axis 1 forces, and the hand-written figure includes error recovery that the
LALRPOP arm simply does not have.

### Evidence

- Hand-written arm: `crates/redox-parser/src/lib.rs`; 11 tests, including
  `recovers_at_item_boundaries_and_reports_every_bad_item` for axis 3.
- LALRPOP arm: built and executed in `.tmp/bakeoff/` (gitignored scratch, excluded from the
  workspace by the root `Cargo.toml`'s `exclude`). Its three measured outputs:
  ```text
  SLICE: ok -- [Item { name: "main", stmts: [Stmt { expr: Call("print", "\"hello\"") }] }]
  MISSING-ARROW: UnrecognizedToken { token: (6, Newline, 6), expected: ["\"->\""] }
  TWO-ERRORS: reported 1 error, stopped: UnrecognizedToken { token: (3, Newline, 3), expected: ["\"->\""] }
  ```
- Dependency counts: `cargo tree` / `Cargo.lock` (8 first-party packages) versus 46 crates
  compiled in the LALRPOP arm.

### Revisit trigger

Re-open this decision if **either** holds:

1. The hand-written expression parser exceeds roughly 1,500 lines — the per-file cap of doc
   02 §5.4 — while still failing to express doc 03 §5.1's call grammar, §5.4's operator
   sections, or §4.2's `...` position rules. That would mean the grammar's real difficulty
   is in precedence and ambiguity rather than in the off-side rule, which is the regime a
   generator is good at.
2. A parser generator appears that produces `Indent`/`Dedent` natively **and** offers a
   trivia-attachment hook. That would retire axes 1 and 2 together, which are the two this
   decision actually rests on.

Do not re-open it on line count alone (see "Not decided on the line count"), and do not
reach for a generator mid-plan without re-running the axes.

---

## 2. Lexer: hand-written, over `logos`

**Chosen: hand-written.** Not a bake-off; a consequence of §1 axis 1 and therefore recorded
rather than measured. Emerald uses `logos` in 116 lines because Ruby-ish syntax with `end`
delimiters needs no whitespace sensitivity. Redox needs four things a regex-dispatch lexer
does not provide:

1. the off-side rule, with an indent stack;
2. trivia capture, so comments reach the generated `.xrs` (doc 02 §10.1);
3. whitespace-adjacency on every token, which doc 03 §5.1's call grammar and §5.4's
   `(-1)`-versus-`(- 1)` rule both require;
4. continuation suppression inside delimiters (plan 02), which is lexer state, not a
   pattern.

**Evidence:** `crates/redox-lexer/src/lib.rs`, 16 tests.

**Revisit trigger:** none. Items 1 and 4 are not expressible as a token pattern set.

---

## 3. Measured: rustc accepts a `.xrs` input file directly

Doc 01 §1.1 left this open:

> The compiler must account for Rust compiler input-file extension handling. If necessary,
> it should use standard `.rs` staging files or compiler input through stdin rather than
> requiring changes to Rust itself.

**Measured** (2026-11-08, rustc 1.97.1, this repository's devenv shell):

```text
$ printf '// probe\nfn main() {\n    println!("hello");\n}\n' > hello.xrs
$ rustc --edition 2024 -o out hello.xrs
exit 0, stderr: 0 bytes
$ ./out
hello
```

**The "if necessary" clause does not fire.** No `.rs` staging file and no stdin plumbing is
needed for the direct-rustc path, so `crates/redox-cli/tests/hello.rs` invokes rustc on the
`.xrs` as-is.

**Scope limit, so plan 09 is not surprised:** this is about `rustc` invoked directly.
**Cargo is different** — it discovers crate roots by name (`src/main.rs`, `src/lib.rs`) and
will not pick up a `.xrs`. That is why doc 02 §6.6 recommends the `build.rs` + `OUT_DIR` +
`include!` route, and this finding neither supports nor weakens that recommendation.

---

## 4. Found: doc 01 §3.1 prescribes four-space indentation, every example uses two

A documentation inconsistency, recorded rather than silently resolved. Doc 01 §3.1 says
"Use four spaces per indentation level", while every example in docs 01, 02, 03 and 05 — the
`hello.sdx` of plan 01 §3 included — uses two.

**How plan 01 resolved it:** `redox-lexer` fixes no width at all. Any increase opens a block
and the matching decrease closes it, so two-space and four-space sources both lex and only
*inconsistency* is an error (`rejects_an_inconsistent_dedent`). This is Python's rule, and
it makes the discrepancy moot for the compiler.

Enforcing one canonical width is a formatter concern (**plan 34**), and that is the venue
where doc 01 §3.1's text should be corrected to whichever width the formatter emits.

Emitted Rust is separately fixed at **four spaces**, matching rustfmt's default so generated
code does not read as foreign. That is independent of both the source width and this
repository's own two-space Rust style (`rustfmt.toml`).

---

## 5. Decided: the emitter does not call rustfmt

**Chosen: emit canonical formatting directly.** Doc 02 §5.3's three reasons, restated
because the temptation recurs:

1. Output would depend on an external tool's version, so the same input could produce
   different `.xrs` on two machines.
2. Determinism would break, and with it the exact-text golden tests (doc 01 §3.4,
   acceptance criterion 7).
3. The span map plan 06 needs would be destroyed, since rustfmt rewrites positions.

A `--rustfmt` flag may be added later as an opt-in, and must then mark the span map invalid.

---

## Cross-references

- Pipeline architecture and crate split: `docs/02 - Redox Update Blueprint.md` §5.
- Lowering per construct: `docs/03 - Redox Surface Design.md` §§5–15.
- Plan sequence and ordering commitments: `docs/04 - Redox Plan of Plans.md`.
