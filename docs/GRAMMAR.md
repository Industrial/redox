# Voltaic's grammar, as a graded inventory

Doc 02 §4.1 asks for this document and says what it is for:

> **any Rust construct marked `UNREACHABLE` is a construct Voltaic users cannot write**, which
> directly violates §26's interoperability charter.

So this is **risk detection**, not documentation. Plan 46
(`docs/40 - 46 - Grammar Inventories.md`) built it; `docs/03 - Voltaic Surface Design.md` §18 is
a summary of one axis rather than a substitute for this one.

**Every row carries evidence, and `crates/voltaic-cli/tests/grammar_inventory.rs` resolves it.**
That is not ceremony. Plan 46 §1.3 measured what an unchecked inventory costs: of sixteen
apparent gaps in a first careless pass, **eleven were the probe's own bug** — and a false
`UNREACHABLE` is worse than a missing row, because it sends a future plan to design a surface
for something that already works.

| Grade | Means | Evidence it carries |
|---|---|---|
| `PASSTHROUGH` | Voltaic writes the Rust construct essentially unchanged | a corpus file that uses it |
| `SUGARED` | reachable, through a different spelling | a corpus file, and the Rust it lowers to |
| `UNREACHABLE` | a Voltaic program cannot express it | a fixture under `tests/unreachable/` that **still fails**, and the nearest thing that does work |

An `UNREACHABLE` fixture failing is asserted, so when a later row closes a gap the fixture
starts compiling and the test breaks. It fails when the compiler gets *better*, deliberately
— the alternative is a document still claiming a gap years after it closed.

---

## 1. Rust's grammar

