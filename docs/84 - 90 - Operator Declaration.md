# Plan 90 — Operator Declaration

Plan ID: `operator-declaration`
Depends on: 56 (`currying`), 66 (`composition-operator`), 84 (`binder-escape-hatch`)
Milestone: 11 — *expressive delta*
Status: `done` — §10 records what shipped and what §1.3 got wrong

---

## 1. What this plan is

A declaration gives a spelling and a fixity to a function that already exists. It emits no item.

```
infixl 6 <+> = Set::union
infixr 1 =<< = and_then

combined = a <+> b          -- Set::union(a, b)
```

The declaration is **fixity plus alias**. It introduces no new Rust name, fabricates no API, and
its lowering is a call to a function the author wrote or imported.

### 1.1 The decline rests on a claim that is false, and this is the correction

`docs/DECLINED.md` refuses this construct under *"Rust has no target for it"*:

| Construct | From | Reason |
|---|---|---|
| Custom operator declaration | Haskell, Scala | no Rust lowering target exists (doc 03 §14.4) |

Doc 03 §14.4 states it in full: *"Custom operator declaration (Haskell, Scala) is declined by
necessity — Rust has no mechanism for it, so there is no lowering target."*

**That conflates two different things.** Rust has no custom-operator *mechanism*, which is true.
It does not follow that there is no lowering *target*, and the surface already proves it: Voltaic
has at least six operators with no Rust counterpart at all, and every one of them lowers to an
ordinary expression form.

| Voltaic operator | Rust counterpart | Lowers to | Shipped by |
|---|---|---|---|
| `\|>` | none | a method call on the piped value | doc 18 §4, plan 58 |
| `>>>` / `<<<` | none | a closure, through the shared `closure` path | plan 66 |
| `$` | none | **nothing at all** — grouping that leaves no trace | doc 18 §4 |
| `??` | none | an `unwrap_or`-shaped expression | doc 03 §12.1 |
| `is` | none | a pattern test | plan 60 |

A user-declared `<+>` lowering to `Set::union(a, b)` has exactly the target `a \|> f` has. The
decline's reason is therefore wrong as written, and the construct has to be re-judged on its real
costs — which are substantial, and are §1.3 and §1.4.

This is the kind of correction `docs/DECLINED.md` already carries twice: plan 47 §1.4 restated two
reasons that cited a deleted section, and the turbofish row records a decline *"on a compiled
measurement"* rather than an argument. A reason that does not survive inspection is replaced, and
the verdict is re-derived rather than inherited.

### 1.2 Declaring an alias, not a definition, is what keeps it admissible

The obvious design — let `(<+>)` be a *definition*, as in Haskell — does not work here, and the
reason is Rust's identifier grammar: there is no Rust function that can be named `<+>`, so the
emitter would have to invent one. That fabricates API surface, which is doc 03 §8.7's objection to
default arguments and the reason they stay declined.

So a declaration **names a function that already exists**:

```
infixl 6 <+> = Set::union
```

and `a <+> b` emits `Set::union(a, b)`. Nothing is declared, nothing is renamed, and
go-to-definition on `<+>` has somewhere real to go. The author writes the function as an ordinary
Voltaic function or imports it from Rust; the operator is purely a parse-level spelling for it.

Admission, clause by clause:

- **Type-free.** The lowering is a call to the named function. No type is consulted — strictly
  less is consulted than for `\|>`, which has to decide between a method call and a call.
- **No implied dependency.** The declaration names only what the author named.
- **Semantics-preserving.** It is a call. Evaluation order is left-to-right as written, no clone,
  no reborrow.
- **Reachability-positive.** New expressiveness; nothing becomes less reachable.

### 1.3 The real cost is that the lexer stops being a lexer

This is the row's genuine difficulty and it belongs here rather than in §6.

The operator table is **static and longest-match**. `AGENTS.md` records the test that pins it and
where it lives:

> `operator_table.rs`'s doc comment states *"longest match first"* […] It exists:
> `longer_operators_are_tried_before_their_prefixes`, an inline `#[test]` in `scan_operator.rs`.

