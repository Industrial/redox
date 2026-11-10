# Redox for Zed

A manifest, a language config, and plan 41's three queries. Zed is
tree-sitter-native, so the grammar does the highlighting.

## Install

1. **Put `redox-lsp` on `PATH`:**

   ```bash
   cargo install --path crates/redox-lsp
   ```

2. **Install this directory as a dev extension.** In Zed: `zed: install dev
   extension`, then pick `editors/zed/`. Zed builds the grammar from
   `[grammars.redox] path` — point it at this checkout's `tree-sitter-redox/`
   if you moved either directory.

## Layout

```
extension.toml                      the manifest
languages/redox/config.toml         suffixes, comments, indent, server
languages/redox/highlights.scm      copies of tree-sitter-redox/queries/
languages/redox/injections.scm
languages/redox/locals.scm
```

`.sdx` is claimed; `.xrs` is not. It is ordinary Rust — the compiler's whole
premise — so Zed's own Rust support handles it, and claiming it here would take
highlighting *away* from the file.

## The queries are copies, and the gate makes that safe

Zed reads its queries from inside the extension, so unlike Neovim — which is
handed the query text at setup time — there is no way to avoid a second copy.
A second copy of a file is a file that drifts.

`crates/redox-cli/tests/editors.rs` therefore asserts these three are
**byte-identical** to `tree-sitter-redox/queries/`. That is the same mechanism
the golden corpus uses: a copy is allowed when a build fails the moment it
stops matching. Re-copy with:

```bash
cp tree-sitter-redox/queries/*.scm editors/zed/languages/redox/
```

## Verified by

`editors.rs` parses `extension.toml` and `languages/redox/config.toml`, checks
every key against an allow-list from Zed's documented schema, asserts every
path the manifest names exists, and compares the three queries byte for byte.

**That is weaker than the other clients and the test's name says so** — it is
`zed_config_parses`, not `zed_works`. There is no `zed` on the machine this was
developed on. Plan 49 §1.4 is the ladder.
