# Plan 79 — The CLI in Redox

`redox-cli`'s 24 files, 1,376 lines of `src/`. The ninth crate ported, and the
one whose `src/` is the `redox` binary — so from this commit the command a
person types to compile Redox is itself compiled from Redox.

Two of its three findings are about **checks that could not see what they were
for**. The port's own gate is a parse, so what survives parsing arrives late;
and the doc-comment check compares the `.sdx` with the Rust it generates, which
cannot notice a comment both sides lost. One of them had already cost
`redox-lsp` a fifteen-line doc comment, three rows ago, silently.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/redox-cli/sdx/**` — 24 files, 1,068 lines |
| Generated | `crates/redox-cli/src/**`, 1,494 lines, untracked from this commit |
| Tests | 42, unchanged — and **not** ported |
| List | `seed/PORTED` gains `redox-cli 79` |
| Seed | refreshed, 925,628 → 924,941 bytes |

**The 9,397 lines under `crates/redox-cli/tests/` stay Rust**, and that is the
layout rule rather than a concession: doc 06 §7 makes a public-API test an
integration test, and an integration test is a separate crate that links the
one it tests. So `port.rs`, `examples.rs`, `ui.rs` and the rest now run
*against* a Redox-sourced `redox-cli` — the harness that checks the port
compiles the thing it checks.

**The bootstrap cycle is broken by the seed, not by this crate.**
`./bootstrap.sh` regenerates every ported `src/` with `seed/redox1.wasm`, which
is `redox-seed` and stays Rust; it never invokes `redox-cli`. Without that the
ninth port would have made the CLI require itself.

---

## 2 What it found

### 2.1 `RDX0115` refused a legal program, and the formatter hid the workaround

`.contains p.name (&text)` is the ordinary receiver-last call — `p.name` is an
argument, `(&text)` is the receiver. It was `RDX0115: method-call syntax is
retired`.

`postfix` refused **any** `(` after a field name. Plan 57 §11.2's reason for
that is sound as far as it goes: `x.f (a)` was `Expr::MethodCall` and the
parenless `x.f a` is an application over an `Expr::Field`, so refusing one
spelling leaves the other emitting the identical `x.f(a)` silently. What it
missed is that `postfix` runs inside `atom`, which **cannot see that there is
an enclosing application**: the `(` it refuses may be the next argument of the
call it is an argument to.

**Adjacency separates the two halves, and it is what separates the two
spellings.** Rust writes `x.f(a)`; a sibling argument has a space before it. So
`postfix` now refuses only a `(` whose span starts where the field name ends,
and `application` keeps refusing a field as an application's *head* — which is
every spaced form, including all three the UI fixture already pinned. Their
`.stderr` is unchanged but for line numbers, because `application` reports the
same span `postfix` did.

**The first fix was a workaround, and it was stable for exactly one run.**
Parenthesising — `.contains (command.name) (&text)` — parsed. Then
`port.sh` formatted, `stands_alone` judged `Expr::Field` safe to juxtapose (it
is, and it was right), the parentheses came off, and the next file-by-file
build reported the same diagnostic from source the formatter had just written.

That is the **fourth** time in six rows that a formatter round-trip has
decided whether a program compiles — after row 74's range, row 77's `move` and
row 78's `as`-in-an-import — and the first where the formatter was **not** at
fault. `stands_alone`'s whole job is to mirror `parser::starts_argument`, and
it mirrored it correctly; the parser disagreed with itself. Which is the useful
version of the lesson: *a formatter that round-trips to unparseable output has
found a disagreement, and it is not always the formatter's.*

`tests/ui/retired_method_call.sdx` gains the adjacent spelling as a fourth
shape, and `parse.rs` gains both directions — the legal non-final field access,
asserted down to the argument list, and the adjacent refusal.

### 2.2 The body writer ate a doc comment, and the doc-comment check cannot see it

`write.py` drops the head's trailing comment block when the body supplies its
own, because the translator leaves a doc comment for the first construct and a
hand-written body usually writes a better one. The condition was *any* line
starting `--`.

`run/commands`' body opens with an ordinary comment — the one-line-per-entry
note relocated out of the Rust's `COMMANDS` table, which is one line here. That
took the 19-line `-- |` block describing `COMMANDS` with it.

**An ordinary comment and a doc comment are different things**, which is the
rule this whole toolchain exists to respect, and only the second supersedes. The
condition is now `-- |`.

**`every_doc_comment_reaches_the_generated_rust` structurally cannot catch
this.** It counts `-- |` in the `.sdx` against `///` in the Rust *generated from
it*, and both sides lost the comment together, so the port was green and
self-consistent. The check that found it compares the **pinned hand-written
Rust** with the generated Rust — a different question, asked once, by hand:

```
crate          orig ///   now ///
redox-cli           322       338      (+16, the doc comments this port adds)
redox-lsp           324       321      LOST 3
```

`redox-lsp`'s `serve/symbols/symbol_of.rs` had a 15-line doc comment and its
`.sdx` had none: plan 78's body for it opens with the ordinary comment about
where the `#[allow]` had to move, and the rule ate the comment above it. Its
`serve/cast.rs` lost the summary sentence *"Deserialise a request's parameters
for a known method"* the same way. Both are restored here.

So doc 02 §10.1's *"comments are trivia, never discarded"* was violated three
rows ago by a tool, and the gate that exists to check exactly that reported
success. **A fidelity check needs a source outside the thing it checks**, and
this one owes a successor row a test: the comparison above is a thirty-line
script and should be `port.rs`'s sixth assertion, reading each crate's pin from
the commit that untracked its `src/`.

### 2.3 Plan 55 §1.6's allow, decided

Five crate roots carried `#![allow(clippy::no_effect)]` with a paragraph
explaining that §1.6's written `()` is what clippy objects to. Four carried
`match_like_matches_macro` with a paragraph explaining that `matches!`'s second
argument is a pattern. Two carried `single_match`; one carried
`while_let_loop`. Every one of them says the same thing: **clippy is asking for
a Rust form this language cannot write.**

`redox-cli` would have been the eighth repetition of the first paragraph. Plan
75 §2.4 called that the wrong answer; plan 76 §6 did it anyway and recorded
that it had; plan 78 did it again. The decision was owed, and the two options
named were a workspace-level allow or a change to §1.6 — and §1.6 has an
argument behind it, so it is the allow.

`[workspace.lints.clippy]` now holds the five, with one table saying why, and a
crate opts in with `[lints] workspace = true`. **126 lines come out of seven
crate roots**, and the three still hand-written in Rust — `redox-driver`,
`redox-emit`, `redox-parser` — deliberately do *not* opt in, so clippy keeps
its real verdict on them until their own port row adds those two lines. That is
the whole cost of this per future port, where it used to be a ten-line
paragraph.

### 2.4 A comment asserting something false, caught by rustc rather than the port

`main`'s first draft matched on `run::run(&args)` with **one arm**, and said so:
*"`run::run`'s success type is uninhabited, so there is no `Ok` to match."* It
returns `Result<(), String>`. `E0004: non-exhaustive patterns: Ok(_) not
covered`.

Worth recording because of *when* it was caught. `port.sh` reported `files with
errors: 0` — the `.sdx` parses, and a `case` missing an arm is not a parse
error. It took `port.rs`'s compile step, which stages the output and refuses to
write when rustc rejects it, and that refusal is the only reason
`crates/redox-cli/src/` was not left broken. **A port's own check is a parse**,
which row 78 said about silent defects and which applies just as well to a
confident wrong sentence in a comment.

### 2.5 Smaller things, each already recorded somewhere

| | |
|---|---|
| **An open-ended range has no spelling** | `&args[1..]` is `&args[1 .. n]` with `n = .len args` bound first. Third sighting — plan 68's `walk.sdx`, plan 76's `signature_line.sdx`, now `run.sdx`. |
| **`not` is lexed and not parsed** | `Keyword::Not` has no arm in `unary_op.rs`, so negation is `!` and `if !.is_empty warnings` is the spelling. A keyword the lexer knows and the parser does not is worth a row's attention: it is either an operator or it is not a keyword. |
| **`matches!` with `\|`** | `Command::matches` nested `matches!(arg, "--help" \| "-h" \| "help")` inside a `match`. An arm's `\|` is doc 03 §6.2's guard, so the macro does not carry over; three `if`s say it flatter than `or` inside a `case` inside a `case`. |
| **A test-scoped helper *type*** | `target.rs`'s `Dir` is a newtype with a `Drop` and a `static` counter. Redox puts nothing but `#[test] fn`s inside `mod __rdx_tests` (plan 73 §2.9), so all three live at file scope behind `#[cfg(test)]` — and the counter is the **first module-scope `static` in a port**, where the Rust declared it inside the function. `write.py`'s construct pattern did not know the keyword. |
| **`drop (…)` where the Rust wrote `let _ =`** | `recommended_watcher`'s closure discards a `#[must_use]` send. Plan 75 §2.3's `_dropped = …` needs a statement and a lambda body is one expression. |
| **No tuple-destructuring binding, twice** | `let (tx, rx) = mpsc::channel()` and `let (rust, map) = emit_with_map(…)` become a bound pair and two projections. Edition 2021's disjoint closure capture is what makes `channel.0` moving into a `move` closure leave `channel.1` usable. |
| **A field access that is not the last argument** | Needs no parentheses after §2.1, and would have been wrong with them — see the formatter note there. |

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 24 files are Redox | yes |
| 2 | `redox-cli` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/redox-cli/src/` is gitignored and out of the index | yes |
| 4 | `crates/redox-cli/sdx/` **is** in the index | yes |
| 5 | Every test survives | yes — 42 before, 42 after, same names |
| 6 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 7 | A clean clone bootstraps it | yes |
| 8 | `stage1 == stage2` still holds | yes |
| 9 | `RDX0115` accepts a non-final field access and still refuses `x.f(a)` | yes — both pinned in `parse.rs`, four shapes in the UI fixture |
| 10 | No doc comment lost, here or in the rows the same bug touched | yes — `redox-lsp`'s two restored |
| 11 | The §1.6 allow is stated once | yes — `[workspace.lints.clippy]`, 126 lines removed |

---

## 4 Next

Three crates, 17,441 lines: `redox-driver` (2,981), `redox-emit` (6,203),
`redox-parser` (8,257). They are the compiler core, and they are the three that
do not inherit the workspace allows, so each port's first `clippy` run will say
which of the five that crate actually needs.

Owed to a successor, from this row:

- **The fidelity check in §2.2 should be a test.** It found a three-row-old
  violation of a rule the gate claims to enforce, and it is thirty lines.
- **`not` should be an operator or not a keyword** (§2.5).
- **A keyword collision needs its own diagnostic** — `rust = …` still reports
  *"expected a `"""` block of Rust after `rust`"* — and the translator should
  prefix `r#`. Owed since row 75, now three rows old.
- **`FieldInit` needs a `leading`**, so an attribute on a record-field
  initialiser is carried rather than dropped. Owed since row 78.
