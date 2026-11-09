# Plan 43 — Test Suite Expansion

Plan ID: `test-suite-expansion`
Depends on: 34 (`formatter`, partial), 42 (`fuzzing`) — see §1.2
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

Doc 04's row 43 names four property tests:

| Property | Status |
|---|---|
| Formatter idempotence | **already shipped** — plan 34 leaf 5 |
| `parse(fmt(x)) ≡ parse(x)` modulo trivia | **already shipped** — plan 34 leaf 5 |
| Transpile stability | **already shipped** — plan 04, as `emission_is_deterministic` and `every_example_matches_its_golden_and_is_deterministic` |
| Lexer indentation round-trip | outstanding — this plan |

So the row is one property, not four. §1.1 is why that happened and why it is worth saying
out loud rather than quietly implementing the one.

### 1.1 Three of four arrived from the rows that needed them

Each of the three was built by the plan that could not be proved without it:

- Plan 04's whole subject was emitter determinism, so it asserted determinism.
- Plan 34 needed idempotence and meaning-preservation to make a span-slice formatter safe to
  ship incrementally (its §1.4), so it built both — and both immediately caught bugs (its
  §10.2).

That is the healthy direction. A property belongs to the plan whose claim depends on it,
because that plan has a reason to get the normaliser right — plan 34 §10.5 records its
meaning-preservation test failing on its own normaliser rather than on the subject, which is
exactly the mistake a row that owned properties *in general* would be most likely to make and
least likely to notice.

**So this row is mostly a register correction**, and doc 04's cell should say so. A row that
collects properties after the fact is a row that will write them without a stake in them.

### 1.2 The dependency on row 42 is not real

Row 43 depends on `42 (fuzzing)`, and none of its four properties needs a fuzzer: all four
run over the existing corpora. The dependency reads as though the properties were imagined as
fuzz oracles.

They are better as corpus properties — a fuzzer finds inputs, and these four need no new
inputs to be useful, as plan 34 §10.2's three bugs demonstrate. Row 42 can later point its
generated programs at the same assertions, which is a reason for 42 to depend on 43 and not
the reverse.

Recorded rather than silently ignored, because the register is supposed to be read.

### 1.3 The one real property: indentation round-trip

Redox is off-side-rule: a block is opened by indentation and the lexer turns that into
`Indent` and `Dedent` tokens. The property:

> Re-deriving the indentation from the token stream reproduces the source's block structure.

Concretely — for every `.sdx` in the repository, walking the tokens and tracking
`Indent`/`Dedent` gives, for each line that begins a statement, the same depth the source's
leading whitespace implies.

**Why this is worth a test rather than being obviously true:** the lexer decides depth from a
stack of widths, and doc 03 §3.1's `RDX0002` exists because an indentation width matching no
enclosing level is an error. The round-trip is the statement that the stack and the text agree
— and it is the single assumption every later stage makes without checking. Plan 34's leaf 4
will re-indent from exactly these tokens, so if they disagree with the text, the formatter
will silently change block structure.

That makes this property plan 34's safety net as much as this row's, which is the argument
for building it now rather than after.

---

## 2. Admission tests (doc 03 §2)

Not applicable: no construct, no lowering. This row adds assertions.

---

## 3. Concrete proof this plan targets

The property itself, over every `.sdx` in the repository — the same corpus-wide sweep plan 34
§1.4 established, for the same reason: thirty-odd files already cover every construct that
exists, so a property over them is the cheapest large test available.

Plus a **negative** case, because a property test that cannot fail is decoration: a
hand-written fixture whose indentation is deliberately inconsistent, asserted to be rejected
by the lexer with `RDX0002` rather than silently given a depth.

---

## 4. Leaves

### Leaf 1 — `leaf-indentation-round-trip`

The property, in `redox-lexer/tests/`, plus the negative case.

### Leaf 2 — `leaf-register-correction`

Doc 04's row 43 says what it actually owes, and row 42's dependency direction is corrected.

---

## 5. Acceptance criteria

1. For every `.sdx` in the repository, the depth implied by `Indent`/`Dedent` equals the
   depth implied by leading whitespace, on every line that begins a statement.
2. A file with inconsistent indentation is `RDX0002` and not a depth — the property's
   negative case.
3. `lexical.sdx` is included rather than exempt. It does not *parse*, but it must still
   **lex**, and this property is about lexing — so the one file the other corpora exempt is
   the one this row most wants.
4. Doc 04's row 43 records that three of its four properties were shipped by the rows that
   needed them, and row 42's dependency direction is corrected.

---

## 6. Out of scope, and what it discloses

- **The three shipped properties** are not re-implemented. They live with the plans that
  needed them, per §1.1.
- **A fuzzer.** Row 42's, and §1.2 argues the dependency runs the other way.
- **Generated-input property testing** — `proptest` or `quickcheck`. The corpora are richer
  than anything a generator would produce for a language this shaped, and a generator that
  produced *valid* Redox would be most of row 42's work. If row 42 lands, it points at these
  assertions.
- **A coverage threshold.** `:coverage` exists as a task; a number in CI is a different
  argument and not this row's.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald had no off-side rule, so it had no indentation to round-trip.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Say the row is one property, not four | Implement all four here, duplicating three | They already exist, built by the rows whose claims needed them — and a property owned by the plan that depends on it gets its normaliser right, per §1.1 |
| 2 | Correct row 42's dependency direction | Leave it | None of the four properties needs a fuzzer, and a register that reads wrongly is worse than one that reads short |
| 3 | Include `lexical.sdx` | Exempt it, as the other corpora do | It is exempt elsewhere because it does not parse. This property is about *lexing*, which it must do — so it is the most valuable input, not the one to skip |
| 4 | A negative case | The property alone | A property test over a corpus that all passes cannot distinguish "the property holds" from "the property is vacuous" |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
