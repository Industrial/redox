# Plan 41 — Tree-sitter Grammar

Plan ID: `tree-sitter-grammar`
Depends on: 19 (`parenless-calls`)
Milestone: 7 — *tooling*
Status: `done` — all four leaves shipped 2026-11-09

---

## 1. What this plan is

`tree-sitter-voltaic/` per doc 02 §5.4: `grammar.js`, an external scanner in C for
`INDENT`/`DEDENT`/`NEWLINE`, the generated parser, the three query files, and a corpus suite.

Doc 04's cell discloses the cost rather than hiding it, and the disclosure is the right one:
*"an indentation-sensitive tree-sitter grammar is genuinely fiddly, and it is a second grammar
to keep in sync with plan 46's inventory"*. §1.2 is this plan's answer to the second half,
because a second grammar that tries to be a second **source of truth** is a liability, and one
that does not is a tool.

### 1.1 Verified preconditions

Spiked end to end before this plan was written, because every claim below is about a toolchain
rather than about Voltaic.

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
| **Parity** with `voltaic-parser` | A second implementation of the language, which drifts, and which plan 46's inventory would have to police forever |
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
cd tree-sitter-voltaic && tree-sitter test          # the corpus cases
cargo test -p voltaic-cli --test tree_sitter        # the no-ERROR sweep
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
6. `injections.scm` injects Rust into nothing. Voltaic has no embedded-language construct, and
   the file exists to say so rather than to be forgotten — §6.

---

## 6. Out of scope, and what it discloses

- **Parity with `voltaic-parser`.** §1.2's whole argument. The grammar cannot answer *is this
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
| 7 | Ship an empty `injections.scm` | Omit the file | Voltaic has no embedded language; a file that says so will not be rediscovered as a gap |

---

## 10. Result

Shipped 2026-11-09. `tree-sitter-voltaic/` — `grammar.js`, a nesting-aware external scanner,
the generated ABI 15 parser, three query files, five corpus cases, and a Rust sweep in the
workspace gate.

**All four criteria that can be checked are met**: the corpus parses with no `ERROR` node —
twenty-eight examples and two lint cases, `lexical.sdx` included — an inconsistent dedent
*does* error, the parser is ABI 15 from the pinned CLI, and the highlight query loads and
matches.

§1.2's decision held up and §1.3's prediction came true in the most direct way available.
Everything else below is about the scanner, which is where the row's whole difficulty was.

### 10.1 The scanner took three designs, and only the third can work

A scanner cannot emit NEWLINE and then INDENT at the same place. `advance(lexer, true)` marks a
character as skipped **and consumes it**, so whitespace measured for one token is gone before
the next call, and `mark_end` does not hold it back. Both of the first two designs tried to do
it anyway:

| Attempt | Symptom |
|---|---|
| Consume the indentation while emitting NEWLINE | A zero-width ERROR at the head of **every** indented block — twenty-eight files, one cause |
| Mark the end at column zero, then measure | The same, because marking does not rewind |

The third is tree-sitter-python's: **INDENT absorbs the newline**. One measurement per call,
whichever of the three the width implies, and the grammar has no `_newline` before an
`_indent` — a block's opening *is* the indent.

That is a design the row should have started from. The lesson is narrower than "read the prior
art": it is that **`advance(…, true)` consumes**, and a scanner that needs to look at the same
bytes twice has to remember them instead.

### 10.2 Which is what the two `pending` fields are

Remembering is the other half. A statement ending at a dedent needs its NEWLINE *before* the
DEDENT, and a statement ending at an indent needs its NEWLINE before the INDENT — and in each
case the width has already been consumed, so the token the grammar is not ready for cannot
wait in the input. It waits in the scanner: `pending_dedents` and `pending_indent`.

Getting that wrong produced a zero-width ERROR at the first item after every indented block —
the same symptom as §10.1 and a different cause, which is what made it slow to find. A dedent
of several levels at once needs the count computed once, too, which is where the
inconsistent-dedent check belongs: it is a question about the whole run.

### 10.3 `prec` is static and `conflicts` is dynamic, and the difference cost four rounds

`section` at `prec(2)` broke `((b).c ())`. The parser committed to reading `( (b) . )` as a
left section and had **no alternative left** when `c` arrived, so most of the corpus failed on
a nested parenthesis followed by a postfix.

`prec` tells the generator which reading to keep and **discards the other**. A genuine
ambiguity — one whose resolution depends on what comes next — needs a declared `conflicts`
entry, which tree-sitter's GLR explores at parse time. `pattern_test` had the same bug for the
same reason.

The grammar's `conflicts` list is eighteen entries and every one of them is honest: the
language has real ambiguities, and `f (x) = y` against `f (x)` is the one `voltaic-parser`
resolves by scanning forward to an `=` at depth zero. A conflict here is the same decision made
by a different mechanism.

### 10.4 The flat expression rule was right, and its edges were not free

§1.2's flat rule cost nothing in the places it was defended — no precedence cascade, no second
copy of `BinaryOp::power`. It cost something at the boundary between an expression and the
construct around it, and three times:

