# Redox's grammar, as a graded inventory

Doc 02 §4.1 asks for this document and says what it is for:

> **any Rust construct marked `UNREACHABLE` is a construct Redox users cannot write**, which
> directly violates §26's interoperability charter.

So this is **risk detection**, not documentation. Plan 46
(`docs/40 - 46 - Grammar Inventories.md`) built it; `docs/03 - Redox Surface Design.md` §18 is
a summary of one axis rather than a substitute for this one.

**Every row carries evidence, and `crates/redox-cli/tests/grammar_inventory.rs` resolves it.**
That is not ceremony. Plan 46 §1.3 measured what an unchecked inventory costs: of sixteen
apparent gaps in a first careless pass, **eleven were the probe's own bug** — and a false
`UNREACHABLE` is worse than a missing row, because it sends a future plan to design a surface
for something that already works.

| Grade | Means | Evidence it carries |
|---|---|---|
| `PASSTHROUGH` | Redox writes the Rust construct essentially unchanged | a corpus file that uses it |
| `SUGARED` | reachable, through a different spelling | a corpus file, and the Rust it lowers to |
| `UNREACHABLE` | a Redox program cannot express it | a fixture under `tests/unreachable/` that **still fails**, and the nearest thing that does work |

An `UNREACHABLE` fixture failing is asserted, so when a later row closes a gap the fixture
starts compiling and the test breaks. It fails when the compiler gets *better*, deliberately
— the alternative is a document still claiming a gap years after it closed.

---

## 1. Rust's grammar

Organised by the Rust reference's areas rather than by Redox's constructs, so coverage can be
checked against the host language instead of against this project's history.

### 1.1 Lexical structure

| Area | Grade | Evidence and note |
|---|---|---|
| Integer, float, bool literals | `PASSTHROUGH` | `examples/expressions.sdx` — written through unchanged, suffixes included |
| String literals, escapes | `PASSTHROUGH` | `examples/strings.sdx` |
| String interpolation | `SUGARED` | `examples/strings.sdx` — `#{x}` becomes `format!`'s `{}` with the expression as an argument |
| Raw strings `r"…"`, `r#"…"#` | `PASSTHROUGH` | `examples/lexical.sdx` |
| Byte and byte-string literals | `PASSTHROUGH` | `examples/lexical.sdx` |
| C-string literals `c"…"` | `PASSTHROUGH` | `examples/lexical.sdx` |
| Char literals | `PASSTHROUGH` | `examples/lexical.sdx` |
| Multi-line strings `"""…"""` | `SUGARED` | `examples/strings.sdx` — no Rust equivalent; dedented per doc 01 §7.2 and emitted as one literal |
| Identifiers | `PASSTHROUGH` | `examples/hello.sdx` |
| Raw identifiers `r#type` | `PASSTHROUGH` | `examples/reachability.sdx` — plan 45 leaf 4; the `r#` spelling was already taken by raw strings |
| Line and block comments | `SUGARED` | `examples/comments.sdx` — `--` and `{- -}`, emitted as `//` and `/* */`; comments are trivia and reach the `.xrs` (doc 02 §10.1) |
| Doc comments | `SUGARED` | `examples/comments.sdx` — `---` becomes `///` |

### 1.2 Items

