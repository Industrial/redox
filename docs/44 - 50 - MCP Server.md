# Plan 50 — MCP Server

Plan ID: `mcp-server`
Depends on: 05 (`diagnostics`), 06 (`source-maps`), 11 (`build-script-integration`)
Milestone: 7 — *tooling*
Status: `done` 2026-11-10

---

## 1. What this plan is

The last row of the register. Doc 02 §5.5 puts `voltaic-mcp/` in the layout and §9.3 argues for
it: the driver exposed over the Model Context Protocol, so an agent working in a Voltaic
codebase gets the compiler's answers as data rather than as scraped terminal output.

Four tools, from the register: `voltaic_check`, `voltaic_build`, `voltaic_fmt`, and a source-map
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
| `voltaic_diag::SourceMap::resolve(line, col)` exists and is public | **yes** — the source-map tool is a one-line call |
| `voltaic_fmt::format` is public | yes |
| `voltaic_driver` exposes `compile_dir`, `diagnose`, `lint`, `remap`, `RustcMessage` | yes |
| `voltaic_diag::Diagnostic` derives `Serialize` | **no**, and `voltaic-diag` depends only on `miette` |
| the rustc pass is reachable outside `voltaic-cli` | **no** — `check_command`, `rustc_path` and `ScratchDir` are all `pub(crate)` |
| `.mcp.json` is committed | yes, **15 servers**, none of them `voltaic` |

Spiked before the plan was written, as plan 41 §1.1 and plan 44 §1.1 did: a throwaway crate
outside the workspace compiling one tool with a typed parameter, a typed result and stdio
transport. Two of the rows above are findings *from* that spike rather than from a version
number, which is the whole argument for spiking.

### 1.2 The rustc pass is already duplicated once, and this row would be the third

The decisive precondition. `voltaic check` runs rustc inside `voltaic-cli`:
`check_command` is `pub(crate)`, and so are `rustc_path` and `ScratchDir`. So `voltaic_check`
as an MCP tool cannot reach it.

This is plan 40 leaf 1's situation exactly — *a `pub(crate)` thing is right until a second
reader appears, and a copy would make the two disagree while both compiling* — except that
**the copy has already been made**. `crates/voltaic-bench/build.rs` bakes `VOLTAIC_RUSTC` the same
way `crates/voltaic-cli/build.rs` does, and the two differ **only in their doc comments**:
`main`, `resolve_rustc` and `is_real_rustc` are identical code in both.

A third copy for `voltaic-mcp` is where that stops being tolerable. So **leaf 1 moves the rustc
pass into `voltaic-driver`** — including the build script, because `env!("VOLTAIC_RUSTC")`
resolves in whichever crate carries the script, and AGENTS.md's gotcha is that neither obvious
source of a rustc path is trustworthy here. `voltaic-cli` and `voltaic-bench` then read one baked
path from one verified resolution, and `voltaic-mcp` is a third reader rather than a third
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

So `voltaic-mcp` defines its own wire type and converts. That also respects doc 06 §6 rule 2 —
*only the CLI renders* — because an MCP tool is a second renderer and its shape is its own
business. `path`, `code.text()`, `severity`, `message`, `help` and a `(line, column)` carry
over; `source` does not.

**Line and column are 1-based characters, not UTF-16 code units.** `voltaic-lsp` has
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
can do cheaply, and `voltaic_check` means what the CLI's `check` means: transpile, run rustc,
report rustc's errors against the `.sdx`.

Worth stating because it looks like an inconsistency between two rows and is in fact the same
judgement applied to two transports.

### 1.5 The `.mcp.json` entry is additive, and that is a constraint

`.mcp.json` is committed and holds **15** servers. The entry for `voltaic` must be added
without touching the other fifteen.

This is written as a constraint because it was already got wrong once in this repository: a
script that round-tripped `.mcp.json` through a JSON serializer rewrote the **whole file** —
136 insertions and 112 deletions for an edit that should have been seven lines. A formatter
applied to a config nobody asked to reformat is a diff nobody can review.

So the edit is a textual insertion, and a test asserts the other servers survive it.

---

## 2. Admission tests (doc 03 §2)

Not applicable — no surface syntax. `voltaic-mcp` adds a binary, not a construct, and nothing
about `.sdx` changes.

---

## 3. Concrete proof this plan targets

1. The rustc pass lives in `voltaic-driver`, `voltaic-cli` and `voltaic-bench` both use it, and
   neither keeps a copy of `resolve_rustc`. `voltaic check`'s own UI corpus is unchanged by the
   move, which is what proves it was a move.
2. `voltaic_check` on a `.sdx` with a type error returns **structured JSON** carrying the
   rustc code, the message and a `.sdx` line and column — asserted against the JSON, never
   against rendered text.
3. `voltaic_check` on a clean `.sdx` returns an empty diagnostic list, not an error.
4. `voltaic_build` writes the `.xrs` and names it; `voltaic_fmt` returns the formatted text and
   does not write unless asked.
