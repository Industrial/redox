# Canonical forms

Voltaic has one spelling for each thing, and `voltaic fmt` produces it.

```voltaic
{{#include ../../../examples/draw_condition.sdx}}
```

## What canonical means here

Where the surface admits two spellings of the same construct, one is canonical
and the formatter rewrites the other into it. Parenless application is
canonical; so is receiver-last method calling; so is `and` over `&&` in Voltaic
source.

This is not a style preference with a tool attached. A language with two
spellings has two things to read, two things to search for, and two things for
a reviewer to argue about.

## The formatter is in the gate

`voltaic fmt` existed for a long time in no gate at all — `treefmt` has no
`.sdx` formatter and the only `fmt --check` among the build tasks was Rust's.
The result was 50 of 53 files in one crate non-canonical, and a formatter bug
that **deleted a member's visibility** surviving unnoticed.

It is now `moon run :sdx-format-check`, and it is the *check* rather than the
write, placed beside the lint. A gate that rewrites the source it is grading
cannot fail.

## A larger worked example

```voltaic
{{#include ../../../examples/phase1.sdx}}
```

That file is the repository's phase-one example and has a recorded `.stdout`
beside it, so what it prints is asserted rather than described.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 16, *Canonical forms*.
