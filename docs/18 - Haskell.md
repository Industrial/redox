# Voltaic in Haskell Clothing

Version: 1.0.0
Status: **normative.** This is the Voltaic surface. Where this document and
`docs/03 - Voltaic Surface Design.md` disagree about *spelling*, this document wins; where they
agree about *lowering*, doc 03 remains the reference for the Rust produced.
Supersedes: doc 03 §§4.1, 5.1–5.6, 6.1–6.4, 7.3–7.7, 8.1–8.5, 9.1–9.3, 10.1–10.4, 12.1,
14.2, 21, and decisions 49 and 50.
Amends: doc 01 §§4.1, 16, 17.7, 22, 24.

The same transpiler, respelled. Every lowering in doc 03 stays exactly as written; only the
surface changes, and it changes toward Haskell. The surprise is that the Haskell spelling is
also the smaller one: **nineteen constructs, eight diagnostics and four of the six meanings
of `#`** fall out of the language without taking any capability with them.

```haskell
load : &str -> Result<Config, Error>
load p =
  guard Ok raw <- read p   else return Err Error::Unreadable
  guard Ok cfg <- parse raw else return Err Error::Malformed
  Ok cfg
```

Four tokens carry the whole redesign: **spaced `::`** ascribes a type, **tight `::`** is still
a Rust path, **`<-`** draws a value out of something, and **juxtaposition** is application.

---

## 1. The brief, and the two rules it sets

The question was how far the Voltaic surface can be pushed toward Haskell while the generated
Rust is held fixed. That fixes two rules, and everything below is downstream of them.

**The lowerings do not move.** Doc 03 specifies each construct by the Rust it produces, and a
respelling that changes the Rust is a different language, not a different spelling. So `|>` is
still a receiver pipeline, `??` is still `Option`-only and still lowers to a `match`, a
comprehension still becomes `flat_map`, and a guard still becomes an `if`/`else if` ladder.
Three lowerings are new, all three disclosed in §6; nothing else in the emitter changes shape.

**The admission test does not move either.** Doc 03 §2 admits a construct only if its lowering
is fixed by the source text alone, names nothing outside `core`/`alloc`/`std`, preserves Rust's
semantics, and buys reachability. Haskell is a type-directed language, so the test does most of
the editing here: it is what admits equations, guards, `data` and sections, and what rejects
laziness and every monad abstraction (§8).

**Currying was in that reject list until plan 56** (`docs/50 - 56 - Currying.md`), and the
test is what let it in rather than what kept it out. Test 1 asks that the lowering be fixed by
the source text alone, and for an under-applied call it is: §2.2's signature *states* the
arity, so `add 1` against `add : i32 -> i32 -> i32` is one argument short by counting
top-level arrows and nothing else. What is admitted is partial application — a closure over the
arguments still missing — and not Haskell's semantics, since a Voltaic function is still one
saturated call rather than a chain of one-argument ones. §8's table carries the correction.

### 1.1 The simplicity dividend is not a bonus — it is the argument

Haskell gets its surface from a handful of general mechanisms rather than many special ones.
Adopting those mechanisms deletes the specific constructs Voltaic had grown to cover the same
ground: guards subsume `cond`, relational patterns and `when`; `<-` subsumes `use`, `is`,
`defer` and the comprehension's `for`; the section rule subsumes `&:`. **Every deletion in §7
is a construct whose work another construct already does.**

**And the dividend was paid once more, by plan 55** (2026-11-10). This section
counted the deletions doc 18 could see; it could not see the largest one,
because the mechanism it needed was Rust's rather than Haskell's. Making every
body position an **expression** subsumes three more: an indented block subsumes
`do`, so no keyword announces a block; an expression branch subsumes `then`,
so an `if` has one spelling rather than two; and `try`, `async` and `unsafe`
become *adjectives on that one block* rather than three constructs of one
shape. `then` — one of the nine keywords §7 counts this document as *adding* —
is retired and free again, `do` stays reserved with no construct behind it for
plan 60, and `RDX0101`/`RDX0102` stopped firing at arm and tail positions.
The measurement is plan 55 §1.2's: the fork was pushing `return` into a
declarative surface at seven times the rate the expression form was used,
because `return` was the only way a branch could produce a value.

---

## 2. Five moves make the language Haskell

| | Move | One line |
|---|---|---|
| 1 | `f x y` | Application is juxtaposition, left-associating. Commas leave the call grammar; parentheses become pure grouping. |
| 2 | `f : A -> B` | A signature line above the equations. The generic list and the bound clause are read off it. |
| 3 | `f 0 = …` | Definition by equations over patterns, with `\|` guards and `where` locals. |
| 4 | `data T = …` | One declaration form for product and sum types, with `deriving`. |
| 5 | `class` / `instance` | Traits and impls under their Haskell names, constraints by `=>`. |

### 2.1 Move 1 — application by juxtaposition

Doc 03 decision 49 makes `f x, y` canonical everywhere and pays for it with what §20 calls the
parser's most intricate region: eight interacting rules, the comma below every operator, a
four-member terminator set, six required-parens cases, and `RDX0055` for the nested call it
cannot read. Haskell's rule costs one sentence — **application binds tighter than every
operator and associates left** — and the hazard disappears instead of being diagnosed.

| Voltaic today (doc 03 §5.1) | Haskell-shaped | Rust unchanged |
|---|---|---|
| `send level, format! "{}: {}", code, msg` | `send level (format "{}: {}" code msg)` | `send(level, format!("{}: {}", code, msg));` |
| `add (multiply 2, 3), 4` — parens required | `add (multiply 2 3) 4` | `add(multiply(2, 3), 4);` |
| `add multiply 2, 3, 4` — `RDX0055` | `add $ multiply 2 3 4` | `add(multiply(2, 3, 4));` |
| `fib(n - 1) + fib(n - 2)` | `fib (n - 1) + fib (n - 2)` | `fib(n - 1) + fib(n - 2)` |
| `Some(x)` — parens always | `Some x` | `Some(x)` |
| `user.save!` | `user.save ()` | `user.save();` |

Three consequences worth stating.

- **Parentheses are grouping only**, so the adjacency rule for calls goes away — `f(x)`,
  `f (x)` and `f x` are one expression and the formatter prints the last.
- **`$` is admitted, lowering to nothing at all**: it is grouping that runs to the end of the
  expression, and it is why a nested call rarely needs brackets.
- **Zero-argument calls are written `f ()`**, which retires doc 03 §5.2's `!` suffix and its
  disambiguation rule against macro invocation.

**Required parentheses: six cases down to two.** What survives is nested application —
Haskell's own rule, and no longer a diagnostic because `f g x` has exactly one reading — and a
postfix operator on an application, `(parse x)?` or `(get u).await`. Doc 03's cases 4
(constructors), 5 (forwarding) and 6 (swallowed tokens) all go: constructors are handled by
capitalisation (§5.3), forwarding is deleted (§7), and the comma that did the swallowing is
gone.

### 2.2 Move 2 — the signature line

A signature above the equations is the most recognisable thing in Haskell, and it earns its
place here by carrying information Rust otherwise makes you repeat. **Lowercase type names are
type variables**, as in Haskell; the generic list is the order of their first appearance, and
the bound clause is read off the `=>` constraint. The spine splits on top-level `->`; every
atom between the arrows is still a verbatim Rust type, so doc 03 §14.1's guarantee that nothing
in the type grammar can drift is intact.

```haskell
largest : Ord t => &[t] -> &t
largest xs = best
  where
    best = xs |> iter |> max |> unwrap

index : HashMap<String, Vec<u8>> -> &str -> Option<&Vec<u8>>

const MAX : usize = 1024

square : const u32 -> u32
square n = n * n
```