| Area | Grade | Evidence and note |
|---|---|---|
| Functions | `SUGARED` | `examples/hello.sdx` — a signature line and one or more equations, not `fn name(a: T)` |
| `const fn` | `SUGARED` | `examples/constants.sdx` — marker inside the `::` run |
| `async fn` | `SUGARED` | `examples/async.sdx` — likewise; plan 39 |
| `unsafe fn` | `SUGARED` | `examples/reachability.sdx` — likewise; plan 45 leaf 2 |
| `extern "C" fn` (export) | `UNREACHABLE` | `tests/unreachable/extern_fn_abi.sdx` — plan 45 shipped the *import* direction only. **No alternative** in the surface; inline Rust reaches it |
| Structs (braced) | `SUGARED` | `examples/data.sdx` — one constructor named for the type is a `struct` (doc 18 §2.4) |
| Tuple structs | `SUGARED` | `examples/data.sdx` — `newtype`, and a constructor with positional fields |
| Unit structs | `SUGARED` | `examples/data.sdx` — `data Marker = Marker` |
| Enums, variants, discriminants | `SUGARED` | `examples/data.sdx` — anything other than one self-named constructor is an `enum` |
| Unions | `SUGARED` | `examples/reachability.sdx` — `union`, plan 45 leaf 4 |
| Type aliases | `PASSTHROUGH` | `examples/generic.sdx` — `type` and the type verbatim |
| `const` and `static` items | `SUGARED` | `examples/constants.sdx` |
| Traits | `SUGARED` | `examples/class.sdx` — `class`, with default bodies, associated types and consts, supertraits |
| `unsafe trait` | `UNREACHABLE` | `tests/unreachable/unsafe_class.sdx` — plan 45 §6 asked for this check and plan 45 §10 did not record it. **No alternative**: an invariant rustc cannot check has no safe spelling |
| Impls, inherent and trait | `SUGARED` | `examples/class.sdx` — `instance` |
| `unsafe impl` | `UNREACHABLE` | `tests/unreachable/unsafe_class.sdx` — same row, same fixture |
| Modules, inline and nested | `SUGARED` | `examples/module.sdx` — `module` |
| `use` declarations, groups, globs, aliases | `SUGARED` | `examples/module.sdx` — `import` |
| `extern` blocks | `SUGARED` | `examples/reachability.sdx` — plan 45 leaf 3; `unsafe extern` is emitted because edition 2024 requires it |
| Macro definitions | `UNREACHABLE` | `tests/unreachable/macro_def.sdx` — **declined** under doc 03 §23 item 3. Alternative: inline Rust |
| Attributes, inner and outer | `PASSTHROUGH` | `examples/attributes.sdx` — Rust's own `#[…]`, plan 36. **One exception, and only one:** plan 53's `#[redox::allow(unwrap)]` is *consumed* by the emitter and never reaches the `.xrs`, because an un-consumed `redox::` attribute is rustc's `E0433` — a hard error, not an unknown lint |

### 1.3 Types

| Area | Grade | Evidence and note |
|---|---|---|
| Paths and generic arguments | `PASSTHROUGH` | `examples/generic.sdx` — a type is verbatim text (doc 03 §14.1) |
| References, `&mut` | `PASSTHROUGH` | `examples/class.sdx` |
| Raw pointers | `PASSTHROUGH` | `examples/reachability.sdx` — verbatim, as any type is |
| Tuples | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Arrays and slices | `PASSTHROUGH` | `examples/arrays.sdx` |
| Trait objects `dyn Trait` | `PASSTHROUGH` | `examples/generic.sdx` |
| `impl Trait`, argument and return | `PASSTHROUGH` | `examples/generic.sdx` — plan 25 |
| Function pointers and `Fn` bounds | `PASSTHROUGH` | `examples/generic.sdx` |
| Lifetimes | `PASSTHROUGH` | `examples/strings.sdx` — `&'static str`; carried verbatim, so there is no separate construct |
| Generic parameters and bounds | `SUGARED` | `examples/generic.sdx` — a **bound clause** read off `=>` (doc 18 §2.2), not `where` |
| `where` clauses | `SUGARED` | `examples/generic.sdx` — the same bound clause. Redox's own `where` introduces **local bindings** and is a different construct |
| Associated types and constants | `SUGARED` | `examples/class.sdx` |

### 1.4 Expressions

