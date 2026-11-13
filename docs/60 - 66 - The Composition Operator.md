# Plan 66 — The Composition Operator

Plan ID: `composition-operator`
Depends on: 56 (`currying`) — **corrected 2026-11-10**; this row amends plan 58's
documents rather than consuming its code, and only leaf 4's `xs |> f >>> g` reading
waits for `|>` to become reverse application
Milestone: 9 — *surface coherence*
Status: `done` — all four leaves shipped 2026-11-10

---

## 1. What this plan is

> **Authorship.** The spelling is Tom's decision. This document was written in a
> concurrent session and reviewed in another before registering; two corrections
> were made then — the dependency is 56 rather than 58, and criterion 1 builds
> the prefix-ordering test rather than running one that exists. Both are marked
> in place.


Composition is spelled `>>>` forward and `<<<` backward. Plan 58 chose `>|` and `|<`;
this row replaces that choice before it reaches a line of code.

```
normalise = .trim >>> .to_lowercase
xs |> .map (.trim >>> .to_lowercase)
normalise = .to_lowercase <<< .trim      -- Haskell's direction
```

It is a small row, and it earns being a row rather than a comment on plan 58 for two
reasons. The spelling has an argument plan 58's does not, and **measuring it turned up
two lexer facts plan 58's own orientation missed** — both of which apply to `>|` exactly
as much as to `>>>`, so they are findings against that row rather than costs of this one.

### 1.1 This is not plan 58's fallback firing

Plan 58 §9 decision 3 already names this spelling:

> **If `>|` collides, the fallback is `>>>`/`<<<`.** Recorded now so leaf 1's collision
> check has an answer rather than becoming a design discussion at implementation time.

**The collision check passed.** Plan 58 §11.1 measured it: neither `>|` nor `|<` is in the
operator table, and maximal munch cannot steal a valid program because *Voltaic spells a
closure `\n -> …`, not `|n|`* — so unlike a Rust-shaped surface there is no `|<` sequence
for a greedy token to break. That finding stands and this row does not disturb it.

What is superseded is decision **2**, which chose `>|`/`|<` over `>>`/`<<`, and the ground
is legibility rather than lexing. A reader must not take this row as evidence that the
hazard decision 3 guarded against ever materialised. It did not.

### 1.2 The bar's position carries nothing

`|>` is bar-then-head, and both halves mean something: the bar is the pipe, the head is
the direction. `>|` is head-then-bar. Both heads point right, so both glyphs read
rightward — the only difference between them is *which side the bar sits on*, and that
carries no meaning at all. A reader who goes looking for one finds nothing.

`|<`'s head points left, so it does read backward. The pair is therefore internally
consistent and its *forward* half is the arbitrary one, which is the opposite of how a
family should fail. Plan 58 §6 concedes the point in passing — *"`>|` is less legible than
`>>`"* — and then keeps the spelling anyway.

### 1.3 `>>>` and `<<<` are not invented

They are Haskell's own. `Control.Category` exports `>>>` as forward composition and `<<<`
as backward, and `Control.Arrow` re-exports both; a reader arriving from Haskell needs no
onboarding, and one arriving from F# reads `>>>` as a heavier `>>`, which is exactly what
it is.

That is a stronger position than any other spelling choice in this milestone can claim.
`:=`, `is`, `.name` and `\->` are each argued from Voltaic's own constraints, because Haskell
has nothing to lend for them. This one is borrowed intact, from the language the rest of
the surface is borrowed from.

And it keeps the property that made plan 58 §1.3 reject `>>`/`<<` in the first place:
**the shifts are untouched.** `examples/expressions.sdx`'s `print (1 << 1 + 1)` keeps
asserting shift precedence, and no program doing bit work migrates for the benefit of
programs doing none.

### 1.4 The section spelling, which is where a glyph is most exposed

Doc 18 §4 gives every infix operator a section for free, so a composition operator arrives
in two positions beyond the two plan 58 checked:

```
(>>> g)                  -- a right section: \f -> f >>> g
(f >>>)                  -- a left section:  \g -> f >>> g
xs |> .map (>>> .trim)
```

Set beside `(>| g)` and `(g >|)`, the difference is the whole of §1.2 at its most
compressed: a sectioned `>|` puts a bar directly against a parenthesis, where the reader
is already tracking one delimiter, and asks them to notice which side it fell on.