```rust
fn largest<T>(xs: &[T]) -> &T
where
    T: Ord,
{
    let best = xs.iter().max().unwrap();
    best
}

const MAX: usize = 1024;

const fn square(n: u32) -> u32 { n * n }
```

This retires doc 03 §14.2 — `where` had two roles resolved by position, bound clause before the
body and local definitions after it. Constraints now live in `=>`, which is also Haskell's
spelling, so **`where` keeps one meaning** and the position-sensitivity goes. Constraints take
any Rust bound: `Into<String> t =>` becomes `T: Into<String>`, and `Iterator<Item = u8> t =>`
likewise, so associated-type equality stays reachable.

### 2.3 Move 3 — equations and guards

Doc 03 already has multi-clause definitions (§8.1, from Haskell) but spells each clause as a
closure with a repeated signature, which is why `RDX0045` exists to catch the clauses
disagreeing. With the signature hoisted, a clause is a pattern row and the diagnostic reduces
to arity.

```haskell
fib : u64 -> u64
fib 0 = 0
fib 1 = 1
fib n = fib (n - 1) + fib (n - 2)

tier : u64 -> &'static str
tier spend
  | spend > 10_000 = "platinum"
  | spend > 1_000  = "gold"
  | member         = "silver"
  | otherwise      = "basic"
```

```rust
fn fib(__rdx_a0: u64) -> u64 {
    match __rdx_a0 {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

fn tier(spend: u64) -> &'static str {
    if spend > 10_000 { "platinum" }
    else if spend > 1_000 { "gold" }
    else if member { "silver" }
    else { "basic" }
}
```

The guard bar is **the single highest-yield token in the redesign**, because Haskell puts it in
both equations and case arms and Voltaic had grown three constructs to cover the same ground.
`cond` is deleted. `when` arm guards are deleted. Doc 03 §6.2's relational patterns — `> 65` as
an arm — are deleted, and with them the scrutinee-temporary lowering they needed, because the
user now writes the binder the guard tests.

```haskell
label : u32 -> &'static str
label age = case age of
  n | n > 65        -> "senior"
  18..=64           -> "adult"
  Teen or PreTeen   -> "young"
  _                 -> "newborn"
```

```rust
fn label(age: u32) -> &'static str {
    match age {
        n if n > 65 => "senior",
        18..=64 => "adult",
        Teen | PreTeen => "young",
        _ => "newborn",
    }
}
```

`switch` becomes `case … of` with `->` arms, which lets `=>` leave the expression grammar
entirely and take up its Haskell job in constraints. Doc 03's `or` pattern combinator stays,
unchanged and lowering to Rust's `|`, **precisely so that the bar is unambiguously the guard**;
`and` and `not` patterns are deleted in favour of guards, which also deletes `RDX0042` and
`RDX0043`. `not` is freed to mean what it means in Haskell: boolean negation.

### 2.4 Move 4 — `data` for every shape of type

Haskell's declaration form maps onto Rust's with almost nothing left over, and it collapses
four Voltaic keywords into two. **A single constructor whose name matches the type is a `struct`;
anything else is an `enum`.** That rule is syntactic, which is the whole requirement.

```haskell
data Point = Point { x : f64, y : f64 }
  deriving (Debug, Clone, PartialEq)

data Event
  = Click { x : i32, y : i32 }
  | Key char
  | Close
  deriving (Debug)

newtype Meters = Meters f64
data Marker = Marker
type Handler = Box<dyn Fn(Req) -> Res + Send>
```

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point { x: f64, y: f64 }

#[derive(Debug)]
enum Event {
    Click { x: i32, y: i32 },
    Key(char),
    Close,
}

struct Meters(f64);
struct Marker;
type Handler = Box<dyn Fn(Req) -> Res + Send>;
```

Records use braces and `=` for values, matching `::` for types — so the indented block
initialiser of doc 03 §9.1 is deleted along with its spread-must-be-last diagnostic
(`RDX0052`). Field punning survives in both directions; it came from Haskell in the first
place. Primary constructors (§9.3, from C# 12) are deleted — in Haskell the data constructor
*is* the constructor — which takes `RDX0053` with them.

```haskell
p  = Point { x = 10, y = 20 }
q  = Point { x, y }
p2 = p { x = 99 }
```

```rust
let p = Point { x: 10, y: 20 };
let q = Point { x, y };
let p2 = {
    let mut __rdx_r = p;
    __rdx_r.x = 99;
    __rdx_r
};
```

Two corrections to that table, made when it was implemented. The punned field emits **Rust's
own shorthand** rather than `x: x`: the two are equivalent, and `x: x` is what clippy's
`redundant_field_names` flags in the *user's* build, which fails doc 01 §3.4's readability
requirement for no gain. And the update block spans lines, because rustfmt expands a block of
statements unconditionally — a one-line block is not rustfmt-clean, which is plan 04's
guarantee and not a matter of taste.

### 2.5 Move 5 — `class` and `instance`

Rust's trait system is Haskell's class system with different words, so the words change back.
`class` stops meaning "struct plus impl" (doc 01 §17.7) and means `trait`; `instance` means
`impl`, with or without a class name.

**The receiver is marked, not positional: `self` in a type is the receiver** — plan 67, which
replaced this paragraph's first draft. That draft said *"the first parameter typed by the class
variable lowers to `self`"*, and three things had to be true at once before a reader knew which
parameter that was: which position it was in, which of up to three spellings happened to name
the implementing type, and what borrow prefix it carried. The third is kept and is good design;
the first two are gone.

**An `instance` must be in the same file as its `class`** — measured at plan 91 wave 5,
after two type classes became the first real uses of this move. `Parser` carries
`class_heads: BTreeMap<(String, String), SignatureHead>`, filled by `record_head` when a
`class` is parsed and read by `borrowed_head` when an instance member is parsed, so the
parser learns a member's signature from the class declaration — and **each file gets a
fresh `Parser`**. An instance declared elsewhere parses as a run of top-level declarations
and fails with `RDX0101` on every name in the member line, which reads nothing like the
cause.

This is a constraint of the surface, not a defect: the parser wants the member's *shape*,
not its types, so it stays on the right side of doc 02 §12.4. It does bound the design —
an instance cannot sit beside the type it is for, or beside the function it delegates to,
unless the class is in that file too. `voltaic-emit`'s `Emit` therefore declares the class
and all thirteen instances in `item.sdx`, beside the dispatcher that uses them. Lifting
the constraint means letting an instance member be parsed without its head, which is a
surface decision owed its own row.

```haskell
class Debug => Shape
  type Unit
  const SIDES : u32
  area : &self -> f64
  scaled : &self -> f64 -> Self     -- the receiver first, which is Rust's order
  grown : f64 -> &self -> Self      -- the receiver last, which is the call's order
  unit : f64 -> Self                -- no `self`, so no receiver: an associated fn
  consume : self -> u32             -- by value
  retitle : &mut self -> ()         -- by mutable reference
  describe : &self -> String
  describe self = "a shape with #{Self::SIDES} sides"

class Convert t                     -- `t` is the trait's parameter, not the subject
  wrap : t -> Self

instance Convert f64 Meters         -- impl Convert<f64> for Meters
  wrap : f64 -> Self
  wrap n = Meters n

instance Display t => Describe (Wrapper t)
  describe : &self -> String
  describe self = "#{self.inner}"

instance Counter
  increment : &mut self -> ()
  increment self = self.value += 1
