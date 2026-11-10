# Plan 67 — The Marked Receiver

Plan ID: `marked-receiver`
Depends on: 52 (`type-colon`), 57 (`methods-as-functions`)
Precedes and amends: 63 (`instance-inference`), whose §11.3 is reconciled in §1.6
Milestone: 9 — *surface coherence*
Status: `done` — shipped 2026-11-10

---

## 1. What this plan is

Three changes to one construct, and they are one idea: **stop deriving things from position
that can be written down.**

A `class` stops carrying a type variable for the implementing type and names it `Self`, as
Rust does. A named variable becomes what it looks like — a parameter of the trait. And the
receiver is **marked** rather than found by position: `self` in a type is the receiver.

```
class Show => Shape
  const SIDES : u32
  area : &self -> f64
  scaled : &self -> f64 -> Self
  describe : &self -> &'static str
  describe _ = "a shape"
  unit : f64 -> Self            -- no `self`, so no receiver: an associated fn
  consume : self -> u32         -- by value
  retitle : &mut self -> ()     -- by mutable reference

class Convert t                 -- `t` is the trait's parameter, not Self
  wrap : t -> Self

instance Convert f64 Meters     -- impl Convert<f64> for Meters
  wrap n = Meters n
```

### 1.1 The rule it deletes is one sentence applied to a position

Doc 18 §2.5: *"the first parameter typed by the class variable lowers to `self`"*.
`receiver.rs` implements it as a rewrite of `sig.params[0]` and nothing else;
`receiver_text.rs` decides by comparing the parameter's **type text** against a list that is
`["Self", var]` inside a class and `["Self", target, var?]` inside an instance.

So three things have to be true at once before a reader knows which parameter becomes `self`:
which *position* it is in, which of up to three *spellings* happens to name the implementing
type, and what *borrow prefix* that type carries.

The third is good design and this row keeps it unchanged. Deriving `&self`, `&mut self` and
`self` from `&t`, `&mut Self` and `t` gives all three of Rust's receivers in one notation,
with no sigil invented and nothing for an author to memorise. The first two are what this row
removes.

### 1.2 Plan 57 made the position rule contradict the call

Measured at HEAD, by building a class with a two-argument method:

```
scaled : &t -> f64 -> t    →    fn scaled(&self, __rdx_a1: f64) -> Self
```

and plan 57 calls that `.scaled 2.0 s` — the argument first, the receiver last. The
declaration reads (receiver, argument); the call reads (argument, receiver). For a free
function the two orders agree, because there is no receiver to privilege.

Every Rust method has always had that reversal. The difference is that nobody reads std's
declarations, and a `class` is **the one place a Redox author writes one**.

**Marking the receiver makes the question disappear rather than answering it.** With `self`
in the type, position carries no meaning, so both of these emit the same Rust and an author
writes whichever reads better:

```
scaled : &self -> f64 -> Self        -- Rust's order
scaled : f64 -> &self -> Self        -- plan 57's call order
```

That is worth more than picking one. The alternative rows considered — keep first, or move
to last — each make half the corpus read backwards and leave the rule invisible either way.

### 1.3 The corpus cannot see any of this

`examples/class.sdx` is the only file in the repository with a `class` or an `instance` — plan
63 §1.2 counted it — and **every one of its methods takes a single argument**, where first and
last are the same position.

So the rule this row is replacing is unobservable in this repository. No golden moves if it
changes, no example fails, no gate reports anything. That is exactly the
undiscoverable-construct shape plan 51's `construct_coverage` exists to report, and §4 leaf 4
is where it stops being true.

### 1.4 `trait From<T>` is unreachable, and the near miss fails in the wrong language

`class_decl` reads exactly one `p.ident()` for the class variable, so a second lowercase name
in a member's type becomes a **method** generic rather than a trait one. Measured, by building
and then checking:

```
class Convert t          trait Convert {
  wrap : u -> t      →       fn wrap<U>(__rdx_a0: U) -> Self;
                         }
instance Convert Meters  impl Convert for Meters {
  wrap : f64 -> Meters →      fn wrap(n: f64) -> Meters { Meters(n) }
  wrap n = Meters n      }

redox check  →  error[E0049]: method `wrap` has 0 type parameters but its
                              trait declaration has 1
```

Redox accepts it and rustc rejects it, naming a type parameter the author never wrote. This is
the milestone's **deferred** class precisely — a borrowed surface whose semantics did not come
with it. It also means `impl From<u32> for Meters` and `impl From<f64> for Meters` cannot both
be written, which rules out one of the most common shapes in Rust.

Giving the class a parameter list closes it, and the parameter list is only spellable once the
implementing type stops occupying the one slot there is.

### 1.5 `self` costs one closed-list entry, and `Self` is already accepted

Two measurements make this cheaper than it sounds.

