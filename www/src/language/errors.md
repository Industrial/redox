# Errors

Rust's `Result` and `Option`, with the three operators that remove the
boilerplate around them.

```voltaic
{{#include ../../../examples/errors.sdx}}
```

## `?` propagates

```voltaic
{{#include ../../../examples/fallible.sdx}}
```

As Rust's. One trap, found the hard way: **`?` binds inside a parenthesised
argument**, so where you put the parentheses changes what is propagated.

## `??` and `??=`

```voltaic
{{#include ../../../examples/coalesce.sdx}}
```

`a ?? b` is `a` if it is present and `b` otherwise. `??=` assigns only if
absent. Note `Some *lint` multiplies — a prefix `*` after a constructor needs
parentheses, which is the kind of thing the formatter will fix for you once and
you will then never think about again.

## `catch`

Handles an error in place, as an expression, rather than unwinding a call
stack. Voltaic has no exceptions; `catch` is sugar over `Result`.

## `try` blocks

Scope a run of `?` operations so that the first failure leaves the block rather
than the whole function.

## Typed domain errors

```voltaic
{{#include ../../../examples/bind.sdx}}
```

An `errors` declaration defines a domain error type with its variants and
their messages, which is the shape that would otherwise be a `data`
declaration plus a `Display` implementation plus a `From` chain.

## `unwrap` is discouraged, not forbidden

`.unwrap` raises `RDX1001`. If the panic is intended, say so on the line above
with `#[voltaic::allow(unwrap)]` — an attribute the compiler consumes, so it
never reaches rustc, which would reject it as an unknown lint.

## Optional chaining and the panic family are retired

Both existed and both are gone. The reasoning is in
[Declined and unreachable](../reference/declined.md).

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 12, *Errors*.