```

Three consequences, each of which is the point rather than a side effect.

**`scaled` and `grown` above emit the same Rust.** Plan 57 calls a method `.scaled 2.0 s` — the
argument first, the receiver last — so the declaration and the call read in opposite orders, and
every Rust method has always had that reversal. The difference is that nobody reads std's
declarations and a `class` is the one place a Voltaic author writes one. Marking the receiver makes
the question stop having an answer that matters: `fn scaled(&self, k: f64) -> Self` comes out of
both, down to the generated name a required method's parameter takes.

**`Self` names the implementing type and a `class` head names the trait's own parameters.** The
variable was never doing Haskell's job here: in Haskell it earns its keep because methods mention
`t` in arbitrary positions and there is no `Self`; Voltaic has `Self`, needs it for an inherent
`instance` regardless, and accepted it in a class body from the start. One name for one thing —
and the freed slot is what makes `trait From<T>` reachable. Before plan 67, `class Convert t`
spent that slot on the implementing type, so a second lowercase name in a member's type became a
**method** generic and rustc answered `E0049`, naming a type parameter the author never wrote.

**The last name in an `instance` head is the implementing type.** `instance Convert f64 Meters`
is `impl Convert<f64> for Meters`, which a reader can check by eye because the two read in the
same order. A two-name head is unchanged.

The cost, stated plainly: `self` becomes a reserved spelling in **type** position, one more
exception to §5.3's *lowercase is a type variable* beside the seventeen primitives. And `self` is
not really a type, which is a small lie the notation tells. It is Rust's own lie, told in Rust's
own notation, which is why a reader who knows Rust needs no rule at all.

**Assignment is `:=`, and `=` is a binding** — plan 54. `self.value += 1`
above is a *compound* assignment, which is all docs 01, 03 and this one defined
for a long time: at statement level `x = v` introduces a name, so plain
reassignment had no spelling, and plan 20 §10.2 disclosed the consequence
rather than designing around it — *"a loop can advance a numeric accumulator
and cannot rebind anything else."*

```haskell
increment self = self.value += 1     -- compound: the five arithmetic forms
relabel self = self.name := "next"   -- plain: anything at all
inner = 1                            -- a binding, and still a binding
```

```rust
fn increment(&mut self) { self.value += 1; }
fn relabel(&mut self) { self.name = "next"; }
let inner = 1;
```

Rust spells all six with `=` and tells a binding from an assignment by `let`.
Voltaic has no `let` to lean on, so it tells them apart by the operator instead
— which is one character against the scope tracking the alternative needed and
AGENTS.md forbids.

The assignment target is a general expression, so a field, an index, a deref
target and Rust's destructuring assignment all work: `(a, b) := (b, a)` is a
swap, the one shape no compound operator can express. Nothing checks that the
target *is* a place; `(1 + 2) := 3` is rustc's `E0070`.

**An instance method restates its signature**, and the line above is a correction to this
document's first draft, which omitted it. Rust's `impl` restates it too; Voltaic would have to
resolve the class to copy one from it, and AGENTS.md forbids growing any name resolution. The
cost is one line per method and the alternative was a feature that could not be built.

```rust
trait Shape: Debug {
    type Unit;
    const SIDES: u32;
    fn area(&self) -> f64;
    fn scaled(&self, __rdx_a1: f64) -> Self;
    fn grown(&self, __rdx_a1: f64) -> Self;
    fn unit(__rdx_a0: f64) -> Self;
    fn consume(self) -> u32;
    fn retitle(&mut self);
    fn describe(&self) -> String {
        format!("a shape with {} sides", Self::SIDES)
    }
}

trait Convert<T> {
    fn wrap(__rdx_a0: T) -> Self;
}

impl Convert<f64> for Meters {
    fn wrap(n: f64) -> Self { Meters(n) }
}

impl<T: Display> Describe for Wrapper<T> {
    fn describe(&self) -> String { format!("{}", self.inner) }
}

