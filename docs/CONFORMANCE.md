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

**2 blocking occurrences**, down from 964 — and the construct that was 952 of them is gone. Plan 48 measured macro invocation at 99% of the obstacle and wrote *"spell it, and what remains is the `as` casts below"*; plan 64 spelled it, as `name! \-> e`, and this is what remained.

That is the whole arc of the measurement in three rows: plan 45 §1.5 **found** the gap and declined to spell it because it needed a surface decision in doc 03; plan 48 **priced** it, which is what made the decision worth taking; plan 64 **took** it. A count is what turned an unreachability note into a priority.

The figure is not zero and will not be: it is read off `docs/GRAMMAR.md`'s `UNREACHABLE` rows and moves on any commit that adds Rust, which is why it is generated rather than written down.

---

## Blockers

Each is graded `UNREACHABLE` in `docs/GRAMMAR.md`.

| Blocker | Occurrences |
|---|---:|
| `as` cast | 2 |
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
| `redox-bench` | 13 | 0 | 0% |
| `redox-cli` | 24 | 0 | 0% |
| `redox-diag` | 21 | 0 | 0% |
| `redox-driver` | 49 | 0 | 0% |
| `redox-emit` | 109 | 0 | 0% |
| `redox-fmt` | 57 | 0 | 0% |
| `redox-fuzz` | 6 | 0 | 0% |
| `redox-lexer` | 33 | 0 | 0% |
| `redox-lsp` | 22 | 2 | 9% |
| `redox-mcp` | 19 | 0 | 0% |
| `redox-parser` | 120 | 0 | 0% |
| `redox-seed` | 6 | 0 | 0% |
| **total** | **532** | **2** | **0%** |

---

## Twelve crates are no longer a count

The table above measures what *could* be ported. Twelve crates **have** been, and for them the question is settled by rustc rather than by a blocker count: `redox-diag` (plan 73), `redox-lexer` (plan 74), `redox-ast` (plan 68), `redox-parser` (plan 82), `redox-fmt` (plan 69), `redox-fuzz` (plan 75), `redox-bench` (plan 76), `redox-mcp` (plan 77), `redox-lsp` (plan 78), `redox-emit` (plan 81), `redox-driver` (plan 80) and `redox-cli` (plan 79) are written in Redox, their `.sdx` living in `crates/<crate>/sdx/` beside the `src/` it generates, and the generated Rust is compiled as part of the workspace.

So **the compiler is now partly compiled from Redox** — 526 files of it. That corrects what this document said until plan 70: *"nothing in the build depends on `conformance/`, and the compiler is not compiled from it"*, which was true when plan 48 lifted one 45-line module out of a crate and checked it with `syn`, and stopped being true the moment a whole crate's `src/` was generated.

Plan 48's probe is retired with that sentence. `conformance/clause.sdx` was `clause.rs` expressed in Redox and checked by `syn::parse_file`, because one module lifted out of a crate cannot resolve `crate::Stmt`; the real port has no extraction to concede to, so the weaker oracle expired. Its no-escape-hatch assertion now runs over all 526 files in `port.rs`.

What it found still stands, and is the reason doc 02 §4.8 expected a port to be worth doing: every field doc comment was silently discarded, and `redox fmt` deleted them from the source, because the lexer classified a comment opening a continuation line as *trailing* the previous line and then held it for a `Newline` a continuation never produces. Plans 68 and 69 found three more of that class and two formatter defects, and plan 73 four more in the toolchain that writes the port.

---

## The Rust implementation is permanent

Doc 02 §4.8 corrects doc 01 §34's own wording: keep the Rust implementation *"permanently rather than as a 'fallback'"*. The one crates that are not ported are hand-written Rust and stay that way until a row says otherwise; the twelve that are ported keep their generated Rust **committed**, which is what doc 02 §4.8 means by *"commit the generated `.xrs` so the bootstrap never breaks"* — the committed output is the previous-version compiler every self-hosting language needs, stored as reviewable source rather than as a binary.