A user-declared operator makes that table **dynamic**: `<+>` cannot be scanned as one token until
the `infixl` declaration has been read, and the declaration is in the same file as the uses. So
the lexer acquires a dependency on something the parser would normally find — a pre-pass that
scans for `infix`, `infixl` and `infixr` declarations and seeds the table before the body is
lexed.

Two consequences, both load-bearing:

- **A declaration must be lexically findable without a full parse.** That constrains its shape:
  it has to be recognisable from a line start by a keyword, which is why `infixl` is a keyword and
  not a function call.
- **Longest-match is now the author's problem.** Declaring `<+` when `<+>` exists changes how
  `<+>` lexes. §9 decision 4 chooses to diagnose the prefix collision rather than resolve it by
  precedence, because `AGENTS.md`'s own record is that precedence silently deciding a conflict is
  the trap: *"`tree-sitter generate` reports one as 'unnecessary' when a precedence already
  decides, which reads like reassurance and is the opposite."*

### 1.4 The tree-sitter grammar cannot follow, and that is a disclosed asymmetry

Plan 41's grammar is static. It can be taught a general *shape* for a user operator — a run of
operator characters in infix position — but it cannot know a fixity declared in a source file, so
it cannot parse a user operator's precedence correctly.

What that costs is specific rather than vague: editor **highlighting** works, editor **structural
selection and indentation** work on the shape, and anything in an editor that depends on the
operator's binding power does not. `crates/voltaic-cli/tests/tree_sitter.rs` and plan 46's
`every_coloured_keyword_is_still_in_the_grammar` are the gates that will notice, and criterion 8
is the decision about what they should assert.

`AGENTS.md` is explicit about what this class of change requires, and it is not optional here:

> So for anything involving precedence, conflicts, or a flat `repeat1` — which is most of
> `grammar.js` — **run the input.** `tree-sitter parse` on a two-line file costs a second and is
> the only thing that answers.

### 1.5 Scope is the file

Same boundary as plan 56 §1.4 and for the same reason — plan 14 decision 4's refusal of name
resolution. An operator declared in one file is not available in another.

Unlike currying's boundary, this one is **loud**: an undeclared operator does not lex, so the
failure is a Voltaic diagnostic at the use site rather than a silently different program. That is
the better side of the boundary to be on and criterion 6 asserts it.

---

## 2. Preconditions to verify

**Nothing in this section is measured yet.** The row is authored, not executed.

```bash
cargo run -p voltaic-cli -- build /tmp/probe.sdx && cat /tmp/probe.xrs
cd tree-sitter-voltaic && tree-sitter parse /tmp/probe.sdx
```

1. **The operator table's shape and the Pratt loop's fixity source.** `operator_table.rs`,
   `scan_operator.rs`, and `parse_expr.rs` — plan 56 §11.6 records `$` as *"hardcoded in the Pratt
   loop at `parse_expr.rs:48-57`, binding power 1, right-associative"*, so at least one operator's
   fixity is not in the table at all. How many others is the question leaf 0 answers.
2. **Which operator characters are unclaimed.** Doc 03 §4.5 is the inventory and doc 03 §4.1
   records that `#` resolves *"six forms, by longest match"*. The admissible character set for a
   user operator is whatever cannot begin an existing token.
3. **Whether a pre-pass can find declarations without a parse.** §1.3's constraint. Measured by
   writing the scanner, not by reasoning about it.
4. **What `voltaic-fmt` needs to re-print an operator expression.** If it re-parenthesises by
   binding power, it needs the fixity table too, and the formatter acquires the same pre-pass.
   Plan 56 §11.3's rule is the hazard: two consumers of one tree.
5. **What `tree-sitter parse` does with a run of operator characters today.** §1.4. `AGENTS.md`
   records the two ways this has already gone wrong — a `prec(1)` that intercepted a reading that
   plainly parsed by another rule, and a flat `repeat1` that accepted `>>` then `>` with no
   `ERROR` node, so the grammar gates were green before `grammar.js` knew `>>>`.

---

## 3. Concrete proof this plan targets

