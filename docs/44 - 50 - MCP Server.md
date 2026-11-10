# Plan 50 — MCP Server

Plan ID: `mcp-server`
Depends on: 05 (`diagnostics`), 06 (`source-maps`), 11 (`build-script-integration`)
Milestone: 7 — *tooling*
Status: `planned`

---

## 1. What this plan is

The last row of the register. Doc 02 §5.5 puts `redox-mcp/` in the layout and §9.3 argues for
it: the driver exposed over the Model Context Protocol, so an agent working in a Redox
codebase gets the compiler's answers as data rather than as scraped terminal output.

Four tools, from the register: `redox_check`, `redox_build`, `redox_fmt`, and a source-map
lookup answering *which `.sdx` line is this `.xrs` line*. Stdio transport only, no network
listener. A `.mcp.json` entry so this repository's own agents get it.

### 1.1 Verified preconditions

Measured 2026-11-10, and three of them change the plan. Plan 40 §10.1 is the reason this
section exists in this form: that row measured crate *versions* and not *names*, and still
shipped a wrong claim about `Url` versus `Uri`.

| Claim | Verdict |
|---|---|
| `rmcp` resolves | **3.9.0** |
| `schemars` resolves | **1.2.3** |
| `tokio` resolves | **1.54.1** |
| stdio transport is a default feature | **no** — `rmcp::transport::stdio()` needs `transport-io`, and the defaults are `base64, macros, schemars, server, transport-async-rw, uuid` |
| `rmcp::model::ServerInfo` is the handler's return type | **no, deprecated** in 3.9 — it is `ServerConfig`, and it is `#[non_exhaustive]`, so a struct expression is rejected (E0639) |
| structured JSON is expressible | yes — `CallToolResult::structured(serde_json::Value)` |
| the macro surface the row needs | `#[tool_router]`, `#[tool_handler]`, `#[tool]`, `Parameters<T>`, `ToolRouter<Self>` all compile |
| `redox_diag::SourceMap::resolve(line, col)` exists and is public | **yes** — the source-map tool is a one-line call |
| `redox_fmt::format` is public | yes |
| `redox_driver` exposes `compile_dir`, `diagnose`, `lint`, `remap`, `RustcMessage` | yes |
| `redox_diag::Diagnostic` derives `Serialize` | **no**, and `redox-diag` depends only on `miette` |
| the rustc pass is reachable outside `redox-cli` | **no** — `check_command`, `rustc_path` and `ScratchDir` are all `pub(crate)` |
| `.mcp.json` is committed | yes, **15 servers**, none of them `redox` |

Spiked before the plan was written, as plan 41 §1.1 and plan 44 §1.1 did: a throwaway crate
outside the workspace compiling one tool with a typed parameter, a typed result and stdio
transport. Two of the rows above are findings *from* that spike rather than from a version
number, which is the whole argument for spiking.

### 1.2 The rustc pass is already duplicated once, and this row would be the third

The decisive precondition. `redox check` runs rustc inside `redox-cli`:
`check_command` is `pub(crate)`, and so are `rustc_path` and `ScratchDir`. So `redox_check`
as an MCP tool cannot reach it.

This is plan 40 leaf 1's situation exactly — *a `pub(crate)` thing is right until a second
reader appears, and a copy would make the two disagree while both compiling* — except that
**the copy has already been made**. `crates/redox-bench/build.rs` bakes `REDOX_RUSTC` the same
way `crates/redox-cli/build.rs` does, and the two differ **only in their doc comments**:
`main`, `resolve_rustc` and `is_real_rustc` are identical code in both.

A third copy for `redox-mcp` is where that stops being tolerable. So **leaf 1 moves the rustc
pass into `redox-driver`** — including the build script, because `env!("REDOX_RUSTC")`
resolves in whichever crate carries the script, and AGENTS.md's gotcha is that neither obvious
source of a rustc path is trustworthy here. `redox-cli` and `redox-bench` then read one baked
path from one verified resolution, and `redox-mcp` is a third reader rather than a third
implementation.

Done first and as its own commit, before any MCP code exists, so the move is reviewable as a
move.

### 1.3 The JSON is a projection of `Diagnostic`, not a serialization of it

`Diagnostic` has no `Serialize` and should not get one. Two of its fields make that the wrong
shape for a wire format:

- **`source: String` is the whole file.** Serialized, it would put the complete contents of
  every checked file into every response — for a client that already has the file open.
