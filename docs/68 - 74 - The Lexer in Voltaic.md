# Plan 74 — The Lexer in Voltaic

`voltaic-lexer`'s 33 files, written in Voltaic, with the generated `src/`
untracked. Row 72's second successor, and the first one to need the surface to
grow.

**94 tests before, 94 after** — the same count the hand-written crate had, and
the number that matters most in this document, because for a while it was 87.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/voltaic-lexer/sdx/**` — 33 files, 2,057 lines of Rust expressed |
| Generated | `crates/voltaic-lexer/src/**`, untracked from this commit |
| Tests | 94, all passing — 43 inline and 51 integration, unchanged |
| `as` casts | **5 → 0**, removed rather than worked around |
| List | `seed/PORTED` gains `voltaic-lexer 74`, before `voltaic-ast` which imports its types |
| Seed | refreshed, 924,277 bytes |

Shape, against its three predecessors: `voltaic-ast` was data with no logic,
`voltaic-fmt` one function per file, `voltaic-diag` both at once. This one is
**state** — a cursor and an indent stack threaded through twenty methods — and
it is the first to borrow its input, which is what made it need a feature
rather than a workaround.

---

## 2 What the surface had to grow

Two things, and **neither is new syntax**. Both are an existing rule reaching a
place it should always have reached.

### 2.1 `mut self` — Rust's fourth receiver form

Plan 67 marked the receiver with `self` in type position and `receiver_text`
listed three spellings: `&self`, `&mut self`, `self`. Rust has a fourth, a
by-value receiver the method may mutate, and `LexError::offset_by` and
`Lexer::run` are both written with it.

`receiver_text` strips `&mut ` and `&` and then requires the rest to be
`self`, so `mut self` matched nothing, stayed an ordinary parameter, and
emitted

```rust
fn offset_by(base: usize, self: mut self) -> Self
```

which is not Rust. **A spelling the surface accepts and the emitter turns into
invalid Rust is worse than one the parser rejects**, because the failure
arrives from rustc against a line the author did write and cannot see the
problem in. One `else if` closed it; `every_receiver_form_comes_from_the_type`
now covers all four.

Found by probing rather than by reading, which is worth saying: `receiver_text`'s
doc comment lists exactly the three forms it handles and is accurate about them.
The gap is only visible against Rust's grammar, not against the code.

### 2.2 A declaration synthesises its generic list

`Cursor<'a>` and `Lexer<'a>` borrow their source. `data Cursor<'a>` does not
parse — a declaration has no parameter list — and neither does Haskell-style
`data Pair a`. For an hour that read as a blocker needing its own surface row.

It is not. Doc 18 §5.3 already says **"type variables need no declaration … the
generic list is the order of their first appearance, lifetimes included"**, and
that rule was applied in two of the three places it belongs: `generalise` fills
a *signature*'s list, and `TypeApp::vars` fills an *instance target*'s. A
`data` declaration was simply left out, so

```
data Holder = Holder { s : &'a str }     struct Holder { s: &'a str }
```

emitted an undeclared lifetime. Nothing is written in the source that was not
written before; `generalise_data` reads the constructors' field types through
the same `type_vars`, and `'a` appears in the output because it appears in a
field.

Three smaller things fell out of making it work:

- **`TypeApp::is_var` did not count a lifetime.** It asked whether the first
  character is lowercase, and `'` is not — so `vars()` returned nothing while
  `rust_text()` printed `Cursor<'a>`: the target printed the argument and the
  list that must declare it did not. `'static` and `'_` stay excluded for the
  reasons `type_vars` already records, the second because `fn f<'_>(…)` is
  `E0637`.
- **`rust_name` had to stop uppercasing it**, since `is_var` now says yes and
  `'a` would come out `'A`.
- **`type_app` had to accept it**, because `'a` lexes as a `Label` and not an
  `Ident`, so `instance (Cursor 'a)` was *"expected `)` to close the type"*.

**Four generic declarations exist in the whole workspace** — `Cursor<'a>`,
`Lexer<'a>`, `Sink<'a>` in `voltaic-emit` and `Parser<'a>` in `voltaic-parser` —
every one a single lifetime on a borrowing struct. So the gap was narrow, and
the alternative, making those four own their source text instead of borrowing
it, would have made the code worse to avoid a notation.

**An ordering trap came with it.** `voltaic-ast` is written in Voltaic, so adding
a field to `DataDecl` means editing `data_decl.sdx` — and the generator that
would bless it is built from the thing being generated. The field has to reach
the generated `src/` *before* any code can use it. Seeded by hand for one
build, then blessed from the `.sdx` and the hand edit overwritten; worth
knowing because every future AST change has the same shape.

---

## 3 The finding that would have shipped

**A parenthesised half-open range is an operator section, not a range.**

```
.get (..i) body        body.get(|__rdx_1| __rdx_1 .. i)
```

Doc 18 §4 writes a section as a parenthesised operator with one side supplied,
`..` is an operator, and a parenthesis is how a section is written — so `(..i)`
and `(i..)` are *closures* by construction. The two readings collide and the
section wins.

**Eight files of ranges went through the port's own check green**, because that
check is a parse. They would have met rustc as *expected usize, found closure*
much later, in a crate where the generated Rust is what compiles.

What caught it was `voltaic fmt`. The formatter rewrote `((i + 1)..)` to
`(i + 1 ..)`, which does not parse — an idempotence failure, cheap and loud,
sitting directly on top of a type error two layers beneath it. The gate running
`:sdx-format-check` is what made the cheap failure arrive first.

The spelling that works is a **closed** range: `0..i`, `(i + 1)..len`. Both are
exactly equivalent on a slice, and neither needs a surface change.

---

## 4 The finding the harness could not make

**The port dropped seven tests, and the gate was green.**

`scan_number.rs` has seven inline tests. I read its body, stopped above its
`mod tests`, and wrote the file without them. Every gate passed: the crate
compiled, every test that existed ran, clippy was clean.

`port.rs` compares the `.sdx`'s doc-comment count against the generated Rust's,
which is how plan 73's doubling was caught. **It has no equivalent for tests,
and structurally cannot have one**: once a port lands, the pre-port Rust is
gone, so there is nothing to count against.

What found it was counting the crate's tests before and after — 94 against 87 —
against the pinned commit the build already reads from. That check now belongs
in the port procedure, and it is in `AGENTS.md`.

The general shape is worth more than the instance: **a check that compares
generated output to its source cannot see something absent from both.** Doc
comments survive because they are in the `.sdx`; a test module nobody wrote is
in neither.

---

## 5 Three more compiler defects

### 5.1 `Ok(())` could not be written

`expr_list` collapses a lone unit argument to the empty argument list, because
`f ()` is doc 18 §7's zero-argument call. Its doc comment states the trade:
*"a function genuinely taking a `()` cannot be called by juxtaposition … a unit
parameter is vanishingly rare against a zero-argument call."*

It is not vanishingly rare. It is how **every** `Result<(), E>` function
returns, and this crate has eleven of them: `return Ok ()` emitted
`return Ok();`. No ported crate had needed it before, which is why a documented
trade had never been tested.

The two readings are decidable from the convention the language already runs
on — **the case of the first letter**, which decides a type variable (doc 18
§5.3) and a constructor pattern's path (plan 68). An uppercase last segment is
a constructor, so `Ok ()` is `Ok(())`; a lowercase one is a function, so
`String::new ()` stays `String::new()`. Nothing is taken away: a unit struct is
written `Marker` with no call, and `data` cannot declare a zero-field tuple
struct.

### 5.2 A function type kept its grouping parenthesis

`eat_while : (impl Fn(char) -> bool) -> &mut self -> Span` needs those
parentheses: a type is captured as source text, and the arrow inside
`Fn(char) -> bool` would otherwise split the signature. They are a grouping aid
exactly as doc 18 §2.5's `instance … (Wrapper t)` are — and `capture_type` kept
them, so the Rust read `pred: (impl Fn(char) -> bool)`.

Stripped where it is provably a grouping: a single balanced group whose content
has a top-level arrow **and no top-level comma**. The comma is the half I got
wrong first — `(&str, fn(char) -> bool)` is a tuple *and* has a top-level
arrow, so the first version stripped a tuple's parentheses, and the very next
file to use one reported *"expected `=` in a binding"*. Five cases pin it now.

### 5.3 The ascription lowering parenthesised its value

Carried from part 1 of this row: `let __rdx_t: T = (e);` is
`unnecessary_parens`, and the parenthesis is not the author's to drop —
`(Box::new h) : T` ascribes the call rather than `h`. The lowering looks
through one, as plan 69's `Unary` arm does.

---

## 6 Two process traps

**`git checkout <sha> -- <path>` stages what it writes.** The build script
restores the hand-written Rust from a pinned commit before translating, and
that silently re-tracked the tree this very port untracks — so
`the_generated_trees_are_ignored` failed on every build after the first.
`git restore --source=<sha> --worktree` writes the worktree and leaves the
index alone.

**The order is build → bless → seed-refresh.** The restore puts the
*hand-written* Rust back in `src/`, so refreshing the seed before blessing
builds the seed from the Rust being replaced — and the clean-clone test then
compares the seed's correct output against a tree that is not the port's at
all. It reads as a version skew and is an ordering mistake. Both are written
into the build script.

---

## 7 Surface facts this port had to learn

Each cost one diagnostic, and each is a consequence of a rule already stated
rather than a new restriction:

- **a `for` binder is a bare name** — `for (i, line) in …` is `RDX0103`, the
  same rule a lambda parameter follows;
- **`break` is not an inline arm body**, only a block one;
- **a `case` in statement position followed by a `.name` statement** is
  `RDX0115`. Plan 69's fixture for the block-valued binding says a block in
  *statement* position never reaches `postfix`, with an `if`/`else` as its
  witness — and the `if`/`else` really is safe. So the rule belongs to the
  construct, not to the binding, and that fixture's prose is wrong;
- **`x.1.0` lexes `1.0` as a float**, so a nested tuple index needs a binding
  to break it. Rust has the same wart;
- **a literal `#{` needs `\#`**, which makes this the one crate whose own
  syntax collides with the text it is about;
- **a tuple-destructuring binding** is a named pair and its fields, three times
  here;
- **a helper closure that needs a type annotation cannot be written**, so
  `scan_byte` repeats its error literal five times and `scan_escape` carries
  `Result<char, String>` and attaches the code and span once — which is shorter
  than the Rust and says what the Rust's closure encoded and each caller
  re-stated.

Two places the port made the code **better**, not merely equivalent:

- **`is_ident_start` and `is_ident_continue` take a `char`.** All four callers
  had a `char` and wrote `is_ident_start(c as u8)`; the cast was *truncating*,
  so `'é' as u8` is `0xE9` and the right answer came out by luck.
- **`eat_while` takes `impl Fn`**, where the Rust took `mut pred: impl FnMut`.
  All nine callers capture nothing or capture immutably, so the stronger bound
  and the `mut` binding were never needed.

---

## 8 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 33 files are Voltaic; `crates/voltaic-lexer/sdx` is the source | yes |
| 2 | `voltaic-lexer` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/voltaic-lexer/src/` is gitignored and out of the index | yes — 33 files left it |
| 4 | No escape hatch across all 164 ported files | yes |
| 5 | Every doc comment survives | yes |
| 6 | **Every test survives** | yes — 94 before, 94 after, verified against the pinned commit |
| 7 | Zero `as` casts, removed rather than allowed | yes — 5 → 0 |
| 8 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 9 | A clean clone bootstraps it from the seed | yes — seed refreshed |
| 10 | `stage1 == stage2` still holds | yes |

---

## 9 Owed to successor rows

- **A test-scoped helper** (plan 73 §2.9). This crate has four more
  `#[cfg(test)]` helpers at module scope.
- **`param_text` and the parser disagree** about an annotated lambda parameter.
- **`voltaic fmt` diverges on a nullary lambda in argument position**, one
  parenthesis per pass, unbounded — which is why `scan_byte` does not use a
  nullary closure.
- **Plan 69's UI fixture claims the wrong contrast** (§7, third bullet). Its
  prose should name `case`, not the binding.
- **`RDX1001` has no scoped or file-level allow.**
- **The `no_effect` / plan 55 §1.6 contradiction**, now allowed in four crates.
- **`compile_dir` does not descend** — carried from plan 72.

---

## 10 Next

`voltaic-emit` (6,156 lines), `voltaic-parser` (7,974), `voltaic-driver` (2,981),
`voltaic-cli` (1,376), `voltaic-bench` (597), `voltaic-fuzz` (124), `voltaic-lsp`
(934), `voltaic-mcp` (808) — eight crates, roughly 21,000 lines.

`Sink<'a>` and `Parser<'a>` are the only other generic declarations in the
workspace and §2.2 unblocks both, so nothing known stands in the way of any of
the eight. What this row changes most for them is the procedure: count the
tests against the pinned commit, read every file to its end, and expect
`voltaic fmt`'s idempotence to find what a parse cannot.
