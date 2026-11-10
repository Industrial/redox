# Redox for VSCode and Cursor

One extension, both editors. Cursor is a VSCode fork and consumes the same
contribution points, so there is nothing to fork here.

## Install

**On NixOS, use Nix and skip this section.** `~/.dotfiles` builds this
extension as `features/programming/redox/vscode-extension.nix` and registers
it in `features/programming/cursor/default.nix`, which is the only path that
works against a Nix-managed Cursor — see *Why `--install-extension` may not
work* below. The steps here are for an ordinary Cursor or VSCode install.

1. **Fetch the client dependency.** This is step 1, not step 3: the VSIX
   payload *walks* `node_modules`, so the closure has to exist before the
   package is built.

   ```bash
   cd editors/vscode && npm ci && cd -
   ```

   `package-lock.json` is tracked, so this is reproducible. `node_modules` is
   not tracked, because this command is what produces it.

2. **Put `redox-lsp` on `PATH`:**

   ```bash
   cargo install --path crates/redox-lsp
   ```

   Or set `redox.server.path` to wherever the binary is, which is what the Nix
   path does rather than trusting `PATH`.

3. **Build and install the VSIX.** The packager is in this repository, so there
   is nothing to fetch — `REDOX_VSIX_OUT` asks the test that builds it to keep
   a copy:

   ```bash
   REDOX_VSIX_OUT=$PWD/redox.vsix cargo test -p redox-cli --test editors vsix
   cursor --install-extension redox.vsix     # or
   code   --install-extension redox.vsix
   ```

   Without that variable the test still builds and installs a VSIX, into a
   throwaway directory it then discards — see *Verified by* below, and *Why
   there is no `vsce`*.

## Why `--install-extension` may not work

Plan 71 §1.2. If Cursor came from `pkgs.vscode-with-extensions`, its wrapper
passes `--extensions-dir <store path>` **before** your arguments, and that
path is read-only:

```
exec …/cursor-3.25.9/bin/cursor \
  --extensions-dir /nix/store/…-vscode-extensions/share/vscode/extensions "$@"
```

So `cursor --install-extension` has nowhere to write. Cursor says so when the
flag is given twice — *"Option 'extensions-dir' is defined more than once"* —
which is also why this repository's own gate can install into a throwaway
directory while the command above fails for a user. Declare the extension in
Nix instead.

## What you get

| | From |
|---|---|
| `.sdx` recognised, `--` comments, bracket pairs, 2-space indent | `language-configuration.json` |
| Colour on a cold open, before the server starts | `syntaxes/redox.tmLanguage.json` |
| Diagnostics, document symbols, semantic tokens, formatting | `redox-lsp` (plan 40) |
| `redox_build`, `redox_check`, `redox_fmt`, `redox_source_map` for Cursor's agent | `redox-mcp` (plan 50), registered in Cursor's `mcp.json` |

Two settings: `redox.server.path` and `redox.server.enable`. Turning the server
off leaves TextMate highlighting, which is the useful degraded mode.

`extension.js` is plain JavaScript on purpose — a TypeScript source would put a
compile step between a checkout and a working editor, for a file whose job is
to start a process.

## Why there is no `vsce`

`@vscode/vsce` is the usual packager. It is not installed here and `npx` will
not fetch it offline, and adding it would put an npm network dependency in the
gate for a build step that produces a zip.

A VSIX **is** a zip — an OPC package holding `[Content_Types].xml`,
`extension.vsixmanifest` and the extension under `extension/`. `editors.rs`
writes one with the `zip` crate, which this workspace already resolves.

The cost is disclosed rather than hidden: a hand-written manifest gets none of
`vsce`'s validation, so a mistake in it surfaces as an *install* failure rather
than a packaging error. That is acceptable only because the gate performs the
install.

## Verified by

`crates/redox-cli/tests/editors.rs`, at two strengths:

- `the_vsix_installs_into_cursor` builds the VSIX and installs it into
  **Cursor** with `--install-extension`, into a throwaway `--extensions-dir`,
  and reads it back from `--list-extensions`. Your own extensions are never
  touched.
- `the_installed_extension_can_load_its_entry_module` then has `node` load the
  *installed* `src/extension.js`, with a stub `vscode` standing in for the API
  the editor injects, and asserts the `activate` export is there.

**The second one exists because the first was not enough.** VSCode does not run
`activate` during an install, so plan 49's VSIX — which excluded
`node_modules` — installed cleanly, listed back perfectly and then threw on
`require`. What survives such a failure is the language id, the TextMate
grammar and the editor configuration: exactly `redox.server.enable: false`, so
a broken install was indistinguishable from a deliberately degraded one. Plan
71 §1.2 has the measurement.

**VSCode is still not verified, and the distinction matters.** There is no
`code` on the machine this was developed on. The Cursor rungs are
measurements; the VSCode claim is an inference from Cursor being a VSCode fork
that consumes the same contribution points. Plan 49 §1.2 records that the
register had this the other way round.