**`Self` already works inside a `class` body.** `MethodContext::for_class` builds
`receiver_names` as `["Self", var]`, so `area : &Self -> f64` lowers to `&self` today. This row
mostly *deletes* the second spelling rather than introducing the first.

**`self` has to stop being synthesised as a generic.** Doc 18 §2.2 synthesises a function's
generic list from its types, and doc 18 §5.3 makes a lowercase name a type variable — so
`&self` would otherwise produce `fn area<Self_>(…)`. `type_vars.rs` already excludes the
primitives through `primitive_type.rs`, and `self` joins that exclusion as a **sibling check in
`type_vars`, not as an entry in `primitive_type`**: `self` is not a primitive type and adding it
there would make that function's name a lie about its contents.

The cost, stated plainly: `self` becomes a reserved spelling in type position — one more
exception to §5.3's *lowercase is a type variable*, beside the seventeen primitives. And `self`
is not really a type, which is a small lie the notation tells. It is Rust's own lie, told in
Rust's own notation, which is why a reader who knows Rust needs no rule at all.

### 1.6 What this does to plan 63, which has not shipped

Plan 63 elides an instance method's signature by taking the class's *"with its single type
variable substituted"*. With `Self`, the substitution for the implementing type is the
**identity** — `&self` is `&self` in every instance — so plan 63's resolver has strictly less to
do: it substitutes only the trait's real parameters, which this row makes named and positional
in the instance head.

Plan 63 is `planned`, so this row lands first and leaf 5 amends it. That is the same sequencing
argument plan 66 §9 decision 6 makes: the cheapest moment to change a surface is before the code
that spells it exists.

**Plan 63 §11.3's two settled decisions both survive, and one of them narrows.** That section
settled that the substitution runs as a *collecting pre-pass*, leaving `receiver`,
`substitute_var` and `generalise` where they are inside `Parser::function`, and that leaf 1's
scope is top-level items only. Neither is disturbed: this row keeps `receiver` in the parser
(§1.7 is why), leaves the pre-pass shape alone, and gives `substitute_var` **less** to do —
with `Self` there is no class variable standing in for the implementing type, so the only
substitutions left are the trait's own parameters, which leaf 3 makes named and positional.

### 1.7 Where the work lands, and why the parser is still right

Plan 61 §11.6 states a rule two rows converged on independently: *a pass that rewrites
`Program` has two consumers and will surprise one of them*, because `redox-fmt` and
`redox-emit` read the same tree. Taken at face value it says a receiver rewrite belongs in the
emitter, or `redox fmt` will rewrite the author's signature.

**Measured, it does not — and the reason is already written down.** `redox fmt` on
`examples/class.sdx` is byte-identical today, even though `receiver.rs` destroys
`sig.params[0]` at parse time. `redox-fmt`'s `signature_line.rs` prints a signature **as a
verbatim slice of the source**, and its doc comment says why: `generalise` already rewrites a
signature's types in place, so `largest : Ord t => Option<t> -> t` is stored as
`Ord T => Option<T> -> T` and the author's lowercase `t` is gone from the tree. Printing from
the AST would rewrite every generic declaration in the corpus into Rust's convention.

So the general rule is right and signatures are already exempt from it, by a mechanism that
exists and is tested. This row keeps the receiver work in the parser beside the code that does
it today. §6 records the one thing that would lift the exemption.

---

## 2. Verified preconditions

Measured 2026-11-10 against the working tree, by reading the parser and by building two probe
programs through `cargo run -p redox-cli -- build` and `-- check`.

