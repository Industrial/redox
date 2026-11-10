# Redox for Neovim

Needs nvim **0.9 or newer** (`vim.treesitter.language.add`), and 0.11 or newer
for `vim.fs.root`. Developed against 0.12.6.

No plugin manager, and no `nvim-treesitter`.

## Install

1. **Build the parser.** Plan 41's grammar ships C sources, not a binary; plan
   49 §6 declines to commit one per platform.

   ```bash
   cd tree-sitter-redox
   cc -o redox.so -shared -fPIC -Os -Isrc src/parser.c src/scanner.c
   ```

2. **Put this directory on your runtimepath** and call `setup`. In
   `init.lua`:

   ```lua
   vim.opt.runtimepath:prepend('/path/to/redox/editors/neovim')
   require('redox').setup({
     grammar_dir = '/path/to/redox/tree-sitter-redox',
   })
   ```

3. **Put `redox-lsp` on `PATH`**, from this checkout:

   ```bash
   cargo install --path crates/redox-lsp
   ```

## What `setup` wires

| Option | Default | Effect |
|---|---|---|
| `grammar_dir` | — | tree-sitter highlighting; omit it and you get filetype detection and LSP only |
| `parser` | `<grammar_dir>/redox.so` | where the compiled parser is |
| `server` | `{ 'redox-lsp' }` | the language-server command |
| `lsp` | `true` | set `false` to skip the server |
| `format` | `true` | set `false` to leave `formatexpr` and the indent options alone |

`.sdx` becomes filetype `redox`. **`.xrs` becomes filetype `rust`** — it is
ordinary Rust, which is the compiler's whole premise, so it gets Rust's own
support rather than a Redox mode that would highlight it wrongly.

The three queries are read from `grammar_dir` at setup time rather than copied
here. nvim looks for `queries/<lang>/<name>.scm` and the grammar ships
`queries/<name>.scm`, so the layouts do not line up — and a copy would be a
second set of queries drifting from plan 41's.

## Formatting

`gq` formats through the language server, which is plan 40's hook onto plan 34.

There is no `formatprg` fallback and that is deliberate: `formatprg` pipes the
buffer through a filter on stdin, and `redox fmt` has no `--stdin` — it formats
files in place and refuses to write one it could not parse. For a buffer with
no server, `:!redox fmt %` and then `:e` does the same job on the saved file.

## Verified by

`crates/redox-cli/tests/editors.rs`, which runs `nvim --headless` against this
directory: the parser loads, **every** `examples/*.sdx` parses with no `ERROR`
node, all three queries compile, and the highlight query matches. The editor
runs it — the strongest rung on plan 49 §1.4's ladder.
