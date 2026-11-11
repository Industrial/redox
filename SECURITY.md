# Security policy

## Reporting a vulnerability

Report vulnerabilities **privately** through GitHub's private vulnerability reporting:
[Security → Report a vulnerability](https://github.com/Industrial/redox/security/advisories/new).
This opens a draft GitHub Security Advisory (GHSA) that only the maintainers and you can see.

Do not open a public issue, pull request or discussion for a suspected vulnerability.

Please include the affected crate(s), the commit you saw it on, a description of the impact, and a
minimal reproduction if you have one — a `.sdx` file is the ideal form.

## Response times

Redox has one maintainer, so these are targets, not a contractual SLA:

| Step | Target |
|---|---|
| Acknowledge the report | within 7 days |
| Initial assessment (valid or not, severity) | within 14 days |
| Fix for a confirmed issue | within 90 days of the report; sooner for high or critical severity |

We credit reporters in the advisory unless you ask us not to.

## Supported versions

**Redox is pre-release and nothing is published to crates.io.** There is no released version to
support, and no version ranges to maintain: the supported version is `main`. This table will be
replaced with a real support matrix at the first release.

| Version | Supported |
|---|---|
| `main` | Yes |
| anything else | No |

## Scope

Redox is a source-to-source compiler: `.sdx` in, `.xrs` containing ordinary Rust out. Two things
follow, and they shape what counts as a vulnerability here.

**Redox has no type checker, no borrow checker and no exhaustiveness checker, by design** — rustc
is the authority on types and borrows (`AGENTS.md`). So "Redox accepted a program rustc then
rejected" is a *diagnostics* issue, not a soundness one. Generated `.xrs` that rustc accepts but
which is unsound would be a rustc issue, unless Redox emitted `unsafe` the input did not ask for.

**In scope:**

- Generated `.xrs` that does something the `.sdx` did not say — a lowering that changes the
  meaning of a program, injects `unsafe`, or silently drops a check.
- Path traversal or arbitrary writes from `redox build` / `redox fmt` / the driver's cache or
  output paths.
- Code execution while *compiling* untrusted `.sdx` — the compiler is expected to read hostile
  input and only ever write Rust.
- `seed/redox1.wasm` not matching the provenance recorded in `seed/SEED.md` (`blob.rs` asserts
  this agreement; a mismatch means the blob was replaced).
- The MCP server (`crates/redox-mcp`) and LSP server (`crates/redox-lsp`) reading or writing
  outside the workspace they were pointed at.

**Out of scope:**

- A crash or panic on malformed input with no further impact. Report it as an ordinary bug.
- Advisories in development-only dependencies. `cargo audit` runs in CI; open a normal issue.
- The `.cursor` and `.claude-setup` submodules, which are separate repositories with their own
  policies.
