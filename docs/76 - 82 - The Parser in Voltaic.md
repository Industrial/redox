# Plan 82 — The Parser in Voltaic

`voltaic-parser`'s 117 files, 8,293 lines. **The last crate**, and the one that
makes the parser that reads Voltaic a Voltaic program.

Every crate in the workspace is now written in Voltaic except the three that are
deliberately hand-written Rust. Doc 02 §4.8 asked whether the compiler's own
source could be expressed in Voltaic; from this row the question is settled by
rustc for twelve of thirteen crates, and `docs/CONFORMANCE.md`'s blocker count
falls from **3 to 2** because the one `as` cast it attributed to this crate is
gone — `u32::from c` is exact for a `char`, not a fallible substitute.

It also found two things worth more than the port: **`.kind (.peek p)` is a
method call, not a field access**, which 37 call sites got wrong in a way no
transpilation check could see, and a **generated file can exceed a cap the
hand-written one met**, because the emitter's own formatting is part of the
output being measured.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/voltaic-parser/sdx/**` — 120 files, 6,504 lines |
| Generated | `crates/voltaic-parser/src/**`, 8,714 lines, untracked from this commit |
| Tests | 18, unchanged — same names |
| List | `seed/PORTED` gains `voltaic-parser 82`, after `voltaic-ast` |
| Seed | refreshed, 926,449 → 928,389 bytes |
| Doc comments | 2,362 in, 2,396 out, **none lost** |

**Twelve crates are now Voltaic** — 526 files. The three that remain are
hand-written Rust by decision, not by backlog.

`seed/PORTED` puts `voltaic-parser` **third**, straight after `voltaic-ast`: the
list is in rebuild order, and `voltaic-fmt`, `voltaic-fuzz`, `voltaic-bench`,
`voltaic-mcp`, `voltaic-lsp`, `voltaic-driver` and `voltaic-cli` all sit below
something that names its types.

---

## 2 What it found

### 2.1 `.kind (.peek p)` is a method call, and nothing in the port could see it

The receiver-last spelling makes `.f x` the call `x.f()`. So
`&.kind (.peek p)` is `&p.peek().kind()` — a call to a method named `kind`,
where the Rust reads the **field** `p.peek().kind`. The field access is
`&(.peek p).kind`, postfix, exactly as Rust writes it.

Thirty-seven call sites across eleven of the twelve batches were written the
wrong way, and **every check before rustc passed**:

- `voltaic build` transpiled all 117 files with no diagnostic, because
  `.kind x` is a perfectly good application whose head is a method reference;
- `port.sh` reported `files with errors: 0` for the same reason;
- `voltaic fmt` round-tripped it unchanged, because it is canonical Voltaic.

What caught it was `port.rs`'s compile-and-stage step — 47 rustc errors, 37 of
them this — which is the step that exists because the earlier two cannot
answer this class of question. The same structure caught plan 79's
single-armed `main`: a `case` missing an arm is not a parse error either.

**The lesson is about the spelling, not the mistake.** `.name` is a *method*
in prefix position (plan 57) and a *field* in postfix position, and those are
the same three characters doing different jobs. `postfix.sdx`'s own comment in
this crate says so — *"`.` is field access and nothing else, which is `.`'s one
remaining job"* — and it was ported with the error in it. A diagnostic is not
possible in general (a type may genuinely have a method named `kind`), but the
asymmetry is worth a line in doc 03 §1.1 beside the method rule, and is owed.

### 2.2 A generated file can break a cap the hand-written one met

`line_cap.rs` caps a source file at 1,500 lines. `parse/parser.rs` was **1,482**
hand-written and **1,648** generated — over the cap on a port that changed no
code.

The difference is the emitter's, not the author's. The largest single item is
that **a blank line goes after every `use`**: this file has 42 imports, so 42
lines of the 166. The rest is per-item spacing and rustfmt's wrapping of the
signatures Voltaic writes on one line.

That is worth stating plainly because it is a property of *every* port and this
is the first file large enough to meet it: **the cap applies to the generated
Rust, and the generated Rust is 5% larger than the Rust it replaces.** A crate
whose largest file is within 5% of 1,500 lines will hit this.

**A second `impl` in a child module was tried first, and this repository
forbids it** — correctly. `one_construct_per_file` requires exactly one
top-level construct per file, and an impl-only file declares none; doc 06 §1's
*"`impl` blocks travel with their type"* is the rule, and the test is what
enforces it. An `instance (Parser 'a)` in `parse/parser/cursor.sdx` compiled,
passed clippy and passed all six port criteria before the layout test said no.

So the three largest private methods — `clause`, `where_locals` and `stmt`,
together 261 generated lines — take the shape the other 117 files in the crate
already have: a free function over `&mut Parser`, in a file named after it.
They are `export(super)`, which from a child module is exactly the scope a
private method of `parse::parser` had — visible there and in every descendant.
`parse/parser.rs` is now **1,319** lines.

### 2.3 `deriving` is a keyword, so the function is `r#deriving` — and the layout test could not read it

`deriving` is doc 03 §9.5's keyword, so `deriving : …` is `RDX0103` *"expected
a name to begin a declaration"*. `r#deriving` is the spelling, and it emits
`fn r#deriving`, which **is** the identifier `deriving` — a Rust reader of the
`.xrs` sees the same name, and `data_decl`'s call site reads the same.

`one_construct_per_file.rs` then reported the file as declaring **`r`**: its
name scan takes alphanumerics and `_` and stops at the `#`. The fix is three
characters of `strip_prefix("r#")` and a comment, and it is a tool gap rather
than a layout violation — a raw identifier is the identifier.

The translator has prefixed `r#` on module names, struct fields and
record-variant fields since plan 80. This is the first **function** name that
needed it, which is why that row's work did not cover it.

### 2.4 `Parser<'a>` needs no parameter list, and `instance (Parser 'a)` is the impl head

Plan 74 recorded `data` declarations not taking parameters as a blocker for
`cursor.rs`, `lexer.rs`, `Sink<'a>` and `Parser<'a>` — four declarations in the
workspace. Three were resolved by plan 81's `Sink`, and this is the fourth:
**`generalise_data` reads the lifetime out of the field types.**

```
export(crate) data Parser = Parser {
  toks : &'a [Token],
  …
}
```

emits `pub(crate) struct Parser<'a>`, because doc 18 §5.3's rule — a lowercase
name in a type is a variable of that declaration — applies to `'a` as well, and
`rust_generic` passes a lifetime through unchanged. The impl head is
`instance (Parser 'a)`, and `MethodContext::for_instance` puts `'a` in
`reserved`, which is what keeps every method from getting a `<'a>` of its own.
`peek : &self -> &'a Token` is therefore `fn peek(&self) -> &'a Token`, with no
generic list.

So the blocker plan 74 recorded is closed in full, and nothing was added to the
surface to close it: the rule that synthesises a generic list already covered
lifetimes.

### 2.5 Three constructs with no Voltaic spelling, each with a behaviour-identical rewrite

None of these is a new gap; each is a known one met in a new place, and the
rewrite is recorded here so a reader of the `.sdx` does not have to
reconstruct it.

**`..decl`, Rust's functional record update.** `item`'s tail builds an `FnDecl`
from the one `function` returned, overriding three fields. Three assignments to
a `mut` binding are the same thing written out — including the `span` override,
which reads the old `span` before replacing it, exactly as the update syntax
does.

**A diverging `loop` as a `()` body's tail.** `recover_to_item` and
`recover_to_stmt` are `loop { … return; … }`, and plan 55 §1.6 requires a
`()`-returning body to end with `()`, `return`, `break` or `continue`. A `loop`
is none of the four, and a trailing `()` after one is `unreachable_code`, which
`-D warnings` makes an error. Both become `while running`, which ends with `()`
and needs nothing after it.

**`|=`.** `compound_assign` reads the five arithmetic forms and no bitwise one,
so `ungrouped`'s `top_level_arrow |= depth == 0` is written
`top_level_arrow := top_level_arrow || depth == 0`.

### 2.6 `@` over an alternation, three times, and the row stays owed

Plan 81 §2.5 recorded that `Pattern::rust_text` emits an `@` over an
alternation without the parentheses rustc wants — `E0408`. This crate has three
of them: `pattern`'s literal arm (`kind @ (Int | Float | Char | Byte | True |
False)`) and `prefix`'s two keyword arms.

All three are **stepped around rather than into**, and the two routes are
different:

- `pattern` re-reads the token with `token_text` inside the arm instead of
  binding it, which is the same value by a different path;
- `prefix` splits into one arm per keyword, because each arm already knows
  which keyword it matched and needs no binding at all.

The defect is unchanged and still owed. What this row adds is that the
workaround is cheap in both shapes, so closing it is a correctness fix rather
than an unblocking one.

### 2.7 Four smaller things, each a first

**A continuation is a line *ending* in an operator.** `type_vars`'s five-line
`||` chain and `capture_type`'s five-line `&&` chain both open their
continuation lines with the operator, which is `RDX0102`. Both become one line,
which is long and correct. Two more conditions in `item` and `signature_head`
are the same shape.

**An attribute is not a construct.** `write.py` put a blank line between the
translated head and the body when the head did not end in a comment — and
`postfix.sdx`'s head ends in `#[allow(clippy::result_large_err)]`, which made
`empty_line_after_outer_attribute` an error. It is the first ported file whose
head ends in an attribute rather than a doc comment, and the fix is in the tool.

**Rust's shadowing is sometimes load-bearing, and `RDX1003` forbids it.**
`rust_hatch` writes `let raw = raw.clone()`, where the shadow is what ends the
borrow of `p` before `.advance`. Voltaic ends it by **last use** instead: the
clone is the binding's final mention, so a different local name is legal for the
same reason. Eleven more shadows — `condition`, `class`, `constraints`,
`visibility`, `patterns`, `leading`, `name`, `head`, `sig`, `guard`,
`otherwise` — are renamed with the field keeping its own name.

**A `case` cannot be a one-line lambda body**, because it needs a line of its
own. Four `matches!`-inside-a-closure calls — `section`, `looks_like_a_binding`,
`prefix`'s `any`, `eof_trivia`'s `filter` — become an `if` over a `case`, or a
`guard` and a binding.

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 117 files are Voltaic | yes — in 120 `.sdx`, three of them the split of §2.2 |
| 2 | `voltaic-parser` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/voltaic-parser/src/` is gitignored and out of the index | yes |
| 4 | `crates/voltaic-parser/sdx/` **is** in the index | yes |
| 5 | Every test survives | yes — 18 before, 18 after, same names |
| 6 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 7 | No escape hatch | yes — none, and `HATCH_ALLOWED` still lists only plan 81's three |
| 8 | A clean clone bootstraps it | yes |
| 9 | `stage1 == stage2` still holds | yes |
| 10 | No doc comment lost | yes — 2,362 → 2,396 |
| 11 | No file over the line cap | yes — 1,319 and three files of 95, 104 and 132 |
| 12 | `voltaic fmt` round-trips every ported `.sdx` | yes — no formatter defect in this row, the first port since 78 with none |
| 13 | `docs/CONFORMANCE.md`'s count does not rise | yes — it **falls**, 3 → 2 |

---

## 4 Next

**No crate remains.** `voltaic-ast`, `voltaic-diag`, `voltaic-lexer`, `voltaic-parser`,
`voltaic-emit`, `voltaic-fmt`, `voltaic-driver`, `voltaic-cli`, `voltaic-lsp`,
`voltaic-mcp`, `voltaic-fuzz` and `voltaic-bench` are all Voltaic. The remaining Rust
is the integration-test harness (doc 06 §7), `voltaic-seed`, and the three crates
doc 02 §4.8 says to keep hand-written *permanently rather than as a
"fallback"*.

Owed to a successor, from this row:

- **Doc 03 §1.1 should say that `.name` is a method in prefix position and a
  field in postfix position** (§2.1), beside the method rule it already states.
  Thirty-seven call sites is the price of that sentence being absent.
- **`line_cap.rs`'s cap meets generated Rust, which is ~5% larger than the
  Rust it replaces** (§2.2). Either the cap accounts for that or the emitter
  stops putting a blank line after every `use`; the second is probably right,
  since no hand-written file in the workspace is spaced that way.
- **`Pattern::rust_text` should parenthesise an `@` over an alternation**
  (§2.6), which is `E0408` today. Owed since row 81; three more sightings here
  and a cheap workaround in both shapes.
- **The doc-comment fidelity check should be `port.rs`'s seventh assertion.**
  Rows 79, 80 and 81 each asked; this row used it once more, and it is the only
  check that compares against a source *outside* the thing it checks.
- **A macro's format-template argument should be known by name** (row 80 §2.4,
  row 81 §2.4), or a plain `"…"` in a value position should be diagnosed.
- **A keyword collision needs its own diagnostic**, and the translator cannot
  reach a *use* site. Owed since row 75; `deriving`, `class`, `with`,
  `otherwise`, `guard` and `test` all collided here, and `deriving` is the
  first **function** name among them (§2.3).
