# Patterns

Rust's patterns, plus the forms that let a pattern be used where Rust needs a
match arm.

```voltaic
{{#include ../../../examples/case.sdx}}
```

## `switch`

The multi-arm form. Arms are indented under the scrutinee, and the off-side
rule ends the construct — there is no closing brace and no comma between arms.

## Relational and logical patterns

A pattern may be a comparison (`> 0`) or a combination (`> 0 and < 10`), which
removes the most common reason to write a guard. Guards remain for the cases
that need a whole expression:

```voltaic
{{#include ../../../examples/guards.sdx}}
```

## Record patterns and destructuring

```voltaic
{{#include ../../../examples/destructuring.sdx}}
```

Field punning works: naming a field binds it to a variable of the same name.
The `...` rest pattern is **declined** — see
[Declined and unreachable](../reference/declined.md) — as are `ref` and
`ref mut`, and `|` or-patterns.

## Exhaustiveness is rustc's

Voltaic does not check that a `switch` covers its scrutinee, and will not grow
a checker that does. Rust already errors on a non-exhaustive match, against
your `.sdx` through the diagnostic remap, so a second checker could only
disagree with the first.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 6, *Patterns*.
