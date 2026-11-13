# Plan 52 — The Type Colon

Plan ID: `type-colon`
Depends on: 51 (`construct-coverage`)
Milestone: 9 — *surface coherence*
Status: `done`

---

## 1. What this plan is

One token for a type, one token for a path, and no rule about whitespace.

Today Voltaic has **three spellings for a type annotation** and **one of them is also a path**,
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

The trade is accepted because the thing being borrowed was never the *semantics* — Voltaic's
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
is the one place a `::` survives inside a type position, and it survives because Voltaic never
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
*parser's* adjacency machinery: the `spaced_colon_colon.rs` predicate and `tight_colon_colon.rs`,
plus the call sites that cite §5.2.

**`Token::ws_before` stays, and this leaf's first draft said it went.** Measured
after the fact: the field is still set by the lexer and still read, by
`capture_type.rs:63` and `:118` to decide where a captured type's text ends,
and by plan 51's `construct_coverage.rs:555` and `:604` to tell a tight `::`
from a spaced one. What this row removes is adjacency deciding *what a token
means*, which is the property that made it dangerous — not adjacency being
observable at all.

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

**Done.** `:` is the type marker in signatures, bindings, ascription, lambda
binders, record and `errors` fields, `class` members, `const` declarations and
`extern` blocks; `::` is path-only; `spaced_colon_colon.rs` and
`tight_colon_colon.rs` are deleted along with `Token::ws_before`, and doc 18
§5.2's whitespace rule is gone with them.

### 10.1 The row's own definition of correctness was met exactly

§1 said the generated Rust must not change at all, so every golden diff would
be empty or a comment. Measured: **2 of 30 goldens moved, and both diffs are
comment-only** — filtering the blessed diff for lines that are not comments
returns nothing. Signatures never reach the `.xrs`, which is why. Five
`tests/ui/*.stderr` also moved, all of them the echoed source line.

### 10.2 §2's scale was low by three and a half times

The register and §2 both say "184 substitutions across 29 files". The real
migration is **226 files and 652 lines carrying a spaced `::`**:

| | files |
|---|---:|
| `.rs` — doc comments, inline tests, embedded `.sdx` sources | 122 |
| `.sdx` — `examples/`, `tests/`, `benchmarks/`, `conformance/`, four `crates/*/tests/` trees | 91 |
| `tests/ui/*.stderr` | 5 |
| `docs/*.md` | 5 |
| `tests/golden/*.xrs` | 2 |

§2 counted the `.sdx` corpus under `examples/` and nothing else. The `.sdx`
corpus is in fact spread across **eight** directories, not one, and the
majority of the work was Rust doc comments describing Voltaic syntax.

### 10.3 Five things the plan's map did not contain

1. **`voltaic-fmt` *emits* the type token, at five sites** — `expr_text.rs:168`,
   `item_text/const_text.rs:18`, `item_text/member_text.rs:37`,
   `item_text/constructor_text.rs:27`,
   `expr_text/lambda_text/param_text.rs:11`. Left alone, `voltaic fmt` would have
   emitted source the parser no longer accepts. §4 listed the parser's readers
   and forgot the formatter's writers.
2. **`.sdx` programs written as string literals inside Rust tests broke
   silently-shaped**, and AGENTS.md predicted it: *"Emerald's one syntax
   cutover missed 113 fixtures that lived as string literals inside test
   modules."* Here it was `crates/voltaic-cli/tests/watch.rs`'s three
   `main :: ()` programs, which stopped parsing, so the watcher never emitted
   and all three tests **timed out at the 20-second ceiling** — presenting as
   AGENTS.md's documented load-dependent flake rather than as a syntax break.
   The same class existed in `voltaic-lsp`, `voltaic-fuzz`, `voltaic-driver`,
   `voltaic-diag`, `voltaic-mcp` and `voltaic-cli`: 23 files. `no_literal_expectations.rs`
   polices *expectations*, not *sources*, which is the hole.