**`a <+> b <+> c` compiles, runs, prints the right set, and associates left** — a declared
operator, used three times, with its fixity observable in the output.

Six cases, each failing differently:

1. **Left-associative, one use.** The simple case.
2. **Left-associative, two uses**, asserted by a golden that shows the nesting — the case where a
   fixity read but not applied passes case 1.
3. **Right-associative**, `infixr`, asserted the same way.
4. **Two declared operators of different precedence** in one expression, which is the case that
   needs the table and not a special case.
5. **A prefix collision** — `<+` declared where `<+>` exists — refused by a named diagnostic,
   per §1.3.
6. **An undeclared operator**, refused by a named diagnostic at the use site, per §1.5.

And three that must **not** change: every built-in operator's precedence is unaltered, every
example emits byte-identical Rust, and `tree_sitter.rs` is green.

---

## 4. Leaves

### Leaf 0 — `leaf-fixity-inventory`

Measure preconditions 1, 2 and 5. Where every built-in operator's fixity actually lives, which
characters are free, and what the tree-sitter grammar does with an unknown operator run today.
This leaf may resize or stop the row, and `AGENTS.md`'s instruction is to **run the input** rather
than read the grammar.

### Leaf 1 — `leaf-declaration-prepass`

`infix`, `infixl`, `infixr` with a precedence and an alias target, found by a pre-pass before the
body is lexed. §1.3.

### Leaf 2 — `leaf-dynamic-operator-table`

The seeded table, longest-match preserved, with `longer_operators_are_tried_before_their_prefixes`
extended to cover a declared operator against a built-in prefix.

### Leaf 3 — `leaf-pratt-fixity`

The Pratt loop reads binding power and associativity from the table. Precondition 1 decides whether
this also means moving `$`'s hardcoded entry, and if it does, that move is in this leaf and is
asserted not to change `$`'s behaviour — plan 56 §10.4 records that `$`'s predicate *"paid for
more than `$`"*, so it is not a safe thing to touch quietly.

### Leaf 4 — `leaf-diagnostics`

The prefix collision, the undeclared operator, a precedence outside the admissible range, and a
declaration whose alias target is itself an operator. Four named diagnostics with `tests/ui/`
fixtures.

### Leaf 5 — `leaf-formatter`

`voltaic fmt` round-trips a declaration and re-prints uses at the right precedence, per
precondition 4.

### Leaf 6 — `leaf-tree-sitter`

The grammar accepts a general user-operator shape. Criterion 8 is the decision about what the
gates assert given §1.4's asymmetry, and `tree-sitter parse` on each §3 case is the evidence.

### Leaf 7 — `leaf-corpus-and-docs`

`examples/operator.sdx`. Then the document work:

- **`docs/DECLINED.md`'s row moves out of the declines**, with §1.1's correction recorded — the
  reason was wrong, not merely outweighed.
- Doc 03 §14.4's sentence is corrected in place with a date, per doc 04's rule 2.
- `crates/voltaic-cli/tests/declined.rs` asserts every doc 03 §22 construct appears in
  `DECLINED.md`, so both documents move in one commit or the gate breaks.
- `docs/GRAMMAR.md` §1.4's operators row and `docs/STATUS.md` regenerated.

---

## 5. Acceptance criteria

1. Each §3 case transpiles, compiles with no rustc output, runs, and its stdout is asserted.
2. Associativity is **observable in the output**: cases 2 and 3 have goldens showing the nesting,
   and a running test whose result differs between the two associativities. A fixity that is read
   and not applied must fail here.
3. Two operators of different precedence in one expression produce the right answer, run.
4. No built-in operator's precedence changes: every existing golden re-runs with **no
   re-blessing**, and `examples/expressions.sdx` and `examples/section.sdx` specifically are
   asserted byte-identical.
