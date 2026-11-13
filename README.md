# Voltaic

**A Haskell-shaped surface over Rust's semantics.** Voltaic is a source-to-source compiler: `.sdx`
in, `.xrs` containing ordinary Rust out. You get equations, guards, pattern rows, type classes,
pipelines, sections and composition — and then rustc type-checks and borrow-checks the result,
because Voltaic has no checker of its own and must never grow one.

[![CI](https://github.com/Industrial/voltaic/actions/workflows/ci.yml/badge.svg)](https://github.com/Industrial/voltaic/actions/workflows/ci.yml)
[![CodeQL](https://github.com/Industrial/voltaic/actions/workflows/codeql.yml/badge.svg)](https://github.com/Industrial/voltaic/actions/workflows/codeql.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-A85C32.svg)](LICENSE)

There is no runtime, no garbage collector and no standard library. There is Rust's, because the
output *is* Rust — Voltaic's answer to "where is the `Vec`" is "crates.io".

---

## The language, in one file

This is `examples/guards.sdx`, unedited:

```voltaic
-- A ladder, including `otherwise` as the final catch-all.
tier : u64 -> &'static str
tier spend
  | spend > 10_000 = "platinum"
  | spend > 1_000 = "gold"
  | otherwise = "basic"

-- Several equations, each with a short body on the `=` line.
fib : u64 -> u64
fib 0 = 0
fib 1 = 1
fib n = fib (n - 1) + fib (n - 2)
```

and this is the `.xrs` it compiles to, also unedited:

```rust
// A ladder, including `otherwise` as the final catch-all.
fn tier(spend: u64) -> &'static str {
    if spend > 10_000 {
        "platinum"
    } else if spend > 1_000 {
        "gold"
    } else {
        "basic"
    }
}

// Several equations, each with a short body on the `=` line.
fn fib(__rdx_a0: u64) -> u64 {
    match __rdx_a0 {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}
```

Three things in that output are deliberate and load-bearing.

**The comments survived.** Comments are trivia, never discarded — the `.xrs` is a human-facing
artifact, not an intermediate you are supposed to ignore. You can read it, review it, and step
through it in a debugger.

**It is boring Rust.** No trait objects you did not ask for, no `Box`, no runtime shim. A guard
ladder is an `if`/`else` chain; pattern rows are a `match`. If you can read Rust, you can audit
what Voltaic did.

**Generated names are namespaced.** Anything Voltaic invents carries the `__rdx_` prefix, and your
identifiers may not use it — so a generated binder can never shadow one of yours.

---

## What the surface actually gives you

### Methods are functions, and the receiver is last

A `.name` is a method awaiting its receiver. That one decision is what makes pipelines and
partial application share a single spelling:

```voltaic
.trim raw                     -- raw.trim()
.insert k v m                 -- m.insert(k, v)
.trim _                       -- |x| x.trim()      — awaiting a receiver
.insert k _ m                 -- |v| m.insert(k, v) — awaiting the middle one
(.name)                       -- |x| x.name        — GHC's record dot
```

The `_` is an **application hole**: it marks the argument a closure is still
waiting for, in any position. That is what makes the third line mean what it
says — the fourth row is the reason it has to. A `.name` followed by `)` is the
*selector*, so `(.trim)` is `|x| x.trim`, a field access rustc answers with
*"attempted to take value of method"*. This table claimed `.trim` alone was
`|x| x.trim()` until plan 85 measured it; the hole is the spelling that is.

### Pipelines, composition and sections

```voltaic
-- `|>` is reverse application: the piped value becomes the last argument.
doubled numbers = numbers |> .iter |> .copied |> .filter (> 0) |> .map (2 *) |> .sum

-- `>>>` composes forwards, `<<<` backwards.
lengths xs = xs |> .iter |> .map (.trim >>> .len) |> .sum

-- Sections: an operator with one side supplied, either side.
--   (+ 1)    |x| x + 1
--   (2 *)    |x| 2 * x
--   (> 0)    |x| x.gt(&0)      — the method form, because an adaptor hands you a reference
--   (.name)  |x| x.name        — GHC's record dot
```

`(- 1)` is negation and `(* 2)` is a dereference — unary always wins, so those two want the left
section (`(1 -)`, `(2 *)`) instead.

### `$`, so the nesting goes away

```voltaic
.ok $ std::fs::read_to_string path        -- std::fs::read_to_string(path).ok()
Ok $ Some $ Span { lo = start, hi = end } -- Ok(Some(Span { lo: start, hi: end }))
```

`$` lowers to *nothing at all* — it is grouping that runs to the end of the expression, and it
leaves no trace in the output.

### Data, classes and instances

```voltaic
data Event
  = Click
  | Close
  deriving (Debug)

newtype Meters = Meters f64        -- single-field, so `#[repr(transparent)]`

class Debug => Tally
  current : &self -> u32
  bump    : &mut self -> ()
  start   : u32 -> Self

instance Tally Counter
  current : &self -> u32
  current self = self.value
  bump : &mut self -> ()
  bump self =
    self.value += 1
    ()
  start : u32 -> Self
  start value = Counter { value }
```

`&self`, `&mut self` and `self` in a signature mean exactly what they mean in Rust. Voltaic does
not hide the borrow — it just stops making you write `impl` blocks by hand.

### Off-side rule

Indentation is significant; there are no braces and no semicolons. Blocks are opened by layout,
which is why a guard ladder and a `where` group read as outlines rather than punctuation.

### Abstractions Rust does not have

Four constructs with no Rust counterpart, each lowering to ordinary Rust that rustc then checks
exactly as it would the written-out form.

```voltaic
pattern Origin = Point { x = 0, y = 0 }   -- a named pattern, with parameters
pattern Both a b = (Some a, Some b)

deriving template Forward                 -- a derive, with no proc-macro crate
  instance Display Target
    fmt : &self -> &mut Formatter<'_> -> Result
    fmt self f = Display::fmt (&self.Field0) f

infixl 6 <+> = merge                      -- a spelling and a fixity for an existing function

totals = do .and_then                     -- a block whose statements are bound by one method
  first <- head xs
  rest <- tail xs
  Some (first + rest)
```

A **pattern synonym** is substituted at every use site, so `case p of Origin -> …` compiles to
the written-out pattern and rustc's exhaustiveness check sees everything it would have. A
synonym's parameters are formal: every binder is written at the use site, which is what separates
it from a wildcard import of names.

A **deriving template** names an `instance` whose implementing type is the reserved name
`Target`; every declaration listing it in `deriving (…)` gets that `instance` with `Target`
replaced by its own type name, `Field0`… by its field names, and `Fields` / `FieldNames` by
*every* field as a `self` access and as a string literal — so one template visits declarations of
any width. One file, four lines, no `syn`, no `quote`, no second compilation unit. A name in a
`deriving` list that no template matches stays in `#[derive(…)]` untouched.

An **operator declaration** is fixity plus alias and emits no item: `a <+> b` becomes
`merge(a, b)`. The spellings are a closed set of twelve, `<` · one character · `>`; precedence is
0–9 and always binds looser than arithmetic, so `a <+> b + c` is `a <+> (b + c)`. A declaration
names a function that already exists, so go-to-definition has somewhere real to go.

A **bind block** reads `<-` as whatever the named method does, so one shape covers `Option`,
`Result`, iterators and anything else with a `and_then`-like method — the comprehension Haskell
spells `do`, without Voltaic having to know which monad it is.

```voltaic
over (.addr.zip.code) doubled target   -- target.addr.zip.code = doubled(target.addr.zip.code)
```

**`over` says a path once.** `:=` is the general assignment and reaches a name, a field, an index
and a deref target, so the update already exists — written twice:
`target.addr.zip.code := doubled target.addr.zip.code`. Two spellings of one path that must agree
is the shape of a typo that type-checks, and `… := doubled target.addr.zip.town` compiles whenever
both fields are the same type. The lowering is an assignment through the place, so nothing is moved
and nothing is cloned; the emitted Rust is what the `:=` form emits.

Every one of these lowers to ordinary Rust that rustc then checks exactly as it would the
written-out form — which is the only reason they are admissible. Voltaic has no type checker and
must never grow one.

### String interpolation

```voltaic
println! \-> "hello #{name}, you are #{age} years old"
```

`\->` is a nullary lambda — it is how a macro argument is passed unevaluated.

---

## It compiles itself

**All thirteen crates under `crates/` are written in Voltaic.** The compiler's lexer, parser,
emitter, formatter, diagnostics, driver, CLI, LSP server and MCP server are `.sdx` files;
27,600 lines of them. The Rust under each crate's `src/` is *generated*, `.gitignore`d, and not
in the index.

So a clean clone has no Rust in it at all. What bridges the gap is `seed/voltaic1.wasm` — a 906 KB
WebAssembly build of a previous Voltaic compiler, which `./bootstrap.sh` runs under `wasmtime` to
regenerate every crate. Three tests keep that honest:

| Test | What it proves |
|---|---|
| `blob.rs` | the committed blob is the one `seed/SEED.md` records (provenance) |
| `bootstrap.rs` | a clone *without* the generated Rust rebuilds it byte-identically |
| `fixpoint.rs` | `stage1 == stage2` — the compiler reproduces itself |

The seed is a build artifact rather than a release, so version skew is closed by two commands
instead of rustc's `cfg(bootstrap)` or Go's version chain. `seed/SEED.md` has the whole story.

---

## Quick start

```bash
git clone --recurse-submodules https://github.com/Industrial/voltaic
cd voltaic
./bootstrap.sh                 # generate every crate's Rust from the seed
cargo build
```

Then:

```bash
cargo run -p voltaic-cli -- build examples/hello.sdx   # -> examples/hello.xrs
cargo run -p voltaic-cli -- check examples/hello.sdx   # run rustc, report against the .sdx
cargo run -p voltaic-cli -- fmt   examples/hello.sdx   # the canonical formatter
```

Diagnostics are reported against the `.sdx` you wrote, with 1-based line and column in
characters — not against the generated Rust.

`examples/*.sdx` is the CI-checked source of truth for what works today: 37 programs, each one
compiled, run, and compared against its expected output.

---

## Layout

The workspace splits along pipeline stages.

| Crate | Role |
|---|---|
| `voltaic-lexer` | off-side-rule lexer; spans, trivia, whitespace adjacency |
| `voltaic-ast` | syntax tree types, no logic |
| `voltaic-parser` | hand-written recursive descent (Pratt loop for expressions) |
| `voltaic-emit` | the Rust source emitter |
| `voltaic-fmt` | the canonical formatter |
| `voltaic-diag` | diagnostics and source maps |
| `voltaic-driver` | the incremental driver, cache and lints |
| `voltaic-cli` | the `voltaic` binary, plus the end-to-end and golden harnesses |
| `voltaic-lsp` | language server |
| `voltaic-mcp` | MCP server (`voltaic_build`, `voltaic_check`, `voltaic_fmt`, `voltaic_source_map`) |
| `voltaic-bench`, `voltaic-fuzz`, `voltaic-seed` | throughput, fuzzing, and the wasm seed |

Within a crate, **one cohesive group per file**, `src/` is a shallow module tree, and no
file carries a comment — all three machine-checked.
`docs/06 - Voltaic Code Architecture.md` has the rules; [AGENTS.md](AGENTS.md) has the
conventions that are load-bearing rather than stylistic.

---

## Development

Everything runs through [moon](https://moonrepo.dev), inside a [devenv](https://devenv.sh) shell:

```bash
devenv shell
moon run :check          # cargo check, per crate, cached
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # 884 tests across 13 crates
moon run :build :audit :coverage :docs
moon run :format         # treefmt: Rust, Nix, shell, JS/TS, YAML, TOML
```

The Rust targets are **per-crate and cached**, defined once in `.moon/tasks/tag-crate.yml`. A
one-line edit re-checks one crate, not thirteen.

**Edit the `.sdx`, never the `.rs`.** After a change, regenerate with:

```bash
VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test port
```

which compiles the generated tree before writing it and refuses a generation that does not
compile — so a bad change leaves `src/` untouched rather than broken.

Three blessable corpora, each asserting a different thing about a program:

```bash
VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test examples  # tests/golden/*.xrs — the text
VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test ui        # tests/ui/*.stderr — the diagnostic
VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test run       # *.stdout — what it prints
```

Blessing rewrites the expectations and then fails the run, so a blessed diff is always reviewed
rather than absorbed into a green build.

---

## Documentation

`docs/` is numbered: `01` specification, `02` blueprint, `03` surface design, `04` plan of plans,
`05 - NN -` one per plan, `06` code architecture, plus `COMPILER.md` for toolchain decisions and
`CONFORMANCE.md` for what the surface covers today.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Commits are signed off
([DCO](https://developercertificate.org/)) and follow conventional commits. Security issues go
through [SECURITY.md](SECURITY.md), not public issues.

## License

[MIT](LICENSE).
