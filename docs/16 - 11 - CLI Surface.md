# Plan 11 — CLI Surface

Plan ID: `cli-surface`
Depends on: 07 (`rustc-remapping`)
Milestone: 3 — *the compiler is fast enough to live in an edit loop*
Status: `done`

---

## 1. What this plan is

Doc 01 §27 fixes Redox's command-line surface at six entries and no more:

> `build`, `check`, `fmt`, `watch`, `--version`, `--help` … Cargo owns build, test, docs
> and dependencies.

This plan makes that a **checked** property rather than an intention. The surface becomes
one declarative table; dispatch, the usage text, and the test all read from it; and a
seventh subcommand cannot be added without the test failing.

### 1.1 Why a surface needs defending in code

A CLI grows by accident. Every individual addition is reasonable — `redox fmt-check`,
`redox explain`, `redox init`, `redox test` — and the sum is a tool that has replaced the
build system doc 01 §1.3 says it must not. Doc 02 §3 records that Emerald's CLI reached
**seventeen** subcommands, eleven of which doc 04's plan 11 row describes as ones *"Redox
declines"*.

Plan 08 already met this pressure once, from a plan document rather than a user: doc 04
proposed a `redox test` entry point, and plan 08 §6.1 rejected it on doc 01 §27 grounds.
That is the system working, and it worked because the rule was written down. This plan
gives it teeth.

### 1.2 The distinction worth implementing

`redox fmt` today is **not** an unknown command. It is a declared part of the surface whose
implementation is plan 34. Those two cases deserve different answers:

```console
$ redox fmt
error: `fmt` is not implemented yet — it is plan 34 (`formatter`)

$ redox frobnicate
error: unknown command `frobnicate`
<usage>
```

The first tells a user the feature is coming and names where to look. The second tells them
they made a mistake. Collapsing both into "unknown command" is the kind of small dishonesty
that makes a tool feel unfinished in a way it is not.

---

## 2. Verified preconditions

| Claim | How checked | Result |
|---|---|---|
| `run` dispatches with a hand-written `match` | `redox-cli/src/run.rs` | A `match` on `args.first()`, with `build` and `check` arms |
| `USAGE` is a hand-written string | `run/usage.rs` | A `const &str`, maintained separately from the `match` |
| The two can already disagree | reading both | Nothing connects them; plan 07 added `check` to the match and to the usage text in separate edits |
| Doc 01 §27's list is six entries | doc 01 §27 | `build`, `check`, `fmt`, `watch`, `--version`, `--help` |

### 2.1 The defect this plan removes

`USAGE` and the dispatch `match` are two independent lists of the same thing. Plan 07 had
to edit both, and nothing would have complained if it had edited only one — a subcommand
that works but is undocumented, or is documented and does not work. Neither is caught by
any test today.

Generating the usage text from the same table dispatch reads removes the possibility
rather than the instance.

---

## 3. Concrete proof this plan targets

1. **The surface is exactly doc 01 §27's six entries**, asserted against a literal list in
   the test. Adding a seventh fails until someone edits the list, which means editing a
   line that says *"doc 01 §27 fixes this"*.
2. **`--help` names every entry**, so a declared command cannot be hidden.
3. **A declared-but-unimplemented command says so, and names its plan** — distinct from an
   unknown command, asserted separately.
4. **Exit codes**: `--help` and `--version` succeed; an unknown command and a failed build
   do not.

---

## 4. Leaves

### Leaf 1 — `leaf-command-table`

`redox-cli/src/run/command.rs` — one `Command` per surface entry, carrying its name, a
one-line summary, and its state (implemented, or planned with the plan that owns it).
`run/commands.rs` holds the table.

### Leaf 2 — `leaf-generated-usage`

`USAGE` becomes a function over the table, so the usage text cannot omit or invent a
command. The hand-written const goes.

### Leaf 3 — `leaf-dispatch`

`run` looks the argument up in the table and dispatches, or reports a planned command, or
reports an unknown one with the usage.

### Leaf 4 — `leaf-proof-and-gate`

`redox-cli/tests/cli_surface.rs` for §3, plus the gate and the docs.

---

## 5. Acceptance criteria

| # | Criterion |
|---|---|
| 1 | The command table is exactly doc 01 §27's list, asserted against a literal |
| 2 | `--help` lists every command in the table |
| 3 | A planned command reports its plan, and exits non-zero |
| 4 | An unknown command reports the usage, and exits non-zero |
| 5 | `--help` and `--version` exit zero and write to stdout |
| 6 | `build` and `check` behave exactly as before |
| 7 | Full gate green |

---

## 6. Out of scope, and what it discloses

| Deferred | To | Note |
|---|---|---|
| `fmt`'s implementation | **plan 34** | This plan makes the command announce itself honestly, not work |
| `watch`'s implementation | **plan 12** | Likewise |
| Argument parsing library (`clap`) | **rejected** | Six commands and three flags. `clap` would be a third dependency to replace forty lines, and doc 02 §5.5 requires a dependency to earn its place |
| Shell completions | — | Generated completions are a `clap` feature and would be the main argument for adopting it. Worth revisiting if the surface ever justifies the dependency, which at six entries it does not |

