# Redox for VSCode and Cursor

One extension, both editors. Cursor is a VSCode fork and consumes the same
contribution points, so there is nothing to fork here.

## Install

1. **Put `redox-lsp` on `PATH`:**

   ```bash
   cargo install --path crates/redox-lsp
   ```

2. **Build the VSIX.** The packager is in this repository, so there is nothing
   to fetch — `REDOX_VSIX_OUT` asks the test that builds it to keep a copy:

   ```bash
   REDOX_VSIX_OUT=$PWD/redox.vsix cargo test -p redox-cli --test editors vsix
   cursor --install-extension redox.vsix     # or
   code   --install-extension redox.vsix
   ```

   Without that variable the test still builds and installs a VSIX, into a
   throwaway directory it then discards — see *Verified by* below, and *Why
   there is no `vsce`*.

3. **Install the client dependency**, if you want the language server rather
   than TextMate highlighting alone:

   ```bash
   cd editors/vscode && npm install
   ```

   `vscode-languageclient` is the extension's one dependency and is **not**
   vendored. Without it the extension still contributes the language, the
   TextMate grammar and the editor configuration; `activate` is what needs the
   module.

## What you get

| | From |
|---|---|
| `.sdx` recognised, `--` comments, bracket pairs, 2-space indent | `language-configuration.json` |
| Colour on a cold open, before the server starts | `syntaxes/redox.tmLanguage.json` |
| Diagnostics, document symbols, semantic tokens, formatting | `redox-lsp` (plan 40) |

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

`crates/redox-cli/tests/editors.rs` builds the VSIX and installs it into
**Cursor** with `--install-extension`, into a throwaway `--extensions-dir`, and
reads it back from `--list-extensions`. Your own extensions are never touched.

**VSCode is not verified, and the distinction matters.** There is no `code` on
the machine this was developed on. The Cursor install is a measurement; the
VSCode claim is an inference from Cursor being a VSCode fork that consumes the
same contribution points. Plan 49 §1.2 records that the register had this the
other way round.
