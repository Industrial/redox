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

## The order things have to happen in

The dependency that is easy to miss: `fetchFromGitHub` needs a tag that already
exists, so the nixpkgs submission comes *after* a real release, not before.

1. Rename the GitHub repository to `Industrial/voltaic`.
2. Merge `rename/voltaic`, then the release branch.
3. Add `CARGO_REGISTRY_TOKEN` and rehearse with a dry-run dispatch.
4. Tag `v0.1.0` and push it. crates.io and the GitHub Release happen; the
   `nixpkgs` job reports that there is nothing to bump yet and passes. This is
   the expected first-release outcome, not a failure.
5. **Now** do the one-time submission below, against the `v0.1.0` tag.
6. Once it merges, add `NIXPKGS_PR_TOKEN`. From the next tag on, the bump PR is
   automatic.

Step 6 can be done earlier; it just has nothing to do until step 5 has merged.

## Before the first submission

Two things are not done yet, and both are the owner's:

1. **Rename the GitHub repository to `Industrial/voltaic`.** The code and docs
   already point there, but the git remote is still `Industrial/redox` and
   nothing in `nix/nixpkgs-package.nix` can be fetched until the rename lands.
   This was left deliberately: renaming a repository is outward-facing.
2. **Push a release tag**, so there is something for `fetchFromGitHub` to fetch.
   `docs/RELEASING.md` has that flow.

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

2. Fill the two hashes. They ship as `lib.fakeHash`, which makes the build fail
   and print the real value. `hash` first, then `cargoHash` — the second cannot
   be computed until the source fetch succeeds.

   ```bash
   nix-build -A voltaic          # fails, prints the correct `hash`
   # paste it in, then:
   nix-build -A voltaic          # fails again, prints the correct `cargoHash`
   # paste that in, then:
   nix-build -A voltaic          # builds
   ```

   Or let `nix-update` do both at once, which is what the release job does:

   ```bash
   nix-shell -p nix-update --run "nix-update voltaic --version 0.1.0"
   ```

3. Check it the way a reviewer will.

   ```bash
   ./result/bin/voltaic --help
   nix run nixpkgs#nixpkgs-review -- rev HEAD
   ```

4. Open the PR from your `Industrial/nixpkgs` fork — the same fork the release
   job pushes to, which already exists. Title it `voltaic: init at 0.1.0`,
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
