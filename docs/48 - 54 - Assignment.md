# Plan 54 — Assignment

Plan ID: `assignment`
Depends on: 52 (`type-colon`)
Milestone: 9 — *surface coherence*
Status: `in progress` — leaf 3 outstanding

---

## 1. What this plan is

`:=` reassigns. `=` continues to bind, and only to bind.

Plan 20 §10.2 wrote this row's specification, in a result section, four years of plans ago in
project time and one milestone ago in fact:

> Plain reassignment appears in none of them. `counter = 2` is a binding wherever it is written.
> […] If the limit ever has to go, the spelling is the decision, not the implementation: a
> distinct token (`:=`) costs one lexer arm and no analysis, which is the shape of answer this
> project can afford. That belongs in a surface-design doc, not in a plan.

This row is that decision, taken, with the surface-design note written into doc 03 and the lexer
arm added. The reason the limit has to go is in §1.2.

### 1.1 The current behaviour is the only silent defect in the language

Measured across the new corpus — 12 programs, 1,501 lines, 905 of them code: of sixteen
constructs that refuse or reinterpret, **fifteen are caught by something** — eleven by a named
Redox diagnostic, four by rustc on the generated code. Exactly one is caught by nothing:

```
mut i = 0
i = j            -- emits `let i = j;`  -- a new binding, shadowing
```

Plan 20 §10.1 found the loop form of it first — `while n < 3 { let n = n + 1; }`, an infinite
loop that compiles — and closed that case with compound assignment. What remains is everything
compound assignment cannot express.

### 1.2 What `+=` cannot express, and why that is not a corner

Plan 20 §10.2's disclosure is precise: *"a loop can advance a numeric accumulator and cannot
rebind anything else."* Three shapes fall outside it, and all three are ordinary:

| Shape | What you must write today |
|---|---|
| A swap | `a += b - a` then `b += tmp - b`, with `tmp` bound first |
| A conditional update | `if n > best` then `best += n - best` |
| Moving a cursor | accumulate into a `String` char by char rather than `i = j` |

The third is from a real program. A tokeniser scanning a run of digits wants `i = j` after
finding the end of the run; the written form accumulates the text and advances `i += 1` inside
the inner loop, because the natural spelling silently shadows. The code is fine. The reason it
is shaped that way is not visible in it, which is the cost.

The first two are worse than verbose — `best += n - best` is arithmetic standing in for
assignment, and it is only correct because `best` is numeric. For a non-numeric binding there is
no workaround at all.

### 1.3 `:=` and not the alternatives

Three other spellings were weighed.

**`=` assigns when the name is an existing `mut` binding.** Shortest, reads best, and requires
knowing whether a name is already bound — scope tracking. AGENTS.md: *every check rustc can
make, rustc makes*, and Redox has no scope table. Plan 20 §10.2 refused this on exactly that
ground and the refusal still holds.

**`set i = j`.** A word rather than a sigil, greppable, and `=` untouched. Declined on weight:
it is the only statement form in the language that would need a keyword to say something every
other language says with punctuation, and the milestone is already adding `is`, `>|`, `|<` and
`\->`.

**Stay assignment-free and make the shadow an error.** The declarative option, and the one
closest to Haskell. Declined because Redox has `mut`, `while`, `until`, `for` and compound
assignment already — the language is not pure, and refusing the one operation that makes its
existing mutation vocabulary usable would be a purity it does not have anywhere else.

`:=` is one lexer arm, no analysis, and carries its meaning to anyone who has met Pascal, Go,
Algol or pseudocode. It is what plan 20 §10.2 named.

### 1.4 Shadowing stays available, and stays spelled `=`

`i = j` continues to mean a new binding. That is deliberate and it is the reason this row is
cheap: nothing about the existing statement grammar changes, and no program written before this
row changes meaning.

The cost is that the silent case remains *expressible*. Someone who means to assign and writes
`=` still shadows. What changes is that the correct spelling now exists, so the diagnostic in
§4's leaf 3 can name it — and a lint can see the shape that was previously invisible.

