# Declined and unreachable

Two different lists, and the difference matters.

**Unreachable** means Voltaic has no surface for a Rust construct *yet*. Eight
of those, each with a fixture the gate asserts still fails — see
[Rust reachability](../language/reachability.md).

**Declined** means Voltaic will not grow the construct. Those are decisions,
recorded with reasoning, in `docs/DECLINED.md`.

## What is declined, and why

| Declined | Reasoning |
|---|---|
| Default arguments | Labelled arguments already cover the readable cases; defaults add an invisible second signature |
| `...` rest patterns | Field punning and `..` cover the uses; the spelling collided with range syntax |
| Optional chaining (`?.`) | Shipped, then retired by plan 57 — it overlapped `??` and the receiver-last method call |
| The panic family | Shipped, then retired by plan 64 |
| A type checker | Rust has one. Two checkers means two that can disagree |
| A borrow checker | As above |
| An exhaustiveness checker | As above |
| A standard library | crates.io is the standard library. No plan implements a library function |

## The one that is worth dwelling on

**No standard library** is the rule with the largest measured payoff. A
predecessor project spent 112 of its 203 units of work building one. That is
the cost this rule avoids, and it is why "which crate should I use" is always
the right question here and "does Voltaic have a built-in for this" never is.

## Retired rather than declined

Two constructs — optional chaining and the panic family — were built, shipped,
and then removed. They are listed with the declines because the outcome is the
same for you as a user, but the distinction is recorded in the project's own
documents: a decline is a decision made in advance, and a retirement is a
decision corrected afterwards. Both are kept, because a document that only
shows the decisions that worked is a document that teaches nothing.
