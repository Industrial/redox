# Changelog

Notable changes per release. Nothing has been released yet: `0.1.0` is the
version in the manifests, not a published version. `docs/RELEASING.md` is how a
release is cut.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A release pipeline. `.github/workflows/publish.yml` runs the gate, packages
  every published crate, writes a CycloneDX SBOM per crate, attests the lot with
  SLSA Build L3 provenance, publishes to crates.io in a derived dependency
  order, re-downloads each crate and checks it against the signed provenance,
  creates the GitHub Release, and then opens the nixpkgs version-bump PR.
  `docs/RELEASING.md` and `docs/NIXPKGS.md` are the prose.
- `scripts/validate-crate-packaging.sh` — derives the publish order by
  topological sort of the workspace manifests, checks the dependency rules, and
  proves each packaged crate contains Rust and compiles from its own archive.
- `crates/voltaic-cli/tests/publish_metadata.rs` — the manifest half of those
  rules, in the ordinary gate, so a broken manifest is reported on the commit
  that caused it rather than at the next release.
- `nix/nixpkgs-package.nix` — the nixpkgs expression, for the one-time
  submission described in `docs/NIXPKGS.md`.
- `.github/actions/setup-voltaic` — the pinned toolchain, wasmtime, tree-sitter,
  moon and the seed bootstrap, in one place instead of once per job.
- `include` keys on the eight published crates, which is what makes them
  publishable at all: `src/` is generated from `sdx/` and is `.gitignore`d, and
  `cargo package` enumerates files from git, so without `include` a published
  crate contains no Rust and uploads cleanly anyway.
- `description`, `repository` and `homepage` metadata; crates.io rejects a
  publish with no description.
- A root `LICENSE` file, which did not exist.

### Changed

- **The licence is now MIT**, replacing `CC-BY-SA-4.0`. CC-BY-SA is a content
  licence: it carries no patent grant and its copyleft scope over linked code is
  untested, which is a poor fit for libraries meant to be depended on.
- Intra-workspace dependencies are declared once in `[workspace.dependencies]`
  and inherited with `<crate>.workspace = true`, so each runtime dependency
  carries a version. `cargo publish` refuses a path dependency without one.
  Dev-dependencies deliberately stay path-only, which is the opposite rule and
  matters for the same reason.
- **The project was renamed from Redox to Voltaic**, including all thirteen
  crates and the `voltaic` binary. Not everything carries the new name yet: the
  `RDX####` diagnostic codes and the `__rdx_` generated-identifier prefix still
  use the old one, because each needs its own build → regenerate → reseed cycle.
  A diagnostic still reads `RDX0101`. The `.sdx` and `.xrs` extensions never
  encoded the old name and are unchanged by design.

### Fixed

- `AGENTS.md` placed `build.rs` under the CLI crate. It is at
  `crates/voltaic-driver/build.rs`, and it is the only tracked hand-written
  Rust left under `crates/`.

### Not published

`voltaic-bench`, `voltaic-fuzz` and `voltaic-seed` are internal; `voltaic-lsp`
and `voltaic-mcp` are binary-only and delivered through nixpkgs and
`flake.nix`. All five carry `publish = false`.