---

## 2. Verified preconditions

Measured 2026-11-10.

**`i = j` emits a shadowing `let` and nothing complains.** The only signal is a downstream
`unused variable` warning on the *outer* binding, pointing at the declaration rather than the
assignment:

```
mut i = 0
...
i = j
->  let i = j;          // rustc: unused variable: `i`  (on the outer one)
```

**Compound assignment works, on a binding and on a field.** `n += 1`, and doc 18 §2.5's
`increment self = self.value += 1`. Five operators, per plan 20.

**A swap requires three statements and arithmetic.** Verified running:

```
mut a = 1
mut b = 2
mut tmp = a
tmp += 0
a += b - a
b += tmp - b          -- prints "2 1"
```

**`:=` is unused in every position.** Absent from `operator_table.rs` and from `TokenKind`, so
the lexer arm is additive and collides with nothing. `:` is **already** a token — plan 52 gives
it a type-annotation *role*, it does not create it — and longest-match ordering is already
enforced by `scan_operator`'s `longer_operators_are_tried_before_their_prefixes`. The dependency
on plan 52 is about the role, not about tokenhood.

---

## 3. Concrete proof this plan targets

**A swap, written as a swap, in an example that runs.**

```
mut a = 1
mut b = 2
tmp = a
a := b
b := tmp
```

Three cases chosen because they fail differently:

1. **The swap.** Prints `2 1`. The arithmetic workaround prints the same thing, so only the
   source distinguishes them — which is why the example carries both and a comment saying why.
2. **A conditional update of a non-numeric binding.** `best := name` inside a loop over
   `Vec<String>`. There is no `+=` for this, so it is the case that is currently *impossible*
   rather than merely awkward.
3. **`:=` on a name that is not `mut`.** rustc rejects it — `cannot assign twice to immutable
   variable` — and that is the division of labour working. Asserted as a `tests/ui/` fixture so
   the message a reader meets is a known one.

---

## 4. Leaves

### Leaf 1 — `leaf-lexer`

`:=` as a token. One arm, longest-match against `:` from plan 52.

### Leaf 2 — `leaf-assignment-statement`

The statement form, and its emission as Rust's `=`. Target is a place expression: a name, a
field, an index, a deref — all four, because `*slot := v` and `row[i] := v` are the shapes a loop
actually needs and omitting them would leave the row half-delivered in the way plan 20 §10.1
describes.

### Leaf 3 — `leaf-shadow-lint`

A lint on a bare `=` that rebinds a name already bound **in the same statement list**. This is
not scope tracking: it is one list, in the parser, with no nesting and no resolution — which is
why it is affordable where the `=`-assigns option in §1.3 was not. It suggests `:=`, and it is a
warning rather than an error because shadowing is legal and sometimes meant.

### Leaf 4 — `leaf-corpus`

`examples/control.sdx` gains the swap and the non-numeric conditional update. Plan 20 wrote that
file *within* the limit this row removes, and the record of the limit is **not** in the file — it
is plan 20 §10.2's prose and the emit test `a_plain_equals_is_still_a_binding` in
`crates/redox-emit/tests/control.rs`. That test asserts the boundary this row moves, so it is a
migration target: rewritten to assert that `=` still binds **and** that `:=` assigns, which is
the boundary after this row rather than before it.

### Leaf 5 — `leaf-docs`

Doc 03 gains the surface-design note plan 20 §10.2 asked for. Doc 01 §4.3 and doc 18 §2.5 are
amended to show both operators. `docs/GRAMMAR.md` **gains** a `SUGARED` row for assignment — it
has none today, only `let` bindings and shadowing — so `docs/STATUS.md`'s gap table does not
move, because that table is derived from `UNREACHABLE` rows and assignment was never one.

---

## 5. Acceptance criteria

1. `name := e` emits Rust's `name = e`, for a name, a field, an index and a deref target.
2. A swap and a non-numeric conditional update each appear in a running example with asserted
   stdout.
