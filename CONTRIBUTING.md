# Contributing to Voltaic

Thank you for helping. This page covers the legal part and the gates; the conventions every
change is held to are in [AGENTS.md](AGENTS.md), and the design documents are numbered under
[docs/](docs/).

## The one rule that surprises everyone

**Edit the `.sdx`, never the `.rs`.** Every crate under `crates/` is written in Voltaic. Its
`src/*.rs` is *generated* from its `sdx/*.sdx`, is `.gitignore`d, and is not in the index — so a
clean clone has no Rust in it at all until you bootstrap:

```bash
git clone --recurse-submodules https://github.com/Industrial/voltaic
cd voltaic
./bootstrap.sh      # runs seed/voltaic1.wasm over every crate in seed/PORTED
cargo build         # works from here
```

After changing a `.sdx`, regenerate with:

```bash
VOLTAIC_UPDATE=1 cargo test -p voltaic-cli --test port
```

That compiles the generated tree *before* writing it and refuses a generation that does not
compile, so a bad change leaves `src/` untouched rather than broken. It then fails the run on
purpose, so a regenerated diff is always reviewed instead of absorbed into a green build.

## Gates

Run these from inside `devenv shell`:

```bash
moon run :check :lint :test
```

`:lint` is `-D warnings`. CI runs the same targets plus `treefmt` — see
[.github/workflows/ci.yml](.github/workflows/ci.yml), which bootstraps from the seed first for
the reason above.

## License of contributions

Voltaic is licensed under [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/), as
declared by `[workspace.package] license` in `Cargo.toml`. By contributing, you agree that your
contribution is licensed under the same terms.

## Developer Certificate of Origin (DCO)

Every commit must be signed off. A sign-off certifies the
[Developer Certificate of Origin 1.1](https://developercertificate.org/): that you wrote the
change or otherwise have the right to submit it under the project's license. Add it with
`git commit -s`, which appends:

```text
Signed-off-by: Your Name <you@example.com>
```

Use your real name and an email address you can be reached at. Pull requests with commits
lacking a sign-off are not merged.

The [DCO workflow](.github/workflows/dco.yml) enforces this on every pull request: each
non-merge commit the pull request adds needs a `Signed-off-by:` trailer with the commit author's
email. Commits that predate the workflow are not checked, and commits authored by
`dependabot[bot]` are exempt. To fix a failing check, sign off and force-push:

```bash
git commit --amend -s --no-edit        # the last commit
git rebase --signoff origin/main       # every commit of the branch
```

## Signed commits

A sign-off is a statement; a signature proves who made the commit. Sign your commits with one of:

- **SSH** (simplest if you already push over SSH):
  `git config gpg.format ssh`, `git config user.signingkey ~/.ssh/id_ed25519.pub`,
  `git config commit.gpgsign true`, and add the key to GitHub as a *signing* key.
- **GPG**: `git config user.signingkey <key id>`, `git config commit.gpgsign true`, and add the
  public key to GitHub.
- **[gitsign](https://github.com/sigstore/gitsign)** (keyless, Sigstore):
  `git config gpg.x509.program gitsign`, `git config gpg.format x509`,
  `git config commit.gpgsign true`.

`git commit -S -s` signs and signs off in one step.

## Commit messages

Conventional commits, enforced on `commit-msg` by a [prek](https://github.com/j178/prek) hook
(commitizen). `feat(parser): …`, `fix(emit): …`, `chore(deps): …`.

## Security issues

Do not open public issues for vulnerabilities; follow [SECURITY.md](SECURITY.md).
