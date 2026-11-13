# Plan 03 — Parser Core

Plan: 03 of 48 (`docs/04 - Voltaic Plan of Plans.md`, Milestone 1)
Slug: `parser-core`
Status: **`done`** — shipped 2026-11-08; see §10 for evidence per criterion
Depends on: plans 01, 02
Discloses: plans 05, 19, 20, 21, 23, 24, 26

---

## 1. What this plan is

Plan 02 finished the lexical surface. Its fixture, `examples/lexical.sdx`, lexes cleanly and
is **rejected by the parser** at line 7 — the first binding plan 01's three-production
grammar cannot express. This plan builds the grammar that accepts it.

Scope: **expressions, signatures and bindings**. Specifically:

- a full expression grammar with a **Pratt loop on Rust's precedence table**, which doc 02
  §5.2 fixes as the rule — a Voltaic expression that reads like Rust and binds differently is
  the worst available failure mode;
- function **parameters and return types**, so doc 01 §31 Phase 1's own acceptance test
  parses;
- **typed and mutable bindings** — `name: Type = value`, `mut name = value`;
- **multi-error recovery** at item and statement boundaries.

Not in scope, and each named to a successor in §6: control flow, patterns, data
declarations, the general parenless call grammar, and the emitter work that would make the
new expression forms produce running Rust. This plan makes the AST rich; plan 04 makes the
emitter keep up with it.

### 1.1 Types are captured verbatim, not parsed

Doc 03 §14.1 fixes type expressions as **copied through verbatim**: every present and future
Rust type form is reachable for free and nothing can drift. So this plan does not build a
type grammar. It captures a type as the raw source text of its token run, which is both less
work and strictly more capable than parsing a subset would be.

That makes "where does a type end?" the only real question. The answer is a depth-counted
scan to the first `,`, `)`, `=` or `->` at angle/paren/bracket depth zero — with one
refinement that earns its place: **an `->` preceded by `)` does not end a type**, because
that is `Fn(i32) -> i32` rather than a signature's arrow. `: i32 ->` terminates because
`i32` precedes the arrow; `&dyn Fn(i32) -> i32)` does not, because `)` does.

---

## 2. Verified preconditions

None beyond plans 01 and 02. The one fact this plan depends on — that the lexer emits a
token stream rich enough to parse doc 03's surface — is what plan 02's 117 tests establish.

---

## 3. Concrete proof this plan targets

**Doc 01 §31 Phase 1's own acceptance test**, which plan 01 deliberately deferred:

```
add = (a: i32, b: i32): i32 ->
  a + b

main = ->
  print (add 10, 20)
```