3. `:=` against a non-`mut` binding is a `tests/ui/` fixture carrying rustc's message.
4. `i = j` still binds, and the new lint fires on it with a `:=` suggestion.
5. The lint does **not** fire across statement lists — a `=` inside a nested block shadowing an
   outer name is legal and silent, asserted, because the alternative is scope tracking.
6. Compound assignment is unchanged; all five operators — `+=`, `-=`, `*=`, `/=`, `%=` — still
   work, asserted.
7. `a_plain_equals_is_still_a_binding` is rewritten to assert both halves of the new boundary,
   not deleted.
8. `construct_coverage` green with `:=` indexed.
9. Plan 20 §10.2's disclosure is marked resolved in doc 04's register, naming this row.

---

## 6. Out of scope, and what it discloses

**The silent case is narrowed, not closed.** `=` still shadows, by design (§1.4). The lint
covers one statement list; a shadow across a block boundary is invisible to it and legal. That is
the honest residue, and the alternative — full scope tracking — is the thing AGENTS.md forbids
and plan 20 §10.2 already refused.

**No destructuring assignment.** `(a, b) := (b, a)` is Rust's and would be the natural way to
spell a swap. It is declined here because plan 33 made destructuring a *binding* form, and
giving the same patterns an assignment meaning doubles the pattern grammar's interpretations for
one idiom. Recorded as a candidate, not a gap.

**No `+=` on the new token.** `:+=` is not a thing. The five compound operators keep `=`,
because they already mean assignment and always did — this row does not need to make them
consistent with a token that exists to disambiguate from binding.

**Expression-position assignment is plan 55's.** Under expression orientation `a := b` is an
expression of type `()`, as it is in Rust. Here it is a statement, which is all the current
grammar has a place for.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout asserted; the
non-`mut` refusal in `tests/ui/`; `construct_coverage` green.

---

## 8. Emerald reuse

Emerald had assignment from the start and therefore never recorded why. The reusable item is
Redox's own: **plan 20 §10.1's finding that a row's scope can be smaller than the thing it
claims to deliver** — a `while` whose body cannot change its condition is not a loop — and that
is the fourth such case plan 20 counted.

This row is the fifth instance of the same lesson and the first to be *planned* from it rather
than discovering it at the gate. The disclosure plan 20 wrote is why.

---

## 9. Decision log

1. **`:=`, as plan 20 §10.2 named it.** The alternatives are in §1.3; two cost analysis Redox
   must not grow, and one costs a keyword for no gain.
2. **`=` keeps shadowing.** Changing it would make every existing program's `=` ambiguous and
   would require the scope table that rules out the `=`-assigns design.
3. **Place expressions, all four, in one leaf.** Plan 20 §10.1's lesson applied before the gate
   rather than after it: a row that ships assignment-to-names-only has shipped something a loop
   cannot use.
4. **The lint is one statement list, deliberately.** It catches the common case with no
   resolution. Extending it would be the first step toward the thing AGENTS.md forbids, and the
   boundary is asserted by criterion 5 so a future row cannot drift across it quietly.
5. **No destructuring assignment.** §6. One idiom is not worth doubling the pattern grammar's
   meanings.

---

## 10. Result

**Leaves 1, 2, 4 and 5 done; leaf 3 outstanding.** `:=` assigns, `=` still
binds, and the limit plan 20 §10.2 disclosed is gone. The shadow lint has not
been built — §10.6 says what it needs and why it was deferred.

### 10.1 One node, not two, and no new match arm anywhere

§4 and the orientation both expected a new `StmtKind` variant and budgeted
four exhaustive matches gaining an arm. None did. `StmtKind::Assign` was
already *compound* assignment, `a += b` is `a = a + b`, and the two are one
construct with a modifier — so `op` became `Option<BinaryOp>` and the whole
AST change is one field.

That matters beyond the line count. `emit/stmt.rs`'s match is deliberately
exhaustive — *"a new `StmtKind` must fail to compile here"* — and the same is
true of `redox-fmt`'s `kind_text.rs`. A design that adds a variant pays that
tax at four sites; one that refines a field pays it at the two that actually
print the operator, which is where the real decision lives:

```rust
// redox-emit                        // redox-fmt
op.map_or("", |op| op.rust_text())   op.map_or(":", |op| op.redox_text())
```

Redox's two spellings collapse to Rust's one, and the asymmetry in those two
lines *is* the row: Rust tells a binding from an assignment with `let`, and
Redox has no `let` to lean on. Printing a bare `=` in the formatter would turn
an assignment back into a binding.

### 10.2 Criterion 3 was in the wrong corpus, and the reason generalises

Criterion 3 named `tests/ui/`. It cannot live there: `ui.rs` runs
`redox build`, which parses, lints, emits and writes **without ever invoking
rustc**, so a program whose only fault is `E0384` builds clean and the harness
fails on its own `assert!(!out.status.success())`.

The fixture is `crates/redox-cli/tests/check/immutable_assignment.sdx`, and
`redox check` reports

```text
error[E0384]: cannot assign twice to immutable variable `total`
  --> immutable_assignment.sdx:19:3
```

against the `.sdx`. `is_mut` is set only by the literal `mut` keyword and the
emitter writes `mut` only when it is set, so Redox has no mechanism to make
this compile — and the test is what stops one being added as a convenience
later. *"Every check rustc can make, rustc makes."*

### 10.3 §6 declined something that was free, and rustc settles it

The assignment target is a general `Expr`, which is the existing precedent for
compound assignment and the reason `self.value`, `v[0]` and `*r` all work. So
does `(a, b) := (b, a)` — Rust's destructuring assignment, stable since 1.59 —
and §6 and decision 5 declined it.

**The decline is withdrawn.** Refusing it would mean Redox deciding what is and
is not a place expression, and `(1 + 2) := 3` is rustc's `E0070`: it is a check
rustc makes, so rustc makes it. The alternative also costs the corpus its best
line — a swap is the one shape no compound operator can express, and
`(a, b) := (b, a)` is how it should read.

### 10.4 The corpus was missing four of five compound operators

`construct_coverage` reported `-=` unwitnessed the moment this row graded
assignment in `docs/GRAMMAR.md`, and the orientation had measured the same
thing from the other side: only `+=` appeared in any example or golden. Four
operators shipped in **plan 20** and were asserted nowhere but one
string-literal unit test.

`examples/control.sdx` now exercises all five in an order-dependent sequence,
so a transposed lowering changes the printed value.

Two further gaps the same test forced into the open:

- **`docs/GRAMMAR.md` had no assignment row at all** — not for `:=`, which is
  new, and not for compound assignment, which was four plans old. Both are
  graded now.
- **Nothing in the corpus shadowed anything.** The `Shadowing | PASSTHROUGH`
  row cited `examples/expressions.sdx`, which holds exactly one binding; the
  nearest candidate elsewhere was two bindings of `a` in two *different*
  functions, which is not shadowing either. The construct was graded reachable
  and witnessed by nothing. It needed a witness for this row regardless —
  criterion 5 requires that a nested-block shadow be legal and silent — so
  `examples/control.sdx` gained one, and the row cites it.

### 10.5 Two more readers, as plan 52 predicted

§11.5 found no leaf for the tree-sitter grammar, and `tree_sitter.rs` asserts
every corpus `.sdx` — `tests/lint/` included — parses with no `ERROR` node. The
grammar's `compound_operator` is now `assignment_operator` with `:=` at the
head of its `token(choice(...))`. No precedence was needed: tree-sitter takes
the longest match, `:=` is longer than `:`, and neither it nor `::` is a prefix
of the other.

Also as predicted, the formatter is a *writer* of the token and not only a
reader. `kind_text.rs` is in §10.1 above.

### 10.6 Leaf 3 — the shadow lint — is outstanding

Deferred rather than attempted, for two measured reasons:

1. **Leaf 3 names the wrong crate.** It says the lint is *"one list, in the
   parser"*. `crates/redox-driver/src/lint.rs`'s own module doc already
   settled that a lint cannot live in the parser — `ParseFailure` has no notion
   of a warning, and `Diagnostic` is the type that carries a `Severity`. The
   lint belongs in `redox-driver/src/lint/`, over the AST.