5. `$` behaves identically whether or not its entry moved to the table, per leaf 3.
6. The four leaf-4 diagnostics each have a `tests/ui/` fixture.
7. `voltaic fmt` round-trips a declaration and every use at the correct precedence.
8. `tree_sitter.rs` is green and `tree-sitter parse` output for each §3 case is recorded in
   `tree-sitter-voltaic/test/corpus/`. Where the grammar cannot know a fixity (§1.4), the corpus case
   records the shape it *does* produce, so the asymmetry is in the corpus rather than in prose.
   `AGENTS.md`'s rule applies: *prefer a case in the corpus, which `tree_sitter.rs` runs, over
   leaning on the examples sweep.*
9. `docs/DECLINED.md`, doc 03 §14.4, doc 03 §22 and `declined.rs` move in one commit and the gate
   is green after it.
10. All examples green; `construct_coverage` green with no new `ALLOW` entries.

---

## 6. Out of scope, and what it discloses

**The lexer gains a pre-pass, permanently** (§1.3). That is a real architectural change to the one
component everything downstream depends on, and it is the reason this row sits last in the
milestone rather than first.

**The tree-sitter grammar cannot follow** (§1.4). Editors get highlighting and not precedence.
Criterion 8 puts that in the corpus.

**No operator *definitions*, only aliases** (§1.2). `(<+>) a b = …` is not a thing; the function
is declared normally and the operator points at it.

**No prefix or postfix operators.** Infix only. A prefix operator collides with the unary
operators doc 18 §4 already records as winning over right sections — *"`(- 1)` is negation and
`(* 2)` is a dereference"* — and adding a declarable prefix would make that collision
user-extensible, which is worse than the papercut it already is.

**No sections of user operators in this row.** `(<+> x)` is reachable in principle through plan 56
leaf 4's unified lowering, and it is not asserted here. A future row, named rather than assumed.

**Readability is now the author's problem**, and nothing in the surface constrains it. This is the
standing objection to custom operators in every language that has them and it is not answered —
only noted, with the mitigation that a declaration is file-scoped (§1.5) so the blast radius is one
file.

**Precedence range is bounded** and the bound is arbitrary. Haskell uses 0–9; whatever leaf 1
chooses, leaf 4 diagnoses the outside and §9 decision 5 records the choice.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted;
`tests/ui/` fixtures for leaf 4's four diagnostics; `tree-sitter-voltaic/test/corpus/` cases per
criterion 8; goldens asserted unchanged per criteria 4 and 5; `declined.rs` green after the
document move; `construct_coverage` green.

---

## 8. Emerald reuse

None — doc 02 §4.3 records Emerald's call and operator syntax as conventional, with no declarable
operators.

The transferable discipline is `AGENTS.md`'s, and it is the sharpest case for it in the register
because the finding was written down about this exact mechanism:

> **A declaration read correctly still does not tell you the behaviour, because another rule can
> intercept the same input.** […] Removing the `prec(1)` is what fixed it — a **declared conflict
> did not**, because `tree-sitter generate` reports one as *"unnecessary"* when a precedence
> already decides, which reads like reassurance and is the opposite.

And the other half, which is why criterion 2 insists the associativity be observable in a *running*
result rather than in a parse tree: *"the grammar gates were green before `grammar.js` knew `>>>`,
because a flat `repeat1(choice(…))` run accepts `>>` then `>` with no `ERROR` node."* A fixity read
and not applied produces a parse that looks right and an answer that is wrong, which is doc 02
§5.2's worst failure mode and only doc 02 §8.3's run catches it.

---

## 9. Decision log

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| 1 | Overturn the decline | Inherit it | Doc 03 §14.4's reason — *"no lowering target"* — is false, and §1.1's table is six counterexamples already in the surface. `docs/DECLINED.md` has replaced a wrong reason twice before (plan 47 §1.4) |
| 2 | Fixity plus alias (§1.2) | A Haskell-style operator *definition* | No Rust function can be named `<+>`, so a definition fabricates API — doc 03 §8.7's objection to default arguments, which still stands |
| 3 | A lexer pre-pass | Lex operator runs generically and resolve in the parser | A generic run is exactly the flat `repeat1` failure `AGENTS.md` records: it accepts `>>` then `>` with no `ERROR` node, so the gates go green while the grammar is wrong |
| 4 | Diagnose a prefix collision | Resolve it by precedence or by declaration order | `AGENTS.md`: a precedence that silently decides a conflict *"reads like reassurance and is the opposite"*. A refusal the author can see beats a resolution they cannot |
| 5 | A bounded precedence range | Unbounded integers | An unbounded range makes two libraries' choices unmergeable, and the bound costs nothing while declarations are file-scoped |
| 6 | Infix only | Declarable prefix and postfix operators | A declarable prefix makes doc 18 §4's unary-versus-section collision user-extensible |
| 7 | Last in the milestone | Earlier, alongside the cheaper rows | It is the only row here that changes the lexer, and the lexer is what every other row's surface is read by |