5. `voltaic_fmt` on a file that does not parse returns the parse diagnostic and **changes
   nothing** — plan 34 criterion 8's property, through a second interface.
6. The source-map tool answers *which `.sdx` line is this `.xrs` line* from plan 06's map,
   including the honest answer for a **synthetic** line that maps to nothing.
7. Every tool is reachable over a real **in-process MCP session** — the tool list and a call
   each, as plan 40's tests do over an in-process `Connection` rather than against a mock.
8. `.mcp.json` gains `voltaic` and keeps all 15 existing servers, byte for byte.
9. No network transport is compiled in: the dependency list has no HTTP server feature, and a
   test asserts the manifest does not enable one.

---

## 4. Leaves

### Leaf 1 — `leaf-driver-owns-rustc`

Move the rustc check pass and its build script from `voltaic-cli` to `voltaic-driver`. Delete
`voltaic-bench`'s duplicate. `voltaic-cli`'s `check_command` becomes the renderer it should always
have been. Criterion 1, and nothing of this row's own ships in this commit.

### Leaf 2 — `leaf-mcp-server`

`crates/voltaic-mcp/`: the server, stdio transport, and `voltaic_check`, `voltaic_build`,
`voltaic_fmt`. The wire types and the `Diagnostic` projection. Criteria 2–5 and 7.

### Leaf 3 — `leaf-source-map-tool`

`voltaic_source_map`, over `SourceMap::resolve`. Criterion 6.

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
4. **`voltaic_fmt` cannot destroy work.** A file that does not parse comes back with its
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
- **MCP prompts and resources.** Tools only. Resources would mean deciding what a Voltaic
  project's resource tree is, which is a design and not a wiring job.
- **An MCP client.** `rmcp`'s `client` feature stays off. Nothing here consumes MCP.
- **Cancellation and progress.** §1.4's reasoning cuts both ways: request/response means no
  cancellation is *needed*, and it also means long calls block. A `voltaic_check` over a large
  project will take as long as rustc does, and that is disclosed rather than designed around.
- **Authentication.** `rmcp`'s `auth` features stay off. Stdio's trust boundary is the process
  that spawned it.
- **A `voltaic_lint` tool.** `voltaic_driver::lint` exists and plan 38's lints are real, but
  `voltaic_check` already reports them as diagnostics with lint severity; a second tool would be
  a second answer to one question.

---

## 7. Gate

`moon run :format :check :lint :build :test :audit`, with `voltaic-mcp` in the workspace so its
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
| Leaf 1 moves the rustc pass **and its build script** to `voltaic-driver` | `env!("VOLTAIC_RUSTC")` resolves in the crate carrying the script, and AGENTS.md's gotcha is that no unverified rustc path is trustworthy. One resolution, three readers |
| `voltaic-bench`'s duplicate is deleted in the same leaf | It is identical code differing only in doc comments; leaving it would make leaf 1 a second copy rather than a consolidation (§1.2) |
| The wire type is a projection, not `#[derive(Serialize)]` on `Diagnostic` | `source` is the whole file and `span` is byte offsets. Serializing the type would ship both (§1.3) |
| 1-based characters, not `lsp_types::Position` | `position_of`'s `character` is UTF-16 because LSP demands it. Reusing it would be a unit away from right (§1.3) |
| `voltaic_check` runs rustc, unlike the language server | MCP is request/response, so plan 40 §6's cancellation problem does not exist here (§1.4) |
| Stdio only, with a test on the manifest | The register says so, and an unasserted claim about a network listener is the kind that stops being true quietly |
| `.mcp.json` edited textually, not round-tripped | A serializer rewrote the whole file once in this repository — 136 insertions for a seven-line edit (§1.5) |
| No `voltaic_lint` tool | `voltaic_check` already returns lint-severity diagnostics; two answers to one question is worse than one |
| Tested over an in-process session | Plan 40's choice, for plan 40's reason: a mock tests the mock |

---

## 10. Result

Shipped 2026-11-10. `crates/voltaic-mcp` — four tools, stdio only, **22 tests** passing — plus
leaf 1's consolidation in `voltaic-driver` and the `.mcp.json` entry. The register is complete.

### 10.1 §1.1's spike earned its keep twice, and then a third time in the tests

The two preconditions that came from compiling rather than from a version number were both
real, and a third of the same kind appeared only once the tests were written:

1. stdio transport is `transport-io`, **not** a default feature;
2. `ServerInfo` is deprecated for `ServerConfig`, which is `#[non_exhaustive]` — so E0639, and
   the fields go on a `Default`;
3. **`CallToolRequestParams` is `#[non_exhaustive]` too.** Once is a quirk of one type; twice
   is `rmcp`'s convention, so `session.rs` has one `call` helper rather than four
   workarounds.

Plan 40 §10.1's lesson — *that row measured versions and not names* — held up as the reason
§1.1 exists in that form. None of these three is discoverable from a version number, and all
three would have been discovered anyway, later, as compile errors in a half-written crate.

### 10.2 §7's anticipated `allow` was the wrong fix, because the warning was right