2. **Plan 53 landed the machinery it needs, in the same files, in this same
   push.** `lint` now takes an `Option<&Program>` and
   `redox-driver/src/lint/suppressed_spans.rs` already walks items, clauses,
   `where` locals, statements and statement-owned blocks for pragmas. A shadow
   lint is a second consumer of that walk, not a new one — and building it
   concurrently with the walk itself would have meant two agents in one file.

What it still owes when it is built: `RDX1003` (plan 53 took `RDX1002`), a
`tests/lint/` fixture because `code_coverage`'s `NOT_YET_REACHABLE` is `&[]`,
a decision about what counts as "a name" when `Binding.pattern` is a
`Pattern` rather than an `Ident`, and **a way for an author to say a shadow is
meant** — `allow_pragma` is hard-coded to `unwrap`, so an intentional shadow
would warn forever. Plan 53 parametrising the pragma is the cheaper order.

Measured, for whoever builds it: scanning every `.sdx` under `examples/`,
`tests/` and `crates/` for two bindings of one name in the same statement list
found **zero** hits before this row, and the one deliberate shadow added here
is in a *nested* block, where criterion 5 says the lint must stay silent. So
it will fire on nothing that exists, which is the right starting point and
also means the corpus proves very little about it.

### 10.7 Smaller findings

- **`:=`'s only ordering constraint is against `":"`**, not `"::"`, because
  neither of the latter is a prefix of the other.
  `longer_operators_are_tried_before_their_prefixes` enforces the one that
  matters and is correctly silent about the imagined one.
- **Reaching the wrong parse arm would fail loudly**, which is worth knowing
  before relying on it: `looks_like_a_binding` tests for `Eq`, `Colon` or
  `LBrace` at the second token and `reaches_a_binding_eq` scans for a
  depth-zero `Eq`, so a `ColonEq` answers no to both and falls through to
  `parse_expr` with *"expected a line break after the statement"*.
- **`redox-bench` builds the signature marker in a `format!` template**, so
  `both_halves_define_the_same_function` spelled it `::` and survived a
  repository-wide sweep for a spaced `::` — the space comes from `"{} ::"`, not
  from the file. It was the last one left.
- **Three byte offsets derived in comments** moved when the marker lost a
  character; plan 52 §10.4a has them.
- §5's criteria were numbered 1,2,3,4,5,6,7,7,8. Renumbered.
- Plan 20 §10.2's disclosure is marked resolved in doc 04's register row 20,
  naming this row.

---

## 11. Corrections from orientation

Written after reading the code this row targets. The additive premise holds:
`:=` appears nowhere in the lexer, the AST, the parser, the tree-sitter grammar
or the corpus — the only `:=` in the repository is BNF notation in three doc
comments. Seven more claims check out. Five do not, and three of those would
have sent the implementation to the wrong crate or the wrong corpus.

### 11.1 Criterion 3 names a corpus that cannot carry it

`tests/ui/` runs `redox build --plain` and asserts the case **fails**
(`crates/redox-cli/tests/ui.rs`). `build_command.rs` never invokes rustc — it
parses, lints, emits and writes. A program whose only fault is
`cannot assign twice to immutable variable` therefore **builds clean**, and the
ui harness fails on its own assertion rather than on the program.