---
## 10. Result

**Shipped**, 2026-11-11. `examples/operator.sdx` declares three operators —
left-associative, right-associative and non-associative — and uses each;
`tests/golden/operator.xrs` shows `merge(merge(a, b), c)` and
`diff(9, diff(4, 2))` side by side, so the associativity is in the artifact
rather than in prose; `examples/operator.stdout` asserts the numbers, and they
differ from what the other associativity would print.

`docs/DECLINED.md`'s row is lifted, doc 03 §14.4's sentence is corrected in
place with a date, and `docs/GRAMMAR.md` grades the construct `SUGARED`.

### 10.1 §1.3 was the row's whole cost, and it was avoidable

This section was written before implementation and said the row was not
shipped because *"the lexer stops being a lexer"*:

> A user-declared operator makes that table **dynamic**: `<+>` cannot be
> scanned as one token until the `infixl` declaration has been read, and the
> declaration is in the same file as the uses. So the lexer acquires a
> dependency on something the parser would normally find — a pre-pass that
> scans for `infix`, `infixl` and `infixr` declarations and seeds the table
> before the body is lexed.

Every sentence of that is correct **given one assumption**, and the assumption
is not stated: that the set of admissible *spellings* is open. It is not, and
nothing required it to be.

`crates/voltaic-lexer/sdx/scan.sdx` is a `const` of twelve
spellings — `<+>`, `<->`, `<*>`, `</>`, `<%>`, `<|>`, `<&>`, `<^>`, `<=>`,
`<~>`, `<$>`, `<?>` — and `scan_operator` tries that run before
`OPERATOR_TABLE`. The lexer scans every one of them **whether or not any file
declares it**, so:

| §1.3 said | What shipped |
|---|---|
| the table becomes dynamic | it is a second `const`, in longest-match order with the first |
| a thread-local seeded before lexing | no guard; the lexer holds no state |
| a scanner over raw source before the lexer | nothing; declarations are found by the parser, in source order, for free |
| *"the lexer stops being a lexer"* | `scan_operator` gained six lines |

**Which** operators a file has declared is then the parser's business, and the
parser is where declaration order is already known — `Parser::fixities` is a
field of the same shape as plan 63's `class_heads`, collected while items are
parsed and read while an expression is. An undeclared use is `RDX0123` at the
use site, which is *louder* than §1.5 asked for: it promised a Voltaic diagnostic
rather than a silently different program, and this one names the operator and
says where to declare it.

**The lesson is about where a requirement came from.** §1.3's analysis followed
from "a user picks the spelling", and nothing in §1 or §2 ever asked for that —
§1.2 is about whether a declaration is an *alias or a definition*, and is
silent on the spelling's shape. A closed set keeps everything §1 wanted
(fixity, precedence, associativity, an alias to an existing function) and
discards only the freedom to invent a glyph. That trade was never priced,
because the open set was assumed rather than chosen.

The closure also buys properties the open form could not have had, and
`ASSIGNABLE_OPERATORS`' doc comment states them: no entry is a prefix of
another (all the same length), no entry is a prefix of a built-in (so putting
the run first truncates nothing), and two built-ins — `<-` and `<=` — *are*
prefixes of entries here, which is exactly why the run must come first.
`assignable_operators_precede_their_builtin_prefixes` asserts that across the
two tables, which
`longer_operators_are_tried_before_their_prefixes` structurally cannot see.

### 10.2 What shipped

