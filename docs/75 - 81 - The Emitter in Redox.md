# Plan 81 — The Emitter in Redox

`redox-emit`'s 109 files, 6,203 lines. The largest crate ported, and **the one
that answers doc 02 §4.8's question with a "no"**.

Every row since 73 has been able to say the compiler's own source is
expressible in Redox without the inline-Rust hatch. This one cannot: three
files need `thread_local!`, whose body is a `static` declaration, and a Redox
macro argument is an expression. That is not a defect — it is the measurement
the question was asked to get, and it is now a graded `UNREACHABLE` row with a
fixture rather than a sentence in a plan.

It also found the **fifth** formatter round-trip defect and the **third**
sighting of plan 80 §2.4's brace-doubling, which together say something the
earlier rows only hinted at: the two halves of a round-trip are never checked
against each other except by a port.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/redox-emit/sdx/**` — 109 files, 5,513 lines |
| Generated | `crates/redox-emit/src/**`, 6,585 lines, untracked from this commit |
| Tests | 18, unchanged — same names |
| List | `seed/PORTED` gains `redox-emit 81` |
| Seed | refreshed, 925,681 → 926,449 bytes |
| Doc comments | 2,324 in, 2,333 out, **none lost** |

Eleven crates are now Redox — 406 files. One remains: `redox-parser` (8,257).

---

## 2 What it found

### 2.1 `thread_local!` has no Redox spelling, and that is doc 02 §4.8's answer

Three files — `arity_scope`, `binding_scope`, `bind_scope` — hold a guard over
a thread-local table. The declaration is

```rust
thread_local! {
  static IN_SCOPE: RefCell<ArityTable> = RefCell::new(ArityTable::default());
}
```

Plan 64 made macros callable as `name! \-> e`, opening each argument with a
nullary lambda, so **an argument is an expression**. §6 of that plan disclosed
the *pattern* half of the consequence — `matches!`'s second argument is out of
reach — and this is the same gap one step further out: the argument here is an
*item*.

**Nothing else expresses it.** A module-scope `static` is available (doc 03
§13.4) and will not do: a `Cell` and a `RefCell` are not `Sync`, and the
`thread_local!` is there precisely so the suite's parallel tests cannot see
each other's files. Threading the tables through `emit_expr` instead is what
`arity_scope`'s own doc comment forbids at length — §11.3 priced it at
twenty-four importers and thirty-six call sites.

So the three use doc 03 §23's hatch, and three things follow.

**`port.rs`'s criterion 7 changed from "none" to "exactly these".** It asserted
zero `rust` blocks anywhere in the port, with the reason that *"one `rust`
block would make the whole claim vacuous"*. That reason is sound and the
assertion was the right one to write; what it could not anticipate is a
construct with no spelling. So it now carries `HATCH_ALLOWED` — three paths,
each with its reason — and `every_hatch_allowance_is_used` fails if an entry
goes stale. An undeclared hatch still fails, which is the whole of what the
check was for.

**`docs/GRAMMAR.md` gains the row**, so the gap is graded rather than
mentioned: *macro invocation with a non-expression argument*, `UNREACHABLE`,
with `tests/unreachable/macro_item_argument.sdx` asserting it still fails. That
is what makes a future row that closes it break the build.

**The check that forbids the hatch also had a false positive.** It tested for
the bare word `rust` with `--` comments stripped and nothing else, so
`test "translates the one escape rust lacks"` — a test's own description, in a
file that uses no hatch — was reported as an offender. The hatch is the keyword
*and* a verbatim block in both positions, so both halves are now required.

### 2.2 `redox fmt` stripped parentheses the parser needs, for the fifth time

`Some (BindingKind::Called)` as a `case` pattern came back from the formatter
as `Some BindingKind::Called`, which does not parse.

`pattern_argument`'s `stands_alone` is the mirror of what the parser accepts
beside another pattern, and its reasoning was right about the shape it names:
*"an `Apply` with **no** arguments — `None`, `Close` — is a bare name and needs
nothing"*. What it missed is that a bare name and a *path* are different
things: `Kind::Called` is three tokens, and the parser's pattern-argument
reader takes one.

**The fix took two attempts, and the first one is the instructive half.**
Adding `&& !path.contains("::")` to the `Apply` arm changed nothing, because
`pattern_app` keeps a bare constructor as a `Pattern::Binding` carrying the
whole path — its own comment says so, *"a bare constructor stays a `Binding`,
as it was before this plan"* — so a zero-argument `Apply` is a shape the parser
never builds. The guard belongs on the `Binding` arm.

That is the fifth formatter round-trip defect, after plan 69's three and plan
79's one, and the second where the formatter was the half at fault. Plan 79's
lesson stands and sharpens: *a round-trip to unparseable output has found a
disagreement between two halves that nothing else compares, and reading only
one half is how the first fix misses.*

**Fixing it needs a cycle broken by hand**, which is worth recording for the
next row that touches the formatter: `port.sh` runs `redox fmt`, whose
behaviour comes from `redox-fmt/src`, which is generated from `redox-fmt/sdx`
by a bless that first requires every ported `.sdx` to parse — including the one
the old formatter just broke. So the order is: fix the `.sdx`, hand-repair the
file the old formatter mangled, bless, *then* re-run the port.

### 2.3 Brace-doubling, third sighting — and plan 80 §2.4 was incomplete

`macro_argument` brace-doubles every string literal reaching a macro. Plan 80
§2.4 recorded the defect and said position alone cannot settle it, which is
right. What it got wrong is the remedy: it reached for a `char` comparison and
called the spelling absent.

It is not absent. **Doc 03 §11.2's raw literal is the answer** — `r"…"` is
`Expr::RawLit`, which §11.2 calls *"pure passthrough"* and which
`macro_argument` hands straight to `emit_expr` with no doubling at all. Every
brace-bearing assertion in `double_braces`'s own tests is written that way, and
`rust_escapes`'s backslash fixtures need raw literals regardless.

Where a real newline is also needed, `r"…"` cannot help, and the third spelling
is a binding: `String::from "fn main() {\n…"` is an ordinary call, so no
doubling applies, and the assertion compares the binding. `Sink`'s map test
needs that one.

The row is still owed — a plain `"…"` in a value position is silently
rewritten and nothing says so — but it is owed as *a diagnostic or a
macro-name table*, not as a missing spelling.

### 2.4 The doc-comment check learned to read a verbatim block

A `///` inside a `rust """ … """` block has no `-- |` in the `.sdx` to come
from, so `every_doc_comment_reaches_the_generated_rust` reported a comment
appearing out of nowhere. The alternative was to downgrade three real doc
comments to `//` to satisfy the arithmetic, which is the test changing the code
rather than reading it — so the counter now reads a verbatim block's own doc
comments on both sides.

A doc comment *above* the hatch item is not an option: `unused_doc_comments` on
a macro invocation, which `-D warnings` makes an error.

### 2.5 Surface limits met for the first time, each with its spelling

| | |
|---|---|
| **`vec![false; n]`** | The repeat form takes a length after a `;`, and a macro argument is an expression — `[x; n]` is `Expr::ArrayRepeat`, an *array*. `Vec::new` plus `.resize` is the same thing. |
| **A tuple pattern as a closure parameter** | `.filter(\|(_, e)\| …)`. Doc 69 §10.4's rule, met eight times here; the pair is bound and projected. |
| **An annotation that is load-bearing** | `trimmed.starts_with(\|c: char\| …)` — `str::starts_with` is generic over `Pattern`, so a bare `\c -> …` has nothing to infer `char` from. Asking the first character directly needs no closure. |
| **A `let` inside a match guard** | `Expr::Call { … } if let Expr::MethodRef { … } = unparen(callee)`. A guard asks and does not bind, so the two `Call` arms fold into one with an inner `case`. |
| **An irrefutable `let` in a chain** | `&& let rewritten = assert_call(…) &&`. `is` asks a *refutable* question, so an always-succeeding binding cannot be a link. |
| **A record field matched against a path** | `op: BinaryOp::Pipeline` — a field pattern takes a **single-segment** path, so five arms compare in a guard instead. |
| **`@` over an alternation** | `inner @ (A \| B)` emits **without parentheses** and rustc answers `E0408: variable inner is not bound in all patterns`. A guard comparison instead; owed to a successor as an emitter fix. |
| **An `instance` head takes a name** | `instance std::fmt::Write (Sink '_)` does not parse; the trait is imported. And the lifetime is juxtaposed — `Sink '_`, as `Cursor 'a` is — not `Sink<'_>`. |
| **`return` as an inline arm body** | Not accepted, the same limit `break` has; it goes on its own line. |
| **An array literal argument** | `.args ["--edition", …]` is an *index*. `[` is not on `starts_argument`'s list, which `stands_alone`'s doc comment explains, so the argument keeps its parentheses. |
| **An `instance` of a class from another file** | Must restate each signature — `RDX0101` asks for it. An instance whose class is declared directly above needs none. |
| **A second `write()` over one file** | Must carry the declaration: the first consumed the `-- TODO` marker, so the cut then falls at the type. Plan 80 §2.5 recorded it; it bit `sink.sdx` here. |
| **A test-only type needs qualifying** | `redox_ast::Ident` is used only by a `#[cfg(test)]` helper, and an unqualified import is `unused_imports` without the tests. Same shape as plan 80's `Origin`. |
| **An assignment is not a lambda body** | `\slot -> *slot := restored` — a lambda body is one *expression*. `std::mem::replace` says the same thing and is the sibling function's own idiom. |

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 109 files are Redox | yes |
| 2 | `redox-emit` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/redox-emit/src/` is gitignored and out of the index | yes |
| 4 | `crates/redox-emit/sdx/` **is** in the index | yes |
| 5 | Every test survives | yes — 18 before, 18 after, same names |
| 6 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 7 | No *undeclared* escape hatch | yes — three declared, each with its reason, and the allowance is itself checked |
| 8 | A clean clone bootstraps it | yes |
| 9 | `stage1 == stage2` still holds | yes |
| 10 | No doc comment lost | yes — 2,324 → 2,333 |
| 11 | The hatch gap is graded, not mentioned | yes — `GRAMMAR.md` row and `tests/unreachable/macro_item_argument.sdx` |
| 12 | `redox fmt` round-trips every ported `.sdx` | yes — `stands_alone` fixed on the `Binding` arm |

---

## 4 Next

One crate, 8,257 lines: `redox-parser`. It is the last, and it does not inherit
the workspace allows yet, so its first `clippy` run will say which of the five
it needs.

Owed to a successor, from this row:

- **A macro's format-template argument should be known by name** (§2.3), or a
  plain `"…"` in a value position should be diagnosed. Owed since row 80; the
  spellings that avoid it are now written down.
- **`Pattern::rust_text` should parenthesise an `@` over an alternation**
  (§2.5), which is `E0408` today.
- **The doc-comment fidelity check should be a test** — rows 79 and 80 asked,
  and this row used it four times: once for each of the three hatch files and
  once for `ArityScope`'s whole type-level comment, which the translator leaves
  below the marker when the first item is a macro.
- **A keyword collision needs its own diagnostic**, and the translator cannot
  reach a use site. Owed since row 75; `of`, `errors`, `guard` and `otherwise`
  all collided here.