### 1.5 Two lexer facts, and they are plan 58's

Neither is about collision, and neither is specific to this spelling.

**A leading composition operator does not continue a line.**
`Lexer::next_line_continues` (`crates/voltaic-lexer/src/lexer.rs:410`) is a raw text test
for the literal `"|>"`, and its doc comment says why only `|>` qualifies — a leading `.`
is ambiguous against doc 03 §7.5's `with` block. Nothing else is admitted. But the corpus
writes a multi-line pipeline with the operator **leading** the continuation line:
`examples/application.sdx` has three such lines and there is no trailing `|>` anywhere in
`examples/`. So a composition chain written in the house style does not lex, whatever the
operator is spelled, until that function learns a second string. Plan 58 §4 leaf 1 does not
mention it and plan 58 §11 did not find it.

**A type can end in `>>>`, and `capture_type` already holds the pattern that handles it.**
This is the one risk the spelling genuinely adds, and reading the code retires it rather
than deferring it. `crates/voltaic-parser/src/parse/parser/capture_type.rs` already treats a
multi-angle closer as a special case, twice:

```rust
TokenKind::Gt | TokenKind::Shr | TokenKind::RBracket => true,   // :114, a depth-zero terminator
TokenKind::Shr => depth -= 2,                                    // :135, "`>>` closes two"
```

So `Vec<Vec<Vec<i32>>>` needs one arm in each set, by the pattern beside it. The **opening**
side needs nothing: an identifier always separates the angles of a nested generic, so that
type opens as three separate `Lt` tokens and closes as one token.

Worth knowing before leaf 1 is written: the function this sits in is the one plan 52 found
hanging. A stray `>` took `depth` to `-1`, which disabled every termination condition
including `Eof`, and 28 bytes grew until the OOM killer stopped it. The `:114` arm is that
fix. A third angle is being added to the exact code path with that history.

---

## 2. Verified preconditions

Measured 2026-11-10 against the working tree.

| Claim | Verdict |
|---|---|
| `>\|` and `\|<` appear in **22 lines across 5 documents and nowhere under `crates/`** | true — docs 18, `45 - 51`, `48 - 54`, `50 - 56`, `52 - 58`; 18 of the 22 lines are plan 58's own |
| …and the sixth file a naive grep reports is a **false positive** | true — `docs/56 - 62`'s two hits are `\|t: Arc<Thresholds>\|`, a Rust closure parameter inside a quoted measurement. Plan 46 §1.3's lesson, hit again: eleven of sixteen apparent findings were once the probe's own bug |
| `OPERATOR_TABLE` is longest-match-first, checked mechanically | true — `crates/voltaic-lexer/src/lexer/operator_table.rs`, and `scan_operator::tests::longer_operators_are_tried_before_their_prefixes` checks every prefix pair rather than by eye |
| its three-character block holds `...`, `..=`, `??=` and nothing else | true — `>>>` and `<<<` join that block, above `>>` and `<<` |
| `<<` → `Shl` and `>>` → `Shr` are two-character entries | true |
| `TokenKind::Pipe` is `\|>`; bare `\|` is `TokenKind::Or` | true — plan 58 §11.1's naming trap, re-confirmed |
| `BinaryOp::Pipeline` is power 1, the lowest rung, and the rungs are **odd** | true — `crates/voltaic-ast/src/expr/binary_op.rs:122`; the comment says the odd numbering is load-bearing, because left-associativity is `power() + 1` |
| `capture_type` decrements by **two** on `Shr`, and treats it as a depth-zero terminator | true — `capture_type.rs:135` and `:114` |
| `Shl` is in **neither** of `capture_type`'s depth arms | true — pre-existing, §6 |
| a nested generic closer is already exercised by the corpus | true — `examples/fallible.sdx` has `Option<Option<i32>>` and `Option<&Option<String>>` |
| a **triple** closer is in no corpus file | true — the three `>>` lines under `examples/` are all double, so §3 case 3 is a genuinely new fixture |
| `next_line_continues` tests for the literal `"\|>"` | true — `lexer.rs:410` |
| `continues_line` holds `Shl` and **not** `Shr` | true — `crates/voltaic-lexer/src/token/kind.rs:206`, and `:190` records why `>` and `>>` are excluded: they close a generic argument list far more often than they dangle off a comparison |
| `~`, `&:`, `.(` and the `.+` family are lexed and consumed by nothing | true — `Tilde`, `AmpColon`, `DotParen`, `DotPlus` reach only `operator_table.rs`, `kind.rs` and `scan_operator.rs`. `@@` is alive. §9 decision 7 |