The corpus that runs real rustc and remaps its message onto the `.sdx` is
`tests/check/` with `crates/redox-cli/tests/check.rs` — five fixtures
(`clean`, `restructured`, `syntax`, `trivial`, `verbatim`), hand-written line
and column assertions rather than blessed `.stderr`. Criterion 3 belongs there.
(`tests/unreachable/` also runs `redox check`, but is reserved for
`docs/GRAMMAR.md`'s `UNREACHABLE` rows.)

### 11.2 Leaf 3 puts the lint in the crate whose own documentation refuses it

Leaf 3 says the shadow lint is "one list, in the parser".
`crates/redox-driver/src/lint.rs`'s module doc, under *Why the driver and not
the parser*, already settled this:

> Plan 30's `check_callback_tails` is a whole-program pass in the **parser**,
> and it produces `ParseError`s — which become hard errors, because
> `ParseFailure` has no notion of a warning. **A lint cannot go that way.**
> `Diagnostic` is the type that carries a `Severity`, and the driver is where
> `Diagnostic`s are built

So the lint lives in `crates/redox-driver/src/lint/`, over the AST. That brings
a signature change the leaf does not budget: `pub fn lint(path: &Path, source:
&str)` is source-only *because* its one existing lint is lexical. A shadow lint
needs statement lists, so it needs `&Program`. Four call sites, all of which
already hold a parsed `program`: `redox-cli/src/build_command.rs:28`,
`redox-driver/src/check.rs:65`, `redox-lsp/src/serve/diagnostics.rs:41`,
`redox-mcp/src/build.rs:42`.

Two further facts the leaf needs. **Scope tracking is net-new** — nothing in
the repository tracks bindings; `Block { stmts: Vec<Stmt> }` is the whole model
of "one statement list". And `Binding.pattern` is a `Pattern`, not an `Ident`,
so what counts as "a name" for a destructuring binding is undecided and must be
decided here.

The lint's home corpus is `tests/lint/`, not `tests/ui/`: `lint.rs` asserts
`out.status.success()` where `ui.rs` asserts failure, and `code_coverage.rs`
accepts a fixture in either. A new lint code needs one, because
`NOT_YET_REACHABLE` is `&[]`. Next free lint code: **`RDX1002`** — `RDX1001`
(`UnwrapUsed`) is the only warning that exists today.

**Suppression is a hole.** `allow_pragma.rs:23` hard-codes
`trivium.text.trim() == "redox: allow unwrap"`. Leaf 3 says the lint warns
"because shadowing is legal and sometimes meant" — but an author who means it
has no way to say so. Either the pragma is parametrised or an intentional
shadow warns forever. Plan 53 is already changing that file; sequencing 53
before 54 makes this cheaper.

### 11.3 `StmtKind::Assign` is already taken

`crates/redox-ast/src/block/stmt/kind.rs:73-80` is **compound** assignment:
`Assign { target: Expr, op: BinaryOp, value: Expr }`. The new variant needs a
different name, or `Assign.op` becomes `Option<BinaryOp>` and the two share one
node. The plan never mentions the collision.

The `StmtKind` blast radius is also unbudgeted. Four exhaustive matches with no
wildcard gain an arm: `redox-emit/src/emit/stmt.rs` (deliberately exhaustive —
*"a new `StmtKind` must fail to compile here"*),
`redox-fmt/src/stmt_text/kind_text.rs`, `redox-fmt/tests/pattern_printer.rs:86`,
and one in `redox-cli/tests/`. `redox-lsp` and `redox-mcp` match no `StmtKind`.

### 11.4 §6 declines something this row would otherwise ship

Because the assignment LHS is a general `Expr` — which is the right design, and
the existing precedent for compound assignment — `(a, b) := (b, a)` **parses**
(`prefix.rs`'s LParen arm builds `Expr::Tuple`) and emits `(a, b) = (b, a);`,
which is valid Rust: destructuring assignment, stable since 1.59. §6's "no
destructuring assignment" and decision 5 therefore describe a **refusal that
must be implemented**, not a feature left out. As written, the plan ships the
thing its own §6 declines.

### 11.5 There is no leaf for the tree-sitter grammar

`tree-sitter-redox/grammar.js:439` holds the assignment token set
(`token(choice("+=", "-=", "*=", "/=", "%=", "??="))`) and
`crates/redox-cli/tests/tree_sitter.rs` asserts that every corpus `.sdx` parses
with **no `ERROR` node** — including, at `:102`, every file under
`tests/lint/`. So leaf 4's `examples/control.sdx` edit *and* the new lint
fixture both fail that gate until the grammar gains `:=` and the parser is
regenerated. Editors need no change: there are no operator lists under
`editors/`.

### 11.6 Smaller corrections

- **The `:=` ordering constraint is only against `":"`.** `operator_table.rs`
  must list `(":=", …)` before line 55's `(":", …)`; there is **no** constraint
  relative to `"::"`, because neither string is a prefix of the other and both
  are two characters.  `scan_operator.rs`'s
  `longer_operators_are_tried_before_their_prefixes` enforces exactly the prefix
  relation, so it will catch the real mistake and is correctly silent about the
  imagined one.
- **`:=` must be read in `stmt()`'s final arm beside `compound_assign`**, not in
  `try_binding`. Verified it fails loudly rather than silently if it is not:
  `looks_like_a_binding` tests `peek_at(1) ∈ {Eq, Colon, LBrace}` and
  `reaches_a_binding_eq` scans for `TokenKind::Eq` at depth zero, so a
  `ColonEq` answers no to both and `i := j` falls through to `parse_expr`,
  erroring with *"expected a line break after the statement"*.
- **Criterion 3's premise is safe.** `mut` is **never** inferred — `is_mut` is
  set only by the literal keyword at `parser.rs:878`, and `emit/stmt.rs:59`
  emits `mut` iff that flag is set. Redox has no mechanism to add it, so
  rustc's message is reachable.
- **All four place forms already parse**: `Expr::Ident`, `Expr::Field`
  (`postfix.rs`), `Expr::Index` (`postfix.rs:50`) and
  `Expr::Unary{op: Deref}` (`unary_op.rs:13`). There is no place-expression
  concept and none is needed — `StmtKind::Assign`'s own doc comment already
  records why the target is an expression.
- **Criterion 6 is already satisfied**, but its "asserted" rests on one
  string-literal unit test. All five operators work
  (`compound_assign.rs:21-32`, `emit/stmt.rs:91-98`,
  `the_five_compound_operators_assign`), but only `+=` appears in any golden or
  example — `tests/golden/control.xrs:40,63,105`. `??=` is in the tree-sitter
  token set and retired from the parser.
- **The new lint fires on nothing that exists.** Scanning every `.sdx` under
  `examples/`, `tests/` and `crates/` for two bindings of the same name at the
  same indentation found **zero** hits; the nearest pair,
  `examples/fallible.sdx:78` and `:101`, are at different indents and so in
  different statement lists. The `examples`, `run` and golden corpora are safe.
- **A corollary defect worth closing here.** `docs/GRAMMAR.md:138` grades
  `Shadowing | PASSTHROUGH | examples/expressions.sdx`, and that file contains
  exactly one binding and no shadowing at all — so :137's `let mut` citation is
  false for it too. Criterion 5's nested-block fixture has no existing basis,
  and plan 51's `construct_coverage` now reads that row. GRAMMAR.md also has
  **no row for compound assignment**, despite it shipping in plan 20; adding
  one is free here and `examples/control.sdx` already witnesses it.
- **§1's quotation of plan 20 is inexact.** The source reads *"The five
  compound operators are what docs 01, 03 and 18 define, and **plain
  reassignment appears in none of them**."*
- **Criterion 8's target is `docs/04 - Redox Plan of Plans.md:198`**, register
  row 20, ending *"Plain reassignment stays **unspecified**, disclosed in plan
  §10.2 rather than designed around"*. That is the clause to mark resolved.
- **§5 numbers its criteria 1,2,3,4,5,6,7,7,8.** Renumber.
- **Re-blessing is unstated**: `examples/control.stdout` and
  `tests/golden/control.xrs` both move under leaf 4.
- **One hazard plan 52 hands over.** `parse_expr.rs:29-38` consumes a bare
  `Colon` at `min == 0` as ascription, so after plan 52 `total : = 42` and
  `total := 42` differ by one space. The first is a parse error rather than a
  second meaning, so plan 52 §1.1's standard — no semantic difference carried by
  a space — is **not** violated. Worth saying so in §6 rather than leaving a
  reader of the milestone to find it.
