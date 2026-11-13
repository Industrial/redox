# Rust reachability

What a Voltaic program can express, stated as a map rather than a promise.

```voltaic
{{#include ../../../examples/reachability.sdx}}
```

That file is the map, it is in CI, and it is the honest answer.

## Eight constructs have no surface

Macro definitions, macro invocation with a non-expression argument, `as` casts,
`extern "C" fn` export, `ref` and `ref mut`, `unsafe impl`, `unsafe trait`, and
`|` or-patterns.

Each has a fixture under `tests/unreachable/` that the gate asserts **still
fails**. So closing a gap breaks the build until both the grammar document and
this list are updated — the one mechanism that stops a reachability table
drifting into fiction.

Every one of them is reachable through the inline-Rust fence in the meantime;
see [Calling Rust](../tour/calling-rust.md).

## The grading

`docs/GRAMMAR.md` grades every Rust construct and is the authority. Its
`UNREACHABLE` rows are where the eight above come from — this chapter does not
list them a second time, so the two cannot disagree.

## Minimum Rust version

Voltaic emits what the pinned toolchain accepts. The generated code is ordinary
Rust of that edition, with no feature gates and no nightly requirement — a
deliberate constraint, because a source-to-source compiler that needs nightly
makes its output less portable than its input.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 18, *Rust reachability*.
