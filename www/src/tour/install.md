# Install

Two supported routes. Both give you the `voltaic` binary, which is the whole
toolchain: `build`, `check`, `fmt`, `run` and `watch`.

## From crates.io

```bash
cargo install voltaic-cli
voltaic --version
```

The crate is `voltaic-cli` and the binary is `voltaic`. The names differ
because the workspace's libraries are `voltaic-ast`, `voltaic-lexer`,
`voltaic-parser` and so on — the CLI is one crate among them rather than the
root of the set.

## From nixpkgs

```bash
nix profile install nixpkgs#voltaic
```

Or as a flake input:

```nix
{
  inputs.voltaic.url = "github:Industrial/voltaic";

  outputs = { nixpkgs, voltaic, ... }: {
    devShells.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {
      packages = [ voltaic.packages.x86_64-linux.default ];
    };
  };
}
```

The flake builds from the seed, so a clean checkout needs no previously
installed Voltaic in order to bootstrap one.

## Editor support

Five integrations live under `editors/`: VS Code, Zed, Neovim, Helix and Vim.
All five highlight; VS Code and Zed also speak to `voltaic-lsp` for
diagnostics, go-to-definition, references, rename and formatting.

## What else you need

Rust, because Voltaic emits Rust and then asks `rustc` about it. Nothing more.
There is no runtime to install, and nothing is linked into your program that
you did not write.
