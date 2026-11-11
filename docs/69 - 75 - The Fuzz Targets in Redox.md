# Plan 75 — The Fuzz Targets in Redox

`redox-fuzz`'s 6 files, 124 lines. The smallest crate in the workspace, and
the shortest of row 72's successors.

It is here for two reasons beyond the register: it is the first crate whose
whole purpose is **discarding** a result, and it is the one that found the hole
in the port harness that lets a crate pass every gate while a clean clone
cannot build it.

---

## 1 What it is

| | |
|---|---|
| Source | `crates/redox-fuzz/sdx/**` — 6 files |
| Generated | `crates/redox-fuzz/src/**`, untracked from this commit |
| Tests | 4, unchanged |
| List | `seed/PORTED` gains `redox-fuzz 75` |
| Seed | **not refreshed** — byte-identical, so no version skew |

The seed being unchanged is the first time that has happened, and it is the
rule working rather than an omission: plan 72 leaf 6's version-skew rule asks
for a refresh when the seed can no longer produce the committed output, and
this port changed nothing in the compiler.

---

## 2 What it found

### 2.1 A ported crate's `.sdx` was never required to be in git

The port blessed, clippy was clean, and all five of `port.rs`'s assertions
passed — with the entire `.sdx` tree untracked. The failure surfaced two steps
later, in the clean-clone test:

```
error: crates/redox-fuzz/sdx is missing, so redox-fuzz has no source to
generate from.
```

That is late, and it reads as a bootstrap problem rather than a forgotten
`git add`. It is also the exact **mirror** of a check that already existed:
`bootstrap.rs`'s `the_generated_trees_are_ignored` asserts the generated Rust
is *not* tracked, and nothing asserted the source *is*.

`every_ported_crates_sdx_is_tracked` closes it, and was falsified before being
trusted — un-staging one file reports `crates/redox-fuzz/sdx: 6 files, 5
tracked` and the test fails; restoring it passes.

**A pair of invariants about the same two trees should be written as a pair.**
One of them existed for three plans on its own.

### 2.2 `data` is a keyword, so it cannot name a parameter

All four targets take `data: &[u8]`. `data` opens a declaration, so
`lex_source data =` is `RDX0103: expected a parameter name`.

`r#data` would preserve the spelling and reach the generated Rust as `r#data`.
A private parameter is not worth that, so it is `bytes` — which says more than
`data` did.

### 2.3 Discarding a `#[must_use]` result needs a binding

Three of the four targets assert **totality**: the claim is that the call
returns, not what it returns. The Rust writes `let _ = …`, and `Result` is
`#[must_use]`, so a bare call statement is `unused_must_use`.

`_dropped = …` is the spelling: a named binding whose underscore prefix keeps
the binding itself from warning. Worth recording because every remaining crate
has calls whose results are deliberately ignored, and the obvious Redox
spelling — a bare call — is the one that does not work.

### 2.4 Plan 55 §1.6 against clippy, for the fifth time

`no_effect` and `unused_unit` are the same `()`, named differently by clippy
depending on where it sits. §1.6 requires a `()`-returning body to end with
`()` rather than implying it, deliberately, so that a body ending in a loop or
an assignment cannot become `()` by arriving last.

**Five crates now carry the allow.** That is the point at which it stops being
a per-crate note: a successor row owes the decision either a workspace-level
allow or a change to §1.6. Repeating the same paragraph in a sixth `lib.sdx`
would be the wrong answer to it.

---

## 3 Criteria

| | Criterion | Met |
|---|---|---|
| 1 | All 6 files are Redox | yes |
| 2 | `redox-fuzz` is in `seed/PORTED` and `port.rs` covers it | yes |
| 3 | `crates/redox-fuzz/src/` is gitignored and out of the index | yes |
| 4 | `crates/redox-fuzz/sdx/` **is** in the index | yes — and now asserted |
| 5 | Every test survives | yes — 4 before, 4 after |
| 6 | `moon run :check :lint :test :sdx-format-check` green | yes |
| 7 | A clean clone bootstraps it | yes |
| 8 | `stage1 == stage2` still holds | yes |

---

## 4 Next

Seven crates, roughly 20,900 lines: `redox-bench` (597), `redox-mcp` (808),
`redox-lsp` (934), `redox-cli` (1,376), `redox-driver` (2,981), `redox-emit`
(6,156), `redox-parser` (7,974).

`tools/port/port.sh` and `tools/port/write.py` are generic over the crate now, where plan
73 and plan 74 each had their own copy — and three of the four lessons in
`port.sh`'s header had already been learned twice when it was written.
