# Functions

```voltaic
{{#include ../../../examples/application.sdx}}
```

## Signature, then definition

`name : A -> B` then `name a =`. The signature is separate and is not optional
for a top-level item: the parser reads a function's shape out of it, which is
also why **an `instance` must sit in the same file as its `class`** — the
parser learns a member's signature from the class declaration, and each file
gets a fresh parser.

## Multi-clause definitions

A function may be defined by several clauses whose patterns select between
them, which is the form that makes a recursive definition read as its own base
case plus its step.

## `where` — local definitions

```voltaic
{{#include ../../../examples/wheres.sdx}}
```

`where` after a body introduces definitions local to that body. This is the
construct that flattens nesting: a helper used twice goes below the thing that
uses it, at one level of indentation, rather than above it at the top level or
inside it as a closure.

`where` has a **second, unrelated role** in a signature, where it carries
Rust's own predicates verbatim. The two are told apart by position, which is
the kind of overload the surface design admits only when the positions cannot
collide.

## Receiver-last methods, and currying

```voltaic
{{#include ../../../examples/receiver.sdx}}
```

```voltaic
{{#include ../../../examples/currying.sdx}}
```

`.len x` is `x.len()`. Partial application works as the signature's arrows
suggest.

## Closures

`\x -> e` is a closure. A parameter is a **bare name** — not a pattern and not
an annotated binding — which is worth knowing because the Rust you are
translating from often annotates its closure parameters for inference.

## Default arguments remain declined

Labelled arguments exist; defaults do not. The reasoning is in
[Declined and unreachable](../reference/declined.md).

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 8, *Functions*.
