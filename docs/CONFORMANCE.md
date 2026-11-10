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

Plan 48 (`docs/42 - 48 - Conformance and Self-Hosting.md`) built it.

---

## The answer

**No — and one construct is almost all of it.**

Macro invocation accounts for **922 of 934** blocking occurrences (99%). Doc 18 §7 retired `!` as a suffix and left no spelling, so `format!`, `write!`, `assert!`, `println!` and `panic!` — how this compiler formats, asserts and reports — cannot be written.

Spell it, and what remains is the `as` casts below. That makes this document a **priority** rather than a verdict: plan 45 §1.5 declined to spell macro invocation because it needs a surface decision in doc 03, and this is the measure of what that decision is worth.

---

## Blockers

Each is graded `UNREACHABLE` in `docs/GRAMMAR.md`.

| Blocker | Occurrences |
|---|---:|
| Macro invocation | 922 |
| `as` cast | 12 |
| `unsafe trait` | 0 |
| `unsafe impl` | 0 |
| `extern "C" fn` | 0 |
| Macro definition | 0 |
| `\|` or-pattern | *not counted* |

**Four of the seven gaps are zero**, which is the result doc 02 §4.8 did not expect: `unsafe trait`, `unsafe impl`, `extern "C" fn` and `macro_rules!` are real gaps and are not *this* obstacle. Only a count could say so.

`\|` or-patterns are **not counted** rather than estimated. The counter is line-based, and no line-based pattern separates a pattern alternative from a bitwise or; a wrong number would be worse than an empty cell.

---

## By crate

| Crate | Files | With a blocker | |
|---|---:|---:|---:|
| `redox-ast` | 47 | 8 | 17% |
| `redox-bench` | 13 | 5 | 38% |
| `redox-cli` | 24 | 20 | 83% |
| `redox-diag` | 21 | 16 | 76% |
| `redox-driver` | 48 | 24 | 50% |
| `redox-emit` | 83 | 59 | 71% |
| `redox-fmt` | 55 | 35 | 64% |
| `redox-fuzz` | 6 | 0 | 0% |
| `redox-lexer` | 32 | 16 | 50% |
| `redox-lsp` | 22 | 6 | 27% |
| `redox-mcp` | 19 | 7 | 37% |
| `redox-parser` | 107 | 33 | 31% |
| **total** | **477** | **229** | **48%** |

---

## The worked port

`conformance/clause.sdx` is `crates/redox-ast/src/program/item/clause.rs` expressed in Redox, with no escape hatch in it. The generated `conformance/clause.xrs` is committed, per doc 02 §4.8 — *"commit the generated `.xrs` so the bootstrap never breaks"*.

It proves a real compiler data type is expressible: declaration, named fields, per-field visibility, derives, doc comments and verbatim types all survive. It does **not** prove the compiler is expressible — 45 lines is 0.2% of it, and the counts above speak to the rest.

Writing it found a defect the count could not see. Every field doc comment was silently discarded, and `redox fmt` deleted them from the source, because the lexer classified a comment opening a continuation line as *trailing* the previous line and then held it for a `Newline` a continuation never produces. AGENTS.md calls comment preservation load-bearing (doc 02 §10.1). Fixed in `redox-lexer`'s `at_line_start` — and that is the *"hard way"* doc 02 §4.8 said a port would find things.

---

## The Rust implementation is permanent

Doc 02 §4.8 corrects doc 01 §34's own wording: keep the Rust implementation *"permanently rather than as a 'fallback'"*. Nothing in the build depends on `conformance/`, and the compiler is not compiled from it.
