# Redox for Vim

The fallback client. Regex highlighting, no tree-sitter, no language server —
for a Vim with neither. If you have Neovim, use [`../neovim/`](../neovim/README.md)
instead; it gets diagnostics and a real parse.

## Install

Copy the three directories into `~/.vim/`, or add this directory to
`runtimepath`:

```vim
set runtimepath+=/path/to/redox/editors/vim
```

Nothing to build and nothing to install — which is the point of this client.

## What you get

| | |
|---|---|
| `.sdx` → filetype `sdx`, `.xrs` → filetype `rust` | `ftdetect/sdx.vim` |
| Keyword, string, comment, number, type and operator highlighting | `syntax/sdx.vim` |
| 2-space indent, `--` comments, `autoindent` without `smartindent` | `ftplugin/sdx.vim` |
| `:RedoxFmt` | `ftplugin/sdx.vim` |

`.xrs` gets Vim's own Rust support, because it *is* ordinary Rust — the
compiler's whole premise.

`nosmartindent` is deliberate: Redox is off-side-rule, so keeping the previous
line's indentation is right and reindenting on a brace is wrong.

## Formatting is `:RedoxFmt`, not `formatprg`

Forced rather than chosen. `formatprg` pipes the buffer through a filter on
stdin and `redox fmt` has no `--stdin`: it formats files in place and refuses
to write a file it could not parse (plan 34 criterion 8). `:RedoxFmt` writes
the buffer, formats the file, reloads it and restores the view — the same
safety property, through the only interface the formatter has.

## What regex highlighting cannot do

This client highlights by pattern rather than by parse, so it is the one here
that can be wrong. `syn region` keeps keywords out of strings and comments; it
does not make the file a parser. And Redox has no type checker, so a `syntax`
file certainly cannot tell a type from a constructor — both are `UpperCamel`
and both are coloured `Type`, which is the honest answer rather than a
heuristic that is sometimes wrong.

The keyword list is checked against `tree-sitter-redox/grammar.js` by
`crates/redox-cli/tests/editors.rs`, so it cannot drift from the tree-sitter
and TextMate highlighters.

## Verified by

`editors.rs` sources this syntax file under **`nvim`** and asserts the
highlight group at a keyword. There is no `vim` on the machine this was
developed on, so that is a *compatible editor* running it, not vim — the
third rung on plan 49 §1.4's ladder, and the test's name says so.
