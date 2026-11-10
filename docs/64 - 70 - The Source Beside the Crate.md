# Plan 70 — The Source Beside the Crate

Status: `done` 2026-11-10
Depends on: 68 (the AST in Redox), 69 (the formatter in Redox)

> **A record, not a forecast.** This row was executed directly from a request rather than
> planned first, and the document is written after the fact. §1 is the survey `AGENTS.md`
> cites; §10 is what happened. There is no §4 of leaves because there were none — the work
> was four mechanical steps and four things they exposed.

---

## 1. The survey: where every self-hosting language keeps its previous compiler

A self-hosting compiler cannot build itself from nothing. Something has to break the
cycle, and across the field that something takes exactly two forms.

| Project | Stage0 is… | Form |
|---|---|---|
| **MoonScript** | the generated Lua, committed beside each `.moon` | source |
| **CoffeeScript** | the generated JS in `lib/`, compiled by `bin/` | source |
| **Nim** | `csources_v2`, pre-generated C, frozen | source |
| **rustc** | the previous **beta**, downloaded by bootstrap | binary |
| **Go** | a previous Go toolchain, in a documented chain | binary |
| **Zig** | `zig1.wasm`, 2.4 MiB, checked in | binary |
| **OCaml** | `boot/ocamlc`, committed bytecode | binary |

**The split is not taste. It is what the compiler emits.** The source camp is every project
whose output is source in a language whose compiler is *already* a prerequisite —
MoonScript needs Lua, CoffeeScript needs node, Nim needs a C compiler, and all three ship
their previous version as text because they can. The binary camp emits machine code, so
there is nothing to ship but a blob.

Redox is squarely in the first camp: it emits Rust, and rustc is a hard prerequisite. So
its previous-version compiler can be **reviewable, diffable, platform-independent text** —
which is what the committed `.rs` under each ported crate's `src/` is.

Three measured facts made this row keep it that way.

**Zig's objection does not apply at this scale.** Zig *rejected* committing generated C on
two grounds: the `wasm2c` output was **181 MB and target-specific**. Redox's generated Rust
is **6,397 lines, 239 KB**, and target-independent. The main historical argument against
committed output is simply absent here.

**Committed source is a better stage0 than a binary.** rustc downloads one, Go chains one,
Zig and OCaml check in opaque artifacts. None of them would refuse what Redox already has;
they cannot have it.

**And the version-skew cost is the decisive one.** rustc's policy is explicit: the compiler
may use a feature only once it is in the stage0 beta, and otherwise carries `cfg(bootstrap)`
double implementations. Go pins a chain (1.5–1.19 ← 1.4, 1.20–1.21 ← 1.17, 1.22–1.23 ←
1.20, now N ← N−2) and bumps it about once a year. Under a release-N−1 stage0, Redox's own
`.sdx` could use only syntax that release understood — and plans 52–67 changed the surface
in **every row**, with plan 69 finding three more gaps. The multi-statement lambda plan 69
wrote, which exposed the `lambda_text` defect, would have been unusable in the compiler's
own source.

**This survey is what plan 72 overturns, deliberately and on a narrower argument** — see
that row's §1.2 and §1.3. The conclusion above is correct *for the transitional state*, in
which most of the compiler is still hand-written Rust and the committed `.rs` is a small
derived minority. It stops being correct when the derived tree is the whole compiler.

---

## 2. What this row did

Four steps, from a direct request:

1. `conformance/redox-ast/**` → `crates/redox-ast/sdx/**`, and the same for `redox-fmt`.
   110 renames, no content change.
2. `conformance/` removed.
3. `.gitattributes` marks both `src/` trees `linguist-generated` and `-diff`.
4. `AGENTS.md` gained the convention, and the reason the committed `.rs` must stay.

The crate is now self-describing: a reader of `crates/redox-ast/src/expr.rs` finds
`crates/redox-ast/sdx/expr.sdx` beside it, rather than in a directory named for a different
plan's probe.

**What this row did *not* do** is change what is committed. §1 is why.

---

## 3. Gate

`moon run :check :lint :test` green, 849 tests, 170 in `redox-cli` — two fewer than before,
being plan 48's retired probe tests. `docs/STATUS.md` and `docs/CONFORMANCE.md` reblessed.

---

## 10. Result

**Done 2026-11-10**, at `c7d9c37`. The move was mechanical; the four things it exposed were
not.

### 10.1 `moon run :test` could not see a stale `.sdx`