---

## 7. Gate

```bash
moon run :check :lint :test :cargo-check :cargo-test :ci-format
```

---

## 8. Emerald reuse

**Lifted as a shape, minus most of it:** `crates/emerald-cli/src/main.rs`'s dispatch.
Doc 04's row for this plan is explicit — the shape *"minus eleven subcommands Redox
declines"*. Emerald's CLI is the negative example this plan is designed to prevent: not
because any one of its seventeen commands was wrong, but because nothing in its code said
when to stop.

---

## 9. Decision log

- **One table, three readers.** Dispatch, usage and the test all read it, so the three
  cannot disagree (§2.1). The previous arrangement could already have drifted.
- **A planned command is distinct from an unknown one** (§1.2), and names its plan. The
  user learns whether to wait or to retype.
- **`clap` is rejected**, with the condition under which that would change recorded:
  completions. Six commands do not justify a dependency.
- **The test compares against a literal list, not against the table.** A test that read the
  table would pass for any table. The literal is a second copy *on purpose* — the one place
  in this repo where duplication is the point, because its job is to notice a change.

---

## 10. Result

`done`. 305 tests pass; `fmt`, `clippy -D warnings` and the full gate are green.

The surface, as `redox --help` now prints it:

```text
redox — a MoonScript-inspired surface for Rust

usage:
  redox build      transpile a .sdx to .xrs
  redox check      transpile, then report rustc's errors against the .sdx
  redox fmt        format a .sdx in place (plan 34, `formatter`)
  redox watch      rebuild on change (plan 12, `watcher`)
  redox --version  print the version
  redox --help     print this message

Cargo owns build, test, docs and dependencies (doc 01 §27).
```

Four commands work, two announce their plan, and the last line says where everything else
went. Every line of it is generated from the table dispatch reads.

| # | Criterion | Evidence |
|---|---|---|
| 1 | The table is doc 01 §27's list | `cli_surface.rs`: `the_surface_is_exactly_doc_01_section_27`, against a literal; `commands.rs`: `the_surface_has_six_entries` |
| 2 | `--help` lists every command | same test, both directions — nothing missing, nothing extra |
| 3 | A planned command names its plan, exits non-zero | `a_planned_command_names_its_plan`, for both `fmt` and `watch` |
| 4 | An unknown command reports usage, exits non-zero | `an_unknown_command_is_reported_as_unknown`, asserting it does *not* say "not implemented" |
| 5 | `--help`/`--version` exit zero on stdout | `help_and_version_succeed_on_stdout`, across `--help`, `-h`, `help`, no-args, `--version`, `-V` |
| 6 | `build` and `check` unchanged | `every_ready_command_dispatches`, plus plans 01–10's tests |
| 7 | Full gate green | `moon run :check :lint :test :cargo-check :cargo-test` |

### 10.1 The defect removed, not the instance

`USAGE` was a hand-written `const` and dispatch was a hand-written `match`: two independent
lists of the same six commands. Plan 07 edited both when it added `check`, and nothing
would have complained had it edited only one — leaving either a subcommand that works and
is undocumented, or one that is documented and does not work.

Generating the usage from the table makes both impossible rather than merely absent. Three
readers now share one source: dispatch, the usage text, and `cli_surface.rs`.

The one place duplication is kept **on purpose** is `SURFACE` in the test: a list that read
`COMMANDS` would pass for any table. Its job is to notice a change, so it has to be a
second copy — and the comment above it says so, because an unexplained duplicate is the
thing doc 06 §3 rule 6 otherwise tells you to delete.

### 10.2 Planned is not unknown

`redox fmt` reports:

```text
error: `fmt` is not implemented yet — it is plan 34 (`formatter`)
```

and `redox frobnicate` reports `unknown command` with the usage. Both exit non-zero and the
tests assert each does **not** produce the other's message. The difference is small and it
is the difference between a tool that is unfinished and one that appears broken: the first
tells a user to wait and names where to look, the second tells them they mistyped.

### 10.3 `unreachable!` with a test behind it

`run`'s match ends in `unreachable!("`{other}` is Ready in COMMANDS but has no dispatch
arm")`. A panic rather than a silent `Ok`, because a `Ready` command with no arm is a bug
in Redox and not in the invocation — and `every_ready_command_dispatches` is what turns it
into a test failure rather than something a user discovers.

### 10.4 Disclosures

- **`clap` is rejected, with the condition that would change it.** Six commands and three
  flags do not earn a third dependency. The real argument for `clap` is generated shell
  completions; if the surface ever justifies those, it justifies the dependency, and that
  is recorded in §6 rather than left as a matter of taste.
- **`fmt` and `watch` do not work.** They are declared because doc 01 §27 declares them,
  and they say so. Plans 34 and 12.
- **Argument parsing past the subcommand is still per-command.** `parse_build_args` handles
  `build` and `check` alike, which is correct while they take the same arguments and will
  need splitting when they diverge.
- **The gate failed once on the combined invocation and passed on re-run**, with every task
  passing individually. Five blaze tasks at once on 17 GB available is the memory pressure
  AGENTS.md now documents — not a defect, and worth noting only because the first run's
  output named every crate in the workspace.