| Area | Grade | Evidence and note |
|---|---|---|
| Operators and precedence | `PASSTHROUGH` | `examples/expressions.sdx` |
| Method calls, field access | `SUGARED` | `examples/section.sdx` — adjacency decides a call from a section |
| Function calls | `SUGARED` | `examples/hello.sdx` — parenless application, Haskell's rule (doc 18 §2.1) |
| Indexing | `PASSTHROUGH` | `examples/arrays.sdx` |
| Tuple indexing `.0` | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Ranges | `PASSTHROUGH` | `examples/control.sdx` |
| Struct and record literals | `SUGARED` | `examples/record.sdx` — braces with `=` for values |
| Array literals and repeats | `SUGARED` | `examples/arrays.sdx` — `[v; n]`, and `:: Vec<T>` for the growable form (plan 29) |
| Closures | `SUGARED` | `examples/expressions.sdx` — `\x -> e` |
| `move` closures | `SUGARED` | `examples/async.sdx` |
| Blocks as expressions | `SUGARED` | `examples/control.sdx` — `do`, with labels |
| `async` blocks | `SUGARED` | `examples/async.sdx` — plan 39 |
| `unsafe` blocks | `SUGARED` | `examples/reachability.sdx` — plan 45 leaf 2 |
| `try` blocks | `SUGARED` | `examples/fallible.sdx` — an immediately-invoked closure, because Rust has try blocks only unstably |
| `?` operator | `PASSTHROUGH` | `examples/fallible.sdx` |
| `.await` | `SUGARED` | `examples/async.sdx` — the draw arrow `<-` (doc 30) |
| `as` casts | `UNREACHABLE` | `tests/unreachable/as_cast.sdx` — `as` is reserved for `import … as`. Nearest: `u8::try_from n`, which is **not** a cast — `as` truncates, `try_from` is fallible |
| Macro invocations | `UNREACHABLE` | `tests/unreachable/macro_call.sdx` — doc 18 §7 retired `!`. Nearest: inline Rust |
| Inline Rust | `SUGARED` | `examples/reachability.sdx` — doc 01 §23.1, plan 45 leaf 1. No Rust equivalent: it *is* Rust |

### 1.5 Patterns

| Area | Grade | Evidence and note |
|---|---|---|
| Literal and wildcard patterns | `PASSTHROUGH` | `examples/case.sdx` |
| Identifier and binding patterns | `SUGARED` | `examples/destructuring.sdx` |
| `@` bindings | `PASSTHROUGH` | `examples/case.sdx` |
| Struct and record patterns | `SUGARED` | `examples/destructuring.sdx` |
| Tuple patterns | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Slice patterns, with `..` | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Range patterns | `PASSTHROUGH` | `examples/case.sdx` |
| `ref` and `ref mut` | `UNREACHABLE` | `tests/unreachable/ref_pattern.sdx` — regraded 2026-11-10. `ref` is **not reserved**, so `ref a` lexes as two identifiers and the pattern parser answers `RDX0101`; the old note claimed `PASSTHROUGH` against `examples/destructuring.sdx`, which contains no `ref`. Nearest: **match ergonomics**, which is what that example actually uses — `case &pair of (a, b) ->` binds by reference with no keyword. What is lost is the case where ergonomics do not apply and a binder must be explicitly `ref mut` |
| `\|` or-patterns | `UNREACHABLE` | `tests/unreachable/or_pattern.sdx` — doc 18 made `\|` a **guard**, so `1 \| 2 ->` parses and means `1` guarded by `2`. Nearest: **separate arms**, which match the same values; what is lost is one arm body shared across patterns |

### 1.6 Statements and control flow