- **`span` is byte offsets.** Correct for `miette`, useless to an agent, which quotes line and
  column.

So `redox-mcp` defines its own wire type and converts. That also respects doc 06 §6 rule 2 —
*only the CLI renders* — because an MCP tool is a second renderer and its shape is its own
business. `path`, `code.text()`, `severity`, `message`, `help` and a `(line, column)` carry
over; `source` does not.

**Line and column are 1-based characters, not UTF-16 code units.** `redox-lsp` has
`position_of`, and it is deliberately *not* reused: it returns an `lsp_types::Position` whose
`character` is a UTF-16 offset because the protocol demands it. An agent reading a file counts
characters. Reusing it would be a type and a unit away from right, which is worse than not
reusing it — §1.2's argument does not generalise to a helper whose semantics differ.

### 1.4 The rustc pass is out of scope for the LSP and in scope here, and the reason is the transport

Plan 40 §6 put the rustc pass out of the language server's scope: cancellation and debounce
are a larger design than the rest of that row, because a server runs on every keystroke and
must abandon work the user has already invalidated.

**MCP is request/response.** A tool call is one explicit request with no successor to race,
nothing to debounce and nothing to cancel. So the thing plan 40 declined is the thing this row
can do cheaply, and `redox_check` means what the CLI's `check` means: transpile, run rustc,
report rustc's errors against the `.sdx`.

Worth stating because it looks like an inconsistency between two rows and is in fact the same
judgement applied to two transports.

### 1.5 The `.mcp.json` entry is additive, and that is a constraint

`.mcp.json` is committed and holds **15** servers. The entry for `redox` must be added
without touching the other fifteen.

This is written as a constraint because it was already got wrong once in this repository: a
script that round-tripped `.mcp.json` through a JSON serializer rewrote the **whole file** —
136 insertions and 112 deletions for an edit that should have been seven lines. A formatter
applied to a config nobody asked to reformat is a diff nobody can review.

So the edit is a textual insertion, and a test asserts the other servers survive it.

---

## 2. Admission tests (doc 03 §2)

Not applicable — no surface syntax. `redox-mcp` adds a binary, not a construct, and nothing
about `.sdx` changes.

---

## 3. Concrete proof this plan targets

1. The rustc pass lives in `redox-driver`, `redox-cli` and `redox-bench` both use it, and
   neither keeps a copy of `resolve_rustc`. `redox check`'s own UI corpus is unchanged by the
   move, which is what proves it was a move.
2. `redox_check` on a `.sdx` with a type error returns **structured JSON** carrying the
   rustc code, the message and a `.sdx` line and column — asserted against the JSON, never
   against rendered text.
3. `redox_check` on a clean `.sdx` returns an empty diagnostic list, not an error.
4. `redox_build` writes the `.xrs` and names it; `redox_fmt` returns the formatted text and
   does not write unless asked.
5. `redox_fmt` on a file that does not parse returns the parse diagnostic and **changes
   nothing** — plan 34 criterion 8's property, through a second interface.
6. The source-map tool answers *which `.sdx` line is this `.xrs` line* from plan 06's map,
   including the honest answer for a **synthetic** line that maps to nothing.
7. Every tool is reachable over a real **in-process MCP session** — the tool list and a call
   each, as plan 40's tests do over an in-process `Connection` rather than against a mock.
8. `.mcp.json` gains `redox` and keeps all 15 existing servers, byte for byte.
9. No network transport is compiled in: the dependency list has no HTTP server feature, and a
   test asserts the manifest does not enable one.

---

## 4. Leaves

### Leaf 1 — `leaf-driver-owns-rustc`

Move the rustc check pass and its build script from `redox-cli` to `redox-driver`. Delete
`redox-bench`'s duplicate. `redox-cli`'s `check_command` becomes the renderer it should always
have been. Criterion 1, and nothing of this row's own ships in this commit.

### Leaf 2 — `leaf-mcp-server`

`crates/redox-mcp/`: the server, stdio transport, and `redox_check`, `redox_build`,
`redox_fmt`. The wire types and the `Diagnostic` projection. Criteria 2–5 and 7.

### Leaf 3 — `leaf-source-map-tool`

`redox_source_map`, over `SourceMap::resolve`. Criterion 6.

### Leaf 4 — `leaf-mcp-json`

The `.mcp.json` entry, additive, with the test that the other fifteen survive. Criteria 8
and 9.