impl Counter {
    fn increment(&mut self) { self.value += 1; }
}
```

**A bound on a trait parameter is not spellable.** `trait Convert<T: Display>` needs somewhere
to put the bound, and the `=>` clause in a class head is read as the *superclass* list — a
constraint on `Self`. A row that wants it is opening the question of where a `where` clause lives
in a class head. **A blanket impl is not reachable either**: `instance Display t => Describe t`
parses and emits `impl Describe for t`, with a lowercase target rustc rejects, because `TypeApp`
reads type variables out of a type application's *arguments* and a bare variable is the head.
`tests/unreachable/blanket_impl.sdx` is the fixture; inline Rust is the alternative.

Dropping `@name` and `@@name` for `self.name` and `Self::name` is the cost of this move, and it
is a cost worth paying twice over: Haskell has no such sigil, and `@` is left with exactly one
meaning, the as-pattern `whole @ Move x y` — which is Haskell's spelling too.

---

## 3. One arrow, one bar, one draw

| Token | Position | Means | Replaces |
|---|---|---|---|
| `->` | in a type | function arrow; last RHS is the return type | `(a: A): B` signatures |
| `->` | after `\` binders | lambda body | `(x) -> e` |
| `->` | in a `case` arm | arm body | `=>` arms |
| `\|` | opening a guard list | guard | `cond`, `when`, relational patterns |
| `\|` | inside `[ … ]` | comprehension bar | `for`/`when` clauses |
| `\|` | between `data` constructors | sum alternative | `enum` body |
| `<-` | statement in a block | callback bind | `use x <- f`, `defer` |
| `<-` | in a condition or guard | pattern test, binding | `is` |
| `<-` | after `for`, or in a comprehension | generator | `for x in xs` |
| `::` | spaced | type signature or ascription | `:` annotations, `into`, turbofish |
| `::` | tight | Rust path unchanged | — |

### 3.1 The draw arrow absorbs four constructs

`<-` reads the same way everywhere — *a name drawn out of something* — and the four positions
are grammatically disjoint, so no lookahead is needed to tell them apart. A bare binder in
statement position is doc 03 §7.6's `use`; a refutable pattern in condition position is §5.5's
`is` with the operands in Haskell's order.

| Voltaic today | Haskell-shaped | Rust unchanged |
|---|---|---|
| `use conn <- pool.with_connection` | `conn <- pool.with_connection` | `pool.with_connection(\|conn\| { … })` |
| `if msg is Move(x, _) && x > 0` | `if Move x _ <- msg && x > 0` | `if let Move(x, _) = msg && x > 0 {` |
| `while x is Some(v)` | `while Some v <- x` | `while let Some(v) = x {` |
| `for c in candidates` | `for c <- candidates` | `for c in candidates {` |
| `defer` + body | deleted — see §6.1 | — |

`guard` survives, because `let`-`else` has no Haskell analogue and no other spelling, and it
takes `<-` for consistency: `guard Ok raw <- read p else return …`. Its lowering is doc 03
§7.1's, character for character.

### 3.2 Comprehensions become Haskell's, and there is only one kind

```haskell
pairs = [ (x, y) | x <- 1..3, y <- 1..3, x != y ]
names = [ u.name | u <- &users ] : Vec<String>
seen  = [ c | c <- cs ] : HashSet<char>
word  = [ c | c <- cs ] : String
```

```rust
let pairs = (1..3)
    .flat_map(|x| (1..3).filter(move |y| x != *y)
                        .map(move |y| (x, y)))
    .collect::<Vec<_>>();
let seen = cs.iter().collect::<HashSet<char>>();
```

The bar, the draw arrow and the comma do what `for` and `when` did. Doc 03's second
comprehension level — Dart's element-wise `if`/`for` inside a literal, with its `push`/`insert`
lowering — is deleted; Haskell has one comprehension and so does this. The `into` clause is
deleted too, because `::` already says the type and reads better doing it.

**Two corrections to the Rust above, made when this was implemented.** The guard is lowered
with `filter_map` rather than `filter`, and every guard lands at the innermost level: `filter`
hands its closure a *reference*, which is why the block above has to write `x != *y` — and
Voltaic cannot know when to insert that `*`, having no types. `filter_map` hands over the value,
so the guard reads as written. Every binder is in scope at the innermost level, so the two
placements differ in *when* a guard runs and never in what the comprehension produces. And the
generator's source is parenthesised when it is an operator expression, because `.` binds
tighter: `1..3.into_iter()` is `1..(3.into_iter())`.

---

## 4. Sections, pipelines, lambdas

| Section | Lowers to | Note |
|---|---|---|
| `(+ 1)` | `\|__rdx_1\| __rdx_1 + 1` | right section, doc 03 §5.4 |
| `(1 +)` | `\|__rdx_1\| 1 + __rdx_1` | left section — **new**, closes the gap |
| `(> 0)` | `\|__rdx_1\| __rdx_1.gt(&0)` | method form, for auto-ref §1.1 |
| `(0 <)` | `\|__rdx_1\| __rdx_1.gt(&0)` | flipped, same lowering |
| `(.name)` | `\|__rdx_1\| __rdx_1.name` | field selector, as in GHC's record dot |
| `(- 1)` | `-1` | always negation; no subtraction section |
| `add 1` | `\|__rdx_1\| add(1, __rdx_1)` | partial application — **plan 56**, the sixth row of the same mechanism |
| `.name` | `\|__rdx_1\| __rdx_1.name()` | method reference — **plan 57**, and the row `(\|> name)` used to hold |

The `(.name)` and `(- 1)` rows each delete something — *"the last two"* until plan 56 added a
row below them and plan 57 another. `(.name)` retires doc 03 §8.5's `&:name` sigil. And
following Haskell in having no subtraction section retires the `(-1)`-versus-`(- 1)` adjacency
rule (decision 50): write `\x -> x - 1`.

**`(|> name)` was a row here and plan 57 removed it**, which is that row's own §1.3: a
**method is a function** whose last argument is its receiver, so a bare `.name` is a method
awaiting that argument and lowers to the same `|__rdx_1| __rdx_1.name()`. Two surfaces for one
construct is the redundancy this milestone exists to remove, and plan 65's rule decides which
row removes it — whoever builds the replacement. Writing it is `RDX0116`.

`(.name)` stays, and the discriminator is **a `)` immediately after the name**: `(.age)` closes
and `(.cmp y)` does not. That is one token of lookahead, and §6's honest note about it is that
a parenthesis now carries a semantic difference — worse than the rest of this milestone
produces, and adopted because Voltaic has no types and *something* must carry it. Outside
parentheses it is **position**: a `.` in head position calls a method, a `.` after an
expression selects a field, and `.` has exactly one job in each place.

One thing the replacement does not reach: a bare `.name` is only spellable in **head**
position, because a `.` after an expression is field access. So a zero-argument method used as
an *argument* — `map (|> len)` — is a lambda now: `map (\t -> .len t)`.

```haskell
result = numbers
  |> iter
  |> map (\n -> n * n)
  |> filter (> 0)
  |> sum

labels  = &users |> iter |> map (.name)
lengths = &users |> iter |> map (\t -> .len t)
f = move \x : i32 -> x + captured
```

```rust
let result = numbers
    .iter()
    .map(|n| n * n)
    .filter(|__rdx_1| __rdx_1.gt(&0))
    .sum();

let labels = users.iter().map(|__rdx_1| __rdx_1.name);
let lengths = users.iter().map(|__rdx_1| __rdx_1.len());
let f = move |x: i32| x + captured;
```

**Why `|>` stays and `.` does not.** Haskell's composition operator is `.`, which in Voltaic is
field access — the one Haskell token that cannot be had at any price. Reverse application in
Haskell is `&`, which is Rust's reference sigil. So the receiver pipeline keeps Elm's and F#'s
spelling, and composition is declined for lack of a lowering target: without currying there is
nothing to compose.

**Amended by plan 56** (`docs/50 - 56 - Currying.md`): there is now something to compose, which
is why the table above has a seventh row. An operator section and a function partial
application are **one mechanism with one lowering** — `emit::closure` writes both, and they
differ in the table consulted and nowhere else — so the two cannot drift into two
specifications of the same closure. Neither carries `move`: a `move` capture of a borrowed
argument turns rustc's borrow error into a move error, which is a worse message about the same
mistake. The decline above narrows to the *spelling*, because `.` is still field access; plan
58 names it and **plan 66 chooses `>>>` / `<<<`**, which is `Control.Category`'s own spelling
(plan 66 §1.3); plan 58's `>|` / `|<` is superseded.

---

## 5. Lexical surface: four collisions resolved, four meanings of `#` retired

### 5.1 Comments become `--`, and `#` almost empties

Doc 03 §4.1 orders six rules longest-first to give `#` six meanings outside strings. Taking
Haskell's comment syntax leaves two, and the longest-match table becomes a line.

| Today | Haskell-shaped | Rust |
|---|---|---|
| `# line` | `-- line` | `// line` |
| `### … ###` | `{- … -}` | `/* … */` |
| `## outer` | `-- \| outer` | `/// outer` |
| `##! inner` | `-- ! inner` | `//! inner` |
| `#[attr]` | `#[attr]` kept | verbatim §13.2 |
| `#{1, 2}` | deleted — `[1, 2] : HashSet<_>` | `HashSet::from([1, 2])` |
| `"#{expr}"` | `"#{expr}"` kept | `format!` §11.1 |

`-- |` is Haddock's own outer-doc marker. `-- !` for the inner form is this document's
invention, since Haskell has no module-level analogue of `//!`; it is chosen to echo Rust's own
`/`-to-`!` step. **The one new hazard is real and worth naming:** `x - -1` needs its space,
because `x --1` is a comment. Haskell has the same hazard and the same mitigation — `--` opens
a comment only when followed by a space or end of line — and `voltaic fmt` enforces the spacing.

### 5.2 Adjacency lexing is gone — `:` annotates, `::` is a path

**Superseded by plan 52.** This section specified the last surviving use of
adjacency lexing *in the type grammar*: *a spaced `::` ascribes, a tight `::`
is a path*. That rule is deleted, along with the two parser predicates that
read it.

`Token::ws_before` itself stays. It is still how `capture_type` decides where a
captured type's text ends, and the heading above is therefore narrower than it
first looks: adjacency is no longer load-bearing for *what a token means*,
which is the property that made it dangerous, and remains available for
deciding where one ends.

What it was, and why it went. Doc 03 §21 recorded adjacency as load-bearing for
three decisions — calls, subtraction sections and unary minus. Juxtaposition
removed the first and Haskell's missing subtraction section removed the other
two, and this rule arrived to replace all three. The argument for it was that
it agrees with both communities' formatters: Haskell always spaces `::`, Rust
never spaces a path, so nobody would have to learn it.

That argument is true and insufficient. It made whitespace **semantic** in one
place in the language: the same two characters meant two different things
depending on a space, so a formatter that normalised spacing could change what
a program meant, and a reader could not tell the two apart without counting
columns. Giving ascription its own token costs one character and removes the
category.

So the type marker is `:` everywhere a type is written — signature, typed
binding, ascription, lambda binder, record and `errors` field, `class` member,
`const`, `extern` — and `::` joins a path and does nothing else:

```haskell
load : &str -> Result<Config, Error>
kind = Error::Malformed
n : u64 = 0
total = xs |> iter |> sum : u64
set = [1, 2, 3] : HashSet<i32>
```

```rust
fn load(p: &str) -> Result<Config, Error>
let kind = Error::Malformed;
let n: u64 = 0;
let total = { let __rdx_t: u64 = xs.iter().sum(); __rdx_t };
let set = HashSet::from([1, 2, 3]);
```

Two consequences worth naming. The Rust column now reads as a transliteration
of the Voltaic one rather than as a translation, because `:` *is* Rust's type
marker — which is the same argument §2.4's record fields already made. And
`::<T>` becomes spellable, where the old rule had no way to write it; doc 56's
plan declines to spell it for separate reasons.

A spaced `::` is now `RDX0110` rather than an ascription. It has to be a
diagnostic and not silence: `y = x : u64` would otherwise be consumed as the
path `x::u64` — `u64` is a valid segment — and rustc would report an
unresolved path against a line nobody wrote, which is precisely the failure
mode this section's rule existed to prevent.

### 5.3 Capitalisation becomes lexical, and pays for itself twice

Haskell's rule — constructors and types begin uppercase, values and type variables lowercase —
is a convention Rust already follows and a lexical fact Voltaic can use. It does three jobs no
other mechanism here can do.

- **Constructors need no parentheses.** `Some x` is a constructor application because `Some` is
  uppercase, so it reads identically to the pattern that destructures it. Doc 03 §5.1.2 case 4
  required `Some(x)` precisely to keep constructor and pattern aligned; capitalisation keeps
  them aligned for free, and more strictly.
- **Record update needs no type name.** `Point { x = 1 }` constructs and `p { x = 1 }` updates,
  told apart by the case of the head — which is why the update form is admissible at all.
- **Type variables need no declaration.** `t` in a signature is a generic parameter, emitted as
  `T`; the generic list is synthesised from first appearance, lifetimes included.

**One correction, made when this was implemented.** "Lowercase type names are type variables"
cannot be read literally: `i32`, `usize`, `f64`, `bool`, `char` and `str` are lowercase and
concrete, and taking the rule at its word makes `add : i32 -> i32 -> i32` a generic function
of one parameter. A lowercase name is a type variable **unless it is one of Rust's primitive
type names** — a closed, lexical list, which keeps the rule decidable from the source text
alone (doc 03 §2's admission test 1). The cost is that a user cannot name a type in lowercase,
which §5.3 was already spending to make type variables undeclared.

**A second correction, made when plan 67 marked the receiver.** `self` is lowercase and in type
position it is §2.5's receiver marker, so the rule would otherwise synthesise `fn area<SELF>(…)`
from `area : &self -> f64` — a program that still compiles in simple cases, which is the worst
shape a defect can take. `self` therefore joins the exceptions, and it joins them as **its own
check rather than as an entry in the primitive list**: `self` is not a primitive type, and a
list whose name stops describing its contents is how a reader learns to stop trusting the names.

### 5.4 The guard bar against bitwise or

One rule: **the first `|` following an equation head or a `case` pattern opens the guard list**;
everywhere else `|` is Rust's bitwise or, unchanged. Pattern alternation keeps doc 03's `or`
keyword rather than Rust's `|`, which is what makes the first rule unambiguous —
`Teen or PreTeen -> …` cannot be read as a guard.

---

## 6. Four new lowerings, disclosed

The brief holds the emitter fixed, and it very nearly is. Four lowerings are new. Each is
type-free, each exists only because a Haskell construct arrived without one, and each is listed
here rather than discovered later.

| Construct | Lowering | Why it is type-free |
|---|---|---|
| `e : T` ascription | `{ let __rdx_t: T = e; __rdx_t }` | The type is written, not inferred. Replaces the `into` clause and most turbofish needs. |
| `p { x = 1 }` record update | `{ let mut __rdx_r = p; __rdx_r.x = 1; __rdx_r }` | Needs no type path, which `Point { x: 1, ..p }` would. Moves `p`, as `..p` does. |
| `[a, b] : C<_>` ascribed literal | `{ let __rdx_t: C<T> = C::from([a, b]); __rdx_t }` | The container is named in the ascription. Covers the deleted set and map literals. |
| `data T = A \| B` | the `enum`, plus `use T::*;` | Syntactic: the declaration names the type and the constructors, and nothing else is consulted. |

The third row is written here as the ascription block *containing* `C::from`, which is a
correction: `C::from([a, b])` alone drops the type's generic arguments, so `[] : Vec<i32>` has
nothing to infer its element from and `[1, 2] : Vec<i64>` becomes a `Vec<i32>` because an
unsuffixed integer literal defaults. The two lowerings compose; neither replaces the other.

The fourth was **not in this document's first draft**, and it is not optional sugar. In Haskell
a constructor is in scope under its own name; in Rust it is not. Without the `use`, `Close` is
an ordinary name — as an expression it fails to resolve, and as a `case` pattern it silently
becomes an *irrefutable binding*, so a `describe` written exactly as §2.4 and §2.3 write it
returns its first arm for every input. That is doc 02 §5.2's worst failure mode reached by
following this document, which is the strongest argument available for disclosing lowerings
rather than discovering them.

Two constructors colliding is rustc's error to give, which is the division of labour Haskell
has as well: both languages reject the collision, neither resolves it.

### 6.1 And one lowering deleted outright

Doc 03 §7.3's `defer` is the single construct in the language that emits a helper item — a
`Drop` guard in a generated `__rdx` module — and doc 03 §23 item 1 asks for an explicit
decision about it. Haskell brackets resources with a callback-taking function and `<-` already
flattens exactly that, so **`defer` is deleted** and the request is answered: Voltaic emits no
runtime scaffolding at all, and doc 01 §36 needs no exception.

---

## 7. The simplification dividend

| | |
|---|---|
| **19** | constructs deleted |
| **8 / 14** | diagnostics retired |
| **6 → 2** | meanings of `#` |
| **6 → 2** | required-parens cases |
| **3 → 1** | uses of adjacency |
| **0** | capabilities lost |

| Deleted | doc 03 § | What does its work now |
|---|---|---|
| `cond` | 7.4 | Guards |
| Relational patterns `> 65` | 6.2 | Guards — and the scrutinee temporary goes with them |
| `and` / `not` patterns | 6.2 | Guards; `not` becomes boolean |
| Anonymous record patterns | 6.3 | Guards over field access |
| `when` arm guards | 6.4 | The guard bar |
| `is` | 5.5 | `<-` in condition position |
| `use x <- f` | 7.6 | `<-` as a statement |
| `defer` | 7.3 | `<-` over a bracketing function |
| `with` blocks | 7.5 | An ordinary binding — no construct needed |
| Broadcasting `.+`, `.(` | 10.2 | `\|> map`; `RDX0054` goes too |
| Collection `if`/`for` | 10.1 | The one comprehension |
| `into` clause | 10.3 | `::` ascription |
| Set, map and `%w` literals | 10.4 | `[…] : C<_>` |
| Block struct initialiser | 9.1 | Brace records; `RDX0052` goes |
| Primary constructors | 9.3 | `data` constructors; `RDX0053` goes |
| Labelled arguments | 8.2 | Nothing — deleted with its best-effort `RDX0051` |
| `...` forwarding | 8.4 | Nothing — `RDX0048` goes; `...` leaves the language |
| `&:name` | 8.5 | `(\|> name)` — and that in turn by a bare `.name` (plan 57) |
| `(\|> name)` | §4 | A bare `.name`: a method is a function, receiver last (plan 57) |
| `?.` optional chaining | 13.1, 12.3 | `try` over a `?` run, with an explicit `&` (plan 57) |
| `x.f ()` method-call syntax | 5.1 | `.f x`, `x \|> .f` — `.` is field access only (plan 57) |
| `!` zero-argument suffix | 5.2 | `f ()` — and the token itself went to **macro invocation** in plan 64, which is the one row in this table where retiring a spelling cost a capability. See the note below |
| Chained comparison `0 < x < 9` | 5.6 | `&&` |
| `??=`, postfix `if`/`unless`/`until` | 12.1, 7.7 | Their expanded forms |
| `@` / `@@` sigils | 4.3 | `self.` / `Self::`; `@` is the as-pattern only |
| Two roles for `where` | 14.2 | `=>` constraints; `where` keeps one role |
| `=>` as an arm arrow | 6.1 | `->`; `=>` becomes the constraint arrow |

**The keyword count is flat, and saying otherwise would be dishonest.** Eleven keywords leave
— `is`, `and`, `use`, `with`, `defer`, `cond`, `into`, `derive`, `mod`, `unless`, `switch` —
and nine arrive: `case`, `of`, `then`, `otherwise`, `data`, `newtype`, `instance`, `deriving`,
`module`. (`do` was already reserved, and `switch` belongs on the leaving list because §2.3
replaces it with `case … of`; the first draft of this table counted ten and ten, which was
wrong in both columns and wrong by the same amount, so the conclusion stands.) What falls is
the number of **constructs**, which is what the parser, the emitter and the formatter are each
sized by. Operators fall outright: `=>` (as an arm), `??=`,
`.+ .- .* ./ .%`, `.(`, `...`, `%w[`, `#{` (set), `&:`, `@`, `@@` and `!` leave; `::`, `<-`,
`\`, `$`, `|`, `--` and `{- -}` arrive.

**Corrected by plan 64: one capability *was* lost, and it was the largest in the
language.** This section's table reads `0` capabilities lost, and that is true of
every row but one. Retiring `!` as a suffix also spent the token doc 01 §23 had
**retained** macro-invocation syntax on, and no row here noticed — so `name!` had no
spelling at all, and `effect!`, `caps!`, `hlist!`, `vec!`, `json!`, `bail!`, `info!`,
`query!`, `html!` and every proc-macro were unreachable in a language whose premise
is *"Voltaic has crates.io"*.

It stood for nineteen rows. Plan 45 §1.5 found it and left the spelling decision;
plan 48 ported the compiler's own source to Voltaic and **priced** it, with
`docs/CONFORMANCE.md` carrying the figure. Worse than unreachable, plan 45 measured
`assert! true` *appearing* to work — it read as the `assert` keyword applied to
`!true` and silently emitted `assert!(!true)`, an inverted assertion.

Plan 64 takes the token back: `foo!` is a macro head and `! foo` is negation, told
apart by **adjacency** — which also makes this section's `3 → 1 uses of adjacency`
row read `3 → 2` once plan 52 and plan 64 are both counted. The transferable lesson
is the sequence rather than the token: **a specification retained a construct whose
enabling token a later document spent**, and three plans found the consequence
before one took it. Plan 51's cross-checks exist to make that visible sooner.

### 7.1 What it does to doc 03 §20's estimate

Directional, by the same method as §20 and no more reliable than that table was. The three
costs §20 names as honest all shrink, and the exception it discloses disappears.

| Crate | §20 | Here | Why |
|---|---|---|---|
| `voltaic-parser` | 3.8k | ~3.0k | The call grammar's eight rules collapse to juxtaposition; no `RDX0055` detection |
| `voltaic-emit` | 3.5k | ~3.0k | Nineteen fewer lowerings; no scaffolding emitter |
| `voltaic-fmt` | 2.0k | ~1.4k | §16's canonicalisation table loses half its rows |
| `voltaic-lexer` | 1.2k | ~1.2k | Layout and trivia are unchanged; one sigil table shrinks, one adjacency rule remains |

Call it **12–15k lines** against §20's 14–18k, with the same caveat §20 carried: the number is
a projection from surface area, not a measurement.

---

## 8. Where Haskell-likeness stops

A design register is worth reading for what it refuses. Each entry below is a Haskell feature
with no type-free lowering into Rust, or one whose Haskell spelling would make a Voltaic
expression read as Rust and mean something else — doc 02 §5.2's worst available failure mode,
and the reason doc 03's first decision was to adopt Rust's range spelling.

| Declined | Reason |
|---|---|
| ~~Currying and partial application~~ | **Admitted by plan 56** (`docs/50 - 56 - Currying.md`), and this row was the central limitation until it was. `f x y` is still one saturated call — a Voltaic function is not a chain of one-argument functions — but an **under-applied** one is now a closure over the arguments still missing: `add 1` is `\|__rdx_1\| add(1, __rdx_1)`. Admission test 1 passes because §2.2's signature *states* the arity, so the lowering is fixed by counting top-level arrows and nothing else. The boundary is the file: arity is known for a bare name with a signature in the file being compiled, and every other callee — a path, a `where` local, an equation binder, a lambda parameter, an imported name — is emitted as a call for rustc to judge. §4's table and the composition row below are amended with it. |
| Inclusive `..` | Haskell's `[1..10]` includes 10; Rust's excludes it. Doc 03 decision 34 corrected doc 01 for exactly this reason, and it is not being re-broken for cosmetics. `1..10` exclusive, `1..=10` inclusive. |
| `/=` for inequality | Rust reads `/=` as divide-assign. A silent arithmetic mutation where the author wrote a comparison is the same failure mode as the range inversion. Inequality stays `!=`. |
| `.` for composition | Field access, which §5.2, §6.3 and the whole pipeline depend on. ~~Without currying there is nothing to compose anyway.~~ **The second sentence is retired by plan 56**: there is now something to compose. What stays declined is the *spelling* — `.` is taken — and **plan 66 chooses `>>>` / `<<<`**, superseding plan 58's `>\|` / `\|<`. |
| Cons patterns `(x:xs)` | Rust slices are not cons lists; there is no `Cons` to match. Slice patterns stay Rust's: `[a, rest @ ..]`. |
| Laziness, infinite lists, `seq` | Rust is strict. An iterator is the nearest thing and is already reachable; pretending otherwise would change evaluation order, which admission test 3 forbids. |
| `do` over user monads; `>>=`, `<$>`, `<*>` | Admissible in the narrow sense — `x >>= f` could emit `x.and_then(f)` and rustc would resolve it — but it duplicates `\|> and_then`, and §16 allows one spelling per construct. Declined on simplicity, not on type-freedom; recorded as the venue if `\|>` proves insufficient. |
| Type application by juxtaposition *in types* | `Vec t` would require translating every type expression, which doc 03 §14.1 declines because the translation would lag Rust's type grammar. Only the signature *spine* becomes Haskell; each atom stays verbatim Rust, so `&'a mut [T; 4]` and `impl Iterator<Item = u8>` keep working. |
| Return-type polymorphism, newtype deriving, higher-kinded classes | Each needs type information to pick a lowering. Admission test 1. |
| Haskell's module header and export list | `module M (f, g) where` cannot express Rust's per-item, per-field, scoped visibility. `export`, `export(crate)` and the rest stay as doc 03 §9.6 has them. |
| Custom operator declaration | Already declined by doc 03 §22 for the only reason that matters: Rust has no lowering target. |
| Purity and IO | Voltaic is Rust with a different surface. `mut`, `&mut` and side effects stay exactly as doc 03 has them — test 3 again. |

**Six** Voltaic constructs are **kept unchanged** despite having no Haskell analogue, because
deleting them would cost reachability or ergonomics with nothing to show: `??` and `catch` (doc
03 §12 keeps `Option` and `Result` visually distinct, and both still lower to a `match`),
`try` blocks, string interpolation with Rust's format specs, the `errors` declaration, and
`test` blocks.

This said *"Four"* and listed seven, and `?.` was one of the seven. **Plan 57 deleted it**, and
the reason is that the justification above was false for it rather than merely weak: doc 02
§6.3's lowering was *already* a `?` run inside a closure returning `Option`, so with a method
callable as a function the chain is writable as what it lowered to, and deleting the construct
cost reachability nothing. Against keeping it, `?.` was **one token**, so `(e)?.m ()` silently
meant optional chaining rather than `?` followed by a call — one of the five cases in this
milestone's opening survey that reached no Voltaic diagnostic at the point of the mistake.

The cost it did carry is disclosed rather than denied: a field access **moves**, so the
replacement needs an explicit `&` the lowering used to insert. `examples/fallible.sdx` writes
it, and `benchmarks/pairs/borrowed_chain.sdx` is the IR-equality pair that moved with the
construct, so the zero-cost claim is still measured and not merely asserted.

---

## 9. A page of it, end to end

The same module in both surfaces, with the Rust it produces. **Nothing in the right-hand column
is new.**

```haskell
-- ! Configuration loading.
module config

import std::collections::HashMap
import std::fs

data Config = Config
  { path    : String
  , retries : u32
  , flags   : HashMap<String, String>
  }
  deriving (Debug, Clone)

errors ConfigError
  Missing String        "missing key: #{0}"
  from io::Error as Io  "io error: #{0}"

-- | Read and validate a configuration file.
load : &str -> Result<Config, ConfigError>
load p =
  guard Ok raw <- fs::read_to_string p
    else return Err ConfigError::Io
  flags <- parse_flags raw
  Ok Config { path = p.to_owned (), retries, flags }
  where
    retries = lookup raw "retries" ?? 3

describe : &Config -> String
describe cfg
  | cfg.retries == 0 = "#{cfg.path} (no retries)"
  | otherwise        = "#{cfg.path} (#{cfg.retries} retries)"

summarise : &[Config] -> Vec<String>
summarise cs = [ describe c | c <- cs, c.retries > 0 ]

instance Display Config
  fmt : &Self -> &mut Formatter<'_> -> fmt::Result
  fmt self f = write! f "{}" (describe self)

test "a default config retries three times"
  assert (load "fixtures/empty.toml" |> unwrap).retries == 3
```

**The `module config` line is in the wrong place**, found when plan 14 built it:
`module` is Rust's `mod`, so a header inside the config module declares a child module named
`config` inside itself, and rustc says *"file not found for module `config`"*. What §9 means
is that some *other* file declares it; this page needs no header, because §8 declined the one
Haskell would have given it. The Rust below is unchanged either way — the line is removable.

```rust
//! Configuration loading.
mod config;

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
struct Config {
    path: String,
    retries: u32,
    flags: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    Io(io::Error),
}
// Display, Error and From impls as doc 03 §12.5

/// Read and validate a configuration file.
fn load(p: &str) -> Result<Config, ConfigError> {
    let Ok(raw) = fs::read_to_string(p)
        else { return Err(ConfigError::Io) };
    parse_flags(raw, |flags| {
        let retries = match lookup(raw, "retries") {
            Some(__rdx_v) => __rdx_v,
            None => 3,
        };
        Ok(Config { path: p.to_owned(), retries, flags })
    })
}

fn describe(cfg: &Config) -> String {
    if cfg.retries == 0 {
        format!("{} (no retries)", cfg.path)
    } else {
        format!("{} ({} retries)", cfg.path, cfg.retries)
    }
}

fn summarise(cs: &[Config]) -> Vec<String> {
    cs.iter().filter(|c| c.retries > 0)
        .map(|c| describe(c))
        .collect::<Vec<_>>()
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", describe(self))
    }
}

#[cfg(test)]
mod __rdx_tests {
    use super::*;
    #[test]
    fn a_default_config_retries_three_times() {
        assert_eq!(load("fixtures/empty.toml").unwrap().retries, 3);
    }
}
```

---

## 10. What this does to doc 03's open items

| Doc 03 §23 | Status |
|---|---|
| 1 — confirm `defer`'s generated helper | **Resolved.** `defer` is deleted; no scaffolding is emitted, so doc 01 §36 needs no exception. |
| 7 — validate the universal parenless call against real code | **Resolved differently.** The decision that needed falsifying is replaced by Haskell's rule, which has no ambiguous reading to count. |
| 8 — consider Scala-style placeholder sections | **Closed.** Left sections make the reversed case expressible; no placeholder, no adjacency rule. |
| 6 — run the parser bake-off | Unchanged in substance, cheaper to lose. Layout, equations and guards still rule out an LALR(1) generator, but the intricate call grammar that raised the stakes is gone. |
| 2, 3, 4 — `unsafe`, `extern`, FFI, macro definition, `union`, raw identifiers, `async` blocks | **Untouched.** Haskell has nothing to say about any of them; they remain escape-hatch-only and still need spelling. |
| 5 — re-verify the stabilisation table | **Untouched**, and the floor is unchanged at Rust 1.88 / edition 2024, since the let-chain form survives as `<-` in a condition. |
| 9 — write the two grammar inventories | **Still owed**, and now larger: the MoonScript axis is mostly moot, and the Rust axis needs re-grading against this surface. |

### 10.1 The three things to check before committing further

1. **Does `<-` carry four jobs without reader confusion?** The positions are grammatically
   disjoint, so the parser is fine; the question is whether a statement bind and a condition
   pattern-test look too alike at a glance. Write the examples corpus first, as doc 03 §16
   insists, and read it.
2. **Is the signature line a tax at small scale?** Haskell makes signatures optional because it
   infers; Voltaic cannot, so every function needs one. That is honest — Rust needs them too —
   but a one-line private helper now costs two lines, and `where` locals are where that will be
   felt.
3. **Does the spaced-`::` rule survive contact with real paths?** It is the one adjacency rule
   left and the one piece of lexing where a stray space changes meaning. The mitigation is that
   the formatter normalises both sides and the diagnostic for a malformed signature should name
   the spacing explicitly.

---

## 11. Conversion: what this means for the twelve plans already built

Everything above is the language. Plans 01–12 shipped a working compiler for the *old* surface,
so this section says exactly which parts of it change now and which parts of this document are
the specification for plans 13 onward. The distinction is load-bearing: a document that
declared the whole surface "done" would be claiming an implementation that does not exist.

### 11.1 Converted — `done`

The constructs the compiler actually implements today are the lexical surface, the function
signature, and expression application. All three are converted, and the suite is green with
`fmt` and `clippy -D warnings`:

| From | To | Touches |
|---|---|---|
| `# line` | `-- line` | `voltaic-lexer` `scan_hash` → `scan_dash`; every `.sdx` in the repository |
| `### … ###` | `{- … -}` | the fenced-comment scanner |
| `## outer` | `-- \| outer` | trivia kinds unchanged, markers change |
| `##! inner` | `-- ! inner` | likewise |
| `add = (a: i32, b: i32): i32 ->` | `add : i32 -> i32 -> i32` + `add a b = …` | `voltaic-parser` signature and item parsing, `voltaic-emit` |
| `print add(10, 20)` | `print (add 10 20)` | the expression grammar: juxtaposition |
| `f(x)` | `f x` | likewise |

The `::` adjacency rule arrives with the signature line, because the signature is the first
place it is needed.

Since then, one construct per commit:

| Commit | Construct | § |
|---|---|---|
| `3944640` | `<-` is a token at all — it was lexing as `<` then unary `-` and silently emitting `x < -f` | §3.1 |
| `f3069a2` | ascription `e : T`, and the spaced-vs-tight `::` rule that distinguishes it from a path | §2.2 |
| `0abcdf0` | lambdas `\x -> …`, `$`, and parentheses that are only grouping | §4, §2.1 |
| `2ce7485` | the `\|>` receiver pipeline — parsed since plan 03, previously a `compile_error!` | §4 |
| `3bb6710` | multi-clause equations with pattern binders | §2.3 |
| `aafdb08` | the `\|` guard ladder, and short bodies on the `=` line | §2.3 |
| `a0114c8` | the keyword table: eleven words leave, nine arrive | §7 |
| `643b9b2` | expressions carry their column, so a multi-line lowering is rustfmt-clean | §6 |
| `2f29272` | `case … of`, with guards, alternation and ranges | §2.3, §5.4 |
| `9c4bf31` | `data`, `newtype`, `deriving` and `type` declarations | §2.4 |
| `9c5c48d` | record literals: construction, punning and update | §2.4, §6 |
| `accbd58` | `class` and `instance` | §2.5, §5.3 |
| `c50dbb6` | sections, all six forms | §4 |
| `2e36106` | `??`, and the unit expression `()` | doc 03 §12.1, §7 |
| `a20f038` | synthesised generics, `=>` bounds and `where` locals | §2.2, §5.3 |
| `b12eed1` | comprehensions, list literals, and §6's ascribed-literal lowering | §3.2, §6 |
| `31cee34` | `errors`, and `?` | doc 03 §12.5, §8 |
| `fb80617` | `test` blocks, `assert`, and the panic family | doc 03 §15, §12.6 |
| *this* | `export` and its three scopes | doc 03 §9.6, §8 |

The guard commit closed a hole the clause commit had left: doc 18 §2.3's own example
`fib 0 = 0` did not parse, because the clause body required a line break and an indent. It
was found by writing the doc's example into a *guards* test, not by anything in the clause
commit's own tests — which is the argument for §2.3's examples being fixtures rather than
prose.

### 11.2 What is left to other plans

**Nothing in this document is unimplemented.** Every construct §§1–8 specify is parsed,
lowered, exercised by a test, and — wherever it can be made to run — present in the
`examples/` corpus, which real rustc compiles and executes and which rustfmt is asked to
reformat and does not.

What remains is outside this document: the full pattern grammar (a constructor pattern with
fields, slices, tuples, `@`) is doc 04's plan 33, string interpolation is plan 23, and the
formatter is plan 34. Doc 04's plan rows for those constructs are re-pointed at this document
rather than doc 03; the lowerings they must produce are doc 03's, unchanged, as §1 requires.

`<-` is implemented in statement position (§3.1), which is the position the flattening
lowering belongs to; the other three positions in §3 are listed above with the constructs
that introduce them.

### 11.3 What does not change

The whole of plans 05–12 is surface-independent and survives untouched: diagnostics and their
`RDX` catalogue, the source map, rustc remapping, the run and UI corpora, cargo integration,
the cache, the CLI surface and the watcher. That is the dividend of having built the toolchain
before the language — a surface rewrite costs the lexer, the parser and the emitter, and
nothing downstream of them.

Eight diagnostics are retired by §7 (`RDX0042`, `RDX0043`, `RDX0045`, `RDX0048`, `RDX0051`,
`RDX0052`, `RDX0053`, `RDX0054`, `RDX0055`), and all of them belong to constructs that were
never implemented — so plan 05's `code_coverage.rs` is unaffected, and the catalogue shrinks on
paper rather than in code. All eighteen declared codes are still reached by the UI corpus,
including `RDX0006` by the new `{- -}` fence.

### 11.4 What the conversion actually cost, measured

**The generated Rust is byte-identical.** Four examples, four goldens, and the only diff in
`tests/golden/` is a comment the fixture's own header changed. That is §1's rule — *the
lowerings do not move* — not asserted but measured, and it is the strongest evidence that
this was a respelling rather than a new language.

| | |
|---|---|
| Constructs deleted from the parser | `stmt_call`, `parse_signature`, `parameter`, `POSTFIX_POWER` |
| Constructs added | `application`, `atom`, `parse_spine`, `binders`, `signature_from`, `scan_comment` |
| Net files in `voltaic-parser` | −1 |
| Golden output changed | one comment line |

Two of those deletions are worth naming, because they are §1.1's "simplicity dividend"
arriving on schedule:

- **`stmt_call` is gone.** It existed because doc 03's call grammar could not read a
  parenless call anywhere but statement position, and its own doc comment reserved the
  general case for "plan 19's whitespace-adjacency rule". Juxtaposition made statement
  position unremarkable, so the special case dissolved — and no adjacency rule was needed,
  because doc 18 §2.1 removed the ambiguity instead of resolving it.
- **`POSTFIX_POWER` is gone.** Postfix was a rung on the precedence ladder; it is now
  consumed inside `atom`, so the constant had nothing left to compare against.

### 11.5 Three findings from converting rather than designing

**The comment rule in §5.1 contradicts itself, and the stricter reading was taken.** The
hazard sentence says `x --1` *is* a comment; the mitigation sentence says `--` opens one only
when followed by a space or end of line, which makes `--1` arithmetic. `scan_comment` takes
the mitigation — so `x --1` is subtraction — and records that real Haskell takes the looser
reading. A `---` divider still comments, because people write those.

**Every test that writes a position down by hand failed, and every one should have.** Doc
18's two-line declaration form moves every line number in every fixture, so thirteen
assertions across the lexer, the diagnostics, the source map, `check` and `hello` broke. That
is what those literals are for: a test that computed the line from the file would have passed
while proving nothing, and plan 06 §10.1 and plan 08 §10.2 had both already said so.

**Plan 08's `no_literal_expectations.rs` did not catch the embedded sources this touched.**
Its heuristic wants three or more `\n`-terminated values, and a two-line probe has two — so
`cache.rs`'s `SOURCE` and a dozen like it were found by the compiler rejecting them, not by
the checker. The checker is still worth having (it caught `hello.rs` and `check.rs` in plan
08), and its blind spot is now known: **short** embedded programs pass it. Widening the
heuristic to two lines would flag ordinary two-line string literals, so the honest fix is a
`.sdx`-shape test rather than a line count, and that is a plan of its own.

---

## 12. Result

Doc 18 is the Voltaic surface, and the three constructs the compiler implements speak it. The
rest of this document is the specification plans 13 onward are written against; doc 04's rows
for those constructs are re-pointed here.

```haskell
-- Doc 01 §31 Phase 1's own acceptance test: a typed function, a call, and a
-- print. Prints 30, computed by rustc rather than asserted by hand.
add : i32 -> i32 -> i32
add a b =
  a + b

main : ()
main =
  print (add 10 20)
```

```rust
// Doc 01 §31 Phase 1's own acceptance test: a typed function, a call, and a
// print. Prints 30, computed by rustc rather than asserted by hand.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("{}", add(10, 20));
}
```

344 tests pass; `fmt`, `clippy -D warnings`, `cargo audit` and
`moon run :check :lint :test :cargo-check :cargo-test` are green.

---

Voltaic surface specification · a Haskell-shaped respelling of `docs/03 - Voltaic Surface Design.md`
v0.1.0. Lowerings are quoted from doc 03 unchanged except the three disclosed in §6. Section
numbers in references are doc 03's unless this document is named.