§7 recorded a measured risk: the spike's `tool_router` field raised `field is never read`, and
`:lint` is `clippy -D warnings`. It said the field *"needs an `allow` with a reason naming the
macro"*.

It does not. `#[tool_handler]` resolves the router through `Self::tool_router()` —
**statically** — and never reads a field, so the warning is correct and the field is dead
state that `rmcp`'s own examples carry. `Server` is a unit struct and there is no `allow`.

Worth recording as a near-miss of a particular kind: the plan had already decided the fix
before checking whether the diagnostic was right. An `allow` would have silenced a true
statement, permanently, with a comment explaining why the compiler was wrong.

### 10.3 Both error fixtures failed for the wrong reason, and only probing caught it

Written, then checked before any assertion was made against them. Both were wrong:

- `type_error.sdx` was `let n :: u32 = "not a number"`, which comes back **`RDX0101`** —
  Voltaic's own code. The whole point of `voltaic_check` is that *rustc* is the authority
  (AGENTS.md: every check rustc can make, rustc makes), so a fixture carrying an `RDX` code
  would not have exercised the tool at all while looking exactly like it did. Replaced with a
  wrong-typed argument, which is **`E0308`**.
- `syntax_error.sdx` was `print (1`, an unbalanced delimiter — which is **`RDX0012`**, a
  *lexical* code, because the lexer tracks bracket depth. Plan 40 §10.3 found precisely this
  and split its test in two for it; this row re-earned the lesson rather than reading it.
  Renamed `parse_error.sdx` and rewritten as `then` with no `if`, which is **`RDX0102`**.

Plan 46 §1.3's ratio again: of that row's first-pass findings, 11 of 16 were its own probe's
bugs. A fixture that fails for the wrong reason is the same error, one layer down, and the
only defence is checking what it actually does before asserting anything about it.

### 10.4 Narrowings, each stated rather than done quietly

Three places the implementation is smaller than §3 or §5 asked for:

- **`voltaic_fmt` never writes**, so there is no `write` parameter. §5 criterion 4 said *"does
  not write unless asked"*, which implies one. A formatter tool that only returns text cannot
  destroy work at all, the client has the file open, and an agent that wants the change
  written can write it.
- **`voltaic_build` takes no output path.** `voltaic build` has `-o`; this does not. The output is
  `<stem>.xrs` beside the input, which is the rule plan 07's remap and plan 11's build script
  already depend on — and a client that could redirect it could put the generated Rust
  somewhere nothing else will look for its source map. Both written paths come back in the
  response, so nothing has to be inferred.
- **A diagnostic carries a position, not a range.** `voltaic_diag::Diagnostic` keeps a span so
  `miette` can underline a statement; an end column that is sometimes a whole statement and
  sometimes one token is a field nobody could rely on.

### 10.5 The two kinds of failure are the row's load-bearing distinction

Not a finding so much as the thing everything else is arranged around, and it is worth saying
once plainly. Every tool returns `Result<T, String>`, and the split is:

- **`Ok` with `ok: false`** — an answer *about the program*. It does not compile, it does not
  parse, the line is synthetic. These are what the tool is for.
- **`Err`** — the *environment* cannot answer. No rustc, an unreadable path, a corrupt map.
  These become protocol errors.

A reader has to be able to tell *your code is wrong* from *this machine cannot answer*, and
collapsing them is the easy mistake: a `voltaic_check` that returned `Err` for a type error
would be indistinguishable from one that could not find rustc. `server_content` is where the
split is enforced and `session.rs` asserts it on the wire.

### 10.6 Leaf 4's verification was wrong in the way leaf 4 was guarding against

§1.5 made the `.mcp.json` edit textual because a JSON round-trip had once rewritten the whole
file — 136 insertions for a seven-line change. The insertion worked: `git diff --stat` reports
**6 insertions, 0 deletions**.

The script's own *check* that nothing else moved then failed. It filtered out every line whose
text matched an inserted line — and `"type": "stdio"` and `"disabled": false` appear in the
neighbouring entries too, so it deleted theirs and concluded the file had changed. The edit
was right and the assertion about it was wrong, which is the same shape as the bug being
guarded against, one level up. Replaced with a structural splice check.

Two smaller things in the same family: the first version of the script assumed two-space
indentation and the file uses **tabs**, so its anchor matched nothing; and two probe scripts
in this row grepped for `Adding` and `Summary` in coloured `cargo` and `nextest` output and
silently matched nothing — AGENTS.md's own documented gotcha, hit three times in one session.
*Check exit codes, not output text.*

### 10.7 What `.mcp.json` now has, and what asserts it

Sixteen servers, the fifteen that were there plus `voltaic`, each of the fifteen **named** in
`manifest.rs` so a lost one fails the gate rather than being noticed by whoever next needs
it. The stdio-only claim is asserted by reading the manifest text for `rmcp`'s network feature
names, because a feature flipped on by a future `cargo add --features` would open a port
without changing a line of Rust — and the `client` feature is asserted to be a dev-dependency
**only**, since `session.rs` needs a real client and the shipped binary must not contain one.
