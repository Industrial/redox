# voltaic

The Voltaic compiler. `.sdx` in, `.xrs` containing ordinary Rust out.

```bash
cargo install voltaic-cli     # installs the `voltaic` binary
voltaic build hello.sdx       # transpile to hello.xrs
voltaic check hello.sdx       # run rustc, report against the .sdx
voltaic fmt hello.sdx         # canonical formatting
```

Diagnostics are reported against the `.sdx` you wrote, with 1-based line and
column in characters. rustc is the authority on types and borrows — Voltaic has
no type checker, no borrow checker and no exhaustiveness checker of its own, by
design.

Voltaic is written in Voltaic: the `.sdx` under each crate's `sdx/` is the
source, and the Rust under `src/` is generated from it. A clean clone has no
generated Rust at all and bootstraps from a checked-in seed compiler; see
`seed/SEED.md` in the repository.

- Repository, documentation and the language specification:
  <https://github.com/Industrial/voltaic>
- Releasing and the publish pipeline: `docs/RELEASING.md`

MIT licensed.