| Claim | Verdict |
|---|---|
| `class_decl` reads exactly **one** class variable | true — `class_decl.rs`, `let var = p.ident("expected the class variable, as in `class Shape t`")?` |
| a second lowercase name becomes a **method** generic, and the impl then mismatches | true — `fn wrap<U>(__rdx_a0: U) -> Self` against `fn wrap(n: f64)`, **`E0049`** from `redox check` |
| the receiver is `sig.params[0]` and nothing else | true — `receiver.rs`; it returns early unless the *first* parameter's type matches |
| the decision is made from the type's **text**, against a list of spellings | true — `receiver_text.rs`; `["Self", var]` in a class, plus the target's own name in an instance |
| `&Self` therefore already lowers to `&self` inside a `class` | true — `MethodContext::for_class` puts `"Self"` in the list unconditionally |
| a required method's non-receiver parameter is named by the emitter | true — `fn scaled(&self, __rdx_a1: f64)` and `fn unit(__rdx_a0: f64)`, measured |
| a member with no receiver-typed parameter emits an associated function | true — `fn unit(__rdx_a0: f64) -> Self`, measured; `receiver.rs`'s doc says so and the build confirms it |
| `_` works as the receiver's binder in a default body | true — `describe _ = "a shape"` emits `fn describe(&self) -> &'static str` |
| `examples/class.sdx` is the only file with a `class` or `instance` | true — plan 63 §1.2 |
| …and every method in it takes one argument | true — `area`, `describe`, `perimeter`; §1.3 |
| `type_vars` excludes lowercase primitives through a closed list | true — `type_vars.rs` calls `primitive_type.rs`, seventeen names |
| `instance_decl` splits `instance Show Point` as head-is-class, **first argument is the target** | true — `instance_decl.rs`; §9 decision 3 reverses this for three-or-more names |
| `redox fmt` on `examples/class.sdx` is **byte-identical**, despite the parser destroying `params[0]` | true — run against the working tree |
| …because `redox-fmt` prints a signature line as a **verbatim source slice** | true — `signature_line.rs`, and its doc comment gives `generalise`'s in-place type rewrite as the reason |
| the superclass clause is read speculatively, keyed on the variable | true — `constraints.rs`, because `class Shape t` and `class Debug t => Shape t` begin identically; §4 leaf 2 |

---

## 3. Concrete proof this plan targets

1. `area : &self -> f64` emits `fn area(&self) -> f64`, in a running example with asserted
   stdout.
2. `consume : self -> u32` and `retitle : &mut self -> ()` emit the other two receivers, in the
   same example — all three forms witnessed, where the corpus has only ever shown one.
3. **A two-argument method**, written both ways, emits byte-identical Rust:
   `scaled : &self -> f64 -> Self` and `scaled : f64 -> &self -> Self` both give
   `fn scaled(&self, k: f64) -> Self`. This is §1.2, asserted rather than argued.
4. `unit : f64 -> Self` emits an associated function and is called `Square::unit 2.0`.
5. **`trait From<T>` is reachable**: `class Convert t` with two instances for one type compiles
   and runs, and `redox check` is silent where §1.4 measured `E0049`.
6. `class Show => Shape` emits `trait Shape: Show` — the superclass clause survives losing the
   variable.
7. `instance Square` — inherent, no class — is unchanged, byte-identical in its golden.
8. `examples/class.sdx` migrates and its stdout is unchanged, which is the row's correctness
   definition: the surface moved and the program did not.

---

## 4. Leaves

### Leaf 1 — `leaf-self-in-type`

The marker, and the three places it reaches:

| Site | Change |
|---|---|
| `receiver_text.rs` | match the literal `self` with its borrow prefix, instead of a list of type names |
| `receiver.rs` | find the marked parameter at **any** position, and move it to the front |
| `type_vars.rs` | exclude `self`, in a sibling check rather than in `primitive_type` — §1.5 |

The borrow-prefix logic is kept exactly as it is. `receiver.rs` grows a search where it had an
index, and the move-to-front is the only new behaviour in the file.

**It stays in the parser, and §1.7 is the evidence rather than the habit.** The move-to-front
is a tree rewrite, which is exactly the shape plan 61 §11.6 warns about — but a signature line
is printed by `redox-fmt` as a source slice, so the reorder cannot reach the formatter. An
implementer who moves this to the emitter on §11.6's general advice is solving a problem
`signature_line.rs` already solved, and would have to carry the borrow-prefix reading with it.

### Leaf 2 — `leaf-class-head`

`class_decl` stops requiring a variable and takes an optional parameter list:
`class Shape`, `class Convert t`, `class Pair t u`. `MethodContext` loses `var` and
`receiver_names` entirely — with the receiver marked, there is nothing left for it to carry
except the subject, and the subject is always `Self`.

`constraints` is keyed on the variable today, because `class Shape t` and
`class Debug t => Shape t` begin identically. Without a variable the speculation is simpler, not
harder: `class Show => Shape` has a `=>` before the name and `class Shape` does not.

Emission writes `trait Name<Params>` with the parameters in source order.

### Leaf 3 — `leaf-instance-head`

The instance head's names: **the last is the implementing type, the rest are the trait's
arguments.** `instance Convert f64 Meters` is `impl Convert<f64> for Meters`, which reads in the
same order as the Rust it becomes.

A two-name head is unchanged — `instance Show Point` is `impl Show for Point` under both the old
split and this one — so the reversal in §2's last row is reachable only with three or more
names, which no corpus file has. The parenthesised forms are untouched:
`instance Describe (Wrapper t)` still means an inherent-or-trait impl on a generic target.

One question this leaf has to settle rather than assume: whether a **blanket** impl —
`instance Display t => Describe t`, a bare variable as the implementing type — falls out of this
reading. It may already parse. Leaf 3 asserts the answer either way, in a fixture or in
`docs/GRAMMAR.md`'s `UNREACHABLE` rows.

### Leaf 4 — `leaf-corpus`

`examples/class.sdx` migrated — two class signatures and one inherent — and its golden
re-blessed with stdout unchanged.

Then the three shapes the corpus has never had, which is the larger half of this leaf:

- a **two-argument** method, written both ways, with the byte-identical assertion from §3 case 3;
- an **associated function**, called as `Type::f`;
- a **generic trait** with two impls for one type, which is §1.4's closed gap.

`construct_coverage` gains rows for all three so that none of them can quietly disappear again.

**`construct_coverage.rs` is at its ceiling, and this leaf has to split it.** Exactly 1,500
lines against `line_cap`'s `MAX_LINES = 1_500`, checked as `lines > MAX_LINES` — so the
witnesses above cannot be added until the file is divided. Plan 66 landed net-zero there by
rewriting an existing witness; several new rows cannot. The split follows the
one-construct-per-file rule rather than being a line-count exercise.

Done: `construct_coverage/corpus.rs` and `construct_coverage/shape.rs`, 1,500 → 1,040 lines.

### Leaf 5 — `leaf-plan-63-and-docs`

Plan 63's §1 and §4 restated: the substitution for the implementing type is the identity, and
only the trait's parameters are substituted — §1.6.

Then doc 18 §2.5, doc 03's class section, and `examples/class.sdx`'s own comment, which plan 63
§1.1 already records as carrying a **wrong pointer**: it cites doc 18 §11.5 where the deviation
is recorded in §2.5. That correction is owed either way and this is the leaf that touches the
line.

### Leaf 6 — `leaf-grammar-and-editors`

The tree-sitter grammar and its five editor clients, per plan 52 §2's four readers: a surface
row has `examples/`, the Rust doc comments that quote Redox source, `redox-fmt`'s emit sites,
and the grammar-plus-clients, which are checked against the same corpus and so turn every
example into an `ERROR` node if they drift.

---

## 5. Acceptance criteria

1. All three receiver forms are witnessed by a running example with asserted stdout.
2. A two-argument method written receiver-first and receiver-last emits byte-identical Rust.
3. An associated function emits no receiver and is called through its type.
4. `class Convert t` emits `trait Convert<T>`, and two impls for one type compile and run.
5. `redox check` is silent on the §1.4 program that returns `E0049` today.
6. `class Show => Shape` emits `trait Shape: Show`.
7. `instance Square` is byte-identical in `tests/golden/class.xrs`.
8. `examples/class.sdx`'s stdout is unchanged.
9. A lowercase `self` in a type synthesises no generic parameter, asserted directly — the
   failure mode is `fn area<Self_>(…)` and it would still compile in simple cases.
10. `construct_coverage` green, and `line_cap` green — which requires leaf 4's split, with the two-argument method, the associated function and the
    generic trait all indexed.
11. `redox fmt` round-trips the migrated example and `formatting_is_idempotent` holds.
12. The tree-sitter corpus parses every example with no `ERROR` node.

---

## 6. Out of scope, and what it discloses

**The borrow form still comes from the type, and that is deliberate.** `&self`, `&mut self` and
`self` are read as a prefix on the marker, exactly as they are read today on `&t`. This row
changes *which* parameter is the receiver, not how its form is derived.

**A bound on a trait parameter is not in scope.** `class Convert t` gives the trait a parameter;
constraining it — `trait Convert<T: Display>` — needs a place to put the bound, and `constraints`
currently reads a clause about the *implementing* type. A row that wants it is opening the
question of where a `where` clause lives in a class head, which is larger than this.

**Associated types and consts are unchanged.** `type Unit` and `const SIDES : u32` are Rust's
own and have no Haskell ancestor; nothing here touches them.

**Coherence is rustc's.** Two overlapping impls, an orphan impl, a conflicting blanket — all of
them are errors rustc already produces, and AGENTS.md's rule is that Redox grows no check it can
hand over.

**The `instance` head reversal is a real behaviour change with no corpus witness.** §9 decision 3
chooses the reading; §2's last row records what it reverses; leaf 4's fixtures are the only thing
that will ever observe it. Stated here rather than discovered later.

**Leaf 1's safety rests on `signature_line.rs` slicing, and that is a dependency worth naming.**
Its own doc comment records what would lift the slicing: the AST keeping the source spelling and
the emitter applying `rust_generic` when it writes, which moves `generalise`'s third step from
parse time to emit time. A future row that does that makes every parser-side signature rewrite
visible to `redox fmt` — this one included — and must carry the receiver with it. It is not a
reason to pre-emptively move this row's work; it is a reason for that row to read §1.7.

**This is not the `trait`/`impl` rename.** That option was weighed and declined — §9 decision 6.

---

## 7. Gate

Every plan clears, before it is `done`:

```bash
moon run :check          # per-crate `cargo check` — rustc's verdict
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus the three blessable corpora, `cargo test -p redox-fuzz`, and the tree-sitter corpus — leaf 6
is a grammar change and the grammar is checked against the same examples the compiler is.

