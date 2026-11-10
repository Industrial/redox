# Conformance

**Can the compiler's own source be expressed in Redox, without the
inline-Rust escape hatch?** Doc 02 §4.8 phrases the question that way and
calls it *"the property actually worth proving"*.

**Generated.** Measured from the working tree by
`crates/redox-cli/tests/conformance.rs` and compared in the gate. Rewrite
it with:

```bash
REDOX_BLESS=1 cargo test -p redox-cli --test conformance
```

Plan 48 (`docs/42 - 48 - Conformance and Self-Hosting.md`) built it, and
plan 64 changed its answer.

---

## The answer

**Almost — and what is left is the `as` cast.**

**12 blocking occurrences**, down from 964 — and the construct that was 952 of them is gone. Plan 48 measured macro invocation at 99% of the obstacle and wrote *"spell it, and what remains is the `as` casts below"*; plan 64 spelled it, as `name! \-> e`, and this is what remained.

That is the whole arc of the measurement in three rows: plan 45 §1.5 **found** the gap and declined to spell it because it needed a surface decision in doc 03; plan 48 **priced** it, which is what made the decision worth taking; plan 64 **took** it. A count is what turned an unreachability note into a priority.

The figure is not zero and will not be: it is read off `docs/GRAMMAR.md`'s `UNREACHABLE` rows and moves on any commit that adds Rust, which is why it is generated rather than written down.

---

## Blockers

Each is graded `UNREACHABLE` in `docs/GRAMMAR.md`.

| Blocker | Occurrences |
|---|---:|
| `as` cast | 12 |
| `unsafe trait` | 0 |
| `unsafe impl` | 0 |
| `extern "C" fn` | 0 |
| Macro definition | 0 |
| `\|` or-pattern | *not counted* |

**Four of the five remaining gaps are zero**, which is the result doc 02 §4.8 did not expect: `unsafe trait`, `unsafe impl`, `extern "C" fn` and `macro_rules!` are real gaps and were never *this* obstacle. Only a count could say so — and the same count is what identified the one that was.

`\|` or-patterns are **not counted** rather than estimated. The counter is line-based, and no line-based pattern separates a pattern alternative from a bitwise or; a wrong number would be worse than an empty cell.

---

## By crate

| Crate | Files | With a blocker | |
|---|---:|---:|---:|
| `redox-ast` | 53 | 0 | 0% |
| `redox-bench` | 13 | 1 | 8% |
| `redox-cli` | 24 | 0 | 0% |
| `redox-diag` | 21 | 1 | 5% |
| `redox-driver` | 48 | 1 | 2% |
| `redox-emit` | 109 | 0 | 0% |
| `redox-fmt` | 57 | 0 | 0% |
| `redox-fuzz` | 6 | 0 | 0% |
| `redox-lexer` | 33 | 3 | 9% |
| `redox-lsp` | 22 | 2 | 9% |
| `redox-mcp` | 19 | 0 | 0% |
| `redox-parser` | 115 | 1 | 1% |
| **total** | **520** | **9** | **2%** |

---

## The worked port

`conformance/clause.sdx` is `crates/redox-ast/src/program/item/clause.rs` expressed in Redox, with no escape hatch in it. The generated `conformance/clause.xrs` is committed, per doc 02 §4.8 — *"commit the generated `.xrs` so the bootstrap never breaks"*.

It proves a real compiler data type is expressible: declaration, named fields, per-field visibility, derives, doc comments and verbatim types all survive. It does **not** prove the compiler is expressible — 45 lines is 0.2% of it, and the counts above speak to the rest.

Writing it found a defect the count could not see. Every field doc comment was silently discarded, and `redox fmt` deleted them from the source, because the lexer classified a comment opening a continuation line as *trailing* the previous line and then held it for a `Newline` a continuation never produces. AGENTS.md calls comment preservation load-bearing (doc 02 §10.1). Fixed in `redox-lexer`'s `at_line_start` — and that is the *"hard way"* doc 02 §4.8 said a port would find things.

---

## The Rust implementation is permanent

Doc 02 §4.8 corrects doc 01 §34's own wording: keep the Rust implementation *"permanently rather than as a 'fallback'"*. Nothing in the build depends on `conformance/`, and the compiler is not compiled from it.
