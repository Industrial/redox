# Editor integrations

Five clients, one directory each, each installable from a checkout with no
build step beyond compiling the tree-sitter parser. Plan 49
(`docs/43 - 49 - Editor Integrations.md`).

| Client | Directory | Highlighting | Diagnostics | Format |
|---|---|---|---|---|
| Neovim | [`neovim/`](neovim/README.md) | tree-sitter | `redox-lsp` | LSP, or `redox fmt` |
| Vim | [`vim/`](vim/README.md) | regex | — | `redox fmt` |
| VSCode / Cursor | [`vscode/`](vscode/README.md) | TextMate, then tree-sitter via the server | `redox-lsp` | LSP |
| Helix | [`helix/`](helix/README.md) | tree-sitter | `redox-lsp` | LSP |
| Zed | [`zed/`](zed/README.md) | tree-sitter | `redox-lsp` | LSP |

Install steps live in each client's own README, not here.

## What "CI-checked" means here, per client

Plan 49 §1.4. Four of the five editors are not installed on the machine this
was developed on, so *checked* cannot mean the same thing for all five — and a
gate reporting one word for all of them would report the weakest as though it
were the strongest. `crates/redox-cli/tests/editors.rs` therefore runs a
**ladder**, and each test's name carries its own strength:

| Client | What the gate actually does | Strength |
|---|---|---|
| **Neovim** | runs `nvim --headless`: loads the parser, parses every `examples/*.sdx` with no `ERROR` node, compiles all three queries, matches the highlight query | the editor runs it |
| **Cursor** | builds a VSIX, installs it with `--install-extension` into a throwaway `--extensions-dir`, reads it back from `--list-extensions`, **and has `node` load the installed entry module** | the editor installs it, and the module loads |
| **VSCode** | nothing — **inherited** from Cursor, which is a VSCode fork consuming the same contribution points | a claim, not a measurement |
| **Vim** | `nvim` sources `syntax/sdx.vim` as legacy vimscript and reports the highlight group at a keyword | a compatible editor runs it — not vim |
| **Helix** | `languages.toml` parses and every key is one Helix documents | the file is well-formed |
| **Zed** | `extension.toml` parses and names files that exist | the file is well-formed |

A test called `helix_config_parses` is not a test called `helix_works`, and the
difference is the point.

**Plan 71 found that the Cursor rung was itself over-read, which is the same
mistake one level up.** `the_vsix_installs_into_cursor` is a true name, and an
install is not an activation: VSCode does not run `activate` during
`--install-extension`, so an extension whose entry module throws on `require`
installs cleanly and lists back. Plan 49's VSIX excluded `node_modules` by
design, so that is what it did — and the surviving behaviour, TextMate
highlighting with no server, is byte-identical to the documented
`redox.server.enable: false`. A second rung now loads the module, and the row
above says so rather than leaving *"the editor installs it"* to carry a claim
it cannot.

The install instructions were also wrong in a way no test could catch, because
no test followed them: on a Nix-managed Cursor the wrapper forces
`--extensions-dir` to a read-only store path, so the documented
`cursor --install-extension` cannot write. The gate passed because it supplies
its own. `editors/vscode/README.md` says this now, and the Nix declaration is
the supported path on that machine.

Every check **skips with a reason** when its tool is absent, so the gate passes
on a machine with no editors and says which rungs did not run. A silently
skipped check is worse than a missing one.

## One keyword list

The Vim syntax file and the VSCode TextMate grammar each carry a keyword list,
and `tree-sitter-redox/queries/highlights.scm` carries a third. Three
highlighters that disagree are worse than one, so
`every_grammar_keyword_is_coloured` checks all three against the keyword
literals in `tree-sitter-redox/grammar.js` — the grammar being the only one of
the four that the compiler's own gate already exercises.

It found the gap it was written for: plan 45 added `rust`, `unsafe`, `extern`
and `union` to the grammar and to the compiler, and no highlighter coloured any
of them.