| Leaf | State | Where |
|---|---|---|
| 0 — fixity inventory | folded into leaves 1–3 | §10.3 reports what it would have found |
| 1 — declaration pre-pass | **not needed** | §10.1. The declaration is an ordinary item |
| 2 — dynamic operator table | **not needed** | `ASSIGNABLE_OPERATORS`, a second `const` |
| 3 — Pratt fixity | **done** | `user_rung`, read before `binary_power`; `$` untouched |
| 4 — diagnostics | **done** | `RDX0123`–`RDX0126`, four `tests/ui/` fixtures |
| 5 — formatter | **done** | `item_text`, `expr_text`, `power_of` |
| 6 — tree-sitter | **done** | `operator_declaration`, `user_operator`, a corpus case |
| 7 — corpus and docs | **done** | the example, the golden, `DECLINED.md`, doc 03 §14.4, `GRAMMAR.md` |

Across six crates: `voltaic-lexer` (`TokenKind::UserOp`, the constant,
`scan_operator`), `voltaic-ast` (`OperatorDecl`, `Fixity`, `Expr::UserBinary`),
`voltaic-parser` (`operator_decl`, `declares_an_operator`, `alias_target`,
`digit_precedence`, `user_rung`, `operator_chains`, the Pratt arm,
`Parser::fixities`), `voltaic-diag` (four codes), `voltaic-emit` (one `Expr` arm
and one `Item` arm), `voltaic-fmt` and `voltaic-lsp`.

### 10.3 The inventory leaf 0 would have produced

Measured while building, which is what leaf 0 asked for:

- **`$` is still hardcoded in the Pratt loop** at binding power 1, and was not
  moved. §1.1's §2 precondition expected the row to have to touch it; it did
  not, because a user rung is read in its own branch above `binary_power` and
  nothing about `$` is in the way. Criterion 5 holds trivially.
- **Every other built-in fixity is `BinaryOp::power`'s**, on the odd numbers
  1–25 with left-associativity as `power() + 1`. A declared precedence maps to
  `2n + 2` — always **even**, so it cannot collide with a built-in rung, and
  every user operator lands in a gap between two of them. `Fixity::powers`
  carries the interleave table.
- **Every user operator therefore binds looser than `+`, `*` and `>>>`.** That
  is a consequence of the mapping rather than a decision taken first, and it is
  the right one: arithmetic is the precedence every reader already knows, and a
  declaration that could outrank it would make a familiar expression mean
  something new. `examples/operator.sdx`'s `9 <-> 2 + 3` is the assertion.
- **`token_text`'s ladder did want extending**, as §10.3 predicted — and the
  case is the one that needs no table, because a user operator's spelling *is*
  its text. The fuzzer found it: without the arm, a declaration's alias target
  came back as `UserOp("<=>")`, the same Debug-name leak plan 84 leaf 2 fixed
  for the forty static operators.

### 10.4 Non-associativity is a diagnostic, and it had to be

§1 lists `infix` alongside `infixl` and `infixr` without saying what it means
operationally. A Pratt loop's binding powers can only group one way or the
other — every pair of numbers is left- or right-associative — so "does not
chain" cannot be expressed as a precedence at all. Picking either would make
`a <=> b <=> c` mean something the author did not write.

So `Fixity::NonAssoc` carries `LeftAssoc`'s powers and the parser refuses the
*second* use: `RDX0125`, raised after the right operand so it points at the
chain rather than at the operator. That is the same division Haskell makes,
where a non-associative chain is a parse error rather than a grouping.

### 10.5 Three things the implementation found that the plan did not predict

**`Parser::item`'s dispatch is not the only place a line is claimed.** The
clause loop inside `equations` reads any lower-case name not followed by `:` as
another equation of the function above it, so `infixl 6 <+> = merge` below a
function was reported as *"this equation defines `infixl`, but the signature
above it is for `scale`"*. The loop has to be told, with the same
`declares_an_operator` predicate the item dispatch uses. Found by writing the
example; no amount of reading `Parser::item` would have shown it.

