# Plan 62 — Ascription Completion

Plan ID: `ascription-completion`
Depends on: 52 (`type-colon`), 55 (`expression-orientation`)
Milestone: 9 — *surface coherence*
Status: `planned`

---

## 1. What this plan is

Ascription becomes the one way to pin a type, and starts working in the three positions it is
currently missing from. Its lowering stops emitting a block where a `let` will do.

Plan 52 moved the token to `:`. This row finishes the construct:

| | Today | After |
|---|---|---|
| Binding position | a three-line block | `let x: T = e` |
| In a pattern | `RDX0101` | `Err e : VersionError ->` |
| On a `try` block | ambiguous `E0282` | `r : Result<i32, Oops> = try …` |
| Type arguments | no spelling | **still none**, deliberately — §1.4 |

### 1.1 The block lowering is the visible cost

```
HashMap::new () : HashMap<String, u32>

->  {
        let __rdx_t: HashMap<String, u32> = HashMap::new();
        __rdx_t
    }
```

Three lines where one would do, and in a receiver position it puts a brace in a method chain.
`examples/module.sdx` works around exactly that, and says so:

> Each map is bound by a `where` local first: `.len ()` on an ascription block would put a brace
> in a method chain's receiver, which rustfmt reformats.

So the lowering has already cost the corpus a workaround and a comment explaining it. In binding
position — which is the overwhelming majority of uses — the correct emission is Rust's own
annotated `let`, and the block is only needed where the ascription is genuinely nested.

### 1.2 The three missing positions, each met while writing a real program

**In a pattern.** Pinning an error type at the arm that handles it is the natural spelling and is
`RDX0101: expected `->` after a `case` pattern`:

```
case parsed of
  Err e : VersionError -> "error: #{e}"
```

**On a `try` block.** A `try` whose `?`s widen through `From` has an ambiguous error type, and
rustc says so — `E0282: type annotations needed for Result<(bool, Version), _>`. Today the only
fix is to give the block its own function so the *signature* pins it, which is what
`semver`-shaped code does and which is a real restructuring forced by a missing annotation.

**On a binding whose value is a call.** Covered by §1.1's `let` emission.

### 1.3 How much ascription is actually load-bearing

Measured across the new corpus's 905 code lines:

| | Count |
|---|---|
| `::` type annotations total | **186** |
| …of which are signature lines | 102 |
| …**remaining: ascription at a value** | **84** |

Eighty-four in 905 lines — one every eleven lines. Ascription is not an occasional convenience;
it is how this language talks about types at the value level, and three missing positions in a
construct used that often is a gap worth a row.

### 1.4 Turbofish is declined, on a measurement

Plan 52 made `::<T>` *spellable* by giving `::` a single meaning. This row declines to spell it,
because the case that motivated wanting it turns out not to need it.

The hard case was reading an id_effect capability, where the type appears nowhere in the
expression:

```
limits : Effect<Arc<Thresholds>, Never, GateEnv>
limits = need () |> effect
```

`need::<Thresholds> ()` is `RDX0103` today, so the working spelling was a lambda parameter
annotation. Verified by compiling the hand-written Rust: **inference reaches it from the
signature alone.** `need().effect()` resolves `T = Thresholds` from the declared return type,
and so do `need().map(|t: Arc<Thresholds>| …)` and a fully unannotated variant. All three
compiled and ran.

So the declared type is sufficient wherever it is written down, and this row's job is to make it
writable everywhere rather than to add a second mechanism. One way to pin a type is better than
two, and the one that composes with the signature line is the one the language already has.

Disclosed in §6: a position where neither a signature nor an ascription can reach is a position
where this decision is wrong, and none has been found.

---

## 2. Verified preconditions

Measured 2026-11-10, against `redox` at `194fdf7`.

**The block lowering, in a receiver position.** Written in the syntax that parses *today* — `::`
for ascription, and Rust's method call — because this row's measurements must be reproducible at
`194fdf7` rather than in plan 52's and plan 57's surface:

```
(HashMap::new () :: HashMap<String, u32>).len ()
->  ({ let __rdx_t: HashMap<String, u32> = HashMap::new(); __rdx_t }).len()
```

`tests/golden/module.xrs` carries the same shape for both of `examples/module.sdx`'s maps.

**Pattern ascription is refused:** `Err e : VersionError ->` is `RDX0101`.

**A `try` block's error type is ambiguous without a signature:**

