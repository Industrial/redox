# Strings

```voltaic
{{#include ../../../examples/strings.sdx}}
```

## Interpolation

`#{expr}` interpolates. A format specification follows the expression, so
`#{x:>8.2}` is width, alignment and precision in Rust's own mini-language
rather than a second one invented here.

```voltaic
{{#include ../../../examples/template.sdx}}
```

## Literal kinds

Plain, raw, and byte literals, matching Rust's set. A raw literal keeps its
backslashes: `r#"back\slash"#` is eight characters and a backslash is one of
them.

## Fences are verbatim

The triple-quoted form used by the inline-Rust escape hatch emits its body
**undecoded** — a `\n` inside it stays two characters rather than becoming a
newline. That is what makes it usable for embedding Rust that itself contains
string literals.

## No sigils beyond `%w`

Deliberate. A surface that grows a sigil per string flavour becomes
unreadable, and every flavour Rust has is reachable by a literal kind instead.

---

The full statement, with every decision and its reasoning, is `docs/03 - Voltaic Surface Design.md`
section 11, *Strings*.