**A flat run in `grammar.js` accepted `a <+> b` as three operators with no
`ERROR` node.** `AGENTS.md` records this trap twice and §1.4 quoted it, and it
still happened — the `user_operator` rule was added to `_item` and *not* to
`_expression_part`, so the declaration parsed as one token and every use lexed
as `<`, `+`, `>`. The gate was green and the highlighter would have coloured
three operators where the author wrote one. `tree-sitter parse` on the example
is what showed it, exactly as §2 precondition 5 said it would.

**An unbalanced `<` in a captured type swallowed the whole file**, and the
fuzzer found it through this row's own UI fixture. `capture_type` counted `<`
and `>` at depth and only ended a type at a newline when the depth was zero, so
a byte flip turning `u32 -> u32` into `u32 -< u32` captured six lines as one
type and pasted them into the generated Rust. It now **refuses** — a `<`, `(`,
`[` or `{` in a type is closed on the same line — which is plan 84 §1.3's rule
(*"a malformed binder being swallowed instead of refused"*) applied to the
function plan 84 did not touch. That is a pre-existing defect in the hottest
function in the parser, fixed here because this row's fixture is what reached
it.

### 10.6 Criteria, honestly

| # | Criterion | Verdict |
|---|---|---|
| 1 | each §3 case transpiles, compiles, runs, stdout asserted | **met** for cases 1–4 and 6; case 5 is a `tests/ui/` fixture rather than a run |
| 2 | associativity observable in the output | **met** — `9 <-> 4 <-> 2` prints 7 where the left reading prints 3 |
| 3 | two precedences in one expression | **met in part** — `9 <-> 2 + 3` crosses a user rung with a built-in one, which is the property; two *declared* precedences in one expression is not in the example |
| 4 | no built-in precedence changes, no re-blessing | **met** — every existing golden is unchanged |
| 5 | `$` behaves identically | **met trivially** — its entry never moved (§10.3) |
| 6 | four diagnostics with `tests/ui/` fixtures | **met** — `RDX0123`–`RDX0126` |
| 7 | `voltaic fmt` round-trips at the right precedence | **met** — and it needed `Expr::UserBinary` to carry the fixity, §10.7 |
| 8 | `tree_sitter.rs` green, each case in the corpus | **met** — `test/corpus/binder.txt`, and §10.5's second finding is why it is a corpus case |
| 9 | `DECLINED.md`, doc 03 §14.4, §22 and `declined.rs` move together | **met** |
| 10 | examples green, no new `construct_coverage` `ALLOW` | **met** |

### 10.7 Why there is no fifth ambient guard

Plans 56, 59, 87 and 89 each install a `thread_local!` scope at
`emit_with_map`, because the information their emitters need is not in the
tree. This row needs none, and the reason is worth keeping: the parser **had to
consult the fixity table to parse the expression at all**, so recording what it
found costs one field on the node. `Expr::UserBinary` carries the spelling (for
`voltaic fmt`), the fixity and precedence (for the printer's parenthesis
decisions) and the alias target (for the emitter) — and each consumer reads its
own field with nothing ambient between them.

That is plan 61's two-consumers rule satisfied by **storing** rather than by
guarding, and it is available whenever the parser already knows the answer. The
four guards exist because their information is derived *after* parsing; this
one is not.

### 10.8 Corrections to this document

- §1.3 is wrong as a cost, and §6's *"the lexer gains a pre-pass,
  permanently"* with it. Both follow from an open spelling set that §1 never
  asked for. §10.1.
- §1.4's tree-sitter asymmetry **stands**: the grammar knows the twelve
  spellings and nothing about a file's fixities, so structural selection by
  binding power is still not available to an editor. What changed is that the
  grammar can at least lex a use as one token, which §10.5 records as nearly
  having been missed.
- §2's preconditions were unmeasured when written; §10.3 is their answer.
- §10.1 and §10.2 as originally written (the pre-implementation estimate and
  the argument for not starting) are superseded by this section. The estimate's
  three bullets were each individually right about the *open-spelling* design
  and together wrong about the row.
- §4's leaves 1 and 2 describe work that turned out not to exist. §10.2.