```
parsed = try
  v = (parse version_text)?
  r = (parse_req req_text)?
  Ok ((satisfies (&r) (&v), v))
->  rustc E0282: type annotations needed for `Result<(bool, Version), _>`
```

**Signature-driven inference reaches a capability read.** Three forms compiled and ran against
`id_effect 0.13.0`: `need().effect()`, `need().map(|t: Arc<Thresholds>| …)` and an unannotated
closure. This is §1.4's evidence and the reason turbofish is declined rather than deferred.

**`as` casts are `UNREACHABLE`.** `docs/GRAMMAR.md` grades them so (plan 46), and plan 48
counted **12** blocking lines carrying one. Related to this row by proximity and not by
mechanism — see §6.

---

## 3. Concrete proof this plan targets

**`examples/module.sdx` loses its `where`-local workaround**, and the comment explaining the
workaround is deleted because the reason is gone.

Five more:

1. **Binding position emits `let x: T = e`**, asserted in a golden. This is the row's main
   output change and the goldens are where it is visible.
2. **A nested ascription still emits a block**, also asserted — because the rule is
   position-dependent and a reader should be able to see both halves.
3. **Pattern ascription**, on the `Err e : VersionError` shape from §1.2.
4. **A `try` block ascribed at its binding**, replacing the forced-function-extraction pattern,
   with the function version deleted from the example that used it.
5. **`xs |> collect : Vec<i32>`** — ascription binds loosest of anything in an expression, which
   is recorded in `parse_expr.rs` and not in doc 18 §6 (that section gives only the lowering), so
   this must ascribe the pipeline rather than the `collect`. The precedence case plan 52 moved the
   token without changing.

---

## 4. Leaves

### Leaf 1 — `leaf-let-emission`

Binding-position ascription emits `let x: T = e`. One emitter branch, keyed on whether the
ascription is the whole right-hand side of a binding.

### Leaf 2 — `leaf-nested-block`

The block lowering retained for genuinely nested positions, unchanged. Kept as a separate leaf so
§3 case 2's assertion has an owner and the two paths are visibly two.

### Leaf 3 — `leaf-pattern-ascription`

`pattern : T` in a `case` arm, a `guard`, and an equation row. Emits Rust's pattern type
annotation where one exists and falls back to annotating the scrutinee where it does not — a
distinction leaf 3 must establish, because Rust's pattern grammar does not accept a type
everywhere Redox's will offer one.

### Leaf 4 — `leaf-block-ascription`

Ascription on a `try` or `async` block, which after plan 55 are block modifiers rather than
constructs — so this is ascription on a block expression and needs no per-construct handling.

### Leaf 5 — `leaf-corpus-and-docs`

`examples/module.sdx`'s workaround removed; `ascription.sdx` extended with the three new
positions. Doc 18 §6 amended — note it discloses **four** lowerings, not three, so the ascription
row is one of four and the others must not be disturbed. `docs/GRAMMAR.md` regraded, and the turbofish
decline is written into `docs/DECLINED.md` with §1.4's measurement as its reason — plan 47 built
that document for exactly this.

---

## 5. Acceptance criteria

1. Binding-position ascription emits `let x: T = e`; a nested one emits a block. Both asserted in
   goldens.
2. `examples/module.sdx` no longer needs a `where` local to avoid a brace in a receiver, and the
   comment is deleted.
3. Pattern ascription works in a `case` arm, a `guard` and an equation row.
4. A `try` block is ascribable at its binding, and the forced-function-extraction workaround is
   removed from the example that used it.
5. `xs |> collect : Vec<i32>` ascribes the pipeline, asserted.
6. Turbofish is **not** added, and `docs/DECLINED.md` carries the decline with §1.4's three
   verified forms as evidence.
7. All 29 examples green, no rustc output, stdout unchanged.
8. `construct_coverage` green with the three new positions indexed.
9. Doc 18 §6 amended; the generated-Rust shape change is reflected in the re-blessed goldens and
   reviewed.

---

## 6. Out of scope, and what it discloses

**`as` casts stay unreachable, and ascription is not a substitute.** The confusion is natural and
the answer is firm: `e : T` **asserts** a type and `e as T` **converts** a value. Emitting `as`
for an ascription would silently truncate a numeric, which is doc 02 §5.2's worst failure mode
with arithmetic attached. Plan 46 graded the gap and plan 48 priced it at 12 lines; closing it is
a separate row and should not be folded in here on the grounds that both involve a type and a
colon.

