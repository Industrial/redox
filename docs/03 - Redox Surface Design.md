# Redox — Surface Design

Version: 0.1.0
Status: design register / review draft
Derived from: a comparative survey of Ruby, Haskell, C#, Gleam and Elixir, with
secondary reference to Clojure, Dart, Kotlin, Swift, Zig, Scala, F#, OCaml, Julia,
Koka, Nim, Smalltalk, MoonScript and Rust itself.
Amends: `docs/01 - Redox.md` (the specification) and `docs/02 - Redox Update Blueprint.md`
(the implementation blueprint), by addition and by named correction.

---

## 1. Purpose and relationship to the preceding documents

`docs/01 - Redox.md` specifies a MoonScript-inspired surface for Rust.
`docs/02 - Redox Update Blueprint.md` audits that specification against a working
compiler of comparable ambition and proposes an implementation architecture, a lowering
discipline, and thirteen named corrections.

This document decides the **surface**: which constructs the language admits, how each is
spelled, and what Rust each produces. It is a decision register, not an exploration —
every entry below is settled, with its provenance and its lowering. Open items are
confined to §23.

Three properties distinguish this document from doc 01:

1. **Every construct carries an explicit lowering.** Doc 01 gives a mapping table
   (§3.3) and worked examples; this document gives the generated Rust for each admitted
   construct, because the lowering *is* the specification for a transpiler.
2. **Every construct is justified against the type-free constraint** (§2). Doc 02 §6.2
   established the rule; this document applies it as an admission test and records what
   it excludes.
3. **Collisions are resolved.** Expanding the surface created genuine lexical conflicts
   — six meanings for `#`, three for `...`, two for `where`, two for `and`. §4 and §9
   resolve each.

### 1.1 Correction to doc 02 §6.2

Doc 02 §6.2 stated the governing rule correctly but applied it too conservatively,
treating several constructs as type-dependent that are not. The reason is that **rustc,
not Redox, resolves method calls.** Any lowering that emits a method call is therefore
type-free, which admits monadic chaining, callback flattening, comprehension desugaring
to `flat_map`, broadcasting, and the operator sections of §5.4. The constraint excludes
only lowerings that must *choose between two different shapes* on the basis of a type.

Three mechanisms do most of the work, and are named here because they recur throughout:

| Mechanism | What it buys | Used by |
|---|---|---|
| **`?` inside a closure returning `Option`** | rustc performs the entire `and_then`/`map` unification | `?.` chains (§12.3) |
| **`match` lowering** | avoids method-arity and method-resolution ambiguity entirely | `??` (§12.1), `catch` (§12.2), relational patterns (§6.2) |
| **Method-call auto-ref / auto-deref** | one spelling works whether the receiver is `T` or `&T` | comparison sections (§5.4), method references (§8.5) |

The third is the least obvious and the most useful. Rust's method-call syntax inserts
`&`, `&mut` or `*` as needed to reach a receiver, while operator syntax does not. So
`x.gt(&0)` type-checks for both `x: i32` and `x: &i32`, whereas `x > 0` type-checks only
for the former. Wherever a lowering would otherwise have to know whether a closure
parameter arrives by value or by reference, the method form removes the question.

---

## 2. The admission test

A construct is admitted to the Redox surface if and only if it satisfies all four:

1. **Type-free.** Its lowering is determined by the source text alone. Where two Rust
   shapes are possible, the user's own syntax selects between them — never an inference.
2. **No implied dependency.** Its lowering names nothing outside `core`, `alloc` and
   `std`. This is what excludes regex literals (§11.3) and property-test blocks (§15).
3. **Semantics-preserving.** It does not alter Rust's ownership, borrowing, mutability or
   evaluation-order semantics, and never inserts a clone (doc 01 §25.1).
