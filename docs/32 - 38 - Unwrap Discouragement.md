# Plan 38 — `unwrap` Discouragement

Plan ID: `panic-and-lints`
Depends on: 05 (`diagnostics-miette`), 17 (`comments-as-trivia`), 23 (`strings-and-literals`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

One lint: **`RDX1001`**, doc 03 §12.7's `unwrap` discouragement. Row 38's last item.

```text
warning[RDX1001]: `unwrap` panics — the ergonomic path is `?`, `??` or `catch`
  --> config.sdx:4:11
   |
 4 │   port = raw.unwrap ()
   │              ^^^^^^
```

§12.7 in full: *"`unwrap` and `expect` are lintable rather than prohibited: the ergonomic
path is `?`, `??` and `catch`, and a panicking accessor produces a warning carrying its own
diagnostic code (`RDX1001`), suppressible per call site. This is Gleam and Elm's safety
posture expressed as a lint, and it costs nothing because no analysis is involved."*

### 1.1 Three things do not exist yet, and §12.7's last clause is why it matters

*"It costs nothing because no analysis is involved"* is true of the **detection** and false
of everything around it. This is the first `RDX1000`-range code and the first diagnostic in
the project that is not an error, so:

| Missing | Why it is needed |
|---|---|
| A **severity** on `Diagnostic` | Every diagnostic so far is an error. `Diagnostic` has `code`, `message`, `span`, `help`, `path`, `source` — and nothing that says *warning* |
| A **non-fatal** path through `check` and `build` | A warning must not fail the build. Today a `Diagnostic` reaching the CLI *is* the failure |
| A **suppression spelling** | §12.7 requires *"suppressible per call site"* and does not say how |

What the project does already have: `DiagnosticCode` carries rustc's own levels through plan
07, `Code`'s doc comment reserves `RDX1000`+ *"for lints, from doc 03 §17's `RDX1001`"*, and
plan 30 established the whole-program-pass shape this lint needs.

### 1.2 Severity is a field, and miette already renders it

`miette::Severity::Warning` exists and `miette::Diagnostic::severity` is the hook. So the
work is a `Severity` enum on `redox-diag`'s `Diagnostic`, defaulting to `Error` so no
existing construction changes, and one `fn severity` in the miette impl.

Plan 07's path is unaffected: it carries rustc's level as the code's text when rustc gave no
code (`remap.rs`), which is a separate mechanism for a foreign diagnostic and stays as it is.
This plan's severity is for **Redox's own** diagnostics, and §9's decision 2 records why the
two are not unified: a foreign level is a string Redox must not interpret, and a Redox
severity is a closed enum that decides whether the build fails.

### 1.3 Suppression: a comment, not an attribute

§12.7 asks for per-call-site suppression and leaves the spelling open. The obvious candidate
is Rust's own `#[allow(…)]`, and it does not work:

- Attributes are **trivia** (plan 18 §1.3) and reach the `.xrs` verbatim. `#[allow(unwrap)]`
  passed through makes rustc say `unknown lint`, so a program that suppressed a Redox lint
  would stop compiling.
- Consuming it instead — recognising one attribute and dropping it — makes plan 18's rule
  conditional. *"An attribute is not a declaration, and trivia is Redox's mechanism for text
  that belongs to a construct without being one"*; a carve-out for a single spelling weakens
  the rule that made nine item kinds work at once.
- `#[redox::allow(unwrap)]` needs `register_tool`, which is unstable.

So the marker is a **comment pragma**, which reaches nothing downstream because a comment
becomes a `//` line in the `.xrs` and means nothing to rustc:

```text
port = raw.unwrap ()  -- redox: allow unwrap
```

Comments are trivia and a `Stmt` already carries `trailing: Option<Trivia>`, so the text is
there to read with no new plumbing. The namespace is deliberate: `-- allow unwrap` alone
would make an ordinary English comment into a pragma by accident.

This is the `# noqa` and `# rubocop:disable` convention, which has the virtue of being
recognisable and the vice of being a second syntax inside comments. §9's decision 3 takes it
and names what it costs.

### 1.4 The unit is the statement, which is coarser than "call site"

A comment attaches to a statement, so `port = a.unwrap () + b.unwrap ()` is suppressed
wholly or not at all. §12.7 says *"per call site"*, and a line with two is one unit.

Disclosed rather than smoothed over: the finer-grained alternative is an attribute, which
§1.3 just ruled out, and the practical unit people reach for in every language with this
convention is the line. A program that wants one of two suppressed can split the line, which
is also better code.

### 1.5 What is detected, and what deliberately is not

A **method call** named `unwrap` or `expect`, by name, syntactically. No analysis, no types,
no knowledge of what it is called on — which is §12.7's *"costs nothing"* and doc 03 §2's
first admission test.

That means a user's own method named `unwrap` on a type that cannot panic also warns. That is
the correct trade for a lint: it is suppressible, it is a warning, and the alternative needs
a type checker AGENTS.md forbids. §6 records it.

`unwrap_or`, `unwrap_or_else`, `unwrap_or_default` and `expect_err` do **not** warn — they
are the ergonomic path, not the panicking one, and matching a prefix rather than the whole
name would flag the cure as the disease.

---

## 2. Admission tests (doc 03 §2)

Not applicable: this plan adds no construct. It adds a **lint**, and the governing rule is
the one AGENTS.md states — *every check rustc can make, rustc makes*. Clippy has
`unwrap_used`, so rustc's ecosystem *can* make this check; what it cannot do is report it
against the `.sdx` with a Redox code and a Redox suggestion naming `??` and `catch`.

That is the whole justification, and it is narrower than the other rows': this lint earns its
place by pointing at **Redox's** alternatives, not by catching something clippy misses. §9's
decision 1 states the alternative — delegate to `clippy::unwrap_used` — and why doc 03 §12.7
does not take it.

---

## 3. Concrete proof this plan targets

Two UI cases, because this plan's deliverable is a message and the UI corpus is the only one
that asserts rendered text:

- `tests/ui/unwrap_warning.sdx` — a program that **compiles successfully** and prints the
  warning. This is new for the corpus: every case so far fails to compile, and this one must
  not, which is criterion 4.
- `tests/ui/unwrap_suppressed.sdx` — the same program with the pragma, whose `.stderr` is
  **empty**. An expectation that asserts nothing is printed is the only way to prove
  suppression works.

Plus emit tests for the detection: `unwrap`, `expect`, the four `unwrap_or*` forms that must
stay silent, a nested call site, and a user method named `unwrap` that warns anyway because
§1.5 says it must.

The suppressed case is the one a golden could not catch, and it is also the one that fails
loudest if severity is wrong: if the warning is still an error, that program does not
compile and the corpus says so.

---

## 4. Leaves

### Leaf 1 — `leaf-severity`

`Severity::{Error, Warning}` on `redox-diag`'s `Diagnostic`, defaulting to `Error`, with
`fn severity` in the miette impl. Plus the CLI change that makes a warning not fail the
build.

### Leaf 2 — `leaf-unwrap-lint`

The pass: a walk over the AST finding `unwrap`/`expect` method calls, a whole-program pass as
plan 30's `check_callback_tails` is, for the same no-double-reporting reason.

### Leaf 3 — `leaf-suppression`

The `-- redox: allow unwrap` pragma, read from a statement's trailing trivia.

### Leaf 4 — `leaf-proof-and-gate`

Two UI cases, the emit tests, and the gate.

---

## 5. Acceptance criteria

1. `x.unwrap ()` warns with `RDX1001`, pointing at the method name.
2. `x.expect "why"` warns the same way.
3. `unwrap_or`, `unwrap_or_else`, `unwrap_or_default` and `expect_err` do **not** warn.
4. A program whose only diagnostic is `RDX1001` **compiles and succeeds** — asserted by the
   UI case's exit status, because a warning that fails the build is an error with a
   misleading label.
5. `-- redox: allow unwrap` on the statement suppresses it, and the UI case's `.stderr` is
   empty.
6. An unrelated comment does not suppress it, so the namespace is load-bearing.
7. The warning renders as `warning[RDX1001]`, not `error[RDX1001]`.
8. Every existing diagnostic still renders as an error — asserted, because `Severity`
   defaults and a default that leaked would turn every error into a warning.
9. `RDX1001` is in the `RDX1000`+ band, and `code.rs`'s range test knows about that band —
   it currently allows `1..=39`, `40..=99` and `100..=199` and nothing above.
10. `code_coverage.rs` still passes: `RDX1001` is reached by the UI corpus.

---

## 6. Out of scope, and what it discloses

- **A `-W`/`-A` command-line flag.** §12.7 asks for per-call-site suppression and nothing
  else. A global flag is a `redox.toml` question and there is no `redox.toml`.
- **Any other lint.** `RDX1000`+ is a band, and this plan opens it with one code. The next
  lint gets the second number and this plan's machinery.
- **Knowing whether the receiver can actually panic.** That needs types. A user's own
  `unwrap` warns, per §1.5, and the suppression exists for exactly that.
- **Linting `unwrap` inside a `test` block.** Arguably it should be silent there — panicking
  is what a test does — and §12.7 does not say so, so it is not carved out. If it turns out
  to matter the exception belongs in a later lint-configuration row, not hidden here.
- **`panic`, `todo`, `unimplemented` and `unreachable`** are already implemented and are
  **not** linted. They are deliberate statements of intent; `unwrap` is an accident waiting.

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format :audit
```

---

## 8. Emerald reuse

**None.** Emerald had no lint band and no severity — every diagnostic was fatal, which is
the design this plan has to change the first thing about.

---

## 9. Decision log

| # | Decision | Alternative rejected | Why |
|---|---|---|---|
| 1 | Redox lints it, with a Redox code | Delegate to `clippy::unwrap_used` | Clippy's message names neither `??` nor `catch`, and cannot point at the `.sdx`. §2 admits this is the narrowest justification in the project |
| 2 | A `Severity` enum separate from plan 07's level strings | One mechanism for both | A foreign level is text Redox must not interpret; a Redox severity is a closed enum that decides whether the build fails. Merging them would make rustc's output able to change Redox's exit code by spelling |
| 3 | Suppression is a comment pragma | `#[allow(unwrap)]`, consumed or passed through | Passed through, rustc says `unknown lint` and the program stops compiling. Consumed, plan 18 §1.3's attributes-are-trivia rule becomes conditional for one spelling. The comment reaches nothing — §1.3 |
| 4 | The namespace is `redox:` | Bare `-- allow unwrap` | An English comment would become a pragma by accident |
| 5 | Whole-name match, not a prefix | `starts_with("unwrap")` | `unwrap_or` is the *cure*. A prefix match flags it as the disease |
| 6 | The unit is the statement | Per call site exactly | A comment attaches to a statement. §1.4 discloses it; the finer unit needs the attribute §1.3 ruled out |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