- **`=`, `->`, `=>`, `,`, `;` and `:` are separators, not operators.** Accepting them in the
  flat run ate the construct: a guard's `| n % 2 == 0 = Some (n / 2)` had its `=` taken by the
  condition, leaving the guard no body. Each belongs to a rule that names it.
- **A comma had been riding in the `operator` token.** Removing the separators took two more
  files down until `parenthesized` grew a comma list of its own — which it should have had
  anyway, since `(a, b)` and `x.m (a, b)` are both it.
- **An ascription is a rule, not a `::` followed by whatever.** `:: Result<i32,
  core::num::ParseIntError>` has commas inside angle brackets, and a type is a type.

The generalisation: a flat rule is cheap in the middle and needs care at both ends.

### 10.5 Five surface facts the grammar got wrong by assuming

Each found by running the corpus, each a one-line fix, and all five are the same mistake —
writing the rule from what the construct looked like rather than from what the corpus says:

| Assumed | Actually |
|---|---|
| A `const`'s name is lowercase | SCREAMING_CASE, so it lexes as a `type_identifier` |
| A `class` member's `const` has a value | It does not; the `instance` supplies it |
| An import's alias is lowercase | Usually a type — `… as Sorted` |
| A record literal's head is a type | May be a variable: `r = p { x = 99 }` is functional update |
| `deriving` follows the constructors | Sits on its own indented line, ended by the **dedent** |

And one that is the same shape as plan 34 §10.25: **`visibility` had to become a token**,
because as a rule its optional `(crate)` competed with an equation's parenthesised parameter,
and `export(crate) type Depth = f64` came out as an ERROR on `export` plus a pattern `(crate)`.

### 10.6 A pattern is a *run*, in four places

`Some v`, `Poll::Ready v`, `rest @ ..` and `guard Some v = o else` are all **two or more**
patterns juxtaposed, and four rules wanted one. The fix is a `pattern_run` rule used by the
constructor argument list, the `case` arm head, the slice elements, the binding, the guard and
the pattern test.

This is plan 34 §10.19's finding arriving from the other side: there, *parenthesise unless
atomic* turned out to be three rules because what follows differs by position. Here, *a
pattern* turned out to be a run because juxtaposition is the application form for patterns as
much as for calls. Both are the same underlying fact about the language, met twice.

### 10.7 A fenced comment nests, which no regex can express

`{- a {- b -} c -}` is one comment, and `examples/lexical.sdx` has one. So the fence is an
**external token** — the one thing in the scanner that is not about indentation, there for the
same reason the others are: a regex cannot count.

Two things about it are worth keeping. It must run **after** the deferred tokens, because
before them it fired on the call that owed an INDENT and a block whose first line was a fence
never opened. And the fence skipper has to be **one function**: there were two, and the copy
inside `skip_to_content` did not nest — so a nested fence at top level parsed and the same
fence inside a block did not, because the indentation path stopped at the inner `-}` and
measured a width from the middle of a comment. A duplicated algorithm with one copy fixed is
worse than either, which is §1.2's argument turning up inside the row's own scanner.

### 10.8 §1.3 was right about hand-written expectations, immediately

The five corpus cases were written by hand. **Four of them were wrong about the tree** — not
about the block structure, which was the point of writing them, but about the surrounding
nodes. They were regenerated with `tree-sitter test --update` and then read.

That is exactly §1.3's argument for making the sweep the real assertion, and it arrived within
minutes of the file being written. The corpus cases keep their value — they are the only way to
state *this nesting, this dedent, this error* — but as a record of agreed structure rather than
as an independent check.

The sweep found its own working-directory trap on the way: `tree-sitter parse` locates a grammar
by where it was **invoked**, so running it from the repository root gives *"No language
found"* for every file, which reads as a broken grammar and is a broken invocation.

### 10.9 A query naming a node that does not exist is an error

`highlights.scm` listed `"self"` and `"Self"` as keywords. The grammar spells them as one
`self_expression` token, so the query failed to load — and failing to load is the **useful**
behaviour, because a query that silently matched nothing would have left `self` uncoloured with
nothing to find.

Criterion 5 is therefore only half-checked here: `the_highlight_query_loads_and_matches`
asserts the query loads and captures something, which is a real check on the pair, but nothing
asserts that the query's coverage *equals* plan 40's semantic-token table. Naming that rather
than claiming the criterion: the two tables agree by construction today, and the assertion that
keeps them agreeing would need a shared list neither tool can currently read.

### 10.10 The gate's blaze targets SIGKILLed, and AGENTS.md was right about it

`:test` died with `could not build <every crate>: the daemon's worker ended (signal: 9
(SIGKILL): edit-to-binary: 7.4s)` while all five rustc-side targets — `:cargo-check`,
`:cargo-test`, `:lint`, `:ci-format`, `:audit` — passed, and `:check` passed too.

AGENTS.md's gotcha covers this exactly: a blaze failure naming *every* crate with SIGKILL and
no diagnostic is an environment report. `free -h` showed 14 GiB available of 61, with 47 used
from outside this repository, and the workspace had just grown by a crate. `blaze stop` and a
re-run made `:test` and then the whole gate pass unchanged.

Worth recording because it is the first sighting since the entry was written, and the entry
worked: the question *is this memory or is this the tmpfs* was answerable in two commands
because both numbers were already named as the things to check.
