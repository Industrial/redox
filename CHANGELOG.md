# Changelog

Notable changes per release. `docs/RELEASING.md` is how a release is cut, and
`docs/NIXPKGS.md` covers the downstream bump.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-11-13

### Added

- **A visual identity.** `design/` holds two sources — `geometry/pile.json` for
  the mark and `tokens/tokens.json` for the palette — and `design/build.sh`
  emits everything else: seven SVG masters in two optical cuts, a
  `prefers-color-scheme` favicon, `tokens.css` and `tokens.toml`.
  `design/rasterize.sh` renders 31 platform files from those masters — favicon
  and ICO, apple-touch, PWA icons including maskable and monochrome, an iOS
  appiconset with the dark and tinted appearances, a macOS `.icns`, a Windows
  `.ico` with MSIX tiles, and the Linux `hicolor` tree. Output is
  byte-reproducible across runs.
- **A documentation site.** `www/` is an mdBook of 22 chapters deployed to
  GitHub Pages by `.github/workflows/pages.yml`. Every code sample is
  `{{#include}}`d from `examples/`, which CI already compiles and runs against
  a recorded expectation, so a sample cannot drift from the compiler. Ctrl-K
  quick search ranks over mdBook's own index rather than building a second one.
  `llms.txt`, `llms-full.txt`, `sitemap.xml`, JSON-LD and per-page canonical
  links make it legible to crawlers and language models.
- **Editor themes for all five integrations** — VS Code, Zed, Helix, Neovim and
  Vim — generated from the same palette by `design/themes.sh`, plus a `.sdx`
  file icon for VS Code.
- Seventeen gates covering the above, each falsified before being trusted:
  generated assets match their sources, every ICO frame is the size it claims,
  no page scrolls sideways at either of two widths in either theme, and every
  palette pairing clears WCAG AA.

### Changed

- **Two optical cuts replace one scaled drawing.** Five stacked plates above
  32px, three below, because five collapse into a smear at favicon size. Each
  raster frame renders from the cut appropriate to its size rather than being
  downsampled from 1024.
- Three palette values moved for legibility. `ink-faint` was 2.69:1 on the code
  surface and is now 4.56:1; the syntax `string` class moved off `zinc`
  entirely, since zinc is 2.93:1 on white and solving for a readable variant
  landed within one hex of `ink-faint`. Zinc is the mark's second metal and is
  no longer used for text anywhere.

### Fixed

- The MIT switch was incomplete. `README.md` still carried a CC-BY-SA badge and
  licence section, `CONTRIBUTING.md` still told contributors their work was
  CC-BY-SA, and `deny.toml` still allowed `CC-BY-SA-4.0` in the dependency
  list. All three now say MIT. The references under `docs/` are left alone:
  they are plan documents recording what was true when written.

### Known

- `deny.toml` is read by nothing — no gate runs `cargo-deny`, and `:audit` runs
  `cargo audit`, which is the advisory database rather than the licence policy.
  That is why its licence entry survived the switch unnoticed.
- Every page of the book carries two `h1` elements: mdBook's
  `<h1 class="menu-title">` plus the page's own. Not a WCAG failure, and fixing
  it means forking `theme/index.hbs` and carrying it across mdBook versions.
- The book loads its typefaces from Google Fonts rather than self-hosting them.

## [0.1.0] - 2026-11-13

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