| Area | Grade | Evidence and note |
|---|---|---|
| `let` bindings, `let mut` | `SUGARED` | `examples/expressions.sdx` — `x = e` and `mut x = e` |
| Assignment | `SUGARED` | `examples/control.sdx` — `x := e`, because `=` is a binding (plan 54). Covers a name, a field, an index, a deref target and Rust's destructuring assignment, the target being a general expression |
| Compound assignment | `SUGARED` | `examples/control.sdx` — all five of `+=`, `-=`, `*=`, `/=`, `%=`. Shipped in plan 20 and graded here for the first time |
| Shadowing | `PASSTHROUGH` | `examples/control.sdx` — an inner `n = 1` inside a `do` block over an outer `n`, which is the contrast that makes `:=` necessary. Cited `examples/expressions.sdx` until plan 54; that file holds exactly one binding, and nothing in the corpus shadowed anything |
| `if` / `else` | `SUGARED` | `examples/control.sdx` — an expression, with layout |
| `if let` | `SUGARED` | `examples/control.sdx` — `if P = e` |
| `let` chains | `SUGARED` | `examples/draw_condition.sdx` — the draw arrow in a condition (doc 30) |
| `while`, `while let` | `SUGARED` | `examples/control.sdx` |
| `loop` | `SUGARED` | `examples/control.sdx` |
| `for` | `SUGARED` | `examples/control.sdx` — `for x in xs` |
| `break`, `continue`, with value | `PASSTHROUGH` | `examples/control.sdx` |
| Labelled loops and blocks | `SUGARED` | `examples/control.sdx` — `do 'label`, `break 'label v` |
| `match` | `SUGARED` | `examples/case.sdx` — `case … of`, with guards |
| Guards | `SUGARED` | `examples/guards.sdx` — `\|` in an arm, which is what shadows Rust's or-pattern |
| `return` | `PASSTHROUGH` | `examples/control.sdx` |

### 1.7 Beyond the grammar

Two areas that are not productions but are what a user actually hits.

| Area | Grade | Evidence and note |
|---|---|---|
| `std` and crate paths | `PASSTHROUGH` | `examples/fallible.sdx` — any path works, which is what makes the macro gap survivable |
| Error types and conversions | `SUGARED` | `examples/errors.sdx` — `errors`, the one declaration that emits more than one Rust item |
| Tests | `SUGARED` | `examples/testing.sdx` — `test "name"`, gathered into one `#[cfg(test)]` module per file |

---

## 2. MoonScript's grammar

Doc 02 §4.1 asked for this axis graded `KEEP` / `MODIFY` / `REMOVE` against Emerald's
761-line Ruby inventory. **Doc 18 then replaced the surface**, and doc 18 §10 records this
axis as *"mostly moot"*.

Grading a discarded grammar production by production would be archaeology, so what follows is
the question a reader actually has — what survived — with doc 18 §§2.1–2.7 cited for the
rest. Plan 46 §6 records the decline.

| MoonScript trait | Fate | Replaced by |
|---|---|---|
| Significant indentation | **KEEP** | unchanged: the off-side rule is the one thing both source languages agree on |
| `->` and `=>` for functions | **REMOVE** | equations and a signature line (doc 18 §2.1) |
| `if`/`unless`/`while` as expressions | **MODIFY** | kept as expressions, Haskell's layout (doc 18 §2.3) |
| Implicit return of the last expression | **KEEP** | unchanged, and it is Rust's rule too |
| Comprehensions | **KEEP** | `examples/comprehension.sdx` |
| `class` with `@` fields and `self` | **REMOVE** | `data` plus `class`/`instance`, which are Haskell's trait and impl rather than an object (doc 18 §2.5) |
| String interpolation `#{…}` | **KEEP** | `examples/strings.sdx` |
| Table literals | **REMOVE** | records and `data`; Redox has no table type |
| `switch`/`when` | **REMOVE** | `case … of` (doc 18 §2.3) |
| `with` blocks | **REMOVE** | doc 18 §7 deleted them; the keyword was reused for the callback bind (doc 30 §3.3) |
| `do` blocks | **MODIFY** | a labelled block expression (doc 03 §7.2) |

---

## 3. What this document is not

- **A list of declines.** `UNREACHABLE` means *cannot be written*; a decline means
  *deliberately not offered*. Doc 03 §22's declines are plan 47's `docs/DECLINED.md`, and
  conflating the two would make both useless. The one row that is both — macro definition —
  says so.
- **A claim that every citation exercises its construct.** The test asserts the cited file
  exists. Asserting it *contains* the construct needs a parse and a query per row, which is
  plan 41's grammar doing this document's job; the `UNREACHABLE` fixtures carry the stronger
  property instead. Plan 46 §6 records it.
- **A library inventory.** This is the grammar. `std` reachability is a different question
  with a different answer, and plan 47's status document is where that gap table belongs.
