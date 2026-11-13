# Voltaic

Voltaic is a source-to-source compiler. You write `.sdx`; it emits `.xrs`
containing **ordinary Rust**, which `rustc` then compiles with no runtime, no
shim and no library of its own.

That constraint is the whole design. Voltaic has no type checker, no borrow
checker and no exhaustiveness checker, because Rust already has all three and
having two of anything means having two that can disagree. What Voltaic changes
is the *surface*: the off-side rule instead of braces, parenless calls,
receiver-last methods, pipelines, comprehensions, and a `where` that holds
local definitions.

```voltaic
{{#include ../../examples/hello.sdx}}
```

Every sample in this book is included from `examples/` in the repository, and
CI compiles each one and compares its output against a recorded expectation. A
sample here cannot drift from what the compiler does, because there is no
second copy of it to drift.

## Where to start

Not used Voltaic before? Read the **Tour** in order — four short chapters from
nothing to calling into Rust. Looking something up? The **language** chapters
map one-to-one onto the sections of `docs/03 - Voltaic Surface Design.md`,
which stays the authority on every decision recorded here. **Reference** holds
the grammar, the diagnostic codes, and the list of things Voltaic deliberately
will not do.

Press <kbd>Ctrl</kbd>+<kbd>K</kbd> anywhere to jump to a chapter or heading.

## What Voltaic is not

It is not a new language with Rust as a backend. There is no Voltaic standard
library — crates.io is the standard library — and no plan implements a library
function. It is not a dialect that hides Rust: types, lifetimes and traits are
Rust's, spelled differently, and `rustc` is the final word on all of them.

Diagnostics are reported against the `.sdx` you wrote, at the line and column
of your source rather than of the generated Rust.
