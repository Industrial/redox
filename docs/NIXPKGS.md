# Getting Voltaic into nixpkgs

Two different jobs, and only the second is automated.

| | Who | How often |
|---|---|---|
| **Adding** the package to nixpkgs | a human, reviewed by nixpkgs | once |
| **Bumping** it on each release | the `nixpkgs` job in `.github/workflows/publish.yml` | every release |

nixpkgs reviews new packages by hand, so the first submission cannot be
automated and no amount of CI will change that. Everything after it can be, and
is: once `pkgs/by-name/vo/voltaic/package.nix` exists upstream, each tagged
release opens its own version-bump PR with `version`, `hash` and `cargoHash`
already rewritten and `nix-build` already passed.

Until that first PR merges, the `nixpkgs` job finds no attribute to bump and
says so — a `::notice`, not a failure. It is the last job in the workflow and
nothing depends on it, so a nixpkgs problem can never be mistaken for a bad
crates.io release.

## Why a fork

Because nobody outside the nixpkgs committer set can push to `NixOS/nixpkgs`,
and a GitHub pull request has to come *from a branch somewhere*. For an external
contributor that branch lives in a fork, and the PR goes fork → upstream. This
is how every nixpkgs contribution works, by hand or automated; there is no
API that opens a PR without a branch behind it.

`Industrial/nixpkgs` already exists, so there is nothing to create. The release
job resets it to upstream `master` on every run rather than merging, so the fork
never drifts and never needs maintaining — a stale fork is the usual way this
kind of automation rots, and resetting makes the fork's history irrelevant.

## Where this stands

The dependency that is easy to miss: `fetchFromGitHub` needs a tag that already
exists, so the nixpkgs submission comes *after* a real release, not before. That
is no longer a constraint — two releases have shipped.

| | |
|---|---|
| Repository is `Industrial/voltaic` | done |
| `CARGO_REGISTRY_TOKEN` set, pipeline rehearsed twice with dry runs | done |
| `v0.1.0` and `v0.1.1` published — 8 crates each, GitHub Releases with archives, SBOMs and SLSA provenance | done |
| `nix/nixpkgs-package.nix` builds from the published tag, hashes filled | done |
| **The one-time submission to nixpkgs** | **outstanding — the only blocking step** |
| `NIXPKGS_PR_TOKEN` | outstanding, but nothing for it to do until the above merges |

Both releases' `nixpkgs` jobs reported that there was no attribute to bump and
passed, which is the designed first-release behaviour rather than a failure.

**Keep the `version` in `nix/nixpkgs-package.nix` at the newest release until
the submission lands.** It is the file that gets copied, so a stale version there
means submitting a stale package. Once it is merged upstream, this copy stops
mattering: `nix-update` maintains the nixpkgs one, and the release job runs it.

## The one-time submission

1. Copy the expression and format it for its destination. nixpkgs uses
   `nixfmt-rfc-style`; this repository uses `alejandra`, so the in-tree copy is
   in the wrong dialect on arrival. That is deliberate — the file's formatter
   belongs to where it is going, and `treefmt` owns everything that lives here.

   ```bash
   cd /path/to/nixpkgs
   mkdir -p pkgs/by-name/vo/voltaic
   cp /path/to/voltaic/nix/nixpkgs-package.nix pkgs/by-name/vo/voltaic/package.nix
   nix run nixpkgs#nixfmt-rfc-style -- pkgs/by-name/vo/voltaic/package.nix
   ```

   No `all-packages.nix` entry is needed: `pkgs/by-name` is picked up by path.

2. **Nothing — the hashes are already real and the expression already builds.**
   Both `hash` and `cargoHash` are filled for `v0.1.1`, and this was verified by
   building it against the published tag rather than reasoned about:

   ```
   $ nix-build -E "with import <nixpkgs> {}; callPackage ./nix/nixpkgs-package.nix {}"
   …
   Compiling voltaic-cli v0.1.1 (/build/source/crates/voltaic-cli)
   Compiling voltaic-lsp v0.1.1 (/build/source/crates/voltaic-lsp)
   Compiling voltaic-mcp v0.1.1 (/build/source/crates/voltaic-mcp)
   /nix/store/…-voltaic-0.1.1
   ```

   It fetched the GitHub tarball, found no Rust in it, bootstrapped from the
   seed, and produced all three binaries. The output's runtime closure contains
   `rustc-wrapper`, which is the `VOLTAIC_RUSTC` reference the comment on
   `nativeBuildInputs` describes — so `voltaic check` keeps working after a
   garbage collection, as claimed.

   For a later version, `nix-update` rewrites all three fields at once, which is
   what the release job does:

   ```bash
   nix-shell -p nix-update --run "nix-update voltaic --version <version>"
   ```

3. Check it the way a reviewer will.

   ```bash
   ./result/bin/voltaic --help
   nix run nixpkgs#nixpkgs-review -- rev HEAD
   ```

4. Open the PR from your `Industrial/nixpkgs` fork — the same fork the release
   job pushes to, which already exists. Title it `voltaic: init at 0.1.1`,
   per nixpkgs' commit convention.

### What a reviewer will ask about

Worth pre-empting in the PR body, because all three look odd without the reason:

- **It builds from a checked-in `.wasm`.** `seed/voltaic1.wasm` is the
  previous-stage compiler. Voltaic is written in Voltaic, so a source checkout
  contains no Rust at all and something has to generate it; `preBuild` runs the
  seed over every `.sdx`. This is the same bootstrap problem every self-hosting
  language has. `seed/SEED.md` is the blob's provenance.
- **`wasmtime` and `rustfmt` are build inputs.** They are what runs that seed
  and formats what it writes. Both are `bootstrap.sh`'s own declared
  requirements.
- **`rustc` is a build input it does not appear to need.**
  `crates/voltaic-driver/build.rs` bakes a verified rustc path into the binary
  as `VOLTAIC_RUSTC`, so nix retains it as a runtime reference. That is what
  keeps `voltaic check` working after a garbage collection. See the comment on
  `nativeBuildInputs` in the expression.

## After it merges

Nothing. The `nixpkgs` job takes over on the next tag. Two independent things
will then bump the package — that job, and nixpkgs' own r-ryantm bot, which
`passthru.updateScript` opts into — and whichever arrives second finds nothing
to do.

If the automated PR ever needs re-running, re-run the `nixpkgs` job: it
force-updates the same branch and reuses the open PR rather than opening a
second one.

## The faster path, if nixpkgs latency is a problem

nixpkgs review takes days to weeks, which is a poor fit for releasing often.
`flake.nix` in this repository already builds all three binaries, bootstrap
included, so a user can have a release the moment it is tagged:

```bash
nix run github:Industrial/voltaic
nix profile install github:Industrial/voltaic
```

Pointing that at a binary cache so users do not compile it themselves is a
separate, small piece of work and is not part of the release pipeline today.