---

## 3. Concrete proof this plan targets

1. `normalise = .trim >>> .to_lowercase` transpiles, compiles and **runs**, with asserted
   stdout — the same witness plan 58 §3 asks for, in the new spelling.
2. A chain with `>>>` **leading** its continuation line parses. This is §1.5's first
   finding, and it fails today for `|>`'s successor whatever it is called.
3. `wide : Vec<Vec<Vec<i32>>> -> usize` captures as a type and the function compiles. No
   corpus file has a triple closer, so this fixture is the row's new coverage.
4. `print (1 << 1 + 1)` still prints `4` — plan 58 §1.3's protected assertion, unchanged
   and asserted again here because this row is the one that could break it.
5. `(>>> g)` and `(f >>>)` each produce a function, asserted by running rather than by
   compiling: a section with its operands swapped compiles and prints differently.
6. `<<<` and `>>>` with swapped operands are the same function — plan 58 §3 case 4's
   equivalence, which is what makes keeping both directions honest rather than decorative.
7. No document mentions `>|` or `|<` except as a superseded spelling with a pointer here.

---

## 4. Leaves

### Leaf 1 — `leaf-triple-angle-lexing`

`>>>` and `<<<` in the operator table's three-character block; two `TokenKind` variants;
and the three places a multi-angle token is already special-cased:

| Site | Change |
|---|---|
| `capture_type.rs:114` | the new forward token joins the depth-zero terminator set |
| `capture_type.rs:135` | `depth -= 3` beside the existing `Shr => depth -= 2` |
| `kind.rs:206` | `<<<` joins `continues_line`; the forward token stays **out**, for `:190`'s reason |

Plus §3 case 3's triple-closer fixture, which has to land before anything depends on the
tokens. The opening side needs nothing — §1.5.

### Leaf 2 — `leaf-leading-continuation`

`next_line_continues` grows from a literal `"|>"` to a small ordered set — `"|>"`, `">>>"`,
`"<<<"` — longest first, for the same reason the operator table is ordered that way.

This is plan 58's defect as much as this row's, and taking it here means plan 58 ships with
a multi-line composition chain working rather than finding out from a corpus file. The test
is §3 case 2 and it belongs in `crates/voltaic-lexer/tests/continuation.rs`, beside the cases
that already assert what does and does not continue.

### Leaf 3 — `leaf-plan-58-amendment`

Plan 58 restated in the new spelling: §1.3, §3, §4, §5, §6 and §9 decision 2, with decision
2 marked superseded by this row and **decision 3's discharge left standing** — §1.1 is the
distinction that must survive the edit.

Then the four other documents, each a single mention:

| Document | Where |
|---|---|
| `docs/18 - Haskell.md` | §4, *"Plan 58 chooses `>\|` / `\|<`."* |
| `docs/45 - 51 - Construct Coverage.md` | §1.4's list of constructs the milestone adds |
| `docs/48 - 54 - Assignment.md` | §1.3's list of tokens the milestone is already adding |
| `docs/50 - 56 - Currying.md` | §6's closing pointer |

### Leaf 4 — `leaf-precedence-and-direction`

`BinaryOp::Compose` and `BinaryOp::ComposeBack` at power **25**, above `Mul`, sharing a
rung, left-associative.

Power 25 is both the faithful choice and the free one. Haskell's `.` sits at its tightest
operator level and `>>>` is reached the same way; and 25 is the next odd rung above the
table's current top, so nothing is renumbered — which matters, because `binary_op.rs`'s own
comment says the odd numbering is what keeps `power() + 1` collision-free, and the
formatter reads the same table to decide whether a parenthesis is redundant.

Composition binding tighter than `|>` is what makes `xs |> f >>> g` mean `(f >>> g) xs`,
which is the reading the corpus will want.

---

## 5. Acceptance criteria