---


**The examples sweep catches one of leaf 6's three changes, and the reason differs per
change.** Measured before the row ran, then corrected by running it:

| Change | Caught by `every_example_parses_with_no_error`? | Why |
|---|---|---|
| `class Shape` with no variable, and `class Convert t u` | **yes** | `class_declaration` requires a single `field("variable", $.identifier)`, so both new heads are an `ERROR` node |
| `area : &self -> f64` | **no** | `signature_type` is a flat `repeat1(choice(…))`, so `&self` parses as punctuation plus an identifier in a grammar that knows nothing about the marker |
| `instance Convert f64 Meters` | **yes**, and not for the predicted reason | — |

The third row was predicted as *not caught*, because `instance_declaration`'s `target` is
already `repeat1(choice($.type_identifier, $.identifier))` and a three-name head therefore
parses. The rule text is right and the conclusion was wrong: **a precedence on another rule
wins first.** `constraints` was `prec(1, seq(sep1(seq($.type_identifier, $.identifier), ","),
"=>"))`, and `Convert f64` matches `seq(type_identifier, identifier)` — so the parser
committed to reading the head as a constraint clause and failed when no `=>` arrived,
giving `(ERROR (identifier))` on `f64`. Only an all-uppercase run parsed. Removing the
`prec(1)` is what fixed it; a declared conflict did not, because `tree-sitter generate`
calls it *unnecessary* when a precedence already decides.

