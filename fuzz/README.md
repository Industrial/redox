# Fuzz harnesses

Plan 42 (`docs/37 - 42 - Fuzzing.md`) leaf 2. Four libFuzzer targets, each three
lines around a function in `crates/redox-fuzz`.

## What this needs, and does not have here

```bash
cargo install cargo-fuzz      # not pinned in devenv.nix
rustup toolchain add nightly  # libFuzzer needs it
cargo fuzz run parse_program
```

§1.1 measured that **none of this is present on the machine the row was written
on** — no `cargo-fuzz`, no nightly, no AFL. That is why the target logic lives
in `crates/redox-fuzz` as stable Rust and is exercised by `moon run :test`
through a deterministic sweep: without that split the row would ship code
nobody could run, and a fuzz target that has never executed reads as coverage.

Plan 42 §6 declines to pin `cargo-fuzz` and a nightly toolchain, and says why:
it doubles the toolchain this repository installs for a tool that runs on
nobody's inner loop, and these harnesses work the moment someone adds it.

## Why this directory is outside the workspace

`libfuzzer-sys` does not build on stable, and a workspace member that does not
build breaks `cargo check` for everyone. The bare `[workspace]` table in
`Cargo.toml` is what detaches it, and `cargo-fuzz` requires that.

## The seed corpus

`examples/` is the seed corpus — thirty-odd files that already cover every
construct the language has:

```bash
cargo fuzz run parse_program ../examples
```

`lexical.sdx` is **included** deliberately, unlike in the compiler's own corpus
sweeps. It is exempt there because it does not parse, which makes it the most
valuable seed here rather than the one to skip — plan 43 criterion 3 and plan
41 decision 5 made the same call.

## What the targets assert

Three assert **totality**: lexing, parsing and emitting never panic. A product
property rather than hygiene — plan 40's language server runs the parser on
every keystroke, so a panic on malformed input crashes an editor in front of
someone who has merely not finished typing.

The fourth, `roundtrip`, carries the row's real oracle, and the check lives in
`crates/redox-fuzz/tests/sweep.rs` rather than here:

> If `parse_source` accepts the input, `emit`'s output **parses as Rust**.

Doc 04 calls that *"a stronger oracle than Emerald had"*, and it is: Emerald's
targets assert *does not panic*, which finds crashes and nothing else. Checking
the output is possible only because Redox is a source-to-source compiler —
there is an output to check.

## What the sweep already found

A **stack overflow**. The parser is recursive descent with no depth limit, so
deeply nested input aborted the process — not a panic that can be caught, an
abort, which takes the whole program with it.

Measured rather than guessed: 24 levels of nested parentheses parse and 32
abort, in a debug build. `redox_parser::MAX_NESTING` is now 16 and `RDX0109`
reports it.

Found by handing the parser three hundred nested parentheses, which nothing had
ever done before — and found on the first run of the sweep, before any fuzzer
was involved at all.