1. `>>>` and `<<<` lex as single tokens, with the prefix-ordering test green — it checks
   every pair mechanically, so a table entry in the wrong block fails rather than
   truncating silently.

   This criterion was briefly amended to say the check had to be *built*, on the finding that
   `operator_table.rs` states *"longest match first"* in a doc comment with nothing enforcing
   it. **That amendment was wrong and is withdrawn.** The test exists:
   `longer_operators_are_tried_before_their_prefixes`, an inline `#[test]` at
   `crates/voltaic-lexer/src/lexer/scan_operator.rs:35`, walking every pair of entries and
   asserting a longer token precedes any prefix of it.

   It is **inline rather than under `tests/`** because `OPERATOR_TABLE` is private, which is
   what AGENTS.md's rule requires — *"inline tests are for private constructs only"* — and
   that is precisely why a search of `operator_table.rs` and `crates/voltaic-lexer/tests/`
   does not find it. Recorded here so the next reader looks in the right place.
2. A running example witnesses forward composition with asserted stdout.
3. A multi-line chain with the operator leading the continuation line parses, asserted in
   `voltaic-lexer`'s continuation tests.
4. `wide : Vec<Vec<Vec<i32>>> -> usize` compiles, in a corpus file.
5. `print (1 << 1 + 1)` prints `4`, and `examples/expressions.sdx` is byte-identical in its
   golden.
6. Both sections — `(>>> g)` and `(f >>>)` — produce a function, witnessed by stdout.
7. `f >>> g` and `g <<< f` are the same function, witnessed by stdout.
8. `grep` for `>|` and `|<` under `docs/` returns only superseded-spelling notes, and
   nothing under `crates/`, `examples/*.sdx` or `tests/`. Exclude
   `examples/cargo-project/target/` when checking: compiled dependency artifacts match
   both spellings as bytes, which is the probe bug §2 already caught once.
9. `construct_coverage` green with both operators indexed — plan 58 criterion 9, inherited.
10. `voltaic fmt` round-trips every example and `formatting_is_idempotent` holds, because a
    new precedence rung is a formatter input as much as a parser one.

---

## 6. Out of scope, and what it discloses

**`Shl` is in neither of `capture_type`'s depth arms, and that is a pre-existing defect
this row does not fix.** `Option<<T as Trait>::Assoc>` lexes its two adjacent angles as one
`Shl`; `depth` never rises; the depth-zero `as` terminator — which exists for doc 03
§12.5's `from io::Error as Io` — then ends the type at `Option<<T`. A qualified path in a
type is broken today, with or without this row.

It is named rather than quietly repaired because the fix looks like one arm and is not: it
needs a fixture, and it needs a decision about what a qualified path means inside a type
captured as verbatim text. Plan 46 §1.3 is the standing reason not to take it here — a
change with no fixture is how eleven false findings happened once already. A row that owns
qualified paths owns this.

**Keeping both directions is a judgement, and the backward one has no measured customer.**
`$` already reads right to left, and `f <<< g` is `g >>> f` with the operands swapped, so
the only thing `<<<` buys is a Haskell reader's muscle memory. That is worth two table
entries and it is not worth defending if a later row counts the corpus and finds it unused.
Recorded so that row can drop it on evidence rather than on taste.

**No change to `|>`, `$`, or the shifts.** This row changes how composition is spelled and
nothing about what it means; plan 58 §1.4's argument for keeping both `$` and `|>` is
untouched, and so is `examples/expressions.sdx`.

**No cross-module story.** Composition inherits plan 56 §1.4's boundary exactly: arity
comes from a signature visible in the file being compiled, so `f >>> g` across a module
edge gets plan 58 §6's named diagnostic and not a pass-through to rustc.

**No retirement of the dead tokens.** §2's last row found four spellings the lexer scans
and nothing consumes. Whether they should go the way plan 29 took `PercentW` and
`HashBrace` is a real question and not this one's — §9 decision 7.

---

## 7. Gate

Every plan clears, before it is `done`:

```bash
moon run :check          # per-crate `cargo check` — rustc's verdict
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus the three blessable corpora, and `cargo test -p voltaic-fuzz` — a new multi-character
token in `capture_type`'s depth arithmetic is precisely the shape of change that produced
plan 52's hang, and the fuzzer is what found that one.

And the four readers plan 52 §2 enumerated, because this is a surface row: `examples/`,
the Rust doc comments that quote Voltaic source, `voltaic-fmt`'s emit sites, and the
tree-sitter grammar with its five editor clients. Two new operators are a grammar change,
and the grammar is checked against the same corpus.

---

## 8. Emerald reuse

None. Emerald had no composition operator and no pipeline operator — plan 58 §8 records
that its reading-direction argument therefore has no prior evidence and is a design
judgement stated as one. The same is true here, minus one thing: the *spelling* is not a
judgement, because `Control.Category` already made it.

The transferable habit is doc 04's own rule 2 — **correct a stale label in place, with the
date and the evidence that corrected it.** This row is that rule applied to a decision that
was recorded, argued, conceded in its own §6, and never built.

---

## 9. Decision log

1. **`>>>` and `<<<`, superseding plan 58 §9 decision 2.** On legibility, not collision.
   §1.1 is load-bearing: decision 3's fallback never triggered, and a later reader must not
   infer a lexing hazard that §11.1 measured and cleared.
2. **Both directions.** §6 states the cost and names what would overturn it.
3. **Power 25, above `Mul`, sharing a rung.** Haskell's tightest operator level, and the
   one rung that needs no renumbering of a table whose comment says the numbering is
   load-bearing. Composition binds tighter than `|>`, which is what `xs |> f >>> g` needs.
4. **Left-associative, and a mixed chain is legal.** `f >>> g <<< h` parses left to right
   and type-checks; refusing it would be a check, and AGENTS.md's rule is that Voltaic grows
   none. A lint is the right home if it ever earns one — compare plan 54's `RDX1003`, which
   is in the driver and not the parser for the same reason.
5. **Its own row, not an amendment inside plan 58.** The change is five documents and four
   code sites, and §1.5's two findings are defects in plan 58 that this row discharges.
   Folding that into plan 58's §11 would bury a decision inside an orientation, which is
   the failure `plan_orientation.rs` exists to catch from the other direction.
6. **Before plan 58, not after.** `>|` is in no code, so today this row is five documents,
   two lexer sites and a fixture. After plan 58 ships it is a token migration with a corpus
   and a tree-sitter grammar behind it, and plan 52 priced that shape of migration at 226
   files. The cheapest moment to change a spelling is before anything spells it.
7. **`~` is not spent.** `~` is lexed and consumed by nothing, so `~>` / `<~` were
   available at two characters instead of three. Declined: a dead token is cheap to spend
   and expensive to read. `~>` has no composition precedent in any language Voltaic borrows
   from, and `~` reads as *approximately* to most of them. The case for spending it is
   weaker than the case for retiring it, and neither is this row's.
8. **`>>` and `<<` are still not taken.** Plan 58 §1.3's reasoning survives this row
   untouched, and criterion 5 asserts it rather than assuming it.

---

## 10. Result

Executed 2026-11-10. **Nine of the ten criteria are met.** Criterion 8 is met everywhere this
row is allowed to edit and is **not** met in one line of `docs/04 - Voltaic Plan of Plans.md`,
which is outside the brief — §10.7. Criterion 1 is met by a test that already existed, which is
the row's first finding and an error in the criterion itself.

### 10.1 Criterion 1's own correction is wrong: the prefix-ordering test exists

The criterion was corrected before registration to say *"nothing enforces it … build the
check"*. **§2's table row was right and the correction is the error.**
`crates/voltaic-lexer/src/lexer/scan_operator.rs` has held
`longer_operators_are_tried_before_their_prefixes` since plan 02 leaf 7, and it checks exactly
the property the correction asked for — every ordered pair, `text.starts_with(other)` implies
the longer one is listed first.

So the check was not built; it was **verified by breaking it**, which is the half the criterion
genuinely wanted. `(">>>", Compose)` was moved from the three-character block to immediately
after `(">>", Shr)` and the lexer's own tests were run:

```
`>>>` must be listed before its prefix `>>`        longer_operators_are_tried_before_their_prefixes
  left: (Some(Shr), 2)                             scans_the_three_character_operators_whole
 right: (Some(Compose), 3)