So the lesson is narrower and sharper than the prediction: **reading a rule's text does not
tell you what the parser does with it**, because another rule's precedence can intercept
the same token sequence. Leaf 6 still needs its own cases under
`tree-sitter-redox/test/corpus/` — `marked_receiver.txt` is this row's — because the
receiver marker genuinely is invisible to the sweep. Plan 66 found the general shape: its
grammar gates were green before `grammar.js` knew `>>>` existed.

## 8. Emerald reuse

None, and the absence is informative. Doc 02 §4 records Emerald's surface as brace-delimited with
no class system of this shape, so there is no prior art to inherit and no mistake of its to avoid.

The prior art is **Rust's own notation**, which is the point: `&self`, `&mut self` and `self` are
not invented here, they are copied, and a reader who knows Rust needs no rule. The habit this row
borrows from the register rather than from a language is plan 52's: *give one spelling one job,
and delete the positional rule it replaces* — which is what that row did to `::` and what this one
does to `sig.params[0]`.

---

## 9. Decision log

1. **Marked, not positional.** The receiver is written down rather than derived, which makes
   §1.2's declaration-versus-call question stop having an answer that matters: both orders emit
   the same Rust.
2. **`Self`, not a class variable.** The variable was never doing Haskell's job here. In Haskell
   it earns its keep because methods mention `t` in arbitrary positions and there is no `Self`;
   Redox has `Self`, accepts it in a class body already, and needs it for inherent `instance`
   blocks regardless. One name for one thing.
3. **The last name in an `instance` head is the implementing type.** This reverses
   `instance_decl.rs`'s split for three-or-more names, which nothing in the corpus exercises. It
   is chosen so `instance Convert f64 Meters` and `impl Convert<f64> for Meters` read in the same
   order, which a reader can check by eye. The alternative — first-after-class, matching the
   existing code — was rejected because it puts the implementing type between the trait and its
   arguments, where the Rust puts it after both.
4. **`self` goes in a sibling check, not in `primitive_type`.** That function answers *"is this
   lowercase name one of Rust's primitives"* and `self` is not one. A list whose name stops
   describing its contents is how a reader learns to stop trusting the names.
5. **Before plan 63, not after.** Plan 66 §9 decision 6's argument, applied again: plan 63 is
   `planned`, so amending it costs a document, and the substitution it has to perform gets
   simpler rather than harder — §1.6.
6. **Not `trait` / `impl` / `for`.** The words would tell the truth, and `impl Trait` already
   passes through as verbatim type text. Declined because the natural spelling is
   `impl Shape for Square`, which gives `for` a second job in a milestone whose subject is giving
   every token exactly one — and because `instance Shape Square` needs no keyword at all: two
   names mean trait-and-type, one name means inherent. If the Haskell surface is ever abandoned
   as a whole, that is a decision about the language's identity and should be taken as one.
7. **No sigil for the receiver.** `area : .&self -> f64` marks it without reserving a word, which
   is this row's gain without its cost. Declined because plan 57 has just finished making a
   leading `.` mean *method* at a use site, and its own §6 records the one-token ambiguity that
   created as the row's worst property. A third meaning in type position would compound it.
8. **The receiver work stays in the parser.** Plan 61 §11.6's rule about two tree consumers is
   right in general and does not bind here, because `redox-fmt` prints a signature line as a
   source slice — measured, §1.7. Recorded as a decision rather than left implicit, so that an
   implementer reading §11.6 does not relocate it on principle.