**Pattern ascription is narrower than it looks.** Rust accepts a type annotation in some pattern
positions and not others, so leaf 3's fallback — annotate the scrutinee instead — means some
ascriptions move in the generated Rust. Where neither works, the diagnostic must say so rather
than emit something that does not type-check.

**No turbofish, and the condition under which that is wrong is stated.** §1.4. If a position is
found where neither a signature nor an ascription can pin a type, this decision should be
revisited with that position as the evidence. None has been found; the search was not exhaustive.

**The three-line block does not disappear.** It remains for nested positions and will still
appear in generated Rust. The claim is that it stops appearing where a `let` suffices, which is
most places.

---

## 7. Gate

```bash
moon run :check          # per-crate cargo check -- rustc's answer
moon run :lint           # cargo fmt --check && cargo clippy -D warnings
moon run :test           # cargo nextest run, then the doctests
moon run :format         # treefmt
```

Plus: every example transpiles, compiles under rustc with no output, runs, stdout unchanged;
goldens re-blessed and reviewed — this row changes generated Rust, so the diff is the deliverable;
`tests/ui/` fixture for an unascribable pattern position; `construct_coverage` green.

---

## 8. Emerald reuse

Plan 47's `docs/DECLINED.md`, as the home for §1.4's turbofish decline — which is the first
decline in this repository backed by a *compiled measurement* rather than an argument. Doc 03 §22
and `DECLINED.md` exist because Emerald's declines were folklore; this row adds one with evidence
attached, which is the standard the document should hold to.

Nothing else. Emerald had no ascription construct; its type annotations were Rust's, in Rust's
positions.

---

## 9. Decision log

1. **`let x: T = e` in binding position.** The block is correct and verbose, and `examples/module.sdx`
   has already paid for it once in a workaround and a comment.
2. **Keep the block for nested positions rather than finding a one-expression trick.** An
   identity-function call would be one line and would introduce a name into generated Rust that
   no program defines — which is precisely what plan 45 found wrong with `rust """…"""` emitting
   `rust(…)`.
3. **Decline turbofish, on the measurement in §1.4.** Two mechanisms for one job, where the
   existing one was verified sufficient on the hardest case available.
4. **Pattern ascription with a scrutinee fallback.** The honest alternative is to refuse pattern
   ascription wherever Rust refuses a pattern type, which would make the construct's
   availability depend on a Rust detail the author cannot see.
5. **Do not fold in `as` casts.** §6. Same colon, different operation, and conflating them
   changes values.

---

## 10. Result

*Not yet executed.*

---

## 11. Corrections from orientation

Written after reading the code and probing it. **Two criteria are already true,
one contradicts a leaf, one is impossible in Rust, and a live hazard this row
should own is owned by nobody.** The row is real but much smaller than it reads.

### 11.1 Criterion 4 already works — measured both ways

§1's table row 3, §1.2 and leaf 4 all say an ascribed `try` block is the fix for
an ambiguity whose *only* current remedy is to give the block its own function:
*"a real restructuring forced by a missing annotation"*.

Measured. An untyped `try` with two `From`-widening `?`s does reproduce the
ambiguity:

```text
E0282  × type annotations needed for `Result<(i32, f64), _>`
```

And the same block with the ascription at its binding —

```text
r : Result<(i32, f64), Box<dyn Error>> = try
  …
```

— **compiles clean, with no output at all.** The emission is
`let r: Result<i32, String> = (|| { … })();`. So the remedy ships today, in both
directions, and what remains of leaf 4 is a test and an example. This does not
sink the row; it shrinks it.

### 11.2 Criterion 2 contradicts leaf 2

Criterion 2 wants `examples/module.sdx`'s `where` locals — which exist only so
that `.len ()` is not called on an ascription block — to become unnecessary, and
§3 says *"the comment explaining the workaround is deleted because the reason is
gone"*.

Leaf 2 keeps the block for nested positions. **A receiver is a nested
position.** Measured:

```rust
let n = ({
    let __rdx_t: HashMap<String, u32> = HashMap::new();
    __rdx_t
}).len();
```

So the brace stays, the locals stay, and the comment stays true. The only thing
leaf 1 buys that file is re-spelling its locals as
`a : HashMap<String, u32> = HashMap::new ()` — **which already works today**.
Either criterion 2 or leaf 2 has to go, and decision 2's refusal of an
identity-function trick forecloses the third option.