Organised by the Rust reference's areas rather than by Voltaic's constructs, so coverage can be
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
| Generic traits | `SUGARED` | `examples/receiver.sdx` — `class Scale t` is `trait Scale<T>`, plan 67 leaf 2. Unreachable until then: the head's one slot named the *implementing* type, so a second lowercase name became a **method** generic and rustc answered `E0049` (plan 67 §1.4). `crates/voltaic-cli/tests/check/generic_class.sdx` is the program that used to |
| `unsafe trait` | `UNREACHABLE` | `tests/unreachable/unsafe_class.sdx` — plan 45 §6 asked for this check and plan 45 §10 did not record it. **No alternative**: an invariant rustc cannot check has no safe spelling |
| Impls, inherent and trait | `SUGARED` | `examples/class.sdx` — `instance` |
| Two impls of one trait for one type | `SUGARED` | `examples/receiver.sdx` — `instance Scale f64 Meters` and `instance Scale u32 Meters`, which is `impl From<T>`'s shape. The **last** name in the head is the implementing type (plan 67 §9 decision 3), so the head reads in the same order as the `impl Scale<f64> for Meters` it becomes |
| Blanket impls | `SUGARED` | `examples/binder.sdx` — `instance given <T: Display> Describe T` emits `impl<T: Display> Describe for T`, plan 84 leaf 4. Closed 2026-11-11. Through `=>` it is still unreachable and the cause is plan 67 leaf 3's: `instance Display t => Describe t` emits `impl Describe for t`, with a lowercase target rustc rejects, because `TypeApp::vars` reads the *arguments* and a bare variable is the head — so the generic list comes out empty and the bound goes with it. That is a **synthesis** failure rather than a type one, which is why writing the list verbatim is the whole fix and `TypeApp` needed no change |
| `unsafe impl` | `UNREACHABLE` | `tests/unreachable/unsafe_class.sdx` — same row, same fixture |
| Modules, inline and nested | `SUGARED` | `examples/module.sdx` — `module` |
| Items in a function body | `SUGARED` | `examples/local.sdx` — plan 55 leaf 5. A `data`, `errors`, `class`, `instance`, `import`, `const` or `type` is a block element and lowers to the nested Rust item. `test` is refused (`RDX0112`: rustc does not collect a nested `#[test]`) and `module` is out of scope, a bodiless one loading a file |
| `use` declarations, groups, globs, aliases | `SUGARED` | `examples/module.sdx` — `import` |
| `extern` blocks | `SUGARED` | `examples/reachability.sdx` — plan 45 leaf 3; `unsafe extern` is emitted because edition 2024 requires it |
| Nested `fn` items | `SUGARED` | `examples/wheres.sdx` — plan 61 leaf 0. A `where` entry may be a **function**, declared exactly as at module scope: a signature line and one or more equation rows, which emit as a nested `fn`. Mutual recursion among them needs no ordering, because Rust items are order-independent inside a block. Voltaic's `where` had no graded row at all until this one — which is why plan 61's criterion 8 was free, and why it should not have been |
| Macro definitions | `UNREACHABLE` | `tests/unreachable/macro_def.sdx` — **declined** under doc 03 §23 item 3. Alternative: inline Rust, which plan 45 leaf 1 shipped and plan 64 leaf 4 gave an expression position and a layout-delimited form. Invocation is `SUGARED` since plan 64; this row is the half that stays declined, which is plan 46's rule working in both directions at once |
| deriving templates | `SUGARED` | `examples/template.sdx` — plan 89. **Rust has no file-local derive at all**: `#[derive(Forward)]` needs a `proc-macro = true` crate of its own, `syn`, `quote` and a second compilation unit, so a trait one crate wants forwarded three times is written out three times. `deriving template Forward` declares an `instance` whose target is the reserved name `Target`, and every declaration that names `Forward` in its `deriving` list gets that `instance` with `Target` replaced by its own type name and `Field0`… by its field names. The lowering is **textual substitution on the emitted Rust**, so rustc checks the generated `impl` exactly as it would a hand-written one, and a template name is split out of `#[derive(…)]` by a table lookup rather than by name resolution — an undeclared name stays in the derive list, which is `E0433`'s job and doc 03 §9.5's existing behaviour. `template` is Voltaic's **third contextual keyword** after `from` and `pattern`. **Repetition shipped as leaf 3**: `Fields` is every field as a `self` access and `FieldNames` every field's name as a string literal, both comma-separated, so one template visits declarations of any width — `examples/template.stdout` asserts `x=1, y=2` and `a=3, b=4, c=5` from the same template. The loop runs over the declaration's own field list and produces the text that is substituted, which is what `emit_data` already does per field; doc 03 §13.3's line against `comptime` is untouched, because an interpreter runs the author's code and this joins a list the parser produced. What is still missing needs a **construct** rather than a table entry: a per-field *fragment* (a whole `fn` per field, a `match` arm per variant), and `$variants`, which also wants a constructor choice `field_names` refuses to make for a multi-constructor `data`. Plan 89 §10.3 records both |
| Attributes, inner and outer | `PASSTHROUGH` | `examples/attributes.sdx` — Rust's own `#[…]`, plan 36. **One exception, and only one:** plan 53's `#[voltaic::allow(unwrap)]` is *consumed* by the emitter and never reaches the `.xrs`, because an un-consumed `voltaic::` attribute is rustc's `E0433` — a hard error, not an unknown lint |

### 1.3 Types