4. **Reachability-positive.** It either adds expressiveness or closes a gap in the set of
   Rust constructs a Redox program can express (doc 02 §4.1's second inventory axis).

Three corollaries, carried forward from doc 02 and unchanged:

- **Every check rustc can make, rustc makes.** Redox performs no type check, no borrow
  check, no exhaustiveness check and no mutability check.
- **Ownership stays syntactic.** Where a lowering could consume or borrow, the user
  writes which (doc 01 §9.7's discipline, extended in §10.2 to broadcasting).
- **Hygiene is reserved.** All generated identifiers carry the `__rdx_` prefix; user
  identifiers may not (doc 02 §6.3).

---

## 3. Decision register

Forty-eight decisions, in the order taken. Each is specified in full in the section
named.

| # | Decision | Primary provenance | §|
|---|---|---|---|
| 1 | Expansive surface, strictly type-free | — | 2 |
| 2 | `\|>` is a **receiver** pipeline | Elixir, Gleam, F# | 5.3 |
| 3 | `with` is a mutation block with dot-shorthand | MoonScript | 7.5 |
| 4 | `use` flattens callback-taking APIs | Gleam, Koka | 7.6 |
| 5 | `is` is a binding-aware pattern test | C# | 5.5 |
| 6 | Relational and logical patterns via guard lowering | C# | 6.2 |
| 7 | Named record patterns are Rust patterns; anonymous ones are guards | C#, Rust | 6.3 |
| 8 | Slice, as-, range patterns and `when` guards | Rust, Haskell | 6.4 |
| 9 | Both comprehensions and collection-`if`/`for` | Python/MoonScript, Dart | 10.1 |
| 10 | Broadcasting, with a mandatory borrow marker | Julia | 10.2 |
| 11 | Literal shape sets the collect target; `into` overrides | Elixir | 10.3 |
| 12 | Spread, `%w`, set literal | JS/C#/Dart, Ruby, — | 10.4 |
| 13 | Operator sections | Haskell | 5.4 |
| 14 | Multi-clause function definitions | Haskell, Elixir, Erlang | 8.1 |
| 15 | Labelled arguments, declaration-order only | Gleam, Swift, OCaml | 8.2 |
| 16 | `where` locals, `...` forwarding, `&:` method references | Haskell, Ruby, Ruby | 8.3–8.5 |
| 17 | Block struct init with trailing spread | MoonScript, Rust | 9.1 |
| 18 | Field punning, both directions | Haskell, Ruby 3.1 | 9.2 |
| 19 | Primary constructors | C# 12, Gleam | 9.3 |
| 20 | Enum struct variants, `derive` clause, granular visibility, aliases | Rust | 9.4–9.7 |
| 21 | `guard` built on `is` | Swift, Rust | 7.1 |
| 22 | `do` with labelled break-with-value | Zig, Rust | 7.2 |
| 23 | No cascades, no scope functions | — | 7.5 |
| 24 | `defer`, `cond`, `loop`, exhaustive `switch` | Zig/Swift/Go, Elixir/Haskell, Rust, Gleam/Elm | 7.3, 7.4, 7.7, 6.6 |
| 25 | Inline format specifications | Rust, C#, Swift | 11.1 |
| 26 | Raw, char, byte and C-string literals | Rust | 11.2 |
| 27 | No sigils beyond `%w` | — | 11.3 |
| 28 | `##` / `##!` doc comments | Emerald | 11.4 |
| 29 | `??` is `Option`-only, with `??=` | C#, JS, Dart | 12.1 |
| 30 | `catch` is `Result`-only, with optional error binding | Zig | 12.2 |
| 31 | `?.` borrows; no implicit clone | — | 12.3 |
| 32 | `try` blocks, typed domain errors, panic family, `unwrap` lint | Rust, Elixir, Rust, Gleam | 12.4–12.7 |
| 33 | Longest-match lexing resolves all six `#` forms | — | 4.1 |
| 34 | **Rust range spelling exactly** — corrects doc 01 §21 | Rust | 4.2 |
| 35 | `'label`, matching Rust; `@` is self-access only | Rust | 4.3 |
| 36 | `and`/`or`/`not` are pattern combinators only | — | 4.4 |
| 37 | `::` everywhere, including imports — corrects doc 01 §22 | Rust | 13.1 |
| 38 | Verbatim attribute passthrough plus a `derive` clause | Rust | 13.2 |
| 39 | Rust's `const` surface only; no transpile-time evaluator | Rust | 13.3 |
| 40 | Module blocks, re-export, glob import, nested import trees | Rust | 13.4 |
| 41 | Rust type syntax, verbatim | Rust | 14.1 |
| 42 | `where` disambiguated by position | Rust + Haskell | 14.2 |
| 43 | Postfix `.await` — corrects doc 01 §24 | Rust | 14.3 |
| 44 | Default bodies, associated items, supertraits, `impl Trait` | Rust | 14.4 |
| 45 | `test` blocks, std-only | Emerald, Rust | 15 |
| 46 | One canonical form per construct, formatter-enforced | Emerald plan 90 | 16 |
| 47 | Array repeat, tuple structs, labelled loops, `move` closures | Rust | 18 |
| 48 | Document named `03 - Redox Surface Design.md` | — | — |
| 49 | `f x, y` is canonical in **every** position; parentheses are grouping | MoonScript, Ruby | 5.1 |
| 50 | `(-1)` versus `(- 1)` resolved by adjacency | — | 5.4 |

---

## 4. Lexical surface

### 4.1 The `#` sigil — six forms, resolved by longest match

Expanding the surface gave `#` six meanings outside strings and one inside them. No true
ambiguity exists, because interpolation occurs only within a string literal and therefore
never competes with the set literal. The lexer orders its rules longest-first:

| Order | Token | Meaning | §|
|---|---|---|---|
| 1 | `###` | multiline comment fence | 11.4 |
| 2 | `##!` | inner doc comment → `//!` | 11.4 |
| 3 | `##` | outer doc comment → `///` | 11.4 |
| 4 | `#[` | attribute, verbatim passthrough | 13.2 |
| 5 | `#{` | set literal | 10.4 |
| 6 | `#` | line comment → `//` | 11.4 |
| — | `#{` *inside a string literal* | interpolation | 11.1 |

Comments are **trivia attached to AST nodes**, never discarded — doc 02 §10.1's
correction to doc 01 §4.1, restated here because the doc-comment forms above depend on it.

### 4.2 Ranges — correcting doc 01 §21

**Doc 01 §21 is corrected.** It makes `..` inclusive and `...` exclusive, inverting Rust,
where `..` is exclusive. Doc 02 §5.2 identified an expression that reads as Rust and means
something else as the worst available failure mode, and a range operator is the most
likely place for it to occur. Redox adopts Rust's spelling exactly:

| Redox | Rust | Meaning |
|---|---|---|
| `1..10` | `1..10` | exclusive |
| `1..=10` | `1..=10` | inclusive |
| `1..` | `1..` | from |
| `..10` | `..10` | to |
| `..=10` | `..=10` | to inclusive |
| `..` | `..` | full |

This has a second benefit: it frees `...` entirely, dissolving what would otherwise have
been a three-way collision. `...` is now unambiguous by position:

| Form | Position | Meaning | §|
|---|---|---|---|
| `[0, ...xs, 9]` | prefix, in a collection literal or argument list | spread | 10.4 |
| `[a, ...rest]` | prefix, in a pattern | rest | 6.4 |
| `f(...)` | standalone, in a parameter or argument list | argument forwarding | 8.4 |

### 4.3 `'` and `@`

Labels use Rust's own spelling, `'name`, which in Rust already shares the lifetime
namespace — so this is passthrough rather than translation. `@` therefore keeps its single
meaning from doc 01 §16.

| Form | Meaning |
|---|---|
| `'a` | lifetime, or a block/loop label |
| `'c'` | char literal — distinguished by the closing quote, exactly as rustc does |
| `@name` | `self.name` (doc 01 §16.1) |
| `@@name` | `Self::name` (doc 01 §16.2) |
| `name @ pattern` | as-pattern (§6.4) — infix, so distinct from the prefix sigil |

### 4.4 `and` / `or` / `not`

These three words are **pattern combinators only** (§6.2). Boolean logic uses Rust's
symbolic operators, so a Redox boolean expression is visually identical to the Rust it
becomes.

```
# Patterns — keywords
switch subscription
  Active and not Expired  => "ok"
  Trial or Promotional    => "limited"
  _                       => "none"

# Booleans — symbols
if a > 0 && b > 0
if ready || forced
if !done
```

One consequence, recorded because an earlier draft had it wrong: a pattern test combined
with a boolean condition uses `&&`, not `and`:

```
if msg is Move(x, _) && x > 0
```

### 4.5 Keyword and operator inventory

New keywords beyond doc 01: `is`, `and`, `or`, `not`, `use`, `guard`, `with`, `catch`,
`defer`, `cond`, `loop`, `where`, `into`, `derive`, `test`, `assert`, `errors`, `try`,
`const`, `static`, `type`, `mod`, `move`, `async`.

New operators and tokens beyond doc 01: `|>`, `??`, `??=`, `?.`, `.+` `.-` `.*` `./`
`.%`, `.(`, `...`, `%w[`, `#{`, `#[`, `##`, `##!`, `&:`, `..=`, `'label`.

Operator precedence follows **Rust's table unchanged** (doc 02 §5.2), with the new
operators placed as follows, loosest to tightest: the argument-list comma below everything;
`??` below `||`; `|>` below `??` and above assignment; `catch` at `??`'s level; the
broadcast operators at their scalar counterparts' levels; `?.` and `.(` at field-access
level. `|>` binds looser than every arithmetic and comparison operator, so a pipeline stage
never needs parenthesising, and it is deliberately placed so as not to collide with bitwise
`|` (doc 01 §15's requirement).

The argument-list comma sitting below every operator is what lets each argument of a
parenless call be a full expression (§5.1.1). `|>`, `??`, `catch` and the postfix
conditionals are additionally **terminators**: they end a parenless argument list rather
than being absorbed into its final argument.

---

## 5. Expressions and operators

### 5.1 Calls — parenless is canonical everywhere

**Doc 01 §6.1 and §6.2 are strengthened.** Doc 01 offers implicit calls and parenthesized
calls side by side, and §6.2 advises that parentheses "should be encouraged for ambiguous
nested expressions" without saying which expressions are ambiguous. That left the canonical
form undecided, which §16 cannot tolerate.

**`f x, y` is the canonical form of a call in every position** — statement, argument,
field value, match arm, pipeline stage. Parentheses are **grouping, not call syntax**, and
appear only where §5.1.2 requires them.

```
print "hello"
add 10, 20
connect host: "localhost", port: 8080
obj.method x, y
println! "{}, {}", a, b
spawn worker_count, () -> run!
```
```rust
println!("hello");
add(10, 20);
connect("localhost", 8080);
obj.method(x, y);
println!("{}, {}", a, b);
spawn(worker_count, || run());
```

Calls are resolved by the **whitespace-adjacency lexing** doc 02 §4.6 recommends: an
identifier immediately followed by `(` with no intervening space is a parenthesized call;
with a space, or with no `(` at all, it is a parenless call whose argument list begins
there. Emerald's plan 90 established that adjacency is the only one of the two candidate
strategies that keeps a nested parenless call legal in argument position, which doc 01
§6.4's own worked example requires. Redox's lexer is hand-written and already column-aware
for the off-side rule (doc 02 §5.1), so adjacency information is free.

#### 5.1.1 Extent of a parenless argument list

A parenless argument list extends rightward, commas separating arguments, until it reaches
one of:

- the end of the logical line (continuation rules per doc 02 §5.1 item 2);
- a `)`, `]` or `}` closing an enclosing delimiter;
- a **terminator operator**: `|>`, `??`, `catch`, or a postfix `if` / `unless` / `while` /
  `until`.

The comma therefore binds looser than every operator, so each argument may be a full
expression; and the terminator set is what allows a parenless call to be piped or defaulted
without parentheses:

```
send level, format! "{}: {}", code, msg
total = compute_base x, y |> clamp 0, 100
name = lookup id ?? "anonymous"
count = parse_int s catch 0
log "retrying" if attempts > 1
```
```rust
send(level, format!("{}: {}", code, msg));
let total = compute_base(x, y).clamp(0, 100);
let name = match lookup(id) { Some(__rdx_v) => __rdx_v, None => "anonymous" };
let count = match parse_int(s) { Ok(__rdx_v) => __rdx_v, Err(_) => 0 };
if attempts > 1 { println!("retrying"); }
```

#### 5.1.2 Where parentheses are required

Six cases, each because omitting them would change the program or lose a token. The
formatter inserts them (§16); writing them elsewhere is accepted and normalized away.

| # | Case | Example |
|---|---|---|
| 1 | A call nested in another parenless call's argument list | `add (multiply 2, 3), 4` |
| 2 | A call used as an operand of any operator, including `is` | `fib(n - 1) + fib(n - 2)` |
| 3 | A call carrying a postfix operator — `?`, `.await`, a field, a method | `parse(x)?`, `get(u).await` |
| 4 | Enum and tuple-struct **constructors**, always | `Some(x)`, `Ok(v)`, `Meters(3.0)` |
| 5 | Argument forwarding | `send(...)` |
| 6 | Where the list would otherwise swallow a following token | `[0u8; square(8)]` |

Case 1 is doc 01 §6.4's own rule, generalized. Without the parentheses,
`add multiply 2, 3, 4` would mean `add(multiply(2, 3, 4))`, because the inner list would
consume the remainder — the greedy resolution Ruby and CoffeeScript both adopt and both
treat as a known hazard. Redox does not adopt it: an unparenthesized nested parenless call
is **`RDX0055`**, a named diagnostic naming the two possible readings.

Case 4 deserves its reason stated. A constructor is not sugar for a call: it must match the
pattern that destructures it (§6.3), and a Redox constructor that reads differently from
the Rust pattern beside it would reintroduce exactly the mismatch doc 02 §5.2 identifies as
the worst available failure mode.

### 5.2 Zero-argument calls

Doc 01 §6.3's `!` suffix is canonical, with doc 02 §10.3's disambiguation rule: `ident!`
followed by `(`, `[`, `{`, a string literal, or an expression on the same line is a macro
invocation; `ident!` at the end of an expression is a zero-argument call.

```
user.save!
config.load!
Vec::new!
```
```rust
user.save();
config.load();
Vec::new();
```

`f()` is accepted and normalized to `f!`, except under §5.1.2 case 3, where a postfix
operator requires the parenthesized form — `next()?`, not `next!?`. Chains of two or more
zero-argument calls are canonically written with the pipeline operator (§16), so
`xs.iter!.max!.unwrap!` is normalized to `xs |> iter |> max |> unwrap`.

### 5.3 The pipeline operator — receiver semantics

**Doc 01 §15 is corrected.** Its worked example lowers to `sum(map(filter(numbers, …), …))`,
and none of `filter`, `map` or `sum` exists as a free function in Rust, so the example
cannot compile (doc 02 §10.8). `|>` instead places its left operand as the **receiver**:

```
result = numbers
  |> iter
  |> map (n) -> n * n
  |> filter (> 0)
  |> sum
```

```rust
let result = numbers
    .iter()
    .map(|n| n * n)
    .filter(|__rdx_1| __rdx_1.gt(&0))
    .sum();
```

Lowering rules:

- `x |> f(a, b)` → `x.f(a, b)`.
- `x |> f` → `x.f()`. A bare identifier in pipeline position is always a method call, not
  a field access — the position supplies what a type would otherwise have to.
- The operand is emitted once, so `|>` introduces no temporary and no re-evaluation.

### 5.4 Operator sections

A parenthesised partial operator application becomes a one-parameter closure. This is the
construct where the auto-ref mechanism of §1.1 is load-bearing, because a closure
parameter arrives by value from `map` and by reference from `filter`, and Redox cannot
know which.

**Arithmetic sections lower to the operator directly.** `std` provides reference
implementations for primitive arithmetic (`impl Add<i32> for &i32` and siblings), so one
spelling type-checks in both positions:

```
(&xs) |> map (+1)      →  xs.iter().map(|__rdx_1| __rdx_1 + 1)
```

**Comparison and equality sections lower to the trait method form**, because `std`
provides no corresponding `PartialOrd<T> for &T`, and so the operator form would
type-check in only one position:

```
(&xs) |> filter (> 0)  →  xs.iter().filter(|__rdx_1| __rdx_1.gt(&0))
(&xs) |> filter (== k) →  xs.iter().filter(|__rdx_1| __rdx_1.eq(&k))
```

| Section | Lowering |
|---|---|
| `(+ e)` `(- e)` `(* e)` `(/ e)` `(% e)` | `\|__rdx_1\| __rdx_1 <op> e` |
| `(> e)` `(>= e)` `(< e)` `(<= e)` | `\|__rdx_1\| __rdx_1.gt(&e)` / `.ge` / `.lt` / `.le` |
| `(== e)` `(!= e)` | `\|__rdx_1\| __rdx_1.eq(&e)` / `.ne(&e)` |

#### Subtraction sections and negative literals

`-` is the one operator whose prefix and infix forms collide, so `(-1)` and `(- 1)` would
otherwise be the same text. **Adjacency decides**, reusing §5.1's existing mechanism rather
than adding one:

| Written | Meaning |
|---|---|
| `(-1)` — no space | a negative literal |
| `(- 1)` — space | a subtraction section |

This matters more under §5.1 than it would otherwise, because sections are now routinely
passed as bare arguments, so `map (-1)` and `map (- 1)` are both ordinary code. The
formatter normalizes spacing to match the parsed meaning (§16), which makes the distinction
visible rather than merely legal.

Two disclosed limitations, stated rather than discovered later:

- The arithmetic forms rely on `std`'s reference implementations, which exist for primitives
  but not for arbitrary user types. A section over a user type whose `Add` is implemented
  only for the owned form is reported by rustc, through the source map, at the section's own
  location.
- **Reversed sections are not expressible.** `(1 - _)`-style sections, where the hole is the
  left operand, have no spelling: the section grammar places the hole implicitly on the left.
  Write the closure out. Adopting Scala's explicit placeholder would make both directions
  expressible and remove the adjacency rule above, at two characters per section; it is
  recorded in §23 as the venue if the adjacency rule proves troublesome in practice.

### 5.5 The pattern-test operator `is`

One operator serves both boolean testing and scoped binding; the target is chosen from
whether the pattern binds and where the expression sits — both syntactic facts.

```
# No binders → matches!
if status is Active
```
```rust
if matches!(status, Active) {
```

```
# Binders, condition position → if-let
if msg is Move(x, y)
  print x
```
```rust
if let Move(x, y) = msg {
    println!("{}", x);
}
```

```
# Binders plus a condition → let-chain
if msg is Move(x, _) && x > 0
```
```rust
if let Move(x, _) = msg && x > 0 {
```

The let-chain form requires Rust 1.88 and edition 2024 (§19). A derived consequence, not a
separate decision: `while x is Some(v)` lowers to `while let Some(v) = x`.

### 5.6 Chained comparison

Doc 01 §14 is retained, with doc 02 §6.3's lowering: the intermediate is bound by
reference so that single evaluation holds for `Copy` and non-`Copy` alike.

```
if 0 < age < 120  →  if { let __rdx_c0 = &age; 0 < *__rdx_c0 && *__rdx_c0 < 120 }
```

---

## 6. Patterns

### 6.1 `switch`

Doc 01 §18's `switch` with `=>` arms and `when` guards is retained, lowering to Rust
`match`.

### 6.2 Relational and logical patterns

Admitted from C#, lowered by binding the scrutinee to a generated temporary and emitting
guards.

```
label = switch age
  > 65                   => "senior"
  18..=64                => "adult"
  not 0                  => "minor"
  Teen or PreTeen        => "young"
  _                      => "newborn"
```
```rust
let label = match age {
    __rdx_s if __rdx_s > 65 => "senior",
    18..=64 => "adult",
    __rdx_s if !matches!(__rdx_s, 0) => "minor",
    Teen | PreTeen => "young",
    _ => "newborn",
};
```

| Pattern | Lowering |
|---|---|
| `> e`, `>= e`, `< e`, `<= e` | bind scrutinee, emit a comparison guard |
| `p1 or p2` | Rust's native `p1 \| p2` |
| `not p` | bind scrutinee, emit `!matches!(__rdx_s, p)` |
| `p and q`, at most one side binding | the binding pattern, with the other side as a guard |
| `p and q`, both sides binding | **rejected** — `RDX0042` |

Two-binder `and` is rejected because Rust has no pattern form that binds from both sides
of a conjunction, and synthesising one would require restructuring the arm in ways that
change binding modes. The rejection is a named diagnostic, not a silent miscompilation.

Guards in Rust take a shared borrow of their bindings and cannot move out of them. A guard
that attempts to is reported by rustc against the pattern's own source location through the
source map.

### 6.3 Record patterns

Two spellings, with different capabilities, because Rust struct patterns require a type
path that Redox cannot infer.

**Named — a real Rust pattern, with binding.**

```
switch p
  Point{x: 0, y} => y
  Point{x, y: 0} => x
  _              => 0
```
```rust
match p {
    Point { x: 0, y, .. } => y,
    Point { x, y: 0, .. } => x,
    _ => 0,
}
```

**Anonymous — field-access guards, test only.** No type path is needed, so the pattern
cannot become a Rust pattern and cannot bind.

```
if u is {active: true, age: > 18}  →  if u.active && u.age > 18
if u is {active: false}            →  if !u.active
```

A boolean field compared against a literal `true` or `false` emits the field or its
negation rather than an `== true` comparison. Attempting to bind inside an anonymous
record pattern is `RDX0043`.

### 6.4 Further pattern forms

All pass through to native Rust syntax.

| Form | Example | Rust |
|---|---|---|
| Slice | `[first, .., last]`, `[a, ...rest]` | `[first, .., last]`, `[a, rest @ ..]` |
| As-pattern | `whole @ Move(x, y)` | identical |
| Range | `1..10`, `'a'..='z'` | identical (§4.2) |
| Guard | `Some(n) when n > 0 =>` | `Some(n) if n > 0 =>` |
| Field pun | `Point{x, y}` | identical (§9.2) |

### 6.5 Destructuring

Doc 01 §11's forms are retained and lower to ordinary Rust patterns. With §4.2's
correction, rest patterns use `...`:

```
(a, b) = get_coordinates!       →  let (a, b) = get_coordinates();
[first, ...rest] = values       →  let [first, rest @ ..] = values;
Point{x, y} = p                 →  let Point { x, y } = p;
Point{x: px, y: py} = p         →  let Point { x: px, y: py } = p;
```

### 6.6 Exhaustiveness

Redox never synthesises a wildcard arm. A `switch` emits exactly the arms written, so a
missing variant is reported by rustc against the user's own arms. This costs nothing to
implement and is the posture Gleam and Elm treat as central; here it follows directly from
§2's "every check rustc can make, rustc makes."

---

## 7. Bindings and control flow

> **Every form in this section is an expression — plan 55** (2026-11-10). This
> section and doc 01 §8 both read as *statements*, and the parser had a
> statement/expression fork to match: an indented `if` could not be a value, a
> `case` arm needed a marker before a block, and a branch that produced one had
> to `return` it. Plan 55 deleted the fork — a block's value is its last
> element wherever the block appears, and a `()`-returning function ends in a
> written `()` (`RDX0113`). Two consequences are spelled out below: §7.2's `do`
> is retired as a construct and reserved as a token, and `then` is retired with
> it, so an `if` has one spelling. `return` stays, as Rust's early exit, and
> §7.1's else branch still requires one.

### 7.1 `guard`

Early-exit binding, built on `is` so that one pattern vocabulary serves `switch`, `if` and
`guard`. Lowers to Rust's let-else, whose else branch must diverge — exactly `guard`'s own
requirement.

```
load = (p: &str): Result<Config, Error> ->
  guard read(p) is Ok(raw) else
    return Err(Error::Unreadable)
  guard parse(raw) is Ok(cfg) else
    return Err(Error::Malformed)
  Ok(cfg)
```
```rust
fn load(p: &str) -> Result<Config, Error> {
    let Ok(raw) = read(p) else { return Err(Error::Unreadable) };
    let Ok(cfg) = parse(raw) else { return Err(Error::Malformed) };
    Ok(cfg)
}
```

The else branch may be any diverging expression — `return`, `break`, `continue`, or a
panic form (§12.6). A non-diverging else is reported by rustc against the `guard` line.

### 7.2 Block expressions and labels

An indented block is an expression, and gains an optional label permitting early exit with
a value. Both forms are native Rust: block expressions, and labelled block expressions
(Rust 1.65).

> **`do` is retired as a construct and reserved as a token — plan 55 leaf 7**
> (2026-11-10). This section read *"doc 01 §19's `do` is retained"*, and the
> snippet below opened `found = do 'search`. Since the fork above went, a
> block **is** an expression in every position, so no keyword announces one and
> the word earned nothing. It stays **reserved** — plan 60 claims `<-` as
> monadic bind and the Haskell meaning of `do` is the one still unclaimed, so
> spending it on an identifier would cost a breaking change later. Writing one
> is `RDX0114`. The label is the single capability `do` carried that nothing
> else provides, and it outlived the keyword.

```
found = 'search
  for c in candidates
    if c.matches q
      break 'search Some(c)
  None
```
```rust
let found = 'search: {
    for c in candidates {
        if c.matches(q) {
            break 'search Some(c);
        }
    }
    None
};
```

Labels apply equally to loops (§18), so `break 'outer` and `continue 'outer` reach an
enclosing loop.

### 7.3 `defer`

Runs a block at scope exit, in last-in-first-out order, on every exit path including
unwinding.

```
open = (path: &str): Result<(), Error> ->
  handle = acquire(path)?
  defer
    release handle
  process(handle)?
  Ok(())
```
```rust
let __rdx_defer_0 = __rdx::Defer::new(|| { release(handle); });
```

**Disclosed exception to doc 01 §36.** `defer` is the one construct in this surface that
emits a helper item rather than a pure rewrite: a scope-guard type implementing `Drop`.
Doc 01 §36 instructs that no Redox runtime scaffolding be introduced, and this is
scaffolding, however small. It is bounded as follows, and flagged in §23 for explicit
confirmation:

- Emitted **only** when `defer` is used in the file.
- Emitted once per file, into a generated `#[doc(hidden)] mod __rdx`.
- Depends on nothing outside `core` — it is a generic struct and a `Drop` impl, roughly
  eight lines.
- No alternative lowering preserves the semantics. Wrapping the remainder of the scope in
  a closure and calling the deferred block afterwards fails on early return and on panic,
  so it is not equivalent.

### 7.4 `cond`

A flat multi-branch form over independent conditions, from Elixir, with Haskell's
MultiWayIf as a second witness. Replaces the nesting drift of a long `else if` ladder, and
lowers to that ladder as an expression.

```
tier = cond
  spend > 10_000 => "platinum"
  spend > 1_000  => "gold"
  member         => "silver"
  else           => "basic"
```
```rust
let tier = if spend > 10_000 { "platinum" }
    else if spend > 1_000 { "gold" }
    else if member { "silver" }
    else { "basic" };
```

### 7.5 `with` — receiver-implicit mutation

**Doc 01 §20 is replaced, and doc 02 §10.7's decline is reversed.** Doc 01's `with`
resolved bare names against the subject's fields, which requires field knowledge Redox
does not have. MoonScript's actual construct uses a leading dot, which is explicit and
needs no type information at all.

```
with response.headers
  .insert "content-type", "application/json"
  .insert "x-request-id", id
```
```rust
let __rdx_w0 = &mut response.headers;
__rdx_w0.insert("content-type", "application/json");
__rdx_w0.insert("x-request-id", id);
```

The subject is evaluated exactly once into a generated binding. A bare name inside a
`with` block refers to the enclosing scope, not to the subject — only a leading dot
addresses the subject, which is what makes the construct type-free.

Cascades (Dart, Smalltalk) and scope functions (Kotlin's `apply`/`also`, Ruby's `tap`,
Clojure's `doto`) are **declined**: each solves the same problem with a second spelling and
no new capability, and §16's canonicalization requires one form per construct.

### 7.6 `use` — callback flattening

From Gleam, with Koka's `with` statement as a second witness. Rewrites the remainder of the
enclosing block into a closure passed as the final argument, flattening arbitrary nesting.

```
run = (): Result<(), Error> ->
  use conn <- pool.with_connection
  use tx <- conn.transaction
  tx.execute(sql)?
  Ok(())
```
```rust
fn run() -> Result<(), Error> {
    pool.with_connection(|conn| {
        conn.transaction(|tx| {
            tx.execute(sql)?;
            Ok(())
        })
    })
}
```

Rules:

- `use a, b <- f(x)` passes a two-parameter closure; the binder list becomes the closure's
  parameters.
- The closure is the **final** argument, after any written arguments.
- Everything after the `use` line in the same block becomes the closure body.

**Disclosed trap, rejected rather than documented away.** A `return` inside a `use` tail
would return from the generated *closure*, not from the enclosing function — a silent
change of meaning. `return` in a `use` tail is therefore `RDX0044`. `?` is unaffected,
because it propagates out of the closure, which is the intended behaviour and is why the
example above type-checks.

### 7.7 Loops

Doc 01 §8's `while`, `until`, `for` and postfix forms are retained. Rust's `loop` is added
(§18), which was previously unreachable and is required by the break-with-value idiom.

```
loop
  chunk = next()?
  break if chunk.empty
```
```rust
loop {
    let chunk = next()?;
    if chunk.empty { break; }
}
```

---

## 8. Functions

### 8.1 Multi-clause definitions

From Haskell, Elixir and Erlang. Several clauses share a name; they merge into one Rust
function whose body matches on the argument tuple.

```
fib = (0: u64): u64 -> 0
fib = (1: u64): u64 -> 1
fib = (n: u64): u64 -> fib(n - 1) + fib(n - 2)
```
```rust
fn fib(__rdx_a0: u64) -> u64 {
    match __rdx_a0 {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}
```

Rules:

- All clauses must share arity and declared parameter and return types. The first clause's
  signature is authoritative; a mismatch is `RDX0045`.
- Clauses must be contiguous. An intervening item is `RDX0046`.
- Each clause may carry a `when` guard, emitted as a match guard.
- Multiple parameters match on a tuple of the generated bindings.
- Exhaustiveness is rustc's to check (§6.6); no wildcard clause is synthesised.

### 8.2 Labelled arguments

From Gleam, Swift and OCaml. Rust has no labelled arguments, so labels are **erased** and
arguments passed positionally. Labels must therefore appear in the parameter's declared
order.

```
connect host: "localhost", port: 8080  →  connect("localhost", 8080)
```

**Honest scope limit.** Detecting out-of-order labels requires the callee's signature.
Redox can see it when the declaration is in the same compilation unit, and reports
`RDX0051` then. For a callee in another file or another crate, the labels are erased
unconditionally and an incorrect order surfaces as a rustc type error, remapped to the call
site. This is stated rather than promised away: the check is best-effort, the erasure is
not.

### 8.3 `where` — local definitions

From Haskell. Appearing **after** a function body, indented, `where` introduces local
definitions, letting the principal expression read first and its helpers follow. §14.2
resolves the collision with Rust's bound clause by position.

```
largest = <T>(xs: &[T]): &T
  where T: Ord ->
    best
  where
    best = xs |> iter |> max |> unwrap
```
```rust
fn largest<T>(xs: &[T]) -> &T
where
    T: Ord,
{
    let best = xs.iter().max().unwrap();
    best
}
```

Lowering rules:

- A `where` definition that is a **function** becomes a nested `fn`, hoisted above the
  body — so mutual recursion among them is permitted, as in Haskell.
- A `where` definition that is a **value** becomes a `let` emitted before the body
  expression, in written order. A forward reference between value definitions is
  `RDX0047`, because Rust `let` bindings are ordered and synthesising a correct order
  would require dependency analysis this surface does not admit.

### 8.4 Argument forwarding

From Ruby. Expressible in Rust only for a known arity, so it lowers by repeating the
enclosing function's declared parameter list verbatim — which covers thin wrappers and
delegation.

```
log_and_send = (level: Level, msg: &str) ->
  record level, msg
  send(...)
```
```rust
fn log_and_send(level: Level, msg: &str) {
    record(level, msg);
    send(level, msg);
}
```

`...` is legal only inside a function whose parameter list is written out; in a function
taking `...` itself it is `RDX0048`.

### 8.5 Method references

From Ruby's symbol-to-proc. `&:name` becomes a closure calling the named **method** on its
argument. Methods only — a field has no call form, and distinguishing them would need type
information.

```
(&users) |> map &:name  →  users.iter().map(|__rdx_1| __rdx_1.name())
```

The receiver is passed as written, so auto-ref applies (§1.1) and one spelling works in
both by-value and by-reference closure positions.

### 8.6 Closures

Doc 01 §5.4's `->` and `=>` are retained. "Receiver context", left undefined by doc 01 and
flagged in doc 02 §10.11, is defined here as **the body of a method inside an `impl` or
`class` block** — the same definition governing `@` and `@@` in doc 01 §16.

Two forms are added (§18), both previously unreachable:

```
move (x: i32) -> x + captured      →  move |x: i32| x + captured
(x: i32): i64 -> x as i64          →  |x: i32| -> i64 { x as i64 }
```

### 8.7 Default arguments remain declined

Doc 01 §5.5 proposed them; doc 02 §10.6 declined them. The decline stands, and §22 records
it. A synthesised wrapper fabricates API surface the user never wrote, which breaks trait
implementations, rustdoc and go-to-definition. Labelled arguments (§8.2) address the
readability motivation; `Option<T>` parameters or a builder address the rest.

---

## 9. Data declarations

### 9.1 Struct initialization

Doc 01 §17.2's indented block is retained. Functional update — Rust's `..base` — is reached
by writing the base as a spread entry, reusing §4.2's token.

```
p = Point
  x: 10
  y: 20

p2 = Point
  x: 99
  ...p
```
```rust
let p = Point { x: 10, y: 20 };
let p2 = Point { x: 99, ..p };
```

The spread entry must be last; elsewhere it is `RDX0052`. At most one is permitted, as in
Rust. C#'s standalone `with` *expression* is not adopted — the spread entry covers the same
ground without a second construct.

### 9.2 Field punning

From Haskell's NamedFieldPuns and Ruby 3.1's hash shorthand. A bare field name means
"take the same-named binding", in both directions. The pattern side is native Rust; the
construction side is a one-line expansion.

```
# Construct
p = Point
  x
  y
                  →  let p = Point { x: x, y: y };
# Destructure
Point{x, y} = p   →  let Point { x, y } = p;
```

Haskell's RecordWildCards is **not** adopted: a wildcard introduces names that appear
nowhere in the source, which conflicts with the reachability-positive admission test
reading of readable generated code.

### 9.3 Primary constructors

From C# 12, with Gleam and Haskell's positional constructors as further witnesses. A
parameter list on the declaration header declares the fields and synthesises the
constructor — which is what doc 01 §17.7 already promises `class` is sugar for.

```
class Counter(value: i32)
  increment = (self: &mut Self) ->
    self.value += 1
```
```rust
struct Counter { value: i32 }

impl Counter {
    fn new(value: i32) -> Self { Counter { value } }
    fn increment(&mut self) { self.value += 1; }
}
```

Rules: the synthesised `new` is omitted if the body defines `new` itself; its visibility
matches the type's; a header parameter list and an explicit field block are mutually
exclusive (`RDX0053`).

### 9.4 Enums

Doc 01 §17.3 covers unit and tuple variants. Struct variants and explicit discriminants are
added (§18), both pure passthrough.

```
enum Event
  Click{x: i32, y: i32}
  Key(char)
  Close
```
```rust
enum Event {
    Click { x: i32, y: i32 },
    Key(char),
    Close,
}
```

### 9.5 `derive`

A first-class clause on a declaration, lowering to a Rust derive attribute — the dominant
case, given doc 02 §10.2 found attributes absent entirely.

```
struct Point derive Debug, Clone, PartialEq
  x: f64
  y: f64
```
```rust
#[derive(Debug, Clone, PartialEq)]
struct Point { x: f64, y: f64 }
```

General attributes remain available by verbatim passthrough (§13.2); the clause is a
spelling for the common case, not a replacement.

### 9.6 Visibility

**Doc 01 §22 is extended.** It offers only `export` → `pub`, which cannot express Rust's
scoped visibilities or per-field visibility, and so cannot support real library work.

| Redox | Rust |
|---|---|
| `export` | `pub` |
| `export(crate)` | `pub(crate)` |
| `export(super)` | `pub(super)` |
| `export(in path::to)` | `pub(in path::to)` |
| (absent) | private |

Per-field and per-variant visibility use the same markers inside a declaration body.

### 9.7 Aliases, unit structs and tuple structs

All three were unreachable; all three are pure passthrough. Tuple structs matter
disproportionately because the newtype idiom underpins type-safe identifiers, units, and
the orphan-rule workaround.

```
type Handler = Box<dyn Fn(Req) -> Res + Send>
struct Meters(f64)
struct Marker
```
```rust
type Handler = Box<dyn Fn(Req) -> Res + Send>;
struct Meters(f64);
struct Marker;
```

---

## 10. Collections

### 10.1 Two comprehension levels

Doc 01 §9's bracket comprehension is retained for whole-expression construction, and
Dart's element-level `if`/`for` is added for literals that mix written and computed
elements. They compose rather than compete.

**Comprehension — an expression over a source.**

```
pairs = [
  (x, y)
  for x in 1..3
  for y in 1..3
  when x != y
]
```
```rust
let pairs = (1..3)
    .flat_map(|x| (1..3).filter(move |y| x != *y).map(move |y| (x, y)))
    .collect::<Vec<_>>();
```

**Collection-`if`/`for` — elements within a literal.**

```
args = [
  "--release"
  "--verbose" if verbose
  for f in files
  "--"
]
```
```rust
let args = {
    let mut __rdx_v = Vec::new();
    __rdx_v.push("--release");
    if verbose { __rdx_v.push("--verbose"); }
    for f in files { __rdx_v.push(f); }
    __rdx_v.push("--");
    __rdx_v
};
```

The insertion method follows the literal shape: `push` for `[...]`, `insert` for `#{...}`
and `{...}`.

### 10.2 Broadcasting

From Julia. Admitted with a **mandatory borrow marker**, extending doc 01 §9.7's
discipline: the user states `into_iter` / `iter` / `iter_mut` by writing the receiver, and
Redox never chooses.

```
(&xs) |> map.(n) -> n * 2      →  xs.iter().map(|n| n * 2)
(&xs) .+ 1                     →  xs.iter().map(|__rdx_e| __rdx_e + 1)
(&a) .+ (&b)                   →  a.iter().zip(b.iter()).map(|(__rdx_l, __rdx_r)| __rdx_l + __rdx_r)
```

Broadcasting yields an **iterator**, not a collection: it is a chain stage, and the caller
collects. An unmarked receiver is `RDX0054`, by exact analogy with doc 01 §9.7's rule
against a silent `into_iter`.

### 10.3 Collect targets

**Doc 02 §6.3 is extended.** Always emitting `collect::<Vec<_>>()` leaves every other
container unreachable through comprehension syntax. The literal shape supplies a default;
an `into` clause overrides it. Both produce a turbofish, so a comprehension remains a
self-contained expression valid in argument position.

| Written | Emitted |
|---|---|
| `[n for n in xs]` | `.collect::<Vec<_>>()` |
| `{k, v for x in xs}` | `.collect::<std::collections::HashMap<_, _>>()` |
| `#{n for n in xs}` | `.collect::<std::collections::HashSet<_>>()` |
| `[n for n in xs into BTreeSet]` | `.collect::<BTreeSet<_>>()` |
| `[c for c in cs into String]` | `.collect::<String>()` |

Doc 01 §10's clause "unless a type context specifies another map" is **deleted**: no type
context exists, and `into` replaces it.

### 10.4 Literals

| Form | Rust | Note |
|---|---|---|
| `[1, 2, 3]` | `[1, 2, 3]` | **corrected** — see below |
| `[1, 2, 3] : Vec<i32>` | `Vec::from([1, 2, 3])` | doc 18 §7's ascribed literal |
| `{k: v}` | `HashMap` from iterator | **deleted** by doc 18 §7 |
| `#{1, 2, 3}` | `HashSet::from([1, 2, 3])` | **deleted** by doc 18 §7 — `[…] : HashSet<_>` |
| `[0, ...xs, 9]` | chained `extend` / iterator chain | **deleted** — `...` leaves the language |
| `%w[get post put]` | `["get", "post", "put"]` | **deleted** by doc 18 §7 |
| `[0u8; 1024]` | `[0u8; 1024]` | added (§18); pure passthrough |

> **Corrected 2026-11-09 by plan 29 (§10.1).** The first row said
> `[1, 2, 3]` → `vec![1, 2, 3]`. It does not and must not: doc 18 §7 replaced
> four of this table's six rows with `[…] : C<_>`, and that form needs the bare
> literal to be an **array**, because `Vec::from` and `HashSet::from` both take
> `[T; N]`. A bare bracketed literal is therefore an array, and `:: Vec<T>` is
> how it becomes a `Vec`.
>
> The stale row mattered: plan 29 built its case for emitting `[v; n]` verbatim
> on the premise that a list meant `vec!`, and the premise was gone. The
> requirement survived the correction — a repeat is a stack array either way —
> but the reasoning had to be rebuilt, which is what a stale table costs.

`%i[...]` symbol arrays are **not** adopted: Rust has no symbol type, so there is nothing
to lower to. This is the clearest instance of the admission test's reachability criterion
working in reverse — a construct with no target is not sugar, it is a new runtime concept.

---

## 11. Strings

### 11.1 Interpolation and format specifications

Doc 01 §7.1's `#{expr}` is retained. Rust's own format-spec grammar is carried inside the
interpolation and copied verbatim into the generated template, making every Rust format
spec reachable without a second construct.

```
"total: #{amount:.2}"   →  format!("total: {:.2}", amount)
"debug: #{cfg:?}"       →  format!("debug: {:?}", cfg)
"|#{name:>8}|"          →  format!("|{:>8}|", name)
"hello #{name}"         →  format!("hello {}", name)
```

Text after `:` up to the closing brace is spec text, passed through unchanged. A malformed
spec is reported by rustc against the interpolation's own location through the source map.

### 11.2 Literal kinds

Doc 01 covers `"..."` and `"""..."""`. Four Rust literal kinds were unreachable; all are
pure passthrough.

| Form | Rust | Why it is needed |
|---|---|---|
| `r"..."`, `r#"..."#` | identical | regex patterns, paths, embedded JSON — and more necessary here than in Rust, since `#{` is live inside ordinary strings |
| `'c'` | identical | Rust's `char` had no spelling at all |
| `b'x'`, `b"..."` | identical | binary protocol and parser work |
| `c"..."` | identical | FFI, which doc 01 §26 requires |

Doc 01 §7.2's `"""` multiline form with common-indentation stripping is retained, and must
preserve line numbering (doc 02 §5.1 item 5) because the source map depends on it.

### 11.3 No sigils beyond `%w`

Elixir's extensible sigils and Ruby's regex literals are **declined**. A regex literal
cannot lower without naming the `regex` crate, which would have Redox inject a dependency
the user never declared — the one thing §2's second admission criterion forbids
absolutely. Regular expressions remain ordinary crate calls:

```
re = Regex::new(r"^\d+$")?
```

### 11.4 Comments and doc comments

| Redox | Rust |
|---|---|
| `# line` | `// line` |
| `### … ###` | `/* … */` |
| `## outer doc` | `/// outer doc` |
| `##! inner doc` | `//! inner doc` |

Doubling the comment marker for documentation mirrors Rust's own relationship between `//`
and `///`. All four forms are trivia attached to AST nodes and re-emitted (§4.1), which is
doc 02 §10.1's correction to doc 01 §4.1.

---

## 12. Errors

Redox keeps Option and Result visually distinct and non-interchangeable: `??` serves
Option, `catch` serves Result, and both lower to a `match` so neither depends on method
arity or method resolution.

### 12.1 `??` and `??=`

`Option`-only, confirming doc 02 §6.3's lowering.

```
name = opt ?? "anonymous"
```
```rust
let name = match opt {
    Some(__rdx_v) => __rdx_v,
    None => "anonymous",
};
```

```
mut cached = None
cached ??= compute!
```
```rust
if cached.is_none() { cached = Some(compute()); }
```

The `match` form is preferred over `unwrap_or_else` for three reasons, recorded because the
choice is not obvious: the closure arity of `unwrap_or_else` differs between `Option` and
`Result`, so a method lowering would be type-dependent; a user type carrying an inherent
`unwrap_or_else` could otherwise hijack the meaning; and the fallback's laziness is
explicit in the generated code.

### 12.2 `catch`

`Result`-only, from Zig, mirroring `??`.

```
n = parse s catch 0
```
```rust
let n = match parse(s) {
    Ok(__rdx_v) => __rdx_v,
    Err(_) => 0,
};
```

`catch` is a terminator operator (§5.1.1), so the parenless argument list ends at it and
no parentheses are needed around `parse s`.

```
n = parse s catch (e) ->
  log e
  0
```
```rust
let n = match parse(s) {
    Ok(__rdx_v) => __rdx_v,
    Err(e) => { log(e); 0 }
};
```

Doc 01 §13.3's `?` is unchanged and remains the propagation mechanism.

### 12.3 Optional chaining

Doc 01 §13.1's `?.` is retained with doc 02 §6.3's lowering, which delegates the entire
`and_then`/`map` unification to rustc via `?` inside a closure returning `Option`. Nested
options are not flattened, satisfying doc 01 §13.1, because `?` does not flatten.

**Doc 02 §13 question 7 is resolved: the chain borrows and never clones.**

```
n = user?.profile?.name
```
```rust
let n = (|| Some(&user.as_ref()?.profile.as_ref()?.name))();
```

> **Corrected 2026-11-09 by plan 28 (§10.1).** This section published
> `.as_ref()` on the **final** link as well. That does not compile when the last field is
> not an `Option` — `.as_ref()` then means `AsRef::as_ref`, which `String` does not
> implement for `String` — and choosing between the two meanings requires a type, so the
> published form fails §2's first admission test. The same test §12.1 calls *"decisive
> rather than merely persuasive"* when rejecting `unwrap_or_else` three sections earlier.
>
> The final link therefore takes **no** method and the whole path takes a leading `&`, which
> borrows whatever is there whatever its type. The claim this section makes is unchanged:
> the chain borrows and never clones.
>
> One consequence worth stating, because the phrasing below suggests otherwise: a final
> field of type `Option<String>` yields `Option<&Option<String>>` — nested once, borrow
> outside — and not `Option<Option<&String>>`.

Owning the result is written explicitly:

```
n = user?.profile?.name.clone!
```

This keeps doc 01 §25.1's no-silent-clone rule absolute. It is more verbose in the common
`String` case, and that cost is visible at the call site rather than hidden in the
lowering.

### 12.4 `try` blocks

A block in which `?` returns from the block rather than the enclosing function. Rust has
wanted this for years and still has it only unstably; lowering to an immediately-invoked
closure gives Redox it on stable Rust today.

```
result = try
  a = parse(x)?
  b = parse(y)?
  Ok(a + b)
```
```rust
let result = (|| {
    let a = parse(x)?;
    let b = parse(y)?;
    Ok(a + b)
})();
```

### 12.5 Typed domain errors

An error-type declaration generating the enum together with its `Display`, `Error` and
`From` implementations — the boilerplate every Rust project either writes by hand or pulls
a derive crate for. Generated from a declaration, naming nothing outside `std`.

```
errors ConfigError
  Missing(String)       "missing key: #{0}"
  Invalid(String, u32)  "invalid value #{0} at line #{1}"
  from io::Error as Io  "io error: #{0}"
```
```rust
#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    Invalid(String, u32),
    Io(io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(_0) => write!(f, "missing key: {}", _0),
            ConfigError::Invalid(_0, _1) => write!(f, "invalid value {} at line {}", _0, _1),
            ConfigError::Io(_0) => write!(f, "io error: {}", _0),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self { ConfigError::Io(e) }
}
```

This generates items for a user-declared type, as `derive` does; it is not runtime
scaffolding in the sense of §7.3.

### 12.6 Panic family

Rust's panic and assertion macros were reachable only through raw macro syntax. Spellings
are added because partial code and invariant assertions are routine.

| Redox | Rust |
|---|---|
| `todo` | `todo!()` |
| `unimplemented` | `unimplemented!()` |
| `unreachable` | `unreachable!()` |
| `panic "message"` | `panic!("message")` |
| `assert cond` | `assert!(cond)` |
| `assert a == b` | `assert_eq!(a, b)` |
| `assert a != b` | `assert_ne!(a, b)` |

### 12.7 `unwrap` discouragement

`unwrap` and `expect` are lintable rather than prohibited: the ergonomic path is `?`, `??`
and `catch`, and a panicking accessor produces a warning carrying its own diagnostic code
(`RDX1001`), suppressible per call site. This is Gleam and Elm's safety posture expressed
as a lint, and it costs nothing because no analysis is involved.

---

## 13. Modules and metaprogramming

### 13.1 Paths — correcting doc 01 §22

**Doc 01 §22 is corrected.** Doc 02 §10.4 found three inconsistent path conventions in the
specification: `::` in expressions (§4.4), `.` in module paths (§13.3), `.` in imports
(§22), and a direct contradiction between `Vec.new!` (§6.3) and `Vec::new!` (§4.4). Redox
uses `::` everywhere, so `.` unambiguously means field or method access — which §5.2,
§6.3 and §8.5 all depend on.

```
import std::collections::HashMap
import std::io::{self, Read, Write}
import std::collections::HashMap as Map
import crate::models::*
export import crate::models::User
```
```rust
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::collections::HashMap as Map;
use crate::models::*;
pub use crate::models::User;
```

### 13.2 Attributes

**Doc 02 §10.2's critical gap is closed.** Any `#[...]` or `#![...]` line is copied through
verbatim, so every present and future Rust attribute is reachable with no per-attribute
work and interoperability (doc 01 §26) becomes unconditional.

```
#[cfg(test)]
#[tokio::main]
#[serde(rename = "id")]
#[allow(clippy::too_many_arguments)]
#![no_std]
```

The `derive` clause of §9.5 is a native spelling for the dominant case, not a replacement.

### 13.3 Compile-time evaluation

Redox exposes **Rust's `const` surface and nothing more**. Compile-time evaluation is
performed entirely by rustc, so Redox never executes user code and remains a translator
rather than an interpreter.

```
const MAX: usize = 1024
const square = (n: u32): u32 -> n * n
buffer = [0u8; square(8)]
```
```rust
const MAX: usize = 1024;
const fn square(n: u32) -> u32 { n * n }
let buffer = [0u8; square(8)];
```

`static` and `static mut` take the same modifier position. Zig- and Emerald-style
transpile-time evaluation is **declined**: it would require shipping a literal-only
interpreter, a scope expansion of the same order as the type checker doc 02 §12.4 declines
permanently, and Emerald's own plan 61 needed a step-limit flag for it.

Doc 01 §23's macro **invocation** syntax is retained. Macro **definition** has no native
spelling and uses doc 01 §23.1's inline-Rust escape hatch — recorded in §23 as an open
item, since a language whose own macros require the escape hatch is a candidate for a
future round.

### 13.4 Module-scope bindings — correcting doc 01 §4.3

**Doc 01 §4.3 is corrected**, per doc 02 §10.5. Two defects: Rust forbids `let` at module
scope, so *every* top-level binding must be `const` or `static` regardless of case; and
Rust `const` and `static` require an explicit type, so the specification's `PI = 3.14159`
cannot be lowered at all.

Module-scope bindings therefore require an explicit type annotation and lower to `const`
by default. The uppercase-name convention is a lint, not semantics.

### 13.5 Module organization

Three further forms, all previously unreachable, all passthrough: inline `mod` blocks
(needed for the near-universal test-module convention), glob imports, and brace-nested
import trees.

```
mod parser
  export parse = (s: &str): Ast -> ...

  #[cfg(test)]
  mod tests
    use super::*
    test "parses an empty input"
      assert parse("").is_empty!
```

---

## 14. Types, generics, async and traits

### 14.1 Type expressions

Type expressions are **copied through verbatim**, confirming doc 01 §4.4 and §26. Every
present and future Rust type form is reachable for free, and nothing can drift.

```
users: Vec<User>
index: HashMap<String, Vec<u8>>
callback: &dyn Fn(i32) -> i32
boxed: Box<dyn Error + Send + Sync>
slice: &'a mut [T; 4]
produce: impl Iterator<Item = u8>
```

Emerald's bracket type arguments (`Array[T]`) are declined: they would require translating
every type expression, risk lagging Rust's type grammar, and collide with indexing and the
array literal.

### 14.2 `where` — two roles, resolved by position

Rust uses `where` for trait bounds; §8.3 adopts Haskell's `where` for local definitions.
The two positions are grammatically distinct, so no lookahead is required and both features
keep the familiar keyword:

| Position | Role |
|---|---|
| In the signature, before the body arrow | trait bounds (Rust) |
| Indented after the body | local definitions (Haskell) |

§8.3 shows both in one function.

### 14.3 Async — correcting doc 01 §24

**Doc 01 §24 is corrected** from prefix `await` to Rust's postfix `.await`. Postfix
composes with method chains, with `?`, and with the receiver pipeline of §5.3; prefix
requires parenthesising to do any of them. It is also verbatim passthrough.

```
fetch = async (id: u64): Result<User, Error> ->
  http::get("/users/#{id}")
    .await?
    .json()
    .await?
```

Redox provides no executor and no runtime (doc 01 §24), so `async` blocks and `async fn`
pass through and the runtime is the user's own dependency. A concurrent-await construct is
declined for the dependency reason of §11.3: every joining form in Rust comes from a
runtime crate.

### 14.4 Traits and implementations

Doc 01 §17.4 specifies trait signatures only. Four forms were unreachable, which together
made most of the standard trait surface impossible to implement without the escape hatch.
All are passthrough.

```
trait Shape: Debug
  type Unit
  const SIDES: u32

  area = (self): f64
  describe = (self): String ->
    "a shape with #{@@SIDES} sides"

impl<T: Display> Describe for Wrapper<T>
  describe = (self): String -> "#{self.inner}"
```

| Form | Why it is needed |
|---|---|
| Default method bodies | how most real traits are written |
| Associated types and constants | required by `Iterator`, `Deref`, most of `std` |
| Supertraits | trait hierarchies |
| Generic impls with bounds | the blanket-impl pattern, central to Rust library design |
| `impl Trait` in argument and return position | the idiomatic way to take a closure or return an iterator |

Operator overloading needs no new construct: it is an ordinary trait implementation, which
doc 01 §17.5 already provides. Custom operator *declaration* (Haskell, Scala) is declined
by necessity — Rust has no mechanism for it, so there is no lowering target.

---

## 15. Testing

A named `test` block lowering to an attributed function in a generated test module,
together with the assertion spellings of §12.6.

```
test "adds two numbers"
  assert add(1, 2) == 3

test "rejects a negative input"
  assert add(-1, 2).is_err!
```
```rust
#[cfg(test)]
mod __rdx_tests {
    use super::*;

    #[test]
    fn adds_two_numbers() { assert_eq!(add(1, 2), 3); }

    #[test]
    fn rejects_a_negative_input() { assert!(add(-1, 2).is_err()); }
}
```

The function name is a deterministic slug of the description; a collision appends an index.
The module is emitted once per file, only when a `test` block is present.

Property-test and benchmark blocks are **declined** for §2's dependency reason: both lower
to calls into crates the user must declare (`proptest` or `quickcheck`; `criterion`), so
either would compile only in projects that happen to have them. Both remain writable as
ordinary attributed functions via §13.2.

---

## 16. Canonical forms

Emerald's plan 90 established the hazard precisely: its formatter always re-added
parentheses because it ran off a bare AST with no type information, so example code
rewritten in the terser style would be silently reverted by the same CI gate that checked
it. The resolution is to decide the canonical spelling **before** writing examples.

**Every construct has exactly one spelling the formatter emits.** Alternatives are accepted
as input and normalized away, which makes `redox fmt` the migration tool doc 02 §4.5 calls
for.

| Accepted | Canonical | Why |
|---|---|---|
| `f(x, y)` in any position | `f x, y` | §5.1 — parenless is the house style everywhere, not only in statement position |
| `f()` | `f!` | §5.2 — except under §5.1.2 case 3, where a postfix operator requires `f()` |
| `x.a!.b!.c!` | `x \|> a \|> b \|> c` | §5.2 — chains of two or more stages use the pipeline |
| parentheses on a call not covered by §5.1.2 | removed | parentheses are grouping, not call syntax |
| a nested parenless call | `RDX0055`, not reparsed | §5.1.2 case 1 — the two readings are not interchangeable |
| `(- 1)` / `(-1)` spacing | normalized to the parsed meaning | §5.4 — makes the section-versus-literal distinction visible |
| `Point{x: 1}` bound to a name | block form | doc 01 §17.2 |
| block form inside an expression | `Point{x: 1}` | a block cannot nest in an argument list |
| `1...10` | `1..=10` | §4.2 |
| `await f()` | `f().await` | §14.3 |
| `Vec[T]` | `Vec<T>` | §14.1 |
| `x.map(f)` where `x` is a chain head | `x \|> map f` | §5.3, chains of two or more stages |
| repeated receiver, three or more times | `with` block | §7.5 |
| `== true` / `== false` | field or its negation | §6.3 |

The formatter is **trivia-preserving**, not a pure AST pretty-printer. Doc 02 §3.2 records
that Emerald's formatter loses every non-doc comment because its lexer discards them; for
Redox this would be a product defect rather than an annoyance, since the same trivia must
also reach the generated `.xrs`. One investment, two consumers.

---

## 17. Diagnostics

Stable codes, per doc 02 §10.13. Those introduced by this surface:

| Code | Condition | §|
|---|---|---|
| `RDX0042` | `and` pattern with binders on both sides | 6.2 |
| `RDX0043` | binding attempted in an anonymous record pattern | 6.3 |
| `RDX0044` | `return` inside a `use` tail | 7.6 |
| `RDX0045` | multi-clause signature mismatch | 8.1 |
| `RDX0046` | non-contiguous clauses of one function | 8.1 |
| `RDX0047` | forward reference between `where` value definitions | 8.3 |
| `RDX0048` | `...` forwarding in a function that itself takes `...` | 8.4 |
| `RDX0051` | labelled arguments out of declared order (best-effort) | 8.2 |
| `RDX0052` | spread entry not last in a struct initializer | 9.1 |
| `RDX0053` | primary constructor combined with an explicit field block | 9.3 |
| `RDX0054` | broadcast receiver without a borrow marker | 10.2 |
| `RDX0055` | nested parenless call without disambiguating parentheses | 5.1.2 |
| `RDX0063` | attribute in a position Rust will reject | 13.2 |
| `RDX1001` | `unwrap` or `expect` used (warning) | 12.7 |

Every diagnostic names a `.sdx` location, which depends on the source map of doc 02 §6.5 —
restated here because this surface's multi-line expansions (`use`, `defer`, `with`,
multi-clause merging, collection-`if`/`for`) widen the gap between source and generated
line numbers considerably. **Line-preserving emission where feasible** (doc 02 §6.5 item 1)
becomes correspondingly more important, and the constructs above are precisely where it
cannot hold.

---

## 18. Rust reachability

Doc 02 §4.1 proposed grading Rust's grammar `PASSTHROUGH` / `SUGARED` / `UNREACHABLE`, on
the grounds that any `UNREACHABLE` construct is one a Redox user cannot write — a direct
violation of doc 01 §26. That inventory, as it stands after this document:

**Closed by this document.** Attributes (§13.2), granular visibility (§9.6), `where`
clauses (§14.2), `impl Trait` (§14.4), match ergonomics — `@` bindings, ~~`|`
alternatives~~ (**corrected by plan 46 §1.2: see below**),
slice patterns (§6.4), `loop` and labelled break (§7.2, §7.7), `const`/`static`/`const fn`
(§13.3), type aliases, unit structs, tuple structs (§9.7), `move` closures and explicit
closure return types (§8.6), raw, char, byte and C-string literals (§11.2), tuple-struct
and struct enum variants with discriminants (§9.4), inline `mod` blocks, re-exports, glob
imports, nested import trees (§13.5), array repeat (§10.4), default trait bodies,
associated types and constants, supertraits, generic impls (§14.4), format specifications
(§11.1).

**Closed by plan 45**, 2026-11-09 (`docs/39 - 45 - Reachability Residue.md`): `unsafe`
blocks and `unsafe fn` — doc 01 §25.4's choice between "native Rust syntax or the inline
Rust escape hatch" settled on native syntax; `extern` blocks and FFI declarations;
`union`; raw identifiers (`r#type`). And **inline Rust itself**, which is the finding that
reshaped the row: see below.

**Closed earlier**: `async` blocks, by plan 39. This paragraph listed them until plan 45
checked.

**Declined, permanently**: macro **definition**, under §23 item 3's own alternative —
*"accept permanently that Redox macros require inline Rust"*. Usable as a decline only
because plan 45 leaf 1 made inline Rust real.

**Still open**: macro **invocation**. Measured by plan 45 §1.5 — `vec![1, 2, 3]`,
`format! "hi"`, parenthesised, ascribed and as a function body are all rejected, while
`std::vec::Vec::new ()` is accepted, because doc 18 §7 retired `!` as a suffix and left it
meaning boolean negation. `assert! true` only appears to work: it reads as the `assert`
keyword applied to `!true`. Plans 29 §6 and 39 §10.5 each found this and neither assigned
it to a row; it needs a *spelling* decision here before a row can implement one. Reachable
meanwhile through inline Rust, which is what makes doc 01 §26's "Rust macros" requirement
met rather than outstanding.

> **Correction, plan 45 §1.2.** This paragraph used to end *"Each is reachable today only
> through doc 01 §23.1's escape hatch"*, and §23 items 2 and 3 are both phrased against
> that resting place. **There was no such resting place.** `rust """…"""` was not a
> construct: `rust` lexed as an ordinary identifier, `"""…"""` as a multi-line string, and
> the pair parsed as a **function call** — emitting `rust("let x = 1;")`, which no program
> defines. The hatch this document named as the fallback for every unreachable construct
> was a trap that produced uncompilable Rust, caught only by `redox check`'s rustc.
>
> So doc 02 §4.1's standard applied even more sharply than this section claimed: the
> constructs were not escape-hatch-only, they were unreachable. Plan 45 leaf 1 implemented
> the hatch, which is why every decline above is now a decline rather than a gap.

> **Correction, plan 46 §1.2.** The "Closed by this document" list above claims **`|`
> alternatives**. It is wrong, and has been since doc 18: that document made Haskell's
> surface the language, and `|` in a `case` arm introduces a **guard**. So `1 | 2 -> true`
> parses and means *`1` guarded by `2`*, failing at rustc with *"mismatched types: expected
> `bool`, found integer"* — a type error about the wrong thing, which is the worst way for a
> gap to present.
>
> Separate arms match the same values, so expressiveness survives and `docs/GRAMMAR.md`
> grades the row `UNREACHABLE` with that as the nearest alternative. What has no spelling is
> Rust's *binding* case, where one arm body serves several patterns.
>
> `@` bindings and slice patterns in the same clause **are** reachable; both were probed.
> The fixture is `tests/unreachable/or_pattern.sdx`.

> **Also recorded by plan 46 §1.2**, against this section's remit rather than its text:
> **`unsafe trait` and `unsafe impl`** are unreachable. Plan 45 shipped `unsafe` as a block
> and a function marker, and its §6 said the spelling *"should extend"* to a trait and an
> impl — *"if it does not, §10 records that"*. It does not, and plan 45 §10 did not record
> it. Fixture: `tests/unreachable/unsafe_class.sdx`. The FFI **export** direction
> (`extern "C"` on a function) and `as` casts are unreachable too, and neither was ever on
> this list; `docs/GRAMMAR.md` is now where that list lives.

---

## 19. Minimum Rust version

This surface's lowerings depend on the following stabilizations. They should be
re-verified against current Rust before the roadmap's Phase 4, rather than taken from this
table.

| Feature | Required by | Approximate stabilization |
|---|---|---|
| `let`-else | `guard` (§7.1) | 1.65 |
| Labelled block expressions | `'label` on a block (§7.2) | 1.65 |
| `let`-chains (edition 2024) | `is` with `&&` (§5.5) | 1.88 |
| `c"..."` literals | §11.2 | 1.77 |
| `matches!` | `is` (§5.5) | 1.42 |
| Subslice patterns with binding | §6.4 | 1.42 |
| Postfix `.await` | §14.3 | 1.39 |

**Resulting floor: Rust 1.88, edition 2024.** Declining the `is`-with-`&&` form would lower
it to 1.77; every other construct is comfortably below that.

---

## 20. Cost, and the minimality constraint

Doc 01 §30 states that "the project's most important engineering constraint is minimizing
lines of code," and §36 instructs that the smallest practical compiler be built. This
surface is substantially larger than the one those sections were written about, and the
tension should be stated plainly rather than left for a reviewer to find.

**What has changed, and what has not.** The constraint doc 01 §30 actually protects is the
*absence of semantic machinery* — no type inference, no borrow checker, no intermediate
language, no optimizer. That constraint is **fully intact**: §2's admission test is
stricter than doc 01 §30's prose, and this document adds no analysis pass beyond the small
resolver doc 02 §6.4 already budgets. What has grown is the parser and the emitter, which
are the two components whose size scales with surface area and nothing else.

**Revised estimate.** Doc 02 §2 projected 8–12k lines of Rust. Against this surface:

| Crate | Estimate | Note |
|---|---|---|
| `redox-lexer` | ~1.2k | off-side rule, trivia, adjacency, §4's sigil table |
| `redox-parser` | ~3.8k | recursive descent + Pratt, multi-error recovery, §5.1's call rules |
| `redox-emit` | ~3.5k | lowerings + source map |
| `redox-diag` | ~0.8k | miette, §17's catalogue |
| `redox-driver` | ~1.5k | query cache, remapping (doc 02 §7) |
| `redox-fmt` | ~2.0k | trivia-preserving, §16's canonicalization |
| `redox-lsp` | ~0.6k | |
| `redox-cli` | ~0.5k | |
| **Total** | **~13.9k** | plus tests and examples |

Call it **14–18k lines** — still an order of magnitude below Emerald's 107k, and for the
reason doc 02 §1.1 identifies: Emerald spent 112 of 203 plans on a standard library, and
22,589 lines on semantics Redox delegates to rustc.

**The honest costs**, each a consequence of a decision above rather than an oversight:

1. **The formatter roughly doubles.** §16's one-canonical-form-per-construct policy means
   `redox-fmt` rewrites constructs, not just whitespace, while also preserving trivia.
   §5.1's universal parenless form is the largest single contributor: the formatter must
   decide, per call site, whether any of §5.1.2's six cases applies.
2. **The call grammar is the parser's most intricate region.** §5.1 needs eight
   interacting rules — adjacency lexing, comma below every operator, the terminator set,
   the six required-parens cases, closure-body extent in argument position, the
   newline-versus-same-line distinction against §9.1's struct-init block, unary-minus
   adjacency, and `RDX0055` detection. This is the one place where the surface's
   expressiveness is bought with parser complexity rather than with a mechanical rewrite,
   and it is why the estimate above moves the parser rather than the emitter.
3. **Multi-line expansions strain the source map.** `use`, `defer`, `with`, multi-clause
   merging and collection-`if`/`for` all break line-preserving emission, which is exactly
   where doc 02 §6.5's remapping earns its place.
4. **`defer` emits a helper item** — the single exception to doc 01 §36's no-scaffolding
   rule, bounded in §7.3 and flagged in §23.
5. **The decline list does real work.** Regex literals, property tests, benchmarks,
   concurrent await, custom operators, transpile-time evaluation, cascades, scope
   functions, `RecordWildCards`, symbol arrays, bracket type arguments and default
   arguments are all declined, most of them because admitting them would have meant
   implying a dependency or guessing a type.

**Staged adoption.** The surface maps onto doc 02 §11's roadmap without changing its shape;
Phases 0–3 are unaffected, since they concern decisions, the vertical slice, diagnostics
and Cargo integration rather than surface area.

| Tier | Contents | Roadmap phase |
|---|---|---|
| **A — corrections** | §4.2 ranges, §13.1 paths, §14.3 await, §13.4 module bindings, §4.1 comments, §13.2 attributes | Phase 0–1; these are corrections, not additions |
| **B — core surface** | `is`, `guard`, `switch` with guards, slice/as/range patterns, `cond`, `loop`, labels, string literal kinds, format specs, doc comments, visibility, aliases, tuple and unit structs, enum forms, `derive` | Phase 4–5 |
| **C — expressive surface** | `\|>`, sections, `??`/`??=`, `catch`, `?.`, comprehensions, collection-`if`/`for`, `into`, spread, literals, `with`, `use`, puns, primary constructors, multi-clause definitions, labelled arguments, `where` locals, forwarding, method references | Phase 6 |
| **D — tooling-adjacent** | `test` blocks, typed domain errors, `try` blocks, `defer`, broadcasting, panic family, `unwrap` lint | Phase 7–8 |

Tier A should land first regardless of schedule: each item is a correction to a published
specification, and two of them (ranges, paths) are cases where current Redox source would
mean something different from the Rust it resembles.

---

## 21. Amendments to the preceding documents

### To `docs/01 - Redox.md`

| § | Amendment | Here |
|---|---|---|
| §4.1 | Comments are trivia and are re-emitted, not removed | 4.1, 11.4 |
| §4.3 | Module-scope bindings require a type and lower to `const`/`static` | 13.4 |
| §5.4 | "Receiver context" defined; `move` and explicit return types added | 8.6 |
| §5.5 | Default arguments declined | 8.7 |
| §6.1, §6.2 | Parenless calls are canonical in **every** position; parentheses become grouping, and §6.2's "encouraged for ambiguous nested expressions" is replaced by §5.1.2's six enumerated cases | 5.1 |
| §6.3 | `!` disambiguation rule stated; `f!` canonical for zero-argument calls | 5.2 |
| §6.4 | Its nested-call rule generalized; the unparenthesized form is `RDX0055` | 5.1.2 |
| §8 | **Control flow is expression-oriented** — plan 55. §8.1's `if`/`else` reads as a statement and was one; every body position is now an expression, so a branch evaluates to its value and no `return` is needed to carry one out. §8.2's `unless` and §8.3/§8.6's postfix forms were already retired by doc 18 §7 | 7, 7.2 |
| §9 | Comprehension lowering unified; `into` added | 10.1, 10.3 |
| §10 | "unless a type context specifies another map" deleted | 10.3 |
| §13.1 | `?.` borrows; no implicit clone | 12.3 |
| §13.2 | `??` is `Option`-only, lowered by `match` | 12.1 |
| §15 | Pipeline is a receiver pipeline; the worked example is corrected | 5.3 |
| §17 | Primary constructors, puns, enum forms, `derive`, visibility, aliases | 9 |
| §18 | Relational, logical and anonymous record patterns | 6.2, 6.3 |
| §19 | **`do` is retired as a construct and reserved as a token** — plan 55 leaf 7. An indented block is an expression, so nothing announces one; the label survives as `'name` on a block, and writing `do` is `RDX0114`. §7.2 kept it until then | 7.2 |
| §20 | `with` replaced by the dot-shorthand mutation block | 7.5 |
| §21 | **Ranges adopt Rust's spelling** — `..` exclusive, `..=` inclusive | 4.2 |
| §22 | `::` everywhere; granular visibility; glob, nested and re-export forms | 13.1, 9.6, 13.5 |
| §23 | Attribute passthrough added; macro definition recorded as open | 13.2, 13.3 |
| §24 | **Postfix `.await`** | 14.3 |
| §30 | Minimality constraint reinterpreted; revised estimate | 20 |
| §36 | `defer`'s scaffolding recorded as a disclosed exception | 7.3 |

### To `docs/02 - Redox Update Blueprint.md`

| § | Amendment | Here |
|---|---|---|
| §4.1 | Second inventory axis largely closed; residue listed | 18 |
| §6.2 | Rule confirmed but applied too conservatively; three mechanisms named | 1.1 |
| §6.3 | `collect` turbofish extended with `into`; sections and `catch` added | 10.3, 5.4, 12.2 |
| §10.7 | `with` decline reversed, via the dot-shorthand design | 7.5 |
| §10.8 | Pipeline resolved as a receiver pipeline | 5.3 |
| §11 | Roadmap unchanged in shape; surface staged in tiers A–D | 20 |
| §12.4 | Decline list extended | 22 |
| §13 | Questions 1–12 answered; residue in §23 | throughout |
| §4.6 | Adjacency lexing confirmed, and now load-bearing for three separate decisions — calls, subtraction sections, unary minus | 5.1, 5.4 |

Doc 02 §13's open questions resolve as: (1) adjacency lexing — §5.1; (2) longest-match
lexing, `#` retained — §4.1; (3) `::` everywhere — §13.1; (4)–(5) unchanged, `build.rs`
with `.xrs` as the human-facing artifact; (6) `@@` retained, `@` freed of the label role —
§4.3; (7) no implicit clone — §12.3; (8) `Option`-only via `match` — §12.1; (9) receiver
pipeline — §5.3; (10) `##` and `##!` — §11.4; (11) reframing accepted; (12) the
hand-written parser recommendation stands, with doc 02 §3.5's bake-off still owed.

---

## 22. Declines

Extends doc 02 §12.4. Each entry is permanent unless a named venue reopens it.

| Construct | Source | Reason |
|---|---|---|
| Regex literals, extensible sigils | Ruby, Elixir | would imply an undeclared crate (§11.3) |
| Property-test and benchmark blocks | Emerald | same (§15) |
| Concurrent-await construct | — | every joining form is runtime-crate-provided (§14.3) |
| Transpile-time evaluation (`comptime`) | Zig, Emerald | requires an interpreter in the compiler (§13.3) |
| Custom operator declaration | Haskell, Scala | no Rust lowering target exists (§14.4) |
| Cascades | Dart, Smalltalk | method-chaining sugar with no Rust lowering of its own — **reason restated by plan 47 §1.4**, see below |
| Scope functions (`apply`, `tap`, `doto`) | Kotlin, Ruby, Clojure | the same |
| `RecordWildCards` | Haskell | introduces names absent from the source (§9.2) |
| Symbol literals and `%i[...]` | Ruby | Rust has no symbol type (§10.4) |
| Bracket type arguments | Emerald | requires translating every type expression (§14.1) |
| Default arguments | C#, Ruby, Python | fabricates API surface (§8.7) |
| UFCS | Nim | requires type knowledge to choose a shape (§2) |
| Two-binder `and` patterns | C# | no Rust pattern form binds from both sides (§6.2) |
| General monadic comprehension | Scala | must choose `flat_map` versus `and_then` (§10.1) |
| A Redox type checker | — | the project's load-bearing constraint (doc 02 §12.4) |
| A Redox standard library or prelude | — | doc 02 §1.1 |

Two entries carry a named venue rather than a permanent decline: `with` as a functional-update
*expression* (§9.1's spread entry covers the ground; revisit if it proves insufficient), and
a general monadic comprehension (revisit only if a type-free discrimination is found).

> **Correction, plan 47 §1.4.** Cascades and scope functions gave their reason as *"duplicates
> `with` (§7.5)"*. Doc 18 §7 **deleted** §7.5's `with` blocks — *"an ordinary binding, no
> construct needed"* — and reused the keyword for doc 30 §3.3's callback bind. So both
> declines cited a construct that no longer existed, and a reader checking §7.5 would have
> found a deleted section.
>
> The **verdict is unchanged**: it was never about the keyword. Both forms are method-chaining
> sugar, and neither has a Rust lowering of its own, which is the same ground §14.4 declines a
> custom operator on.
>
> `docs/DECLINED.md` restates all of these for a reader who has not read this document, and
> `crates/redox-cli/tests/declined.rs` asserts every construct in the table above appears
> there — so a decline added here cannot go unrecorded.

---

## 23. Open items

1. **Confirm `defer`'s generated helper.** §7.3 is the single exception to doc 01 §36's
   no-scaffolding rule. The alternatives are to accept it as bounded, to decline `defer`,
   or to restate §36 to permit `core`-only generated items. This needs an explicit
   decision rather than an implicit one.
2. **Spell `unsafe`, `extern` and FFI.** §18 lists these as unreachable, and doc 01 §26
   names FFI as a requirement, so the escape hatch is not an acceptable resting place.
3. **Spell macro definition**, or accept permanently that Redox macros require inline Rust
   (§13.3).
4. **`union`, raw identifiers, `async` blocks** — minor, but each is currently
   escape-hatch-only (§18).
5. **Re-verify §19's stabilization table** against current Rust before Phase 4.
6. **Run doc 02 §3.5's parser bake-off.** The hand-written recommendation stands on
   argument, not on measurement, and this surface — §5.1's call grammar, multi-clause
   merging, adjacency lexing, two `where` roles, trivia preservation — strengthens the
   argument while raising the cost of being wrong. §5.1 in particular is not expressible in
   an LALR(1) generator at all, which is the point Emerald's plan 90 made for a far smaller
   version of the same feature.
7. **Validate §5.1 against real code before Phase 4.** The universal parenless form is
   the one decision in this surface taken against the weight of comparative precedent:
   Elixir has the feature and its formatter adds the parentheses back; Gleam declined it;
   Rubocop's default requires them. The counter-argument is specific and sound — `f x, y`
   cannot be misread as Rust, so doc 02 §5.2's worst failure mode does not apply — but it
   is an argument, not evidence. Write a few hundred lines of realistic Redox in the
   canonical style before the grammar is frozen, and count how often §5.1.2 forces
   parentheses back in. If the answer is "most nested calls", the statement-position-only
   policy this document previously carried is the fallback.
8. **Consider Scala-style placeholder sections.** `(_ - 1)` would remove §5.4's adjacency
   rule and make reversed sections expressible, at two characters per section. Recorded as
   the venue if the adjacency rule proves troublesome.
9. **Write the two grammar inventories** (doc 02 §4.1): MoonScript graded
   `KEEP`/`MODIFY`/`REMOVE`, and Rust graded `PASSTHROUGH`/`SUGARED`/`UNREACHABLE`. §18 is
   a summary of the second, not a substitute for it.

---

## 24. Summary

Fifty decisions, from a survey of nineteen languages, admitted against a single test: the
lowering must be determined by the source text alone.

The expansion was possible because doc 02 §6.2 understated what that test permits. rustc
resolves method calls, so any lowering that emits one is type-free — which is what admits
`use`, the receiver pipeline, comprehension desugaring, broadcasting, and the operator
sections whose comparison forms use `.gt(&e)` precisely so that one spelling works whether
a closure parameter arrives by value or by reference.

Four decisions matter more than the rest:

1. **Ranges adopt Rust's spelling** (§4.2). Doc 01 §21 inverted `..`, which is the single
   place a Redox expression was most likely to read as Rust and mean something else. The
   correction also freed `...` and dissolved a three-way collision.
2. **Option and Result stay visually distinct** (§12). `??` serves one, `catch` the other,
   both lower to `match`, and `?.` borrows rather than cloning. The error surface is
   therefore larger than doc 01's while doc 01 §25.1's no-silent-clone rule stays absolute.
3. **One canonical form per construct** (§16). Emerald's plan 90 showed what a formatter
   that disagrees with the examples costs: silent reversion by the same gate that checks
   them. Deciding canonical spellings before writing examples is the cheap version of that
   lesson.
4. **`f x, y` is canonical everywhere** (§5.1). This is the one decision taken against the
   weight of comparative precedent — Elixir has the feature and formats it away, Gleam
   declined it, Rubocop's default requires parentheses — on the specific ground that a
   parenless call cannot be misread as Rust, so the failure mode driving decision 1 does
   not apply here. Parentheses become grouping, required only in §5.1.2's six enumerated
   cases, of which the first is doc 01 §6.4's own existing rule. It is also the most
   expensive decision in the document: eight interacting parser rules, and the reason the
   parser estimate moved rather than the emitter's. §23 item 7 is the check that would
   falsify it.

The constraint doc 01 §30 protects — no type inference, no borrow checker, no intermediate
language — is intact. What grew is the parser and the emitter, from roughly 8–12k projected
lines to 14–18k. What did not grow, and must not, is the set of questions Redox answers for
itself.