9. **The borrow-from-the-type rule is kept.** It is the part of doc 18 §2.5 that is good, and
   deleting it to invent `&`-sigils would cost an author three spellings where they currently
   need none.

---

## 10. Result

Executed 2026-11-10 against `429e269`. All twelve acceptance criteria met; the gate is green on
**12 of 14 steps**, with `moon :test` and `status` red on `conformance_is_current` and
`status_is_current` alone — the two generated documents, whose numbers moved because this row adds
source lines, an example, a golden and one `UNREACHABLE` fixture. Both are the owner's to bless and
neither was touched.

### 10.1 The criteria, measured

| # | Criterion | Verdict |
|---|---|---|
| 1 | all three receiver forms in a running example | `examples/receiver.sdx` — `current : &self`, `bump : &mut self`, `total : self`, stdout asserted |
| 2 | receiver-first and receiver-last emit byte-identical Rust | met, and **more strongly than §3 case 3 asked** — see §10.2 |
| 3 | an associated function, called through its type | `start : u32 -> Self` → `fn start(__rdx_a0: u32) -> Self`, called `Counter::start 3` |
| 4 | `class Convert t` is `trait Convert<T>`, two impls for one type compile and run | `class Scale t` with `instance Scale f64 Meters` and `instance Scale u32 Meters` |
| 5 | `redox check` silent on §1.4's `E0049` program | `crates/redox-cli/tests/check/generic_class.sdx`, and `a_generic_class_with_two_impls_for_one_type_is_accepted` asserts stderr is **empty** |
| 6 | `class Show => Shape` is `trait Shape: Show` | `class Debug => Tally` → `trait Tally: Debug` |
| 7 | `instance Square` byte-identical in the golden | met — `tests/golden/class.xrs` moved **only in its comments**, verified by diffing the generated file against the golden before re-blessing |
| 8 | `examples/class.sdx`'s stdout unchanged | met — `examples/class.stdout` is untouched in the diff |
| 9 | the marker synthesises no generic parameter | `the_marker_synthesises_no_generic_parameter` asserts `!out.contains("fn area<")` directly |
| 10 | `construct_coverage` green with all three shapes indexed | **four** rows added, each proved to fail when `examples/receiver.sdx` is moved aside — §10.5 |
| 11 | `redox fmt` round-trips the migrated example | `redox fmt --check` exits 0 on both `class.sdx` and `receiver.sdx`; `redox-fmt`'s own suite green |
| 12 | the tree-sitter corpus parses every example with no `ERROR` | green, and the sweep was **not** the gate that mattered — §10.4 |

### 10.2 Criterion 2 needed one more change than §4 leaf 1 listed

`__rdx_aN` is numbered by the parameter's position, so a **required** two-argument method emitted
`fn scaled(&self, __rdx_a1: f64)` receiver-first and `fn scaled(&self, __rdx_a0: f64)`
receiver-last — not byte-identical, and §2's own row *"a required method's non-receiver parameter
is named by the emitter"* is where the number comes from. §3 case 3's example has a binder (`k`),
so the criterion as *illustrated* held either way; as *written* it did not.

`receiver` now renumbers a generated name by its place in the **Rust** signature. Nothing existing
moved, because a receiver written first already numbered that way — which is why this is a
correction rather than a surface change, and it is asserted in
`a_two_argument_method_emits_the_same_rust_in_either_order` by `assert_eq!` on the whole output.

### 10.3 Leaf 1's move-to-front belongs in the emitter, and §1.7's exemption is narrower than it reads

**§9 decision 8 is right about the parser and §1.7 is right about `signature_line.rs`. Both are
about the signature, and the pattern row is a second consumer neither considered.**

`emit_clauses` pairs `sig.params` with each clause's **pattern row by index** to build a `match`
scrutinee. So moving the receiver to the front of `sig.params` in the parser, as leaf 1 specifies,
misaligns the arms of any method with several equations or a testing pattern — silently, into Rust
that can still compile. Moving the pattern row with it is worse: `redox-fmt`'s `clause_text` prints
an equation's patterns **from the tree**, where `signature_line` slices the signature from the
source, so the formatter would print `scaled : f64 -> &self -> Self` above `scaled self k = …` —
a program that does not re-parse to itself.

So the marker reading and the borrow-prefix logic stay in the parser exactly as decision 8 requires,
and `emit::signature` floats the receiver to the front — four lines, in the one consumer that wants
Rust's order. **Plan 61 §11.6's two-consumers rule does bind here**, for the pattern row, and plan
67 §1.7's measurement that it does not bind is true only of the signature line. Recorded in
`emit/signature.rs`, `parser/receiver.rs` and here, because an implementer reading leaf 1 will
otherwise put it back.

### 10.4 Three findings in the grammar, two of which the examples sweep could not report

The coordinator's amendment said the examples sweep catches one of leaf 6's three changes. Measured,
it catches **two**, and there were three defects rather than one change:

1. **`class Shape` and `class Convert t u`** — caught, as predicted: `field("variable", $.identifier)` was a required single identifier.
2. **`instance Convert f64 Meters` was an `(ERROR (identifier))` on the `f64`**, which the sweep *does* catch, contradicting the amendment's claim that a three-name head already parses. Only an **all-uppercase** run parsed: `Convert f64` is also the shape of the constraint clause `Convert f64 =>`, and `constraints`' `prec(1)` committed the table to that reading before the `Meters` arrived. A declared `[$.constraints, $.instance_declaration]` conflict did not help — `tree-sitter generate` reported it *unnecessary*, because a precedence beats a declared conflict. Removing the `prec(1)` fixed it.
3. **`&mut` was not in `signature_type` at all.** `bump : &mut self -> ()` fell out of the `signature` rule entirely. Doc 18 §2.5's own `increment : &mut Self -> ()` would have shown this years ago; nothing in the corpus had a `&mut` in a signature until this row.

**`area : &self -> f64` needed no grammar change**, exactly as the amendment said: `self` already
lexes as `$.identifier` there. Adding `$.self_expression` to `signature_type` so a highlighter could
colour the marker was tried and **does not work** — `identifier` wins the lexical conflict, measured
as `(identifier)` in the parsed tree — and was removed rather than forced with a token precedence
that would have changed what `Self` means in every type position.

`tree-sitter-redox/test/corpus/marked_receiver.txt` is leaf 6's own gate: three cases, and a
deliberately wrong expectation in the second was confirmed to fail and be named before the file was
corrected.

**One pre-existing grammar defect is disclosed and not fixed.** A signature whose return type is
`()` *inside a block* loses the `[$.signature, $._expression_part]` conflict and comes out as an
`expression_statement` holding an `ascription`, two `(operator)`s and a `(parenthesized)` — with
**no `ERROR` node**, so the sweep cannot see it. `a : u32 -> ()`, `a : &Self -> ()` and
`a : &self -> ()` all do it; the same lines at item level parse correctly, which is why
`classify : i32 -> ()` in `examples/control.sdx` never showed it. `prec.dynamic` on `signature` and
a token `prec` on `_type_punctuation` were both tried and changed nothing, so the fix is structural
and belongs to a row that owns the grammar. Recorded in `grammar.js` beside the rule.

### 10.5 The corpus leaf, and what it had to add before green meant anything

`examples/class.sdx` migrated with its generated **code** byte-identical — only its comments moved —
which is the strongest possible form of criterion 8 and also the measure of how little the corpus
could see. The whole row is invisible in that file.

`examples/receiver.sdx` is new and is where it stops being invisible: three receiver forms, a
two-argument method written **both** ways, an associated function called through its type, a
superclass clause, a parameterised class, and two impls of it for one type. 89 lines, run under real
rustc with asserted stdout, `redox check` silent.

Four `construct_coverage` rows, each verified by moving `examples/receiver.sdx` aside and confirming
the gate **names it**: *Generic traits*, *Two impls of one trait for one type*, *The receiver, marked
in a type*, *Associated functions*. Each witness is built from what makes the construct different
rather than from its own spelling — `has_a_late_receiver` demands a `self` marker **past at least one
`->`**, because `Toks(["&", "self"])` would be satisfied by `examples/class.sdx`'s receiver-first
one-argument methods, which is the decoy it exists to distinguish itself from.

**`construct_coverage.rs` was split**, as the amendment required: it was at `line_cap`'s ceiling
exactly and this row adds four witnesses. 1,500 → 995 lines, by what the pieces are rather than
where the middle fell — `construct_coverage/corpus.rs` is the haystack and the shape of a claim
about it, `construct_coverage/shape.rs` is the predicates a token run cannot express, and the root
keeps the index, the witness table and the assertions. Reached by `#[path]` rather than by renaming
the target to `construct_coverage/main.rs`, so `--test construct_coverage` still names it.

### 10.6 Plan 63's two decisions survive, and §1.6's claim about `substitute_var` is half wrong

Checked against the code rather than assumed, and written up in plan 63 §11.6.

**Both §11.3 decisions survive untouched.** `generalise` is still called from `Parser::function`;
`receiver` and `substitute_var` are still called from `class_member` immediately after it. No call
site moved. Leaf 1's top-level-items-only scope survives and gains a second benign case —
`examples/local.sdx` declares a `class` and an `instance` inside a *function body*, which restates
its signature today and degrades to exactly that.

**§1.6's "strictly less to do" is half right.** `MethodContext` lost `var: Option<String>` and
gained `vars: Vec<(String, String)>`: it lost one entry and **gained a dimension**. For
`class Shape` — every class in the corpus bar one — `substitute_var` is called **zero** times where
it was called once. For `class Convert t` it is called once, with the subject `T` rather than `Self`.
For `class Pair t u` it is called **twice**, which is more than before and was unspellable before,
and which is precisely the shape plan 63 §1.4 already refuses to elide. What this row removed
outright is the substitution *for the implementing type*.

