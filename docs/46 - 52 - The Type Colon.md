# Plan 52 — The Type Colon

Plan ID: `type-colon`
Depends on: 51 (`construct-coverage`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

One token for a type, one token for a path, and no rule about whitespace.

Today Redox has **three spellings for a type annotation** and **one of them is also a path**,
told apart by a space:

| Position | Today | Where |
|---|---|---|
| Signature | `f :: i32 -> i32` | everywhere |
| Typed binding | `total : i32 = 40 + 2` | `examples/expressions.sdx:19` |
| Ascription | `n = 1 :: u64` | `examples/ascription.sdx` |
| **Path** | `Vec::new ()` | everywhere |

Doc 18 §5.2 resolves the collision by **spacing**: a spaced `::` is a type, a tight `::` is a
path. That rule works and it is the wrong kind of rule — it makes a one-character edit change
what a line means, in a language whose other hazards this milestone is removing.

This row takes `:` for every type position and leaves `::` meaning a path and nothing else.

### 1.1 The spacing rule is a hazard of a kind this repository already regrets

The pattern is familiar. Doc 18 §4 retired doc 03 decision 50's `(-1)`-versus-`(- 1)` adjacency
rule for exactly this reason: a semantic difference carried by a space is one a formatter, a
careless edit, or a line-wrap can erase. And doc 18 §5.1 names the hazard its own comment syntax
introduced — *"**The one new hazard is real and worth naming:** `x - -1` needs its space, because
`x --1` is a comment"* — sharpened in §11.5 and implemented in `scan_comment.rs`.

`::` is the same shape of problem with a larger blast radius, because it appears on every
declaration line in the language.

### 1.2 What `:` buys, beyond removing the rule

Three things, and the second is the one that surprised the measurement.

**Paths become unambiguous, which makes `::<T>` available.** With `::` meaning only a path,
Rust's turbofish is a path segment followed by a type-argument list and needs no new lexing.
Plan 62 declines to *use* it — ascription turns out to reach every case tried, including
capability reads — but the ambiguity that made it unaskable is what this row removes.

**`data` field declarations become literally Rust's struct syntax.** `data Point = Point { x :
i32, y : i32 }` emits `struct Point { x: i32, y: i32 }` with the token unchanged. Doc 18 §2.4's
stated reason for record *literals* taking `=` is *"matching `::` for types"* — a symmetry
argument, not a §5.2 availability one — and this row preserves the symmetry while changing both
halves: `:` declares a type, `=` supplies a value. The declaration now reads the same as the Rust
beside it, which is doc 02 §5.2's argument for why the two surfaces should not diverge where they
need not.

**The typed-binding form stops being a third spelling.** `total : i32 = 42` exists in exactly
**one** line of the corpus and duplicates ascription. Under this row it is not retired — it is
the *same* construct as the signature and the ascription, which is what makes three spellings
into one.

### 1.3 What it costs, measured

| Thing | Count | Notes |
|---|---|---|
| Spaced `::` in `examples/` | **184** | every one becomes `:` |
| Tight `::` in `examples/` | 44 | unchanged |
| `.sdx` files touched | 29 | all of them |
| Goldens to re-bless | 28 | generated Rust is **unchanged**, so the diffs are comments only |
| Single-`:` bindings to re-read | 1 | `examples/expressions.sdx` |

The generated Rust does not change at all — this is a surface-only row — so every golden diff is
either empty or a comment. That is doc 18 §1's own measurement discipline: a surface change
whose output is byte-identical is a surface change, and a golden that moves is a bug.

Docs 01, 02, 03 and 18 all use `::` in prose and code blocks. They are amended, not left
standing, per the rule plan 46 and 47 both applied.

### 1.4 Haskell's `::` is given up, and the reason is stated rather than hidden

This is the first row in the milestone to trade a Haskell spelling for a Rust one, and it should
say so plainly. `f :: Int -> Int` is Haskell's most recognisable line, and `f : i32 -> i32` is
ML's and F#'s.

The trade is accepted because the thing being borrowed was never the *semantics* — Redox's
signature line already does two things Haskell's does not (doc 18 §2.2's synthesised generic
list and bound clause) and one thing it cannot (markers like `const` and `async`, plan 16 and
plan 39). The `::` was costume. The whitespace rule is a real defect. Keeping the costume at the
price of the defect is the trade this row declines.

Doc 18 §5.3's lowercase-type-variable rule is untouched: `t` is still a variable and `i32` is
still not, by the closed primitive list.

---

## 2. Verified preconditions

Measured 2026-11-10.

**All three spellings parse today**, which is what makes them three spellings rather than two
and a bug:

```
add :: i32 -> i32 -> i32        -- signature
total : i32 = 40 + 2            -- typed binding, examples/expressions.sdx:19
n = 1 :: u64                    -- ascription, examples/ascription.sdx
print (i32::max 1 9)            -- path, tight
```

**It is the single-`:` binding that is the outlier, and it appears once.** One line in 703.

**Ascription currently emits a three-line block**, which plan 62 fixes and this row does not:

```
HashMap::new () :: HashMap<String, u32>
->  { let __rdx_t: HashMap<String, u32> = HashMap::new(); __rdx_t }
```

**And a type is verbatim text** (doc 03 §14.1), so a generic type carrying `::` inside it —
`Result<i32, std::num::ParseIntError>` — is copied through and is not this row's business. That
is the one place a `::` survives inside a type position, and it survives because Redox never
parses it.

---

## 3. Concrete proof this plan targets

**Every example compiles and runs unchanged, with byte-identical generated Rust, after 184
substitutions.** The corpus is the proof: a surface-only change that alters output is not
surface-only.

Three cases chosen because they fail differently:

1. **A signature with a generic and a bound.** `largest : Ord t => Option<t> -> t`. If the
   parser takes `:` as the start of a type but keeps `::`'s precedence, the `=>` clause is
   mis-split and doc 18 §2.2's synthesis produces the wrong generic list.
2. **An ascription inside a pipeline.** `xs |> collect : Vec<i32>`. Ascription binds loosest of
   anything in an expression, so this must still ascribe the pipeline and not the `collect`.
3. **A path and a type on one line.** `b = Sorted::new () : Sorted<String, u32>` — both tokens,
   adjacent, no spacing rule to lean on.

---

## 4. Leaves

### Leaf 1 — `leaf-lexer-colon`

`:` and `::` are **already both tokens** — `operator_table.rs` lists them and longest-match
already orders them — so the lexer needs no change on that point. What this leaf deletes is the
*parser's* adjacency machinery: `Token::ws_before`, the `spaced_colon_colon.rs` predicate and
`tight_colon_colon.rs`, plus the call sites that cite §5.2. That is the whole of doc 18 §5.2's
machinery and all of it goes.

**Corrected after reading them.** The draft named `field_decl.rs` as the fifth predicate call
site; it is not one. `field_decl.rs` calls `expect(ColonColon)` — it is a *consumer* of `::` as
a type marker, so it belongs with the six sites leaf 2 and leaf 4 retarget, not with the
predicates. The five predicate sites are:

| Site | Predicate | What it decides |
|---|---|---|
| `path_or_ident.rs:21` | `tight_colon_colon` | `Vec::new` is a path, not `Vec : :new` |
| `path_or_ident.rs:27` | `tight_colon_colon` | continuing a multi-segment path |
| `constructor_path.rs:21` | `tight_colon_colon` | `Color::Red` in a pattern |
| `record_pattern.rs:37` | `tight_colon_colon` | a qualified record-pattern path |
| `lambda.rs:35`, `parse_expr.rs:30` | `spaced_colon_colon` | ascription, not a path |

And the six `expect(ColonColon)` sites that become `expect(Colon)`: `parser.rs:377` (the
signature), `parser.rs:411` (the equation lookahead that distinguishes a signature from a
binding), `class_member.rs:48`, `const_decl.rs:32`, `extern_block.rs:61`, `field_decl.rs:26`.

**A hazard the draft missed.** `split_format_spec.rs` walks an interpolation looking for the
`:` that starts a format spec, and it already skips `::` to let `#{Vec::len x}` through — but it
has no reason to skip a *lone* `:`, because today a lone `:` inside `#{…}` can only be a format
spec. After this row `#{total : u64}` is a legal ascription, and the splitter would cut it at the
colon and emit `u64` as a width. Leaf 3 must teach it the difference; `tests/golden/strings.xrs`
is the witness.

