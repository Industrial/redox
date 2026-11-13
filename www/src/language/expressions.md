# Expressions and operators

The chapter with the most surface in it, and one idea underneath: application
should be cheap to write and cheap to read, so parentheses are for grouping
rather than for calling.

```voltaic
{{#include ../../../examples/expressions.sdx}}
```

## Parenless calls are canonical everywhere

`f x y` is `f(x, y)`. Not an alternative spelling — the canonical one, which
`voltaic fmt` will produce. Parentheses group; they do not call.

The consequence worth internalising: **an argument extends as far as it can**.
`escaped ().len ()` reads as `escaped(().len())`, not as
`escaped().len()`. Receiver-last method calls exist partly to remove that
ambiguity, because there is then no stray `()` for the parser to attach
somewhere surprising.

## Zero-argument calls

A nullary function is declared `name : ReturnType` and called as `name`. A
signature of `: () -> T` declares a function taking *unit*, which is a
different thing — and is the mistake to expect if you come from Rust.

## The pipeline operator

```voltaic
{{#include ../../../examples/compose.sdx}}
```

`x |> f` passes `x` as `f`'s **receiver**, not as its first argument. That is
what makes a pipeline of method calls read left to right without the receiver
jumping position at every step.

## Operator sections

```voltaic
{{#include ../../../examples/section.sdx}}
```

`(+ 1)` is a closure adding one. `(.f)` is a **field** section — it emits
`|x| x.f`, with no call — which is the one trap here: `.f x` is the method call
`x.f()`, so `(.f)` and `.f` are different constructs and eta-reducing a method
call to a field section is a type error, not a simplification.

## `is`, and chained comparison

`x is Pattern` tests a pattern and yields a bool, which is what lets a pattern
appear in a condition without a `switch`. And `a < b < c` means what it reads
as, rather than comparing a bool against `c`.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 5, *Expressions and operators*.
