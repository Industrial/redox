# Your first program

```voltaic
{{#include ../../../examples/hello.sdx}}
```

Four things are happening, and three of them are the surface decisions that
define the language.

**`main : ()`** is a type signature, written separately from the definition.
The `:` reads "has type", and `()` is Rust's unit, so this function returns
nothing. Signatures are not decoration — the parser learns a function's shape
from its signature, which is how it knows the line below starts a definition.

**`main =`** begins that definition. Its body is everything indented beneath
it, which is the off-side rule; the next chapter is about that alone.

**`println! \-> "hello"`** is a macro call. `println!` is Rust's own macro,
unchanged, and `\->` introduces its arguments. Voltaic writes macro arguments
after an arrow rather than inside parentheses because a macro takes a token
stream rather than an expression list, and the arrow is where that change of
grammar happens.

**`()`** on its own line is the result. A body is a sequence of statements
whose final expression is the value, so a function returning unit ends with
unit written out. That is not noise: it is the difference between a body whose
tail is the `println!` and one whose tail is `()`.

## Build it

```bash
voltaic build hello.sdx     # emit hello.xrs - ordinary, readable Rust
voltaic check hello.sdx     # run rustc, report against hello.sdx
voltaic run hello.sdx       # build, compile and execute
```

`check` is the one to reach for while writing. It compiles the generated Rust
and then **remaps every diagnostic back onto your `.sdx`**, so a borrow error
arrives pointing at the source you wrote rather than at generated code you
did not.

## The `.xrs` is meant to be read

`voltaic build` writes `hello.xrs`, and that file is an artifact you are
expected to open. Comments survive into it — they are trivia, and trivia is
never discarded — so the output reads as reviewable Rust rather than as a
compiler's scratch file. If you ever stop using Voltaic, the `.xrs` is your
program.