3. **One test derived byte offsets from a source string in a comment.**
   `voltaic-driver/src/remap/narrow.rs`'s multibyte test computed `é` at 13..15
   from `"x :: ()\n"` being 8 bytes; at 7 bytes every number moved by one. The
   assertion two lines below it is what caught the arithmetic.
4. **A blanket substitution is wrong in three places**, all of them tests whose
   subject *is* the old spelling: the two new parser tests in §10.4 below, and
   the `1 :: u64` case in `crates/voltaic-emit/tests/ascription.rs`.
5. **There is a second parser in this repository, and it is checked against the
   same corpus.** `tree-sitter-voltaic/grammar.js` still required `::` in a
   signature, so after the migration `crates/voltaic-cli/tests/tree_sitter.rs`
   and `editors.rs`'s `neovim_runs_the_committed_config` reported an `ERROR`
   node in essentially every file under `examples/`. Five editor clients'
   query files are cross-checked by the same test, which is how a previous plan
   found four dead keywords.

   **This generalises past this row.** Any surface change in milestone 9 has
   four readers, not one: the hand-written lexer and parser, the formatter's
   *writers* (§10.3 item 1), the tree-sitter grammar, and the editor queries.
   §4 of this plan listed the first only. Plans 54 and 57 have the same hole
   and now say so in their own §11.

   **The grammar needed a GLR conflict, not a precedence** — which is the one
   technical thing worth carrying forward. `::` winning over `:` required
   nothing: tree-sitter's lexer takes the longest match, and in every state
   where both could begin, `path` or `operator` is the longer token. What it
   needed was that `:` made a signature and a typed binding start identically,
   so `tree-sitter generate` refused outright:

   ```text
   Unresolved conflict for symbol sequence:
     'module'  identifier  _indent  identifier  •  ':'  …
   Possible interpretations:
     1:  (_expression_part  identifier)  •  ':'  …
     2:  (_pattern  identifier)  •  ':'  …
     3:  (signature  identifier  •  ':'  signature_type  _newline)
   ```

   One line in `conflicts` resolves it: `[$.signature, $._pattern,
   $._expression_part]`. The hand-written parser needed nothing because
   `looks_like_a_binding` scans forward to the `=`; the grammar reaches the
   same answer by letting GLR explore all three readings until the `=` or the
   newline arrives. A precedence would have been wrong for the reason the
   grammar's own header gives about `f (x) = y`.

   **And `"::"` stopped being a node type**, which no amount of reading would
   have predicted. Once the four type-position literals became `":"`, the only
   surviving `::` lives inside the named `operator` token and inside the hidden
   `_type_punctuation`, neither of which emits a visible node — confirmed in
   the regenerated `node-types.json`. A query naming an absent node is an
   error rather than a no-op, so `queries/highlights.scm` and its byte-identical
   copy in `editors/zed/` both had to change.

   The vim and VS Code highlighters additionally still called `::`
   "ascription" and coloured no `:` at all. **The gate does not catch that
   class**: `every_grammar_keyword_is_coloured` compares lowercase word
   keywords only, so a wrong *operator* colour in three disagreeing
   highlighters is exactly what that test exists to prevent and exactly what it
   cannot see.

### 10.3a Two things found next door, for later rows

Neither is this row's to fix, and both are the kind of thing that only surfaces
when somebody has to touch the area:

- **`tree-sitter test` is in no gate**, and was passing 1 of its 5 corpus cases
  because the corpus was written in the old syntax. `tree_sitter.rs`'s module
  doc explains why the corpus is distrusted, but an uncheckable file in the
  repository rots, and this one had. All five pass now with their expectations
  unchanged — which independently says the `:` migration moved no tree shape.
- **The tree-sitter pin has drifted.** `docs/36 - 41 - Tree-sitter Grammar.md`
  records 0.26.11 and `devenv.nix` pins `pkgs.tree-sitter` with no version,
  which currently resolves to 0.26.9. Doc 41's stated reason for pinning is
  that the committed parser must be regenerable identically on another
  machine; it is not. The committed parser is still `LANGUAGE_VERSION 15` and
  `the_generated_parser_is_abi_15` passes, so nothing is broken today.

