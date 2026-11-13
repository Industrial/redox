# Lexical surface

What a Voltaic token is. Most of it is Rust's, and the differences are few
enough to list.

```voltaic
{{#include ../../../examples/comments.sdx}}
```

## Comments are trivia, and trivia survives

`--` opens a comment and `-- |` a doc comment. Both reach the generated `.xrs`,
because the `.xrs` is a human-facing artifact and a compiler that throws your
comments away produces output nobody wants to read.

Inside the compiler's own source the rule is reversed: zero comments, enforced
over every ported crate. Reasoning there belongs in `docs/` and in commit
messages, where it can be dated and corrected.

## The `#` sigil has six forms

`#` is resolved by **longest match**, which is why the inventory matters more
than the individual spellings. The forms cover interpolation (`#{...}` inside a
string), attributes (`#[...]`), and the word-list literal `%w`. There are no
other sigils — a surface that grows one sigil per idea becomes unreadable at
about six, so the budget was spent deliberately.

## Operators are declared, not discovered

The operator table is closed, and a longest-match rule orders it so that `>>>`
is tried before `>>` before `>`. That ordering is load-bearing and is asserted
by a test inside the lexer rather than by the grammar, because the grammar's
flat alternation would accept `>>` followed by `>` without complaint.

## `and` / `or` / `not`

Words, not symbols. `&&`, `||` and `!` remain available as Rust's, but the
worded forms are canonical in Voltaic source — chained comparison and the
pattern-test operator `is` read as English with them and as line noise without.

## This example does not compile, deliberately

`examples/lexical.sdx` is the error-path example: it exercises token forms that
no parser rule accepts, so it is the one file in the corpus asserted to
**fail**. That is also why the repository has 44 examples and 43 goldens, which
is not an off-by-one.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 4, *Lexical surface*.
