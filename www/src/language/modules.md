# Modules and metaprogramming

```voltaic
{{#include ../../../examples/module.sdx}}
```

## `module` and `import`

`module name` declares; `import a::b` is Rust's `use`. Two constraints worth
knowing before you name things:

**A module may not be named after a keyword.** `class`, `data`, `test`, `type`
and `errors` are all keywords, so `module test` is `RDX0103` — and item
recovery then reports a dozen `RDX0101`s on the lines *after* it. Read the
first diagnostic, not the loudest run of them.

**Paths use `::` with no spaces around it.** A spaced separator is its own
diagnostic rather than a parse failure, because the intent is unambiguous and
saying so is more useful than failing.

## Attributes

```voltaic
{{#include ../../../examples/attributes.sdx}}
```

Rust's attributes, passed through. The exceptions are Voltaic's own
`#[voltaic::allow(...)]` lint suppressions, which the compiler consumes.

## Macros

```voltaic
{{#include ../../../examples/macros.sdx}}
```

Macro *invocation* uses `\->` to introduce a token stream. Macro *definition*
has no Voltaic surface and is reached through the inline-Rust fence.

## Module-scope bindings

A binding at module scope is an error (`RDX0019`-family), because Rust has no
such thing and silently promoting one to a `static` or a `const` would be
Voltaic deciding something that is not its to decide. Use `const`.

## Module organisation

One cohesive group per file: the constructs that belong to one concern and are
read together. `lib.rs` and `main.rs` hold only module declarations,
re-exports, and `main`.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 13, *Modules and metaprogramming*.