### 11.3 Criterion 3 is impossible for its own headline example

rustc accepts a type in a pattern in exactly three places: `let PAT: T = e`,
let-else, and function or closure parameters. **A `match` arm is not one of
them**:

```text
error: expected `|`, found `:`
  |         Err(e): String => …
  |               ^ ------ specifying the type of a pattern isn't supported
```

So leaf 3's *"emits Rust's pattern type annotation where one exists"* is vacuous
for the `case`-arm case the row leads with, and the stated fallback — annotate
the scrutinee — **cannot express §1.2's example**: `Err e : VersionError` pins
only the error parameter, and annotating the scrutinee requires naming the `Ok`
type too. The only workable lowering is a `let` in the arm body:
`Err(e) => { let e: VersionError = e; … }`. This row never names it.

For the other two positions the ascription is near-redundant: `guard`'s let-else
annotates the **source** rather than the binder, and an equation row lowers to a
fn parameter, which the signature line already types.

### 11.4 The `split_format_spec` hazard is unowned, and it is live

`split_format_spec` splits an interpolation at the first depth-zero `:` that is
not part of `::`. After plan 52 a lone `:` inside `#{…}` can be an ascription,
and it is taken as a format spec:

```text
print "#{total : u64}"   ->   println!("{: u64}", total);
```

`redox build` **exits 0 silently**, and `redox check` then reports rustc
objecting to a *format string* — *"invalid format string: expected `}`, found
`u`"* — against a line on which the author wrote a legal ascription.

**Nobody owns it.** Plan 52 §4 assigned it to plan 52's own leaf 3 — *"Leaf 3
must teach it the difference; `tests/golden/strings.xrs` is the witness"* — and
plan 52 §10 shipped recording only that it is *"not yet triggered"*. This row
mentions interpolation nowhere: not §1, not §4, not §5. It belongs here,
because this row is what makes a `:` inside an expression ordinary.

Note what the fix may **not** be: splitting on whitespace. That would make
`#{x : T}` and `#{x:.2}` differ by a space, which is the exact hazard class plan
52 existed to delete.

### 11.5 Criterion 8 has no target

*"`docs/GRAMMAR.md` regraded"* and *"three new positions indexed"*. That
document is an inventory of **Rust's** grammar — its §1 is headed *"Rust's
grammar"* — and Rust has no ascription construct, so there is no ascription row
and no pattern-type-annotation row to regrade. Plan 51's `construct_coverage`
reads the same rows, so there is nothing for it to index either.

### 11.6 Verified true, and smaller findings

- **The lowering is as §1 says**, confirmed against `emit/expr.rs` and
  `tests/golden/module.xrs`. Parse is gated at `min == 0`, the loosest binding
  power, with `$` at 1.
- **The section is doc 18 §6, not §7.** §6 is *"Four new lowerings, disclosed"*;
  §7 is *"The simplification dividend"*. This row has it right; a reader coming
  from plan 52 may not.
- **Doc 18 §6's own correction is already implemented.** `ascribed_list.rs`
  takes the head as `ty.text.split('<').next()` and is called *inside* the
  block, so `[1, 2] : Vec<i64>` keeps its generic argument. The correction is
  verbatim in the document and restated in the file's doc comment.
- **Criterion 1 is half already-true.** The `let x: T = e` path exists, keyed on
  `Binding.ty`, and `x : T = e` already parses —
  `examples/expressions.sdx:19` emits `let total: i32 = 40 + 2;`, and
  `mut buffer : String = String::new ()` works. The real content is rewriting
  `x = e : T` into that existing path.
- **Criterion 5 is already true.** `xs.iter () |> collect : Vec<&i32>` ascribes
  the pipeline, not the `collect`, and emits the block around the whole chain.
- **`::<T>` is safely declined.** `::` is now unconditionally a path separator
  and every site after it demands a name, so `Vec::<i32>::new ()` is `RDX0103`.
  Nothing accepts a turbofish by accident. `declined.rs` checks one direction
  only, so adding the entry is free.
- **30 examples, not 29.**
- §1.3's measurement — *"905 code lines, 186 `::` annotations, 84 at a value"* —
  is from the `id_effect` port corpus and **cannot be settled from this
  repository**; no `id_effect` fixture exists in the tree, which also makes
  §1.4's three compiled forms unverifiable here.
