# Plan 16 — Module-Scope Bindings

Plan ID: `module-scope-bindings`
Depends on: 03 (`parser-core`), 14 (`paths-colon-colon`)
Milestone: 4 — *specification-correct*
Status: `planned`

---

## 1. What this plan is

`const` and `static` at module scope, and `const fn`.

Doc 03 §13.4 **corrects doc 01 §4.3**, per doc 02 §10.5, and names two defects in the
specification:

> Rust forbids `let` at module scope, so *every* top-level binding must be `const` or
> `static` regardless of case; and Rust `const` and `static` require an explicit type, so
> the specification's `PI = 3.14159` cannot be lowered at all.

So a module-scope binding takes an explicit type and lowers to `const` by default, and
*"the uppercase-name convention is a lint, not semantics"*.

Doc 18 §2.2 gives the spelling, with the type after the same `::` every other type in Redox
uses:

```text
const MAX :: usize = 1024          const MAX: usize = 1024;
static COUNT :: u32 = 0            static COUNT: u32 = 0;
export const LIMIT :: usize = 8    pub const LIMIT: usize = 8;

square :: const u32 -> u32         const fn square(n: u32) -> u32 {
square n = n * n              →        n * n
                                   }
```

### 1.1 Why `const fn` is in this plan and not row 39's

`const fn` is in **doc 18 §2.2's own example**, two lines below `const MAX`, and it is the
same keyword in the same position: a marker between `::` and the spine. Splitting them
would mean reading `Kw(Const)` after `::` in one plan and acting on it in another, with the
intervening state being *"`const` in a signature is swallowed into the first parameter's
type"* — which is what happens today and is wrong, since `const u32` is not a Rust type.

Row 39 keeps `async`, which is a different keyword and needs an executor story this one
does not.

### 1.2 The error doc 01 §4.3 deserves

`PI = 3.14159` at module scope is already rejected — doc 18 §2.3 reads a top-level name
followed by `=` as an *equation*, so the parser asks for the `::` signature line it has no
reason to expect. The message says `expected `::` after the name, to begin the signature`,
which is true and unhelpful: the author wrote doc 01 §4.3's spelling, and what they need to
know is §13.4's correction.

This plan gives that case its own message. It is the one piece here that is not
passthrough, and it exists because doc 01 §4.3 was *published* — someone will write `PI =
3.14159`, and the specification told them to.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `const` and `static` are reserved | `keyword.rs` | `Keyword::Const`, `Keyword::Static`, both from doc 01's set |
| `const NAME :: T = v` is already parsed **inside a class body** | plan 25, shipped under plan 13 | `ClassMember::AssocConst { name, ty, value }` — the same shape, one level down |
| The item dispatch already reads a visibility marker first | plan 24 | `visibility()` runs before the keyword test, so `export const` needs no new reader |
| `const` in a signature is swallowed today | `redox build` on `square :: const u32 -> u32` | the parameter type becomes `const u32`, which rustc rejects |
| A new diagnostic needs a UI case | plan 05's `code_coverage.rs` | every `Code` variant must be reachable from `tests/ui/`, so §1.2's message needs a fixture |

---

## 3. Concrete proof this plan targets

`examples/constants.sdx`: a `const`, a `static`, an `export const`, a `const fn`, and each
of them used — so rustc's `dead_code` silence is part of the proof, as it is for every other
example.

And a UI case for §1.2, because a message nothing reaches is a message nothing checks.

---

## 4. Leaves

### Leaf 1 — `leaf-const-item`

`Item::Const(ConstDecl { name, ty, value, is_static, visibility, span, leading })`, parsed
from `const`/`static` and emitted as `const NAME: T = v;` / `static NAME: T = v;`.

One node for both, with a flag, because they differ by one keyword and nothing else — and
doc 03 §13.4 treats them as one construct with a default.

### Leaf 2 — `leaf-const-fn`

An optional `Kw(Const)` between `::` and the spine, carried on `Signature` and emitted as
`const fn`. The marker is consumed before `parse_spine`, so no type ever contains it.

### Leaf 3 — `leaf-module-binding-diagnostic`

`Code::ModuleScopeBinding` (`RDX0106`), raised when a top-level name is followed by `=`,
naming doc 03 §13.4's correction. A UI case to reach it.

### Leaf 4 — `leaf-proof-and-gate`

`examples/constants.sdx`, the UI case, emit tests, and the gate.

---

## 5. Acceptance criteria

1. `const MAX :: usize = 1024` emits `const MAX: usize = 1024;`.
2. `static COUNT :: u32 = 0` emits `static COUNT: u32 = 0;`.
3. `export const LIMIT :: usize = 8` emits `pub const LIMIT: usize = 8;`, and the other
   three visibility scopes likewise.
4. `square :: const u32 -> u32` plus `square n = n * n` emits `const fn square(n: u32) ->
   u32`, and **no parameter type contains `const`**.
5. A `const` with no value is a parse error — unlike a class's associated constant, a
   module-scope one must be initialised, and rustc says so less clearly than Redox can.
6. `PI = 3.14159` at module scope raises `RDX0106` naming §13.4, not a `::` message.
7. `examples/constants.sdx` compiles with **zero** rustc output, runs, and matches its
   `.stdout`; a golden covers the text.
8. `code_coverage.rs` is green, which means the new code has a UI case.

---

## 6. Out of scope, and what it discloses

- **`static mut`** is unsafe in Rust and needs the escape hatch doc 03 §19 owns. Not here,
  and not reachable: a `mut` after `static` is a parse error until something wants it.
- **The uppercase-name lint.** §13.4 says the convention "is a lint, not semantics", and
  Redox has no lint pass — row 38's `RDX1001` is the first and is still outstanding. A
  lowercase `const` is legal here and rustc's own `non_upper_case_globals` warning is what
  the author will see, which is the division of labour AGENTS.md states.
- **`const` blocks and `const` generics** are Rust forms this plan does not reach. A `const`
  generic parameter would arrive through doc 18 §5.3's synthesised generic list, which has
  no spelling for it; worth knowing, not worth guessing at.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald's module-scope bindings are its own semantics, not a lowering onto
someone else's.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | One node for `const` and `static`, with a flag | Two items | They differ by one keyword; doc 03 §13.4 treats them as one construct with `const` as the default |
| 2 | `const fn` ships here | Row 39, with `async` | §13.4's `const` and §2.2's `const fn` are the same keyword in the same position, and leaving one out means reading it in one plan and acting on it in another |
| 3 | The marker is consumed before `parse_spine` | Let `capture_type` keep `const u32` and strip it later | A type is verbatim text (doc 03 §14.1); a type that has to be edited afterwards is not verbatim, and the edit would be a second place that knows what `const` means |
| 4 | A module-scope `const` must be initialised | Allow it, as a class's may be | A class's associated constant is *declared* there and supplied by an instance. A module-scope one has nowhere else to come from |
| 5 | §1.2 gets its own code | Reuse `ExpectedToken` | Doc 01 §4.3 was published. The author who writes `PI = 3.14159` followed the specification, and `expected \`::\`` does not tell them what replaced it |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