```

Two tests, and the second prints the silent truncation as a value. The table is restored and
one test is added — `the_shifts_keep_their_own_spellings`, which asserts what the ordering
*buys* rather than that it holds, because a reader of plan 58 §1.3 will want `>>` and `<<`
checked from the other side.

### 10.2 `capture_type` needed two sites beyond §4's table, and both are re-spellings

§4's table names three sites and all three were right. Two more were not named:

| Site | Why |
|---|---|
| `parse/parser/token_text.rs` | a captured type is **re-emitted** from these spellings, and with no `Compose` arm the type's verbatim text ends in the debug marker `Compose` instead of `>>>` |
| `parse/parser/can_end_a_type.rs` | the `TypeStop::Atom` boundary between two juxtaposed payload types, which already lists `Shr` |

The first is load-bearing — criterion 4 fails without it. The second is not reached by any
fixture today, and it is taken anyway because `>>>` is a closer **this row adds**: a new closer
absent from the set that answers *can a type end here* is the shape of the `Shl` gap §6
declines, and §6's reason for declining that one is that it is pre-existing. This one would not
have been.

### 10.3 Sections are not free, and the measurement is `E0282`

§1.4 is right that doc 18 §4 gives every infix operator a section, and wrong that it is free.
Two separate costs, both measured:

- `emit_section`'s non-comparison arms write `op.rust_text().unwrap_or("/* unsupported */")`,
  and composition has no `rust_text` by construction. Doc 18 §4's *"for free"* is free for an
  operator with a Rust **spelling**; this is the first one without.
- **A composition section cannot be a bare closure at all.** `(>>> g)` is `\f -> f >>> g`, so
  the closure binds a *function* and then calls it — and a closure parameter that is called is
  `E0282` with no annotation. Measured for both candidate shapes:

  | Shape | Verdict |
  |---|---|
  | `\|__rdx_1\| move \|__rdx_2\| g(__rdx_1(__rdx_2))` | `E0282: type annotations needed` |
  | `\|__rdx_1, __rdx_2\| g(__rdx_1(__rdx_2))` — plan 56's flat shape | `E0282`, the same |
  | `\|__rdx_1: fn(_) -> _\| \|__rdx_2\| …` | `E0373`, *"closure may outlive the current function"* |
  | `\|__rdx_1: fn(_) -> _\| move \|__rdx_2\| …` | compiles |

  So the lowering is `fn(_) -> _` plus `move`: the narrowest annotation that compiles, with
  both halves still inferred. **The cost is that the operand must be a function item**, not a
  capturing closure — a non-capturing one coerces. That is disclosed here rather than
  diagnosed, because it is rustc's answer and AGENTS.md leaves it to rustc.

The plain binary lowering has no such problem: `emit/compose.rs` folds the chain into nested
applications over one parameter, so `.trim >>> .to_lowercase` is `|__rdx_1|
__rdx_1.trim().to_lowercase()` — a method chain, not two closures called in place.

### 10.4 `voltaic-fmt` needed two methods and nothing else, and then collapsed a corpus line

Criterion 10 cost `BinaryOp::voltaic_text` and `BinaryOp::power`, and **`voltaic_text` was the one
that would have crashed**: its `other =>` arm calls `rust_text().expect(…)`, so a formatter
printing `>>>` would have panicked rather than printed wrongly. Power 25 needed no renumbering
and `power() + 1 = 26` collides with nothing, as §4 predicted. `power_of`, `can_drop_paren` and
`operand` all read the operator's own table and needed no change. `cargo test -p voltaic-fmt`
green.

What the criterion did not predict: **`voltaic fmt` collapses a multi-line composition chain onto
one line.** `pipeline_text` is the only operator with a layout of its own, so a chain written
with the operator leading each continuation line — §3 case 2's shape, which now *lexes* —
is not canonical, and `every_example_is_already_canonical` rejected the corpus witness. The
example therefore writes the chain on one line and says why, and criterion 3's assertion lives
where the criterion puts it, in `voltaic-lexer/tests/continuation.rs`. A row that wants the
multi-line form preserved owns giving composition a layout.

### 10.5 A bare `[">>>"]` witness would have been hollow, and criterion 4 is what fills it

Criterion 9's first draft here was `Witness::Toks(&[">>>"])`, and it was satisfied — by
**criterion 4's own fixture, in the same file**. `wide : Vec<Vec<Vec<i32>>> -> usize` closes as
one `Compose` token whose span text is `>>>`, so the corpus could have contained no composition
at all and the gate would have been green. The fifth hollow witness of this milestone, and the
first where the decoy is a *type*.

The witness is one run, `>>> . «id» <<< .`, which holds both operators and reaches into an
operand: a `.name` cannot close a type at any depth. Verified by breaking it — `examples/`'s
`.trim >>> .len` was temporarily rewritten as `.len <<< .trim`, and the gate failed naming
`Operators and precedence [docs/GRAMMAR.md:100 (1.4 Expressions)]`. No `tests/ui/` or
`tests/run/` fixture contains either operator; outside `examples/cargo-project/target/` the
only two files that do are `examples/compose.sdx` and `tests/golden/compose.xrs`.

`construct_coverage.rs` was **exactly at `line_cap`'s 1,500 lines** at `e266639`, so the
addition had to be net-zero. It is: `+` and `*` were matched apart and are now matched as the
one run `+ «int» *`, which is a better witness for a row named *precedence* — two operators
found in different files witness no precedence at all — and the rationale moved into the
`spelling` field, which is the text the failure message prints. A later row adding a witness to
that file will have to split it, which is what the cap's own message says to do.

### 10.6 The tree-sitter gate does not catch a missing operator

`grammar.js` gained `">>>"` and `"<<<"` and the parser was regenerated, so an editor sees one
token where it would otherwise see two. But §7's claim that *"the grammar is checked against
the same corpus"* is weaker than it reads: `every_example_parses_with_no_error` and
`the_grammars_own_corpus_passes` were **both green before the grammar knew either operator**,
because the flat `_expression_part` run accepts `>>` followed by `>` as two operators and no
`ERROR` node appears. A missing operator lingers; only a wrong *structure* fails. The
`tree-sitter generate` warning about unnecessary conflicts is pre-existing — reproduced from
`e266639`'s own `grammar.js` with the identical set.

### 10.7 Criterion 8 fails on one line this row may not edit

`grep` for the superseded pair returns nothing under `crates/`, `examples/*.sdx` or `tests/`,
and under `docs/` returns superseded-spelling notes only — **except**
`docs/04 - Voltaic Plan of Plans.md`'s row for plan 58, which still reads *"Composition arrives
as `>|` / `|<`"* as a statement of fact rather than as a superseded spelling. The register is
outside this row's editing brief, so the line is reported rather than fixed. It wants the same
one-clause amendment the other four mentions got.

Two smaller notes from the same grep. `docs/56 - 62`'s two hits remain the false positive §2
identified — a Rust closure parameter inside a quoted measurement — and the `crates/` hit that
did appear was this row's own doc comment, rewritten to name plan 58 §9 decision 2 instead of
spelling the pair.

### 10.8 `>>> g` is not writable as a top-level binding, for plan 59's reason

§3 case 1 spells the witness `normalise = .trim >>> .to_lowercase`. That exact line is not
writable at **top level**, and not because of composition: a signature with one arrow over an
equation with no parameters is the clause lowering's business, which plan 59 owns. As a *local*
binding it is also not writable with method-reference operands — `|__rdx_1|
__rdx_1.trim().to_lowercase()` is `E0282`, because method resolution needs a receiver type and
a closure parameter has none until something constrains it.

Both are rustc's constraints rather than Voltaic's, and both have the same workaround, which is
the one the corpus uses: compose **named functions** where the binding stands alone, and
compose **method references** inside an adaptor, where the item type is known. `examples/compose.sdx`
does one of each, so the case is witnessed in the only two positions it can be.

### 10.9 What the gate said

`moon run :check`, `:lint`, `:format`, the three blessable corpora, `construct_coverage`,
`code_coverage`, `grammar_inventory`, `declined`, `plan_orientation`, `editors`, `tree_sitter`,
`one_construct_per_file`, `directory_structure`, `line_cap`, `voltaic-fmt`, `voltaic-lexer` and
`voltaic-fuzz`: green. No new diagnostic code was needed, so `code_coverage`'s empty
`NOT_YET_REACHABLE` is undisturbed.

**`docs/STATUS.md` and `docs/CONFORMANCE.md` are the two red steps, and they are generated.**
This row adds source lines and one example, so `status_is_current` and `conformance_is_current`
compare a stale document against a moved count — 32 examples to 33, 31 goldens to 32, and the
line totals in `voltaic-ast`, `voltaic-emit`, `voltaic-lexer`, `voltaic-parser` and `voltaic-cli`. They
are blessed by whoever reviews the diff, not here.
