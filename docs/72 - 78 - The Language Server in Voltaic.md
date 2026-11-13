# Plan 78 — The Language Server in Voltaic

`voltaic-lsp`'s 22 files, 934 lines. Row 72's sixth successor, and the one that
found **three silent defects** — two in the compiler and one in the translator
— each of which produced correct-looking output and a failure somewhere else.

Eight of twelve crates are now Voltaic.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/voltaic-lsp/sdx/**` — 22 files, 547 lines of Rust expressed |
| Generated | `crates/voltaic-lsp/src/**`, untracked from this commit |
| Tests | 9, unchanged |
| Seed | refreshed, 925,628 bytes |

---

## 2 The three silent defects

Each of these produced a `.sdx` that parses, a port that reports nothing, and a
failure attributed to a file whose author never touched the thing at fault.
That is the shape worth naming: **a port's own check is a parse, so anything
that survives parsing arrives late and wearing someone else's name.**

### 2.1 The translator could not read a wrapped `use`

`capabilities.rs` opens

```rust
use lsp_types::{
  OneOf, SemanticTokensFullOptions, …
};
```

and `capabilities.sdx` came out with **one** import where the Rust has two. The
import branch matches a single line, so a wrapped `use` fell through to the
generic token path — which dropped the statement, or joined its tokens without
the spaces `as` needs.

rustc then said *"cannot find type `ServerCapabilities`"* about a file whose
author never wrote an import.

**Four ported crates went by without this**, and that says what their imports
looked like rather than that the translator was right: `rustfmt` wraps a `use`
only when it exceeds the width, and `voltaic-lsp` is the first crate whose
dependency has long enough type names.

### 2.2 `as` inside an import tree lost its spaces

`import_path` captures the path as verbatim text, and its comment says *"No
spaces are inserted. A path has none at depth zero, and inside a tree Rust's
own style is `{self, Read, Write}`"*. True of doc 03 §13.1's five forms — and
not of a **renamed item inside a tree**. The scan breaks on `as` at depth
*zero*, so the only `as` that reaches the push is inside a brace group, and it
arrived with no space on either side.

`{Notification as NotificationTrait, …}` became
`{NotificationasNotificationTrait, …}`. The path is **one string** in the AST,
so that came out of the emitter *and* the formatter: Rust that does not
compile, and a `voltaic fmt` that rewrites the source to match it.

A top-level `as` survived all along — `import std::fmt::Write as _` goes
through `import_decl`'s own alias handling — which is why four crates used that
form and none caught this.

**This is the third formatter-breaks-its-own-parser defect in three rows**,
after plan 74's range and plan 77's `move`. All three were found by porting and
none by the corpus.

### 2.3 An attribute on a record-field initialiser is dropped

The Rust writes

```rust
#[allow(deprecated, reason = "the field is required by the struct literal")]
deprecated: None,
```

inside a `DocumentSymbol` literal. `FieldInit` has **no `leading` field**, so
the parser discards the attribute before the AST exists: the emitter never sees
it and `voltaic fmt` cannot print it.

That contradicts doc 02 §10.1's *"comments are trivia, never discarded"*, which
attributes joined at plan 18 §1.3. And it is quiet in the worst way — the
`.sdx` parses, and clippy then warns about a deprecated field the author had
explicitly allowed.

Carrying it needs a `leading` on `FieldInit` plus the parser, the emitter and
the formatter: a feature, not a fix, and **owed to a successor row**. The allow
is on the function meanwhile, which is a wider scope and the same effect.

---

## 3 `cast` had to change shape

Its Rust is generic over `R: RequestTrait` and mentions `R` **only** as
`R::Params` and `R::METHOD`, so callers pick it with a turbofish —
`cast::<Formatting>(request)`.

Voltaic has no turbofish, and ascription cannot stand in: `R` is not inferable
from `R::Params`, which is not injective. `type_vars` would not even register
`r` as a variable, because it skips an identifier adjacent to `::` — the rule
that keeps `fmt` in `fmt::Result` from becoming a type parameter.

So the discriminator becomes a **value**: `cast : DeserializeOwned p => &Request
-> &str -> Result<p, …>`, and each caller ascribes the params type it wants.
More explicit than the turbofish, no longer than it, and the general answer —
**a function generic only through an associated type must take its
discriminator as an argument.**

---

## 4 Two keyword collisions, and one running the other way

**`class` is a keyword**, so `decl.class` cannot be read as a field *or* bound
by a pattern: it is `decl.r#class` and `Some implemented`. Plan 68 already
spelled the AST field `export r#class`, so the convention is settled and only
undocumented. `data` is a keyword too, and `lsp_types` named a field `data`, so
`r#data` three times. That is five keyword collisions in five crates.

**And `final` is reserved in *Rust*, not in Voltaic.** A binding called `final`
parses, emits, and then fails in rustc — *"expected identifier, found reserved
keyword"* — against the generated file. What a name has to avoid is the
**union** of both keyword sets, and Rust reserves several words it does not
use: `final`, `become`, `priv`, `typeof`, `unsized`, `virtual`, `abstract`,
`override`, `box`.

Six collisions, five crates, and the diagnostics are no help: `rust = …` still
reports *"expected a `"""` block of Rust after `rust`"*. **A message naming the
collision is owed**, and the translator prefixing `r#` would remove most of the
need for one.

---

## 5 The record-update form, used for the first time

`capabilities` ends `..ServerCapabilities::default()` in Rust. Doc 18 §6 lowers
a **lowercase** head to a mut-bind plus assignments — because Rust's own
`X { f: v, ..base }` needs the type's *path* and the source gives only the
value's name — so the default is bound and the fields are set on it. The
emitted Rust contains no `..` at all.

---

## 6 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 22 files are Voltaic | yes |
| 2 | `voltaic-lsp` is in `seed/PORTED`, and `port.rs`'s five assertions cover it | yes |
| 3 | `crates/voltaic-lsp/src/` is gitignored and out of the index | yes |
| 4 | `crates/voltaic-lsp/sdx/` is in the index | yes |
| 5 | Every test survives | yes — 9 before, 9 after |
| 6 | A wrapped `use` translates, with the import intact | yes |
| 7 | `as` inside an import tree keeps its spaces, with a test | yes |
| 8 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 9 | A clean clone bootstraps it | yes — seed refreshed |
| 10 | `stage1 == stage2` still holds | yes |
| 11 | An attribute on a record field survives | **no — §2.3, deferred in writing** |

---

## 7 Owed to successor rows

- **`FieldInit` needs a `leading`** (§2.3), so an attribute on a record field
  is carried rather than dropped.
- **A keyword collision needs its own diagnostic** (§4), and the translator
  should prefix `r#` — carried from plans 76 and 77 and now overdue.
- **Seven crates carry the `no_effect` allow.** Plan 75 §2.4 called repeating
  the paragraph the wrong answer, plan 76 §6 repeated it, and so does this; the
  decision is owed either a workspace-level allow or a change to plan 55 §1.6.
- Everything plan 74 §9 lists, less the `move`-lambda entry plan 77 closed.

---

## 8 Next

Four crates, roughly 18,800 lines: `voltaic-cli` (1,376), `voltaic-driver`
(2,981), `voltaic-emit` (6,203), `voltaic-parser` (8,241).

The two largest are the compiler's own middle, and `Sink<'a>` and `Parser<'a>`
are the only generic declarations left in the workspace — both unblocked by
plan 74 §2.2.
