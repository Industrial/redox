# Diagnostics

Every diagnostic is reported against the `.sdx` you wrote, at the line and
column of your source.

```voltaic
{{#include ../../../examples/hole.sdx}}
```

## Codes

Codes are `RDX` followed by four digits, in bands: `0001`–`0013` lexical,
`0039`–`0063` structural, `0099`–`0127` parse, `1000`–`1004` lints. The full
list is in [Diagnostic codes](../reference/codes.md).

> The `RDX` prefix is a leftover from the project's former name and has not yet
> been renamed to `VLT`. It is a queued change: every code appears in 73 test
> expectation files, and renaming them needs its own build-regenerate-reseed
> cycle. Until then, the codes a program emits read `RDX`.

## Remapping

The compiler emits Rust, `rustc` judges it, and the resulting diagnostics are
**mapped back onto the `.sdx`** through a source map written beside the
generated file. So a borrow error, a type error or a missing-match-arm error
arrives pointing at your source, not at a line of generated Rust you never
wrote.

That is the mechanism that lets Voltaic have no checkers of its own without
making the error experience worse than Rust's.

## Reading a cascade

Item-boundary recovery means one real error often produces a run of
consequential ones. **Read the first diagnostic, not the longest run of them** —
a `module test` mistake reports its true cause once and then a dozen
`RDX0101`s on the following lines.

## Holes

A hole is a deliberate gap — a placeholder the compiler reports rather than
guesses at, so an unfinished program says where it is unfinished instead of
failing somewhere else.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 17, *Diagnostics*.