generating:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("{}", add(10, 20));
}
```

compiled by real rustc and printing `30`.

Two notes on the shape, both deliberate. The nested call is **parenthesized** because doc 03
§5.1.2 case 1 requires it until plan 19 generalizes the call grammar — writing
`print add 10, 20` would mean `print(add(10, 20))` only under a greedy rule Voltaic declines.
And `print` applied to a non-string lowers to `println!("{}", …)`, since a bare
`println!(expr)` is only valid for a literal format string.

Plus `examples/expressions.sdx`, exercising the precedence table against hand-computed
values, so a precedence error is a wrong *number* rather than a wrong tree.

---

## 4. Leaves

### Leaf 1 — `leaf-ast-expansion`

`voltaic-ast` grows the nodes this grammar produces, one construct per file under the module
tree doc 06 §2 requires:

| Node | Purpose |
|---|---|
| `Binding` | `name: Type = value`, with `mut` and an optional type |
| `Param` | one parameter: name, type, and whether `mut` |
| `Signature` | parameter list plus optional return type |
| `TypeExpr` | a type's verbatim source text and span (§1.1) |
| `BinaryOp`, `UnaryOp` | operator kinds, named not spelled |
| `Expr::{Binary, Unary, Field, Index, MethodCall, Path, Int, Float, Bool, Char, Paren}` | the expression forms |

`FnDecl` gains a `Signature`. Every node keeps its span and, where it owns source lines, its
trivia.

### Leaf 2 — `leaf-precedence-table`

`parse/precedence.rs`: **Rust's precedence table, unchanged** (doc 02 §5.2, doc 03 §4.5),
as a function from `TokenKind` to binding power. The new operators sit where doc 03 §4.5
places them: `??` below `||`, `|>` below `??` and above assignment, `catch` at `??`'s level.

A test asserts the table is strictly ordered and that every binary token in the lexer's
surface has an entry, so adding an operator without a precedence fails here rather than
parsing surprisingly.

### Leaf 3 — `leaf-pratt-expression-parser`

`parse/parser/expr.rs`: a Pratt loop over the table. Prefix: literals, identifiers, paths,
unary `-` and `!`, parenthesized groups, `@`/`@@` receivers. Infix: every binary operator,
left-associative except assignment. Postfix: field access, index, method call, `?`, `.await`,
call parentheses.

Right-binding power controls associativity, so the table is the single place associativity
lives.

### Leaf 4 — `leaf-signatures-and-bindings`

Parameters with types, optional return type, `mut` on both parameters and bindings, and the
type-capture scan of §1.1. The item form becomes
`name = (params): Ret ->` with every part optional except the name, `=` and `->`.

### Leaf 5 — `leaf-error-recovery`

Recovery at **two** levels rather than plan 01's one: an item boundary as before, and a
statement boundary within a block, so one bad statement does not discard its siblings. This
is the axis `docs/COMPILER.md` §1 measured LALRPOP losing — it reports one error and stops —
so the test asserts a file with three independent errors reports three.

### Leaf 6 — `leaf-emitter-keeps-up`

The emitter must render every new node, because an AST node with no lowering is a silent
`todo!()` waiting to happen. Signatures, binary and unary expressions, field access, index,
method calls, paths, and the `print`-of-non-string case from §3.

### Leaf 7 — `leaf-tests-and-gate`

`examples/phase1.sdx` (§3's program) and `examples/expressions.sdx`, both compiled by rustc
and run with asserted stdout; golden files for both; unit tests for the precedence table;
and a recovery test asserting three errors from three mistakes. Full gate, then independent
re-verification.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | §3's program transpiles, compiles under rustc with zero stderr, runs, and prints `30` |
| 2 | `examples/expressions.sdx` prints its hand-computed values, so precedence is checked by arithmetic |
| 3 | Rust's precedence and associativity hold: `2 + 3 * 4` is 14, `10 - 3 - 2` is 5, `!a && b` groups as `(!a) && b` |
| 4 | A type expression round-trips verbatim, including `Vec<HashMap<String, u8>>` |
| 5 | `&dyn Fn(i32) -> i32` parses as one parameter type (the §1.1 refinement) |
| 6 | Three independent errors in one file produce three diagnostics |
| 7 | A bad statement does not discard its sibling statements |
| 8 | Plans 01 and 02's tests all still pass |
| 9 | Every new file obeys doc 06 §§1–2 |
| 10 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| `if`/`unless`/`while`/`for`/`cond`/`loop` | **plan 20** | Expressions first; control flow consumes them |
| `switch`, patterns, `is`, `guard` | **plans 21, 22** | Patterns are their own grammar |
| Structs, enums, traits, impls | **plans 24, 25** | Item forms beyond `name = ->` |
| The general parenless call grammar | **plan 19** | §3 parenthesizes its nested call until then |
| `miette` rendering | **plan 05** | Errors carry spans; rendering is separate |
| String interpolation in the emitter | **plan 23** | The lexer produces `parts`; the emitter still uses `raw` |
| `\|>`, `??`, `catch`, `?.` lowering | **plans 27, 28** | They get precedence here, not meaning |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

Plus: every example `.sdx` transpiles, compiles under rustc, runs, and matches its asserted
stdout.

---

## 8. Emerald reuse

**Consulted as a contrast, no code:** Emerald's plan 26 threaded an `errors: &mut Vec<_>`
parameter through its generated grammar to get multi-error recovery, which `docs/COMPILER.md`
§1 records as one of the six axes the bake-off measured — recursive descent gets it in about
fifteen lines, and leaf 5 extends it to a second level for the same cost.

**Deliberately not taken:** `crates/emerald-parser`'s grammar itself. It is LALR(1) over a
Ruby-derived surface with `end` delimiters; nothing about its productions transfers to an
indentation-sensitive grammar whose precedence table is Rust's.

---

## 9. Decision log

- **Rust's precedence table, unchanged.** Doc 02 §5.2's reasoning is the whole argument: a
  Voltaic expression that reads like Rust and binds differently is the worst available failure
  mode, worse than an unfamiliar operator. The table is the only place associativity lives.

- **Types are captured verbatim rather than parsed** (§1.1). Doc 03 §14.1 already decided
  this for the language; the parser consequence is that `TypeExpr` is text and a span. A
  partial type grammar would be more code and strictly less capable.

- **The `->`-after-`)` refinement is a real rule, not a hack.** Without it `&dyn Fn(i32) ->
  i32` cannot be a parameter type, which would make doc 03 §14.1's "every Rust type form is
  reachable" false on day one. Criterion 5 pins it.

- **Recovery gains a second level.** Plan 01 recovered at item boundaries only, so one bad
  statement discarded every sibling in its block. For a language whose bodies are indented
  blocks, the statement boundary is where a user's mistakes actually are.

- **The emitter grows in the same plan as the AST.** Splitting them would leave nodes with
  no lowering, and an AST node with no lowering is a `todo!()` with a delay fuse. Doc 02
  §8.1–8.2's rule — text, compilation and execution all asserted — is what forces the two
  to move together.

- **§3 parenthesizes its nested call.** Doc 01 §31's own test writes `print add 10, 20`,
  which doc 03 §5.1.2 case 1 rejects as ambiguous until plan 19. Writing the parens now and
  removing them in plan 19 is honest; silently adopting the greedy reading Voltaic declined
  would not be.

---

## 10. Shipped — evidence per criterion

Executed 2026-11-08. Full gate green: all six moon tasks exit 0. **132 tests pass**, up
from 117.

| # | Criterion | Evidence |
|---|---|---|
| 1 | Phase 1 prints `30` through real rustc | `phase1.rs::phase1_transpiles_compiles_and_prints_30` |
| 2 | Hand-computed precedence values | `expressions_print_their_hand_computed_values` — ten asserted values |
| 3 | Rust precedence and associativity | the ten values above, plus `binary_power`'s own 6 unit tests |
| 4 | A type round-trips verbatim | `a_nested_generic_type_round_trips` — `Vec<HashMap<String, u8>>` |
| 5 | `&dyn Fn(i32) -> i32` is one parameter type | `a_function_type_parameter_parses_as_one_type` |
| 6 | Three errors produce three diagnostics | `recovery.rs::three_errors_in_one_file_produce_three_diagnostics` |
| 7 | A bad statement keeps its siblings | `a_bad_statement_does_not_discard_its_siblings`, `two_bad_statements_in_one_body_are_both_reported` |
| 8 | Plans 01 and 02's tests still pass | whole suite green |
| 9 | Doc 06 §§1–2 obeyed | `one_construct_per_file.rs`, `directory_structure.rs` |
| 10 | Full gate green | six moon tasks, exit 0 |

### Three defects found by running it, not by inspection

1. **A declared return type needs a tail expression, not a statement.** `add` emitted
   `a + b;`, so rustc reported *"expected `i32`, found `()`"* with its own "remove this
   semicolon" hint. Doc 01 §5.2's example requires `{ a + b }`. The fix is scoped: a
   function that *declares* a return type ends in a tail expression, one that does not
   returns `()` and ends in a statement — which is why plan 01's `println!("hello");`
   golden is unchanged.
2. **Plan 01's parenless call was too narrow for its own successor.** It accepted only
   `name "string"`, so `print 2 + 3 * 4` and `print add(10, 20)` both failed. Added
   `stmt_call`, a statement-position parenless call taking full expressions — Emerald's
   plan-90 option (A), restricted to the position its own grammar already restricted
   leading unary operators to.
3. **A third blaze shim.** Plan 02 fixed test-time `PATH` resolution by baking `RUSTC` at
   build time; under `blaze build` that *is* blaze's own shim, which refuses general work.
   `build.rs` now **verifies** a candidate by running `--version` and checking the banner,
   rather than trusting either source. `AGENTS.md` records both halves, because each looks
   like the other's fix.

### Deviations from the plan as written

- **§3's proof program changed from `print (add 10, 20)` to `print add(10, 20)`.** The
  parenthesized form is a parenless call *inside parentheses*, which still needs argument-
  position parenless calls — plan 19. The parenthesized-callee form is the shape plan 03
  can actually prove, and it is closer to doc 01 §31's intent than a form that does not
  parse.
- **`examples/expressions.sdx` avoids a statement-initial unary argument.** `print -3 + 10`
  would require the adjacency rule that distinguishes it from `a - b`, which is plan 19's;
  the fixture writes `print (-3) + 10` and says why in a comment.
- **Reassignment is not a statement form.** `total = total + 7` parses as a *new binding*
  and emits `let total = …`, because distinguishing declaration from reassignment needs the
  scope resolver doc 02 §6.4 budgets but plan 03 does not build. Compound assignment
  (`total += 7`) is doc 01 §4.3's own spelling for mutation and belongs to **plan 20** with
  the rest of the statement surface; the fixture uses neither. Recorded here rather than
  left as a surprise.
- **Two helpers became siblings rather than children** — `token_text`, `parameter`,
  `starts_argument` — because doc 06 §2.4 caps depth at three levels below `src/`, which
  the depth check caught for `stmt_call/starts_argument.rs` at level four.

### What plan 04 inherits

The emitter now renders every AST node with **no wildcard arm**, so plan 04's
determinism work starts from a complete lowering rather than a partial one. Two operators —
`??` and `|>` — parse and carry precedence but emit a `compile_error!`, which is deliberate:
they are named to plans 28 and 27 and will fail loudly rather than plausibly until then.