### 10.4 Criterion 2, in two halves — the second one was not free

With a **literal** on the left, `n = 1 :: u64` was a parse error the moment
ascription moved to `:`: a literal cannot begin a path, so there is nothing for
`::` to join.

With a **name** on the left it was not. `y = x :: u64` was silently consumed as
the path `x::u64` — `u64` is a perfectly good segment — and the first sign of
trouble was rustc reporting an unresolved path against a line the author did
not write. That is the exact failure mode doc 18 §5.2's whitespace rule existed
to prevent, which makes it the one thing this row could not retire the rule
without replacing.

It is replaced by `RDX0110`, and the replacement needed no revival of
`Token::ws_before`. Token **spans** survive the deletion, so a gap between the
previous token's end and the `::`'s start *is* whitespace, and the rule

> a `::` with whitespace before it is never a path

is positional rather than a guess about intent — paths are written tight in
Rust and in Voltaic alike. `path_or_ident` carries the check and tracks
`tight_through` across a multi-segment path, so `A :: b::c` and `A::b :: c` are
both caught and `A::b::c` is untouched.

```text
RDX0110

  × `::` joins a path, so it takes no space before it — write `:` to annotate
  │ a type, or close up the `::` to name a path
   ╭─[spaced_path_separator.sdx:4:9]
 4 │   y = x :: u64
   ·         ─┬
   ·          ╰── here
```

The test that recorded the gap is inverted rather than deleted —
`a_spaced_colon_colon_after_a_name_is_refused`, with
`a_tight_colon_colon_is_still_a_path` beside it so the half that must not
regress is asserted too.

**No continuation-line false positive is possible.** A `::` opening a
continuation line would be whitespace-separated, but an identifier does not
continue a line and `|>` is the only leading continuation operator the lexer
recognises, so `Foo\n  ::bar` is two statements and never one path.

### 10.4a Three byte offsets written in comments, all wrong by one

The type marker lost a character, so every offset derived from a source string
containing a signature moved by one. Three assertions carried such a number:
`voltaic-diag`'s `line_col` (20 → 19), `voltaic-driver`'s `narrow` (14..16 →
13..15) and `voltaic-parser`'s `parses_the_hello_slice` (20..25 → 19..24). A
fourth, in `voltaic-emit`'s `sink.rs`, was found during the parser work.

Each was caught by an assertion rather than by review, and in two of the three
the *comment* deriving the number was wrong alongside it. The test immediately
below `parses_the_hello_slice` —
`nodes_carry_spans_that_index_the_source`, which slices the source instead of
writing the offset down — needed no change at all, which is the argument for
writing them that way.

### 10.6 The normative documents were the last thing left, and the worst

This is where the row nearly stopped at *"the compiler is migrated"*. It was
not finished until `docs/18 - Haskell.md` was, and that document **declares
itself normative** — *"This is the Voltaic surface"* — while its §5.2 still
*specified the whitespace rule this row deleted*. A normative section
contradicting the compiler is worse than a missing one, because a reader has no
way to tell which of the two won.

§5.2 is rewritten as the superseded section it is: what the rule was, the
argument for it — it agreed with both communities' formatters, Haskell always
spacing `::` and Rust never spacing a path, so nobody would have to learn it —
and why that argument is true and insufficient. It made whitespace **semantic**
in one place in the language, so the same two characters meant two things
depending on a space, a formatter normalising spacing could change what a
program meant, and a reader could not tell the two apart without counting
columns. Giving ascription its own token costs one character and removes the
category.

41 further type markers in that document and 3 in doc 03 are migrated.

**Historical plan docs keep their contemporaneous spelling.** They record what
was true when they were written, and this section is where the change is
recorded instead. 115 occurrences across 17 doc files: 44 of them were the
language's specification and are migrated; the rest are history.

