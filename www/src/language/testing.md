# Testing

```voltaic
{{#include ../../../examples/testing.sdx}}
```

## `test` is a declaration

`test "name"` declares a test, and emits a `#[test] fn` into a generated
`mod __rdx_tests` in the same file. The name is a string, so it can be a
sentence rather than a mangled identifier.

## What goes in the generated module

Only tests. Voltaic cannot place anything else inside `mod __rdx_tests`, so a
helper that several tests share has to sit at module scope under
`#[cfg(test)]`. That is a known rough edge rather than a design: it was
recorded as owed a surface decision of its own.

## Where tests belong

Two rules the compiler's own source follows, and that are worth borrowing:

**Public-API tests are integration tests**, in `tests/` beside `src/`, not
`#[cfg(test)]` modules. An inline test can reach private internals and
eventually will, at which point it stops testing the API and starts testing
the implementation.

**Inline tests are for private constructs** — the things an integration test
structurally *cannot* reach. If a private constant has an invariant, its test
has to be inline, and that is the one case where inline is correct.

**Fixtures are files.** Never embed program source as a string literal in a
test. The cost of the alternative is paid once per syntax change, forever, and
is measured: a predecessor project missed 113 fixtures living as string
literals inside test modules during a single syntax cutover.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 15, *Testing*.
