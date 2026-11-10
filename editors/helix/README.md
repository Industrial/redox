# Redox for Helix

Two stanzas and a language-server entry. Helix is tree-sitter-native, so it
uses plan 41's grammar directly.

## Install

1. **Put `redox-lsp` on `PATH`:**

   ```bash
   cargo install --path crates/redox-lsp
   ```

2. **Merge `languages.toml` into `~/.config/helix/languages.toml`.** Helix
   merges rather than replaces, so appending the stanzas to an existing file is
   correct. Fix the `[[grammar]]` `source.path` to point at this checkout's
   `tree-sitter-redox/`.

3. **Build the grammar:**

   ```bash
   hx --grammar build
   ```

4. **Copy the queries** to where Helix looks for them. Helix wants
   `runtime/queries/<lang>/highlights.scm`, and the grammar ships
   `queries/highlights.scm`:

   ```bash
   mkdir -p ~/.config/helix/runtime/queries/redox
   cp tree-sitter-redox/queries/*.scm ~/.config/helix/runtime/queries/redox/
   ```

   That copy is a real cost and worth naming: it is a second set of queries
   that will drift from plan 41's. Neovim avoids it by reading the query text at
   setup time, which Helix has no equivalent of.

`.xrs` is deliberately not claimed. It is ordinary Rust — the compiler's whole
premise — so Helix's own `rust` language handles it, and claiming it here would
take highlighting *away* from the file.

## Verified by

`crates/redox-cli/tests/editors.rs` parses `languages.toml` and checks every
key against an allow-list drawn from Helix's documented schema.

**That is weaker than the other clients and the test's name says so** — it is
`helix_config_parses`, not `helix_works`. There is no `hx` on the machine this
was developed on, so the failure mode this file actually has is a typo in a key
Helix would silently ignore, and a key allow-list is the strongest check
available without the editor. Plan 49 §1.4 is the ladder.