| Area | Grade | Evidence and note |
|---|---|---|
| Paths and generic arguments | `PASSTHROUGH` | `examples/generic.sdx` — a type is verbatim text (doc 03 §14.1) |
| References, `&mut` | `PASSTHROUGH` | `examples/class.sdx` |
| The receiver, marked in a type | `SUGARED` | `examples/receiver.sdx` — plan 67. `self` in a type is the receiver at **any** position, and the borrow prefix on the marker chooses between `&self`, `&mut self` and `self`. `minus : u32 -> &self -> u32` and `plus : &self -> u32 -> u32` emit byte-identical Rust |
| Raw pointers | `PASSTHROUGH` | `examples/reachability.sdx` — verbatim, as any type is |
| Tuples | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Arrays and slices | `PASSTHROUGH` | `examples/arrays.sdx` |
| Trait objects `dyn Trait` | `PASSTHROUGH` | `examples/generic.sdx` |
| `impl Trait`, argument and return | `PASSTHROUGH` | `examples/generic.sdx` — plan 25 |
| Function pointers and `Fn` bounds | `PASSTHROUGH` | `examples/generic.sdx` |
| Lifetimes | `PASSTHROUGH` | `examples/strings.sdx` — `&'static str`; carried verbatim, so there is no separate construct |
| Generic parameters and bounds | `SUGARED` | `examples/generic.sdx` — a **bound clause** read off `=>` (doc 18 §2.2), not `where`. **The clause accepts only `Trait var`**, where the trait is a bare type identifier and the variable a bare lowercase word; `Ord t` and `Ord t, Clone t` work and the six rows below do not. This row read as unrestricted until plan 84 leaf 0 measured it |
| `where` clauses | `SUGARED` | `examples/generic.sdx` — the same bound clause, with the same restriction. Voltaic's own `where` introduces **local bindings** and is a different construct, graded on the row below |
| The verbatim binder, `given <…>` | `SUGARED` | `examples/binder.sdx` — plan 84 leaf 1. Doc 03 §14.1's argument applied one level up: a **type** is carried verbatim, and so now is the **binder**. `given` turns doc 18 §2.2's synthesis off for its signature entirely — no generic list, no `=>` bounds, no type rewrite (plan 84 §1.6) — so lowercase spine atoms are ordinary verbatim types. The six rows below are what it bought, each `UNREACHABLE` until this row |
| A bound whose trait has type arguments | `SUGARED` | `examples/binder.sdx`'s `shout` — `given <T: Into<String>>`. **The largest of the six**: `T: Into<String>`, `U: From<T>`, `I: IntoIterator<Item = T>` and `F: Fn(i32) -> i32` are all this shape. Through `=>` it is still unreachable — `Into<String> t` is not `Trait var` — and that is now **`RDX0120`** rather than the silent swallowing plan 84 leaf 0 measured, which emitted `x: Into<String> T FatArrow T` and `E0782` |
| A projection predicate | `SUGARED` | `examples/binder.sdx`'s `total` — a signature-position `where`, which is doc 03 §14.2's declared role for that position and was unimplemented until plan 84 leaf 3. The `=>` clause pairs a trait with a *variable*, so a path has no position in it |
| A relaxed bound `?Sized` | `SUGARED` | `examples/binder.sdx`'s `width` — `given <T: ?Sized>`. Plan 84 leaf 0 found the `?` had **no respelling at all**, so it reached generated Rust as the text `QuestionSized` — a `TokenKind`'s own Debug name. `token_text` is complete as of leaf 1 and `respells_every_operator_the_lexer_produces` asks the lexer rather than a second table |
| A bound between two lifetimes | `SUGARED` | `examples/binder.sdx`'s binder clauses — a `given <'a, 'b: 'a>` or a signature `where 'a: 'b` carries it verbatim. Through `=>` it is unreachable, because the clause has one trait slot and a lifetime bound has a lifetime on both sides |
| Const generic parameters | `SUGARED` | `examples/binder.sdx`'s `first_of` — `given <const N: usize> [u8; N]`. Synthesis cannot reach it because `type_vars` scans lowercase words in verbatim type text without distinguishing a type position from an array's length position, so the `n` in `[u8; n]` became a **type** parameter and `fn first_of<N>(xs: [u8; N])` used a type as a value (`E0423`) |
| Higher-ranked bounds `for<'a>` | `SUGARED` | `examples/binder.sdx`'s `trimmed` — `where for<'a> F: Fn(&'a str) -> &'a str`. Synthesis was worse than absent here: it excludes only `dyn`, `impl`, `mut` and `as`, so `for` and `fn` both became type parameters — `fn calls<'a, FOR, FN>(f: FOR<'a> FN(&'a str) -> &'a str)`, `E0109` and `E0618` |
| Associated types and constants | `SUGARED` | `examples/class.sdx` |
| Associated functions | `SUGARED` | `examples/receiver.sdx` — a member with no `self` in any of its types has no receiver, which is how Rust spells one too: `start : u32 -> Self` is `fn start(__rdx_a0: u32) -> Self`, called `Counter::start 3`. Plan 67 leaf 4; the construct existed before the row and had no witness |

### 1.4 Expressions