It also made plan 63 leaf 2's byte-identity **stronger**: a class's rendered `fn area(&self) -> f64`
and an instance's restated one are now identical **by construction**, because both spell the
receiver the same way. Before this row they agreed by coincidence — the class's `&t` substituted to
`&Self`, the instance's `&Square` dropped as a receiver, and the two met in the middle.

### 10.7 `receiver.rs` and doc 18 §2.5 had **not** drifted, and §2's "only file" claim had

The pair the brief flagged as most likely to disagree agreed exactly. Doc 18 §2.5 said *"the first
parameter typed by the class variable lowers to `self`"*; `receiver.rs` returned early unless
`sig.params.first()`'s type matched, and its own doc comment quoted §2.5 verbatim. The code, the
comment and the specification were one rule. That is the row's premise rather than a defect, and it
is worth stating because it is the opposite of what today's pattern predicted.

**What had drifted was §2's corpus claim.** *"`examples/class.sdx` is the only file with a `class`
or an `instance`"*, attributed to plan 63 §1.2, is **false**: `examples/local.sdx` has
`class Named t` and `instance Named Shade` inside a function body — plan 55 leaf 5's local-items
example. It was found by the examples sweep failing with `E0107` and `E0599` after the migration,
not by reading. One migrated file in a two-file corpus is half of it.

Two smaller drifts in the same area:

- **§2's row *"`&Self` therefore already lowers to `&self` inside a `class`"*** was true and is now
  deliberately false: `receiver_text` matches the literal `self` and nothing else, so
  `perimeter : &Self -> f64` in `examples/class.sdx` had to migrate. §1.5's *"this row mostly
  deletes the second spelling rather than introducing the first"* understates it — both spellings
  were replaced by a third.
- **`substitute_var`'s doc comment** claimed the class variable *"becomes `Self`"*, which this row
  makes false in every case; it now says what it does. Its citation of doc 18 §11.5 for the
  nested-occurrence gap is one of the three stale `§11.5` pointers plan 63 §11.5 counted, and is
  left alone as that row's to fix.

### 10.8 Four smaller findings

- **`TypeApp::is_var` had to learn about primitives.** *"A lowercase first letter and nothing else"*
  made `instance Convert f64 Meters` emit `impl Convert<F64> for Meters`, because a primitive is
  lowercase and concrete — the same correction doc 18 §5.3 already carries for `type_vars`. The
  closed list moved to `redox-ast` so there is **one** copy; the parser's own
  `primitive_type.rs` is deleted and `type_vars` imports it. Asserted by
  `a_primitive_class_argument_is_not_a_type_variable`.
- **A blanket impl parses and emits `impl Describe for t`.** Leaf 3 asked for this to be settled
  either way: `TypeApp::vars` reads a type application's *arguments*, and in
  `instance Display t => Describe t` the variable is the **head**, so the generic list comes out
  empty and `rust_text` writes the head verbatim. `E0425` from rustc. Graded `UNREACHABLE` with
  `tests/unreachable/blanket_impl.sdx`, which is the ninth gap and why `docs/STATUS.md` moves.
- **`redox-fmt` was dropping an `instance` target's parentheses, and the new head split made that
  wrong.** `instance Describe (Wrapper t)` printed flat as `instance Describe Wrapper t`, which
  round-tripped by accident under the old *first*-argument split and does not under the last-name
  one. A target with arguments is now parenthesised — and the inherent generic form
  `instance (Wrapper t)` was losing its parentheses under **both** splits, so this is a fidelity bug
  the change exposed rather than one it introduced.
- **A `class`'s superclass clause could not be spelled at all before this row.** `constraints`
  requires a trait name **and** a variable, so `class Show => Shape` would have rewound and then
  failed at the body. §2's row *"the superclass clause is read speculatively, keyed on the
  variable"* is true and understates the consequence: leaf 2 needed a new `class_supers` reader, not
  an adjustment, and criterion 6 was unreachable until it existed.

### 10.9 What was out of lane and is left

- **Doc 03's class section is not this row's.** Leaf 5 names it, but doc 03 §9.3's `class Counter(value: i32)` is doc 01 §17.7's struct-plus-impl `class`, which doc 18 §2.5 **retired** and `docs/GRAMMAR.md` already grades `REMOVE`. Editing it would document a construct the language does not have.
- **`docs/CONFORMANCE.md` and `docs/STATUS.md` are stale by exactly the counts this row moves**, listed in §10.1. Not blessed.
- **The `-> ()`-in-a-block grammar defect** of §10.4, which is pre-existing, silent, and structural.
