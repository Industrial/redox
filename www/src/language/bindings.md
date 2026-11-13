# Bindings and control flow

```voltaic
{{#include ../../../examples/bindings.sdx}}
```

## Binding, and rebinding

`x = e` binds. `x := e` reassigns, and requires `mut` on the original binding.
Shadowing a name with `=` is a warning (`RDX1003`), suppressed with
`#[voltaic::allow(shadow)]` on the line above — the attribute is consumed by
the compiler and never reaches rustc.

## `guard`

The early-return form. A `guard` whose condition fails runs its `else` branch,
which must diverge. It is the construct that keeps the happy path at one
indentation level instead of nesting every precondition.

## `cond`

A multi-branch conditional with no scrutinee — the form Rust writes as
`if … else if … else`, flattened so each branch sits at one level.

## `with` — receiver-implicit mutation

Inside a `with` block the receiver is implicit, so a run of mutations on one
value reads as a list of operations rather than as the same name repeated.

## `use` — callback flattening

`use` turns a callback-taking call into a straight-line binding, which is the
difference between a pyramid of closures and a sequence of steps. It is the
same shape as a `do` block, and it is why Voltaic needs no async runtime of its
own.

## `defer`

Runs its body when the enclosing scope ends. For the cases where Rust would
want a `Drop` guard and the value is not yours to implement one on.

## Blocks, labels and loops

```voltaic
{{#include ../../../examples/local.sdx}}
```

Block expressions have values. Labels name a loop or block so a `break` can
leave a specific one, carrying a value out if it has one.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 7, *Bindings and control flow*.