| Area | Grade | Evidence and note |
|---|---|---|
| Operators and precedence | `PASSTHROUGH` | `examples/expressions.sdx` |
| Functional update through a path | `SUGARED` | `examples/optic.sdx`'s `over` — plan 88 leaf 3. `over (.addr.zip.code) doubled target` is `target.addr.zip.code = doubled(target.addr.zip.code)`: the path is said **once** in the source and twice in the output, because Rust has no other spelling for *apply through a place*. That is the construct's whole argument — two spellings of one path that must agree is the shape of a typo that type-checks, and `target.addr.zip.code := doubled target.addr.zip.town` compiles whenever both fields are the same type. The lowering is an **assignment through the place**, so nothing is moved, nothing is cloned and rustc judges the borrow; the emitted Rust is byte-identical to the `:=` form. The function is read as an *atom*, which is what lets it be a bare method reference — `over (.addr.city) .to_uppercase u` — because the place is a separate atom rather than the method's receiver. `over` is Voltaic's **fourth contextual keyword**, settled by three tokens: the word, `(`, then `.`. `RDX0127` refuses a selector that is not a path; an operator section cannot reach it, because the dispatch requires `(.` |
| Operator declaration | `SUGARED` | `examples/operator.sdx` — plan 90. **Rust has no custom operator at all**, and `docs/DECLINED.md` refused this for that reason — *"no Rust lowering target exists"* — which plan 90 §1.1 corrected: the surface already carries six operators Rust has no counterpart for, and every one lowers to an ordinary expression. `infixl 6 <+> = merge` gives an **existing** function a spelling and a fixity and emits no item, so `a <+> b` is `merge(a, b)`: nothing is declared, nothing renamed, and go-to-definition has somewhere real to go. A Haskell-style *definition* stays out of reach, because no Rust function can be named `<+>` and inventing one is the fabricated API doc 03 §8.7 refuses. The spelling set is **closed** — twelve `<` · *char* · `>` forms — which is what kept the lexer static where §1.3 priced a permanent pre-pass over raw source; an undeclared use is `RDX0123` from the parser, where declaration order is already known. Precedence is Haskell's 0-9, mapped to the even binding powers between `|>` and `+`, so every user operator binds looser than arithmetic. `infix` is non-associative by **diagnostic** (`RDX0125`), because a Pratt loop's powers can only group one way. `infix`/`infixl`/`infixr` are contextual, settled by a three-token lookahead |
| Method calls, field access | `SUGARED` | `examples/comprehension.sdx`, `examples/application.sdx` — a method is a **function** since plan 57 and its receiver is the last argument, so `.len xs`, `xs \|> .len` and `.insert k v m`. **Position** decides, not adjacency: a `.` in head position calls, a `.` after an expression selects a field, and `(.age)` is the selector because a `)` follows the name. A method whose name is a Voltaic keyword is **unspellable**, which is what plan 55 leaf 7 bought back for one: `Ordering::then` was `RDX0103` while `then` was reserved, and `examples/control.sdx`'s `ordered` now writes it |
| Function calls | `SUGARED` | `examples/hello.sdx` — parenless application, Haskell's rule (doc 18 §2.1) |
| Indexing | `PASSTHROUGH` | `examples/arrays.sdx` |
| Tuple indexing `.0` | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Ranges | `PASSTHROUGH` | `examples/control.sdx` |
| Struct and record literals | `SUGARED` | `examples/record.sdx` — braces with `=` for values |
| Array literals and repeats | `SUGARED` | `examples/arrays.sdx` — `[v; n]`, and `:: Vec<T>` for the growable form (plan 29) |
| Closures | `SUGARED` | `examples/expressions.sdx` — `\x -> e` |
| The composed selector, `(.a.b.c)` | `SUGARED` | `examples/optic.sdx` — plan 88. `(.name)` selected **one** field; this selects a path, at any depth, and is what a sort key or a `map` actually wants. Graded here because leaf 0 found the old reading was not *absent* but **wrong**: `(.addr.city)` parsed as a selector of `addr` followed by a *method call* on the closure it produced, emitting `(\|__rdx_1\| __rdx_1.addr()).city(o)` — not Rust, no Voltaic diagnostic, and `E0282` from rustc. Milestone 9's worst class, found by writing the second dot. The **functional** setter is a different matter and is declined rather than missing: see `docs/DECLINED.md` |
| The application hole, `_` | `SUGARED` | `examples/hole.sdx` — plan 85. `_` in an **argument** position is a closure parameter, so *n* holes make an *n*-parameter closure and the callee is never consulted. It reaches what currying cannot: plan 56 §11.1 proved arity is knowable only for a bare name with an in-file signature, so a Rust path (`String::from _`), a method (`.insert "a" _ m`) and an imported name (`max _ 0`) were all uncurryable. It also abstracts over **any** position where currying takes a suffix. **`_` now has four jobs and position decides** — the other three are a pattern's wildcard, a type's inferred `_`, and the target of `:=`, all unchanged. It was never `RDX0101`: a bare `_` already emitted verbatim, so `add _ 2` gave `add(_, 2)` and rustc's *"`_` can only be used on the left-hand side of an assignment"*, which is still the answer for a hole with no enclosing application |
| A method awaiting its receiver | `SUGARED` | `examples/hole.sdx`'s `trimmed` — `.map (.trim _)`. **Unwritable before plan 85**, and the README's own table was wrong about it: `(.trim)` is the *selector*, because a `.name` followed by `)` selects a field, so it emitted `\|x\| x.trim` and rustc answered `E0615`, *"attempted to take value of method"*. A hole makes the receiver the closure's parameter, which is the `\|x\| x.trim()` the README claimed all along |
| Assignment to `_` | `PASSTHROUGH` | `examples/hole.sdx` — `_ := e` is Rust's discard assignment, `_ = e;`. Reachable since plan 54 and **in no corpus file until plan 85**, which is why `grammar.js` still parsed it as a wildcard pattern followed by an `ERROR`; the assignment row below lists five target shapes and did not list this one |
| `move` closures | `SUGARED` | `examples/async.sdx` |
| Blocks as expressions | `SUGARED` | `examples/control.sdx` — an **indented block**, with an optional `'label`. Spelled `do` and `do 'label` until plan 55 leaf 7 retired the keyword: a block is an expression, so nothing announces one. `do` stays reserved and is `RDX0114`. The three rows below are **modifiers on this one block**, not constructs of their own |
| `async` blocks | `SUGARED` | `examples/async.sdx` — plan 39; `work = async` and the block indented under it. Regraded at plan 55 leaf 7: `async`, `unsafe` and `try` were three `prefix` arms of one shape and are now one arm reading an adjective and then the block above |
| `unsafe` blocks | `SUGARED` | `examples/reachability.sdx` — plan 45 leaf 2; `n = unsafe` and the block indented under it. A modifier on the block above, since plan 55 leaf 7 |
| `try` blocks | `SUGARED` | `examples/fallible.sdx` — an immediately-invoked closure, because Rust has try blocks only unstably. `r = try` and the block indented under it; a modifier on the block above, since plan 55 leaf 7 |
| `do .method` — the named bind | `SUGARED` | `examples/bind.sdx` — plan 86, and **the only decline this project has ever lifted**. `docs/DECLINED.md` refused a *"general monadic comprehension"* for having to *"choose `flat_map` versus `and_then`"*, with the sole reopening venue *"revisit only if a type-free discrimination is found"*; plan 60's block modifier already was one. The fifth construct of the modifier shape and the only one whose modifier **names a method** — so unlike `try` and `async`, which read `<-` as a *postfix* and emit a statement per row, this folds the rows **rightwards into nested closures** with `move` on each, which is what doc 03 §10.1's comprehension already does. Nothing is fabricated, the final row included: choosing between `Ok`, `Some` and `iter::once` would be the inference §2 clause 1 forbids. `do` is still `RDX0114` in every other position, and the grammar reuses the `(reserved_word)` node so the word stays globally reserved |
| `?` operator | `PASSTHROUGH` | `examples/fallible.sdx` |
| `.await` | `SUGARED` | `examples/async.sdx` — the draw arrow `<-` (doc 30) |
| `as` casts | `UNREACHABLE` | `tests/unreachable/as_cast.sdx` — `as` is reserved for `import … as`. Nearest: `u8::try_from n`, which is **not** a cast — `as` truncates, `try_from` is fallible |
| Macro invocations | `SUGARED` | `examples/macros.sdx` — plan 64. `name! \-> e`: `!` marks the name and a nullary lambda opens each argument, so no bracket appears. Interpolation splices, so `bail! \-> "no route for #{id}"` reaches Rust as two arguments. Was `UNREACHABLE` for nineteen rows — doc 18 §7 spent `!` before any row implemented it, and plan 48 priced the gap at the figure `docs/CONFORMANCE.md` carries |
| Macro invocation with a non-expression argument | `UNREACHABLE` | `tests/unreachable/macro_item_argument.sdx` — graded 2026-11-11 by plan 81. Plan 64's `name! \-> e` opens each argument with a **nullary lambda**, so an argument is an expression: `matches!`'s *pattern* (plan 64 §6) and `thread_local!`'s *item* are both out of reach, and the second is what `voltaic-emit` needs three times. Nearest: the inline-Rust hatch, which is where `crates/voltaic-emit/sdx/binding.sdx` and its two siblings go — the first thing in the ported compiler doc 02 §4.8's criterion cannot claim, recorded in `port.rs`'s `HATCH_ALLOWED` |
| Inline Rust | `SUGARED` | `examples/reachability.sdx` — doc 01 §23.1, plan 45 leaf 1. No Rust equivalent: it *is* Rust. Plan 64 leaf 4 added the **expression** position and a layout-delimited body beside the `"""` string form, because a token-stream DSL's input is not a Voltaic expression and no value-passing shape can carry it |

