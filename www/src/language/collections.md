# Collections

```voltaic
{{#include ../../../examples/arrays.sdx}}
```

## Comprehensions, at two levels

```voltaic
{{#include ../../../examples/comprehension.sdx}}
```

A comprehension builds a collection from a generator, optional filters, and an
expression. The two levels are the simple form and the nested form; the second
exists because a flat comprehension over a nested source reads as a bug.

## Broadcasting

An operation applied to a collection distributes over its elements, which
removes the most common one-line `map`. Where it is not what you want, the
explicit `.map` with a lambda is still the shorter thing to read — and under
`.map`, `.filter` and `.all` the explicit lambda is already minimal, because
`(.f)` is a field section rather than a method reference.

## Collect targets

A comprehension's result type is chosen by its target, so the same expression
builds a `Vec`, a `HashMap` or a `String` depending on what receives it. The
mechanism is Rust's `collect`, spelled so the target is visible at the point
the collection is built.

## Literals

Array, slice, tuple and map literals, plus `%w` for a word list — the one
sigil-shaped literal the surface budget paid for.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 10, *Collections*.