### Leaf 2 — `leaf-signature-and-binding`

The signature line and the typed binding take `:`. They become one parse, since a signature is a
binding's type without a value — which is what they always were and what two spellings obscured.

### Leaf 3 — `leaf-ascription-token`

Ascription takes `:`. Precedence is unchanged — loosest binding power there is, which is
recorded in `parse_expr.rs` and not in doc 18 §6; §6 gives only the lowering. Note `$` sits at
binding power 1, *looser than every operator and tighter than ascription*, so the two do not
collide. Lowering is untouched — plan 62 owns the block.

### Leaf 4 — `leaf-record-fields`

`data` and `errors` field declarations take `:`. Record *literals* keep `=` (doc 18 §2.4), and
the asymmetry is now meaningful rather than forced: `:` declares a type, `=` supplies a value.

### Leaf 5 — `leaf-corpus-migration`

All 29 examples, the goldens, `tests/golden/`, `tests/run/`. Hand-migrated and re-read, per the
decision recorded in §9. Goldens re-blessed and the diff reviewed; a non-comment diff is a bug
in this row.

### Leaf 6 — `leaf-docs`

Docs 01, 02, 03, 18 and `docs/GRAMMAR.md` amended. Doc 18 §5.2's spacing rule is struck and
replaced with the one-token rule, and the reason is recorded in §1.4 rather than left to a
reader to infer.

