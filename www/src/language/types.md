# Types, generics, async and traits

All four are Rust's, spelled for the off-side rule. Voltaic adds no type
system of its own and infers nothing Rust would not.

```voltaic
{{#include ../../../examples/generic.sdx}}
```

## Type expressions

Rust's type grammar: paths, references, generics, tuples, slices, `impl Trait`,
`dyn Trait`. Written in signatures after `:` and between `->`s.

```voltaic
{{#include ../../../examples/ascription.sdx}}
```

Ascription pins a type at an expression. The parenthesis in
`(Box::new h) : T` is load-bearing in the *source* — it ascribes the call
rather than `h` — and is not needed in the output.

## `where`, in its other role

In a signature, `where` carries Rust's predicates verbatim:
`where I::IntoIter: Clone`. Position is what distinguishes this from the
local-definition `where` of [Functions](functions.md), and the two cannot
appear in the same place.

## Classes and instances

```voltaic
{{#include ../../../examples/class.sdx}}
```

`class` is `trait` and `instance` is `impl`. The one rule that is not Rust's:
**an `instance` must be in the same file as its `class`.** The parser records a
class's member signatures when it parses the declaration and reads them back
when it parses an instance member, and each file gets a fresh parser. An
instance elsewhere fails with `RDX0101` on every name in the line, which looks
nothing like the real cause.

It is a constraint rather than a defect: the parser needs a member's *shape*,
not its types, so it does not cross the no-type-checker line.

## Async

```voltaic
{{#include ../../../examples/async.sdx}}
```

`async` and `await` as Rust's, with no executor supplied. Pick one from
crates.io — the same answer Voltaic gives for every library question.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 14, *Types, generics, async and traits*.
