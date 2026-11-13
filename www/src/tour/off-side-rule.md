# The off-side rule

Voltaic has no braces and no semicolons. Indentation *is* the block structure,
which is what "off-side rule" means: a construct's body is everything indented
past the column where the construct began.

```voltaic
{{#include ../../../examples/control.sdx}}
```

## Three rules, and that is all of them

**Indent with spaces.** A tab in indentation is `RDX0001`, reported with
*"indent with spaces"*. There is no configurable width and no mixed mode,
because the alternative is a language where two files that look identical are
not.

**A dedent must land on a column that is already open.** Returning to a level
that was never established is `RDX0002` — *"match an enclosing indentation
level"*. This is the error that catches a half-finished refactor, where a line
ends up between two levels and would otherwise silently attach to the wrong
block.

**Blank lines and comments do not close a block.** They carry no indentation
of their own, so a comment between two statements at the same level is just a
comment, not a boundary.

## Why not braces

Braces and indentation both encode the same tree, and when they disagree the
compiler believes the braces while the reader believes the indentation. Every
language with both eventually grows a formatter whose job is to stop them
disagreeing. Voltaic removes the second encoding rather than adding the
formatter — though `voltaic fmt` exists, and what it canonicalises is layout
*within* a line, not which block a line belongs to.

## What this costs

One real thing: a deeply nested expression cannot be broken across lines as
freely as in Rust, because a continuation has to stay clear of the next
construct's column. In practice that pushes you toward the pipeline operator
and toward `where`, both of which flatten nesting rather than indenting it —
see [Expressions and operators](../language/expressions.md) and
[Functions](../language/functions.md).