### 1.5 Patterns

| Area | Grade | Evidence and note |
|---|---|---|
| Literal and wildcard patterns | `PASSTHROUGH` | `examples/case.sdx` |
| Identifier and binding patterns | `SUGARED` | `examples/destructuring.sdx` |
| `@` bindings | `PASSTHROUGH` | `examples/case.sdx` |
| Pattern synonyms | `SUGARED` | `examples/synonym.sdx` — plan 87. **Rust has no pattern abstraction at all**: a pattern cannot be named, reused or parameterised, and `macro_rules!` does not reach the ergonomics, which is why the RFC has stood unaccepted for years. The lowering is **textual substitution at emit time**, so rustc performs every check it would have on the written-out form — exhaustiveness, field names, types, binding modes — and doc 03 §2's corollary holds both ways. Reaches an equation head, a `case` arm and a position **nested inside another pattern**. Parameters are *formal*, so every binder is written at the use site, which is the one property separating this from `docs/DECLINED.md`'s refusal of `RecordWildCards`; `RDX0121` refuses a body that binds anything else. `pattern` is Voltaic's **second contextual keyword** after `from` — reserving it was tried and reverted, because it is a field name in four of this compiler's own declarations |
| A bare constructor as a clause's whole pattern | `SUGARED` | `examples/synonym.sdx`'s `describe` — `describe Origin = …`. Graded 2026-11-11 by plan 87, which **fixed it**: an upper-case `Binding` is a constructor and not a binder, and `signature_from` was lending its name to the parameter — so `pick None = 0` emitted `fn pick(None: Option<i32>)`, a refutable pattern in argument position that rustc answers with `E0005`. Pre-existing, and no corpus file had one, because every existing clause pattern is an `Apply` or a `Record`. The parameter now takes `__rdx_a0` and the constructor moves into the `match` |
| Struct and record patterns | `SUGARED` | `examples/destructuring.sdx` |
| Tuple patterns | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Slice patterns, with `..` | `PASSTHROUGH` | `examples/destructuring.sdx` |
| Range patterns | `PASSTHROUGH` | `examples/case.sdx` |
| `ref` and `ref mut` | `UNREACHABLE` | `tests/unreachable/ref_pattern.sdx` — regraded 2026-11-10. `ref` is **not reserved**, so `ref a` lexes as two identifiers and the pattern parser answers `RDX0101`; the old note graded this `PASSTHROUGH` and cited a destructuring example that contains no `ref` at all. Nearest: **match ergonomics**, which is what that example actually uses — `case &pair of (a, b) ->` binds by reference with no keyword. What is lost is the case where ergonomics do not apply and a binder must be explicitly `ref mut` |
| `\|` or-patterns | `UNREACHABLE` | `tests/unreachable/or_pattern.sdx` — doc 18 made `\|` a **guard**, so `1 \| 2 ->` parses and means `1` guarded by `2`. Nearest: **separate arms**, which match the same values; what is lost is one arm body shared across patterns |

