# Data declarations

```voltaic
{{#include ../../../examples/data.sdx}}
```

## One keyword for both

`data` declares a struct or an enum — which one depends on whether it has
variants. Every `data` declaration emits `use <Name>::*` so that its
constructors are in scope unqualified, and that single fact has a consequence
worth knowing before you organise files: **two enums sharing any variant name
cannot share a module.** `TokenKind::Or` and `Keyword::Or` in one file is
`E0659`, and a pattern match on the bare name then binds instead of matching.

## Records

```voltaic
{{#include ../../../examples/record.sdx}}
```

Field punning in initialisation as well as in patterns: a field whose value is
a variable of the same name needs naming only once.

## Updating through a path

`over` updates a nested field and returns the updated value, which is the
construct that replaces a chain of intermediate bindings. It is also the only
place the grammar reaches a method reference.

```voltaic
{{#include ../../../examples/optic.sdx}}
```

## `derive`

```voltaic
{{#include ../../../examples/synonym.sdx}}
```

Rust's derives, by name. Voltaic adds no derive macros of its own.

## Visibility

```voltaic
{{#include ../../../examples/visibility.sdx}}
```

`export` is `pub`; `export(crate)` is `pub(crate)`. Default is private, and
private means *visible in the defining module and its descendants* — the same
rule as Rust's, which matters when flattening a module tree moves an item away
from the code that used it.

## Constants, aliases, unit and tuple structs

```voltaic
{{#include ../../../examples/constants.sdx}}
```

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 9, *Data declarations*.
