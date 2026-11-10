# Plan 49 — Editor Integrations

Plan ID: `editor-integrations`
Depends on: 40 (`lsp`), 41 (`tree-sitter-grammar`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

Doc 02 §5.4 puts `editors/` in the layout and §9 names the clients Emerald shipped —
VSCode, Neovim, Helix, Zed. Plan 41 was narrowed on 2026-11-08 to hand this row the editor
configs it had been carrying, and the register added Vim and Cursor. So the row is five
clients, each independently installable, each CI-checked.

Two rows already shipped the parts that do the work. Plan 41 committed an ABI 15 tree-sitter
parser with three queries; plan 40 committed `redox-lsp`, a server that is *editor-agnostic
by construction*. **This row contributes no analysis and no grammar.** It contributes the
wiring, and its whole difficulty is in the word *CI-checked*.

### 1.1 Verified preconditions, because every claim here is about an editor

Plan 41 §1.1 spiked its toolchain before writing a line, on the grounds that a plan about
tooling is a plan about things it does not control. This row is further out still — it is a
plan about five programs, none of which this repository builds. Measured 2026-11-10 on the
development machine:

| Claim | Verdict |
|---|---|
| `nvim` present | **0.12.6** |
| nvim's tree-sitter ABI | **15 supported, 13 minimum** — plan 41 generated ABI 15 |
| `cc` builds plan 41's parser to a shared object | yes, `parser.c` + `scanner.c`, no warnings |
| nvim loads it and parses `examples/hello.sdx` | yes, `source_file`, `has_error: false` |
| nvim compiles all three of plan 41's queries | yes — `highlights`, `injections`, `locals` |
| the highlights query does real work | **437 captures, 13 distinct** on `examples/fallible.sdx` |
| nvim runs a legacy vimscript syntax file | yes — `synIDattr` reports `sdxKeyword` |
| `cursor` present | **3.25.9** |
| `cursor --install-extension <path.vsix> --extensions-dir <dir>` | **exit 0**, `--list-extensions` then reports the id |
| `vim`, `code`, `hx`, `zed` present | **no — none of the four** |
| `@vscode/vsce` available | **no**, and not fetchable: `npx` refused with *"missing packages and no YES option"* |
| `zip` present | **no** |
| Python's `zipfile` and `tomllib` | both present |

Two of those invert the register's expectations, and both change the plan.

### 1.2 The row expected VSCode to be the verified one. It is the opposite

The register says Cursor *"consumes the VSCode extension unchanged, and that is **verified by
installing it, not assumed**"* — written as though VSCode were the solid half of the pair and
Cursor the doubtful one needing proof.

**There is no `code` on this machine and there is a `cursor`.** So the verified client is
Cursor and the assumed one is VSCode, exactly backwards from the cell. The extension is one
artifact either way — VSCode contribution points, which is what Cursor consumes — so nothing
about the deliverable changes. What changes is which half of the sentence is evidence:
Cursor's install is checked, and VSCode's is a claim resting on Cursor being a VSCode fork.

§5 criterion 4 states it that way round, and §6 declines to pretend otherwise.

### 1.3 A VSIX is a zip, so the packaging dependency is declined

`vsce` is the usual way to package a VSCode extension, it is absent, and `npx` will not fetch
it. Adding it would put an npm network dependency in the gate for a build step that produces
a zip.

A VSIX is an OPC package: `[Content_Types].xml`, `extension.vsixmanifest`, and the extension
under `extension/`. Spiked before this plan was written — a **1,094-byte** VSIX built with
Python's `zipfile` and nothing else installed into Cursor and listed as `redox.redox-spike`.

So leaf 3 builds the VSIX from the stdlib. The cost is disclosed rather than hidden: a
hand-built manifest does not get `vsce`'s validation, so a mistake in it shows up as an
install failure rather than a packaging error — which is acceptable precisely because the
gate *performs the install*.

### 1.4 The verification ladder, stated per client rather than averaged

This is the row's shaping decision. *CI-checked* cannot mean the same thing for five clients
when two of them are not installed, and a gate that reports one word for all five would be
reporting the weakest as though it were the strongest. Plan 48 §10.2 took the same line about
`syn` versus rustc: use the best oracle available and say which one it is.

| Client | Oracle | Strength |
|---|---|---|
| **Neovim** | nvim itself, headless: loads the parser, parses every example with no `ERROR` node, compiles all three queries, and the LSP config's command resolves | **the editor runs it** |
| **VSCode / Cursor** | `cursor --install-extension` into a throwaway `--extensions-dir`, then `--list-extensions` | **the editor installs it** (Cursor; VSCode by inheritance — §1.2) |
| **Vim** | nvim sourcing the legacy `syntax/sdx.vim` and reporting `synID` at a known keyword | **a compatible editor runs it** — not vim |
| **Helix** | `languages.toml` parses, and every key is one Helix documents | **the file is well-formed** — not accepted by Helix |
| **Zed** | `extension.toml` parses, and the grammar/language stanzas name files that exist | **the file is well-formed** — not accepted by Zed |

The bottom three rows are weaker and the test names say so, which is the point: a test called
`helix_config_parses` cannot be misread as *Helix works*.

### 1.5 Install steps live beside each client

The register asks for this explicitly — *"not buried in a README"*. One `editors/<client>/README.md`
each, holding the install for that client and nothing about the others. The top-level
`editors/README.md` is an index of five links and the verification ladder above, so a reader
learns what is checked before they trust it.

### 1.6 Authoring this plan made plan 47's counter drop a row

Worth recording where it happened rather than in §10, because it is a finding about the
**register** and not about editors.

Writing this row's cell made `docs/STATUS.md` read **49 rows**, one short. Plan 47's
`plan_counts` requires every register row to carry a backticked status — deliberately, because
without that requirement it counted doc 04's `Plan | Document` index as four extra rows and
totalled 54 for a register of 50 (plan 47 §10). So a row whose cell opens *"**Plan authored
2026-11-10**…"* and carries no marker is skipped silently.

The register had **no spelling for authored-but-not-implemented**. Rows 40, 41, 42 and 44 all
read the same way and are only countable because they later gained a trailing `` `done` ``.
So the row now opens `` `planned` ``, and the total is 50 again.

The pattern is plan 47 §10's twice over: the counter over-counted, was tightened, and now
under-counts a state nobody had written down. Both directions were caught the same way — by a
generated document whose number is compared on every build, which is the argument for
generating it at all.

---

## 2. Admission tests (doc 03 §2)

Not applicable — this row adds no surface syntax. Nothing in `editors/` is compiled into the
`redox` binary or affects what `.sdx` means.

---

## 3. Concrete proof this plan targets

1. `nvim --headless` loads plan 41's parser and parses **every** `examples/*.sdx` with no
   `ERROR` node, using the committed queries, from the committed `editors/neovim/` runtime
   path.
2. The same nvim run compiles `highlights.scm`, `injections.scm` and `locals.scm`, and the
   highlights query produces a non-zero capture count on a real example.
3. A VSIX built from `editors/vscode/` installs into Cursor via
   `--install-extension` with a throwaway `--extensions-dir`, and `--list-extensions`
   reports its identifier. The user's own extension directory is never touched.
4. `editors/vim/syntax/sdx.vim` colours a keyword — `synIDattr` returns the expected group —
   under nvim, which is a stand-in for vim and is labelled one.
5. `editors/helix/languages.toml` and `editors/zed/extension.toml` parse, and every key in
   them is on an allow-list taken from each editor's documented schema, so a typo fails
   rather than being silently ignored.
6. Every path a config names — the parser source, the query files, the `redox-lsp` binary
   name, the formatter command — exists or is a target this workspace builds. A config
   pointing at a moved file is the failure mode these clients actually have.
7. Each client directory has its own README, and `editors/README.md` links all five.

---

## 4. Leaves

### Leaf 1 — `leaf-editors-harness`

The test harness and the directory, before any client. `crates/redox-cli/tests/editors.rs`
plus `editors/README.md`. The harness owns: building plan 41's parser to a shared object in
a temp directory, driving `nvim --headless` with a generated Lua probe, and the path-existence
sweep. Skips with a reason when a tool is absent rather than failing, since the gate must pass
on a machine with no editors — and **reports** the skip, because a silently skipped check is
worse than a missing one (plan 46 §1.3).

### Leaf 2 — `leaf-neovim-and-vim`

`editors/neovim/` — filetype detection for `.sdx` and `.xrs`, the parser registered, the three
queries, `vim.lsp` wired to `redox-lsp`, and `redox fmt` as the formatter. `editors/vim/` —
`syntax/sdx.vim` and `ftplugin/sdx.vim`, regex only, no tree-sitter and no LSP. Criteria 1, 2
and 4.

### Leaf 3 — `leaf-vscode-cursor`

`editors/vscode/` — one extension with a TextMate grammar for a cold open, an LSP client over
`vscode-languageclient`, and `redox fmt` as the document formatter. The stdlib VSIX builder,
and the install-into-Cursor test. Criterion 3.

### Leaf 4 — `leaf-helix-zed`

`editors/helix/languages.toml` and `editors/zed/extension.toml`, with the schema allow-list
test. Criterion 5.

---

## 5. Acceptance criteria

1. **Neovim is verified by Neovim.** The parser loads, every example parses with no `ERROR`
   node, and all three queries compile — asserted from the committed config, not from a
   probe written for the test.
2. **The highlights query is asserted to match, not merely to compile.** A non-zero capture
   count on a real example. Plan 41's gate checked only that the grammar parsed; a query that
   compiles and captures nothing looks identical from Rust.
3. **The extension installs.** Built with no npm dependency, installed into Cursor, listed
   back by id, and into a throwaway extensions directory.
4. **The VSCode claim is stated as inherited, not measured.** §1.2's inversion is in the
   README and in the test's own name, so nobody reads a green gate as *VSCode verified*.
5. **Helix and Zed are checked as far as they can be, and labelled.** Parse plus a schema
   allow-list; test names say `_parses`, never `_works`.
6. **No config names a path that does not exist.** Swept, including the query files and the
   binary names.
7. **Every skip is reported.** A machine without `cursor` passes the gate and says which
   check did not run.
8. **The user's editor state is never modified.** Every editor invocation is given an
   explicit throwaway directory.

---

## 6. Out of scope, and what it discloses

- **Publishing.** No extension is pushed to a marketplace, no plugin to a registry. The row
  ships directories that install from a checkout.
- **`vim` itself, `code` itself, `hx` and `zed`.** Not installed, so not verified. §1.4 is the
  disclosure and each test's name carries it. Installing four editors to verify five
  directories is a cost this row declines; what it will not do is call the result verified.
- **LSP features beyond plan 40's four.** Completion, hover and go-to-definition were
  declined by plan 40 for want of name resolution, and a client cannot add them.
- **`nvim-treesitter` as a dependency.** The register names it, and the config instead uses
  `vim.treesitter.language.add` directly, which is core since nvim 0.9 and needs no plugin
  manager. A plugin dependency would make the gate depend on a network fetch, and §1.3 already
  declined one of those.
- **Parser distribution.** Each client is told how to build the parser from
  `tree-sitter-redox/`; no `.so`, `.wasm` or generated binary is committed. Committed binaries
  for five platforms is a release problem, not an editor-integration problem.

---

## 7. Gate

`moon run :format :check :lint :build :test :audit`, plus the new `editors.rs` integration
test inside `:test`. The editor binaries are discovered rather than required, so the gate's
verdict on a machine with none of them is *pass, with skips reported*.

---

## 8. Emerald reuse

Emerald's VSCode, Neovim, Helix and Zed configs give the layout, and doc 02 §9 cites them.
What they do not give is a check: Emerald's editor configs were not tested by any editor, so
the reuse is the *shape* of the directories and none of the confidence. Vim and Cursor are
Redox additions with no precedent to copy.

---

## 9. Decision log

| Decision | Why |
|---|---|
| A ladder of verification strengths, named per client | Five clients, two editors installed. One word for all five would report the weakest as the strongest (§1.4) |
| VSIX built from Python's `zipfile` | `vsce` is absent and unfetchable; a VSIX is a zip; the gate installs the result, which is the validation that matters (§1.3) |
| The VSCode claim is inherited from Cursor, not measured | There is no `code` here and there is a `cursor`. The register had it the other way round (§1.2) |
| `vim.treesitter.language.add`, not `nvim-treesitter` | Core since nvim 0.9; a plugin dependency makes the gate fetch from the network |
| nvim stands in for vim, and is labelled a stand-in | nvim runs legacy vimscript syntax — verified, `synIDattr` returns `sdxKeyword`. It is still not vim |
| Skips are reported, not silent | A check that quietly does not run is indistinguishable from one that passed (plan 46 §1.3) |
| No parser binary committed | Five-platform binary distribution is a release concern; the configs build from source |
| One README per client | The register asks for it; a combined README is where install steps go to be skimmed past |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