### 1.6 Statements and control flow

| Area | Grade | Evidence and note |
|---|---|---|
| `let` bindings, `let mut` | `SUGARED` | `examples/expressions.sdx` — `x = e` and `mut x = e` |
| Assignment | `SUGARED` | `examples/control.sdx` — `x := e`, because `=` is a binding (plan 54). Covers a name, a field, an index, a deref target and Rust's destructuring assignment, the target being a general expression |
| Compound assignment | `SUGARED` | `examples/control.sdx` — all five of `+=`, `-=`, `*=`, `/=`, `%=`. Shipped in plan 20 and graded here for the first time |
| Shadowing | `PASSTHROUGH` | `examples/control.sdx` — an inner `n = 1` inside a nested block over an outer `n`, which is the contrast that makes `:=` necessary. Cited `examples/expressions.sdx` until plan 54; that file holds exactly one binding, and nothing in the corpus shadowed anything |
| `if` / `else` | `SUGARED` | `examples/control.sdx` — an expression, with layout, and **one spelling** since plan 55 leaf 7 retired `then`. A conditional in argument position is written indented inside the brackets (plan 65); inside a `#{…}` interpolation it cannot be written at all, and `bracketed` calls one instead |
| `if let` | `SUGARED` | `examples/draw_condition.sdx` — `if e is P`, plan 60. Spelled `if P = e` until that row moved the pattern test onto `is` and put the scrutinee first |
| `let` chains | `SUGARED` | `examples/draw_condition.sdx` — the draw arrow in a condition (doc 30) |
| `while`, `while let` | `SUGARED` | `examples/control.sdx` |
| `loop` | `SUGARED` | `examples/control.sdx` |
| `for` | `SUGARED` | `examples/control.sdx` — `for x in xs` |
| `break`, `continue`, with value | `PASSTHROUGH` | `examples/control.sdx` |
| Labelled loops and blocks | `SUGARED` | `examples/control.sdx` — `'outer loop`, `'label` on a bare block, `break 'label v`. The label is the one capability `do` carried that nothing else provides (plan 55 §1.7), so it outlived the keyword |
| `match` | `SUGARED` | `examples/case.sdx` — `case … of`, with guards |
| Guards | `SUGARED` | `examples/guards.sdx` — `\|` in an arm, which is what shadows Rust's or-pattern |
| `return` | `PASSTHROUGH` | `examples/control.sdx` — `first_divisor`, returning out of a `for`. Regraded at plan 55 leaf 8: `return` is an **early exit** and nothing else. It used to be the only way a branch could produce a value, which plan 55 §1.2 measured at fourteen non-tail uses against two conditional expressions in the same corpus; every body position is an expression now, so `examples/draw_condition.sdx`'s `describe` dropped its three |

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
| Table literals | **REMOVE** | records and `data`; Voltaic has no table type |
| `switch`/`when` | **REMOVE** | `case … of` (doc 18 §2.3) |
| `with` blocks | **REMOVE** | doc 18 §7 deleted them; the keyword was reused for the callback bind (doc 30 §3.3) |
| `do` blocks | **REMOVE** | an indented block **is** an expression, so no keyword announces one — plan 55 leaf 7. The labelled form survives as `'label` on a block, and the word stays reserved for plan 60's do-notation (`RDX0114`). Graded **MODIFY** — *"a labelled block expression (doc 03 §7.2)"* — until that leaf |

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