---

## 5. Acceptance criteria

1. `x : T` is the only type-annotation spelling: signature, binding and ascription.
2. `::` appears only in a path, and a *spaced* `::` is a named diagnostic suggesting `:`.
3. All 29 examples transpile, compile under rustc with no output, and run with their stdout
   asserted.
4. **Every golden diff is empty or comment-only.** Asserted by review of the blessed diff, and
   stated as a criterion because it is this row's definition of correctness.
5. `largest`, the pipeline ascription, and the path-plus-type line from §3 are each exercised by
   an example.
6. `construct_coverage` (plan 51) is green: the new `:` spelling is indexed and covered.
7. Doc 18 §5.2 no longer specifies a whitespace rule; `docs/GRAMMAR.md` grades the token.
8. A type containing `::` internally — `Result<i32, std::num::ParseIntError>` — still passes
   through verbatim, asserted.

---

## 6. Out of scope, and what it discloses

**Turbofish is not added here.** `::<T>` becomes *spellable* by this row and is not spelled by
it; plan 62 takes the decision and declines it, on the measured ground that signature-driven
inference reaches every case tried.

**Ascription's lowering is not touched.** The three-line block stays until plan 62. This row
moves a token; conflating it with a lowering change would make the "goldens are comment-only"
criterion unavailable, and that criterion is the row's only real proof.

**`as` casts remain unreachable.** `docs/GRAMMAR.md` grades them `UNREACHABLE` (plan 46), and
plan 48 counted 12 blocking lines carrying one. A reader may reasonably expect `:` and `as` to be
related; they are not — ascription asserts a type, `as` converts a value, and conflating them
would silently change numeric values. Recorded because the expectation is natural and the answer
is no.

**Haskell's signature spelling is gone and will not come back.** §1.4 is the argument. A future
row proposing to restore `::` would be reintroducing the whitespace rule, and should be refused
on that ground rather than re-argued on aesthetics.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under real rustc, runs, stdout asserted; goldens
re-blessed with a reviewed, comment-only diff; `construct_coverage` green; `grammar_inventory`
green.

---

## 8. Emerald reuse

Plan 34's formatter, as the migration tool doc 02 §4.5 says it is for. 184 substitutions across
29 files is exactly the shape of change the formatter was justified by — and doc 04's fourth
ordering commitment exists because Emerald's plan 71 cutover missed 113 fixtures.

The commitment's corollary protects this row directly: **every `.sdx` fixture is a file, and
tests reference files**, so there are no string-literal fixtures hiding from the migration.
`no_literal_expectations.rs` asserts it.

---

## 9. Decision log

1. **`:` for every type position, not `::` with a better-specified spacing rule.** A rule that
   has to be specified more carefully is still a rule a space can break.
2. **Migrate by hand, not with a rewriter.** 184 substitutions in 29 files is a day's work; the
   corpus exists to be read, and reading every line during conversion is the point. The same
   answer is reached in plan 57 for its 52 sites, so neither row defers the argument to the
   other: **hand migration is this milestone's standing choice**, and a rewriter would need a
   corpus larger than any row here produces.
3. **One parse for signature and typed binding.** They were always the same construct. Keeping
   two parsers for it was the accident that made `total : i32 = 42` exist in one line of the
   corpus and nowhere else.
4. **Record literals keep `=`.** Doc 18 §2.4's reason was that `:` had no job; the reason is now
   that `:` has a *different* job. The outcome is the same spelling with a better justification.
5. **A spaced `::` is a diagnostic, not silently a path.** Every program written before this row
   has 184 of them, and the error message is the migration guide.
6. **Give up Haskell's `::` explicitly, in writing.** §1.4. The alternative is a future reader
   finding `:` and assuming it was carelessness.

---

## 10. Result

*Not yet executed.*