`redox-cli`'s test task listed `/crates/**/*.rs`, `/examples/**/*`, `/tests/**/*` and
`/docs/**/*.md` as inputs. A `.sdx` matched none of them. Measured:

```text
moon run redox-cli:test                     -> cached, b734ee39
<append a comment to a .sdx; the .rs is now stale>
moon run redox-cli:test                     -> cached, b734ee39   (same hash)
```

A `.sdx` edited without regenerating is **precisely what `tests/port.rs` exists to catch**,
and the gate reported green. It only ever fired under a direct
`cargo test -p redox-cli --test port`, which is how plan 69 was driven by hand — which is
why nobody noticed. `/crates/*/sdx/**/*` is now an input, and the same experiment fails on
`the_port_regenerates_the_committed_rust`.

The move did not fix this by itself: `/crates/**/*.rs` still does not match a `.sdx`.

### 10.2 `port.rs` knew the layout in two places

The mirror test built the `.sdx` path itself, so the move changed one copy of two and the
reverse direction searched a deleted directory — reporting all 53 files unported. **Plan 69
leaf 0 had just removed this exact shape from this exact file**, by replacing a hard-coded
crate name with `PORTED`; it survived in the path. `sdx_root` and `src_root` are now the
only definitions and both directions call them.

It failed loudly, which is the only reason this is a footnote. Had the mirror checked only
the forward direction it would have gone green on a broken layout.

### 10.3 Plan 68 shipped two of its own criteria unmet, and did not disclose it

That row's criterion 10 reads *"`conformance/clause.sdx` is **retired**, with its `syn`-only
tests, because this row supersedes it"*, and criterion 9 asks for `docs/CONFORMANCE.md`
*"with the worked-port section naming a crate and rustc as its oracle"*.

The file, both tests and the prose survived plan 68 **and** plan 69, and the row was marked
`done` with its §10 silent on all of it. Both were carried out here — found by reading the
criterion while asking whether `conformance/` could be deleted. `syn` left `redox-cli`'s
dev-dependencies with the test that wanted it; `redox-fuzz` still uses it on its own terms.

`plan_orientation.rs` exists because an orientation finding that is written down and not
acted on reads as agreement. This is the same failure one level up: a **criterion** written
down and not acted on, with no test for it. Worth a successor's attention.

### 10.4 `CONFORMANCE.md` claimed the compiler is not compiled from Redox

It ended with *"Nothing in the build depends on `conformance/`, and the compiler is not
compiled from it"* — true when plan 48 lifted one 45-line module out of a crate and checked
it with `syn`, and **false since plan 68 generated a whole crate's `src/`**. A generated
document carrying a stale claim is worse than a stale hand-written one, because the
regeneration looks like a refresh. Rewritten.

### 10.5 Two smaller things

Plans 48 and 65 cite the deleted fixture and keep their bodies; each gained an unnumbered
closing note. Plan 65's matters more — it used `clause.sdx`'s multi-line record declaration
as its bracket-layout **witness**, and that witness is now
`crates/redox-ast/sdx/program/item/clause.sdx`, the real port of the same module, compared
byte for byte by `port.rs` and compiled by rustc rather than checked by `syn`.

The notes are **unnumbered** because `## 11.` is reserved for *"Corrections from
orientation"* and `plan_orientation.rs` requires §1–§9 to cite it. Taking that slot gave two
finished plans an orientation they could not possibly have cited, and the test caught it
within a minute.

Plan 69's three `tests/ui` fixtures lost their `-- |` doc blocks for plain `--`, which is
both the convention (45 of 48) and the right comment form. That also unbroke `redox-fuzz`,
whose sweep mutated the item out from under a twenty-line doc block and got a
doc-comment-only program — which emits a dangling `///` that `syn` rejects.

**That is not an emitter defect**, which is the part worth keeping. Two lines of Redox reach
it with no mutation:

```text
-- | A doc comment with no item under it.
-- | Does the emitter write a dangling `///`?
```

The emitted Rust is a faithful translation of an incomplete program, and `redox check`
reports rustc's own *"expected item after doc comment"* **against the `.sdx`, at line 2
column 1**, through plan 07's remapping. The design working, not failing. So the correction
belongs to the fuzz **oracle**, whose doc says *"a syntax error in generated code is Redox's
own bug and nobody else's"* — true of a mangled emission, false of a faithful emission of a
program that is not itself a complete Rust item. First false positive of that class, now
written down beside the assertion.
