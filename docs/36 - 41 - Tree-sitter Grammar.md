# Plan 41 — Tree-sitter Grammar

Plan ID: `tree-sitter-grammar`
Depends on: 19 (`parenless-calls`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

`tree-sitter-redox/` per doc 02 §5.4: `grammar.js`, an external scanner in C for
`INDENT`/`DEDENT`/`NEWLINE`, the generated parser, the three query files, and a corpus suite.

Doc 04's cell discloses the cost rather than hiding it, and the disclosure is the right one:
*"an indentation-sensitive tree-sitter grammar is genuinely fiddly, and it is a second grammar
to keep in sync with plan 46's inventory"*. §1.2 is this plan's answer to the second half,
because a second grammar that tries to be a second **source of truth** is a liability, and one
that does not is a tool.

### 1.1 Verified preconditions

Spiked end to end before this plan was written, because every claim below is about a toolchain
rather than about Redox.

**The tools are present**, and nixpkgs agrees with the system on the version:

```
$ tree-sitter --version
tree-sitter 0.26.11
$ nix eval --raw nixpkgs#tree-sitter.version
0.26.11
$ which node cc
/nix/store/…-nodejs-slim-24.18.1/bin/node
/nix/store/…-gcc-wrapper-15.3.0/bin/cc
```

**An external C scanner compiles and runs.** A throwaway grammar with
`externals: $ => [$._indent, $._dedent, $._newline]` and a sixty-line scanner holding a stack
of indent widths generated, compiled and parsed `alpha / ␠␠beta / ␠␠gamma / delta` to a tree
with no `ERROR` node. So the hard part — an indentation-sensitive external scanner under this
CLI, on this machine — is known to work rather than assumed.

**`tree-sitter test` works**, and its corpus format is `test/corpus/*.txt`: a `===` banner, the
source, `---`, and the expected s-expression. The spike's one case passed.

**One gap, and leaf 1 closes it.** `tree-sitter generate` warns:

> No `tree-sitter.json` file found in your grammar, this file is required to generate with
> ABI 15. Using ABI version 14 instead.

So a grammar without that file is silently generated against the **older ABI**. Leaf 1 runs
`tree-sitter init` so the committed parser is ABI 15.

**And one the register did not mention**: `tree-sitter` is on the *system* `PATH`, not pinned
by `devenv.nix` like every other tool here. Today the two versions match, so nothing is wrong
and nothing is reproducible either — a generated parser is an artefact of its generator, and an
unpinned generator means the committed parser cannot be regenerated identically on another
machine. Leaf 1 adds it to `devenv.nix`'s `packages`.

### 1.2 The grammar's job is highlighting, not parity

The decision that shapes the whole row. A tree-sitter grammar could aim at two things:

| Aim | What it costs |
|---|---|
| **Parity** with `redox-parser` | A second implementation of the language, which drifts, and which plan 46's inventory would have to police forever |
| **Highlighting, folding and selection** | A shallower grammar that is exactly right about block structure and deliberately approximate inside an expression |

This row takes the second, and the argument is not that it is cheaper. It is that **the
compiler is the authority on what parses**, and a grammar that claimed to be a second one would
make every disagreement a bug report against the wrong component. An editor asks a tree-sitter
grammar *where are the blocks, what colour is this token, what do I select when you press
expand* — none of which needs to know that `f (x, y)` applies `f` to a tuple.

So the grammar is **precise about the off-side rule and loose about precedence**. Expressions
are a flat sequence of operands and operators rather than a precedence cascade, which is what
`highlights.scm` wants anyway: a query colours a token, not a tree shape.

**What that buys**: no `grammar.js` conflict resolution, no second precedence table to keep in
step with `BinaryOp::power` — which plan 34 §10.13 already identified as a thing that drifts
silently — and an `ERROR`-free parse of the corpus as a meaningful assertion rather than a
coincidence.

**What it costs**, stated so nobody is surprised: the grammar cannot answer *is this program
valid*, structural selection inside an expression is coarser than a full grammar's, and plan
46's inventory must compare **token kinds and block structure**, not productions. §6 records
the last as that row's constraint.

### 1.3 The corpus check is the real assertion

`tree-sitter test`'s corpus files are hand-written expectations, and doc 02 §9.4 wants the
grammar to be *"an independent check on the grammar's shape"*. Hand-written s-expressions are
not that: they are a check that the grammar does what its author last thought it did.

The assertion with teeth is **every `.sdx` in the repository parses with no `ERROR` node**.
That corpus already covers every construct the language has, it is maintained by the compiler's
own gate, and a grammar that drifts from the language fails it without anyone writing a new
expectation. Plan 34 §1.4 made the same argument for the formatter's properties and plan 43
§1.1 for the lexer's, and this is the third row where the existing corpus is the cheapest large
test available.

So the corpus suite is **two things**: a handful of `test/corpus/*.txt` cases for the block
structure specifically — where a wrong answer is visible and an s-expression is the only way to
state it — plus a sweep over `examples/` asserting no `ERROR`.

`lexical.sdx` is **included** in the sweep, unlike in `examples.rs` and the formatter's
properties. It is exempt there because it does not *parse*, and that exemption is about the
compiler's parser; a highlighting grammar should survive it, and doc 02 §9.4's "independent
check" is worth least on the files the compiler already accepts. Plan 43 criterion 3 made the
same call for the same reason.

---

## 2. Admission tests (doc 03 §2)

Not applicable: no construct, no lowering. This row adds a grammar.

---

## 3. Concrete proof this plan targets

```bash
cd tree-sitter-redox && tree-sitter test          # the corpus cases
cargo test -p redox-cli --test tree_sitter        # the no-ERROR sweep
```

The sweep is a Rust test in the workspace rather than a shell script, so it runs in the same
gate as everything else and a grammar that stops matching the language fails `moon run :test`.
It shells out to the pinned `tree-sitter` CLI — the same one the editors will use — rather than
linking the `tree-sitter` crate, which would put a `cc` build of the generated parser into
every `cargo check`.

**The negative case**: a file whose indentation is inconsistent must produce an `ERROR`. A
grammar whose scanner silently accepted any indentation would pass the sweep and be wrong in
the one place this row is supposed to be precise — the same trap plan 43 §10.1 fell into, where
a negative case that passes for the wrong reason is worse than none.

---

## 4. Leaves

### Leaf 1 — `leaf-pin-the-toolchain`

`tree-sitter` into `devenv.nix`'s `packages`, and `tree-sitter init` so the grammar generates
against ABI 15. No grammar yet — §1.1's two gaps closed first, so the committed parser is
reproducible from the moment it exists.

### Leaf 2 — `leaf-scanner-and-blocks`

The external scanner and the smallest `grammar.js` that gets block structure right: items,
statements, `INDENT`/`DEDENT`/`NEWLINE`. Plus the `test/corpus` cases for nesting and the
negative case.

### Leaf 3 — `leaf-the-surface`

The rest of the grammar to §1.2's depth — the item kinds, the statement kinds, patterns, and a
flat expression rule. Plus the no-`ERROR` sweep, which is what tells this leaf it is finished.

### Leaf 4 — `leaf-queries`

`highlights.scm`, `injections.scm`, `locals.scm`.

---

## 5. Acceptance criteria

1. `tree-sitter generate` is reproducible: the pinned CLI, ABI 15, and the generated parser
   committed.
2. `tree-sitter test` passes, with corpus cases covering a nested block, a dedent to an outer
   level, and the negative case.
3. **Every `.sdx` in the repository parses with no `ERROR` node**, `lexical.sdx` included.
4. A file with inconsistent indentation **does** produce an `ERROR`, so criterion 3 is not
   vacuous.
5. `highlights.scm` colours every token kind the lexer produces, which is the same table plan
   40's semantic tokens built — and the two are checked against each other, because two
   highlighters that disagree are worse than one.
6. `injections.scm` injects Rust into nothing. Redox has no embedded-language construct, and
   the file exists to say so rather than to be forgotten — §6.

---

## 6. Out of scope, and what it discloses

- **Parity with `redox-parser`.** §1.2's whole argument. The grammar cannot answer *is this
  valid*, and plan 46's inventory must compare token kinds and block structure rather than
  productions — recorded here as a constraint on that row rather than discovered by it.
- **The editor configs.** Plan 49's, per the register's 2026-11-08 narrowing.
- **A `tree-sitter` Rust binding crate.** §3's reason: it would compile the generated parser in
  every `cargo check` to serve one test that the CLI already answers.
- **`tags.scm`**, which drives symbol search in some tools. It needs the same name resolution
  plan 40 §6 declined, so it would be a guess.
- **Incremental-parse benchmarks.** Row 44's, if anyone wants them; tree-sitter's whole point
  is incrementality and this row asserts correctness rather than speed.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

Emerald plan 24's `tree-sitter-emerald` layout and its corpus-test convention — a directory per
construct under `test/corpus/`, which is the shape that keeps a corpus file findable when a
case fails.

**Not taken**: Emerald had no off-side rule, so it had no external scanner and nothing to say
about the hard part of this row. Its grammar also aimed at parity, which is the choice §1.2
declines.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Pin `tree-sitter` in `devenv.nix` | Use the system one, which matches today | A generated parser is an artefact of its generator; an unpinned generator means the committed parser cannot be regenerated identically elsewhere |
| 2 | Highlighting-grade grammar, not parity | A second full grammar | The compiler is the authority on what parses, and a second one makes every disagreement a bug report against the wrong component (§1.2) |
| 3 | A flat expression rule | A precedence cascade mirroring `BinaryOp::power` | A second precedence table is exactly the thing plan 34 §10.13 found drifts silently, and a highlight query colours a token rather than a tree shape |
| 4 | The no-`ERROR` sweep over `examples/` is the main assertion | Hand-written corpus expectations alone | Those check what the author last thought; the corpus checks the language, and it is maintained by someone else's gate |
| 5 | Include `lexical.sdx` in the sweep | Exempt it, as the compiler's own tests do | It is exempt there because it does not *parse*; a highlighting grammar should survive it, and the independent check is worth least on files the compiler already accepts |
| 6 | Shell out to the CLI from a Rust test | Link the `tree-sitter` crate | Linking puts a `cc` build of the generated parser into every `cargo check` to serve one test |
| 7 | Ship an empty `injections.scm` | Omit the file | Redox has no embedded language; a file that says so will not be rediscovered as a gap |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