---

## 5. Acceptance criteria

1. **Leaf 1 is a move, not a copy, and removes an existing copy.** After it, one resolution of
   `rustc` exists in the workspace.
2. **Every tool's result is asserted as JSON.** Not as a rendered string. The register's
   *"structured JSON, never scraped stderr"* is a claim a test has to make.
3. **Diagnostics carry 1-based line and column in characters**, and no file contents.
4. **`redox_fmt` cannot destroy work.** A file that does not parse comes back with its
   diagnostic and is not rewritten.
5. **The source map's gaps are reported, not hidden.** A synthetic `.xrs` line answers *no
   `.sdx` line*, distinctly from an out-of-range one.
6. **Tested over a real session.** An in-process client, a `tools/list` and a call per tool.
7. **Stdio only.** No HTTP or SSE feature enabled, asserted on the manifest.
8. **`.mcp.json`'s other fifteen servers are untouched.**

---

## 6. Out of scope, and what it discloses

- **Any network transport.** Stdio only, per the register. `rmcp`'s HTTP and SSE features stay
  off, and criterion 7 asserts it rather than trusting it. A compiler tool that opens a port
  is a decision for a row that says so.
- **MCP prompts and resources.** Tools only. Resources would mean deciding what a Redox
  project's resource tree is, which is a design and not a wiring job.
- **An MCP client.** `rmcp`'s `client` feature stays off. Nothing here consumes MCP.
- **Cancellation and progress.** §1.4's reasoning cuts both ways: request/response means no
  cancellation is *needed*, and it also means long calls block. A `redox_check` over a large
  project will take as long as rustc does, and that is disclosed rather than designed around.
- **Authentication.** `rmcp`'s `auth` features stay off. Stdio's trust boundary is the process
  that spawned it.
- **A `redox_lint` tool.** `redox_driver::lint` exists and plan 38's lints are real, but
  `redox_check` already reports them as diagnostics with lint severity; a second tool would be
  a second answer to one question.

---

## 7. Gate

`moon run :format :check :lint :build :test :audit`, with `redox-mcp` in the workspace so its
tests are in `:test`.

One measured risk for leaf 2: the spike's `tool_router` field raised `field is never read`
under `#[warn(dead_code)]`, and this repository's `:lint` is `clippy -D warnings`. If
`#[tool_handler]`'s generated code does not read the field, it needs an `allow` **with a
reason naming the macro** — not a bare allow, and not a change to the lint level.

---

## 8. Emerald reuse

`crates/emerald-mcp` is **392 lines** and doc 02 §9.3 cites it for the shape: the driver
exposed over MCP, tools that return the compiler's own structures. What transfers is that
shape. What does not is the protocol layer — Emerald predates `rmcp` 3.x — so §1.1's spike was
the real preparation and Emerald's contribution is the decision that the row is worth doing.

---

## 9. Decision log

| Decision | Why |
|---|---|
| Leaf 1 moves the rustc pass **and its build script** to `redox-driver` | `env!("REDOX_RUSTC")` resolves in the crate carrying the script, and AGENTS.md's gotcha is that no unverified rustc path is trustworthy. One resolution, three readers |
| `redox-bench`'s duplicate is deleted in the same leaf | It is identical code differing only in doc comments; leaving it would make leaf 1 a second copy rather than a consolidation (§1.2) |
| The wire type is a projection, not `#[derive(Serialize)]` on `Diagnostic` | `source` is the whole file and `span` is byte offsets. Serializing the type would ship both (§1.3) |
| 1-based characters, not `lsp_types::Position` | `position_of`'s `character` is UTF-16 because LSP demands it. Reusing it would be a unit away from right (§1.3) |
| `redox_check` runs rustc, unlike the language server | MCP is request/response, so plan 40 §6's cancellation problem does not exist here (§1.4) |
| Stdio only, with a test on the manifest | The register says so, and an unasserted claim about a network listener is the kind that stops being true quietly |
| `.mcp.json` edited textually, not round-tripped | A serializer rewrote the whole file once in this repository — 136 insertions for a seven-line edit (§1.5) |
| No `redox_lint` tool | `redox_check` already returns lint-severity diagnostics; two answers to one question is worse than one |
| Tested over an in-process session | Plan 40's choice, for plan 40's reason: a mock tests the mock |

---

## 10. Result

*Filled in when the plan ships, including anything above that turned out wrong.*