### 10.7 The migration found a parser hang, by changing arithmetic nobody was watching

The most valuable thing this row produced is not in its criteria.

`crates/voltaic-fuzz`'s sweep began failing with `signal: 9, SIGKILL` and no
diagnostic. That is AGENTS.md's environment signature, so the machine was
checked first — 26 GB available, both tmpfs mounts under a third full, no stray
blaze workers — and it was not the environment. Running `no_target_panics`
alone, single-threaded, with a probe printing each input, it died on
`crates/voltaic-cli/tests/ui/arity_mismatch.sdx mutation 5`: **36 bytes**.

Reproduced as a file, under a 2 GB and 10 second cap so it reports rather than
taking the box:

```text
add : i32 \r> i32
add a =
  a
```

A `-` flipped to a carriage return. The minimal form has nothing to do with
`\r` at all — `add : i32 !> i32` hangs identically, and 28 bytes is enough:

```text
main : ()
main =
  n : i32 !> i32 = 1
```

**The defect.** `capture_type` tracks bracket depth and decrements on `Gt`
without checking that anything opened, so a `>` with no `<` takes `depth` to
**-1** at depth zero. Every termination condition in that loop — the comma,
the closing paren, `Newline`, **and `Eof`** — is guarded by `depth == 0`, so
none of them is ever tested again; `p.advance()` at `Eof` does not advance; and
the captured text grows until the kernel kills the process.

A **hang**, not a panic, which is the worse failure of the two the sweep was
built to catch: plan 40's language server runs this parser on every keystroke,
so this is a frozen editor in front of somebody who has not finished typing.

**Authorship: it is older than this row, and the row is why it was found.**
A typed binding took a single `:` before plan 52 — the migration left that path
alone — and `n : i32 !> i32 = 1` hangs through it, so `capture_type` owned the
bug already. What this row changed is the *fuzz corpus*: every seed got one byte
shorter, `mutate` derives its position from `index % seed.len()`, and so every
mutation of every seed became different bytes. The sweep had been testing a
different 276 inputs all along.

That is worth stating plainly, because it is an argument about the value of the
fuzz corpus rather than about this row: a mutation sweep whose seeds never
change explores one fixed sample of the input space forever, and looks
exhaustive while doing it.

**The fix is two changes, and only one of them is the cause.** A depth-zero
`Gt`, `Shr` or `RBracket` now *ends* the type, exactly as `RParen` and `RBrace`
already did — at depth zero there is nothing for it to close, so it belongs to
whatever encloses the type and the caller's `expect` is the right thing to
complain. That removes the cause and makes a negative depth unreachable. And
`Eof` now breaks **at any depth**, which is a termination proof rather than a
convenience: without it, any future arithmetic slip in that loop is a hang
again rather than an error.

The sweep's own runtime is the clearest measurement of what was being spent on
it: **530 seconds and then a SIGKILL**, against **0.37 seconds** now. One input
out of 276 was consuming the entire run, and the test it took down —
`no_target_panics` — never reached a verdict on any of the other 275.

Pinned by `a_stray_closer_in_a_type_terminates` in
`crates/voltaic-parser/tests/recovery.rs`, which asserts nothing about the parse
— it tests that the line after the call is reached, so a regression times the
suite out instead of producing a diff.

### 10.5 Also verified

- **A signature line is never misread as a typed binding.**
  `reaches_a_binding_eq` returns false at the first depth-zero `Newline`, and a
  signature carries no depth-zero `=` before its break — an
  `impl Trait<Assoc = T>`-style `=` sits inside `<>`, which increments depth.
  It is moot in practice too: signature lines are only read by `Parser::item`,
  never at statement level where `looks_like_a_binding` runs. Both directions
  are now asserted.
- The typed-binding path already used `TokenKind::Colon`; nothing to change.
- `split_format_spec.rs` is the hazard §4 records, and it is **not yet
  triggered** — no corpus file writes `#{total : u64}`. It will fire the first
  time one does, and `tests/golden/strings.xrs` is the witness to add.
