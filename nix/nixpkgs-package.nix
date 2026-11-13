# The nixpkgs expression for Voltaic, kept here so it is reviewed and updated
# alongside the code it builds.
#
# THIS FILE IS NOT BUILT BY THIS REPOSITORY. `flake.nix` is what `nix build`
# here uses, and it builds from the working tree. This one is a copy destined
# for `pkgs/by-name/vo/voltaic/package.nix` in NixOS/nixpkgs, written in
# nixpkgs' idiom — `fetchFromGitHub` on a release tag, `cargoHash`, `finalAttrs`
# — because that is the form a nixpkgs reviewer expects and the form
# `nix-update` knows how to rewrite.
#
# Adding it to nixpkgs is a one-time, human-reviewed PR; `docs/NIXPKGS.md` is
# that procedure. Every release after that is bumped automatically by the
# `nixpkgs` job in `.github/workflows/publish.yml`, which runs `nix-update`
# against the merged file and opens the PR.
#
# Keep the three build-time facts below in step with `flake.nix`. They are the
# same facts for the same reason, and the reason is that a Voltaic checkout
# contains no Rust.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  wasmtime,
  rustfmt,
  rustc,
  nix-update-script,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "voltaic";
  version = "0.1.1";

  src = fetchFromGitHub {
    owner = "Industrial";
    repo = "voltaic";
    tag = "v${finalAttrs.version}";
    # Both hashes are real, and this expression has been built from this exact
    # tag: it fetched the tarball, bootstrapped from the seed, and produced
    # voltaic, voltaic-lsp and voltaic-mcp. So the file is ready to copy into
    # nixpkgs as-is; there is no fakeHash dance to do first.
    #
    # Never edit them by hand again either — `nix-update voltaic --version <v>`
    # rewrites version, hash and cargoHash together, and the release workflow
    # runs it against the merged upstream copy on every tag.
    hash = "sha256-be92i6aYbIyMNwr9fO1aOTBKuNRyfRyS37XP3zxBnS0=";
  };

  cargoHash = "sha256-f7WnCYdq56UIN0hcSttYA+Chfnz/2ZmUa9k/Mw93QEA=";

  # The two editor servers ship beside the compiler, from one derivation: the
  # three binaries share almost all of their dependency graph, so building them
  # separately would compile voltaic-ast, voltaic-parser and voltaic-emit three
  # times over.
  cargoBuildFlags = [
    "--package"
    "voltaic-cli"
    "--package"
    "voltaic-lsp"
    "--package"
    "voltaic-mcp"
  ];

  # wasmtime and rustfmt are `bootstrap.sh`'s own declared requirements, below.
  #
  # rustc is listed deliberately, and not because the build needs it —
  # buildRustPackage already supplies one. `crates/voltaic-driver/build.rs`
  # resolves a real rustc by running `--version` and checking the banner, then
  # bakes the path in as `VOLTAIC_RUSTC`. Because that path is a string in the
  # binary, nix's reference scanner finds it and keeps it as a runtime
  # dependency, which is what makes `voltaic check` keep working after a garbage
  # collection. Dropping this line would not fail the build; it would ship a
  # `voltaic check` that breaks the first time the store is collected.
  nativeBuildInputs = [
    wasmtime
    rustfmt
    rustc
  ];

  # **A source checkout has no Rust to compile.** Every crate under crates/ is
  # written in Voltaic: the .sdx under each crate's sdx/ is the source, and the
  # .rs under its src/ is generated and untracked. So the GitHub tarball
  # contains zero files under any crate's src/, and the derivation has to do
  # what a developer does before their first build: run the seed compiler over
  # every .sdx. `seed/voltaic1.wasm` is tracked for exactly this, and
  # `seed/SEED.md` is its provenance.
  #
  # HOME is /homeless-shelter in the sandbox and wasmtime wants a writable cache
  # directory, so it gets one.
  preBuild = ''
    export HOME="$NIX_BUILD_TOP/home"
    mkdir -p "$HOME"
    patchShebangs bootstrap.sh
    ./bootstrap.sh
  '';

  # The suite is not runnable in the sandbox, and saying so is better than a
  # checkPhase that quietly skips most of it: editors.rs wants an editor, node
  # and npm on PATH, watch.rs wants a filesystem watcher and twenty seconds, and
  # status.rs reads the git tree, which fetchFromGitHub does not provide. Those
  # gates run in the project's own CI on every push.
  doCheck = false;

  # Lets nixpkgs' own r-ryantm bot bump this package too, independently of the
  # release pipeline's PR. Whichever gets there first, the other finds nothing
  # to do.
  passthru.updateScript = nix-update-script {};

  meta = {
    description = "Source-to-source compiler for the Voltaic language, emitting ordinary Rust";
    longDescription = ''
      Voltaic compiles .sdx source to .xrs files containing ordinary Rust.
      Diagnostics are reported against the .sdx you wrote. It has no type
      checker, no borrow checker and no exhaustiveness checker of its own, by
      design: rustc is the authority on all three.

      This package provides the `voltaic` compiler together with the language
      server and the Model Context Protocol server.
    '';
    homepage = "https://github.com/Industrial/voltaic";
    changelog = "https://github.com/Industrial/voltaic/releases/tag/v${finalAttrs.version}";
    license = lib.licenses.mit;
    mainProgram = "voltaic";
    maintainers = [];
    platforms = lib.platforms.unix;
  };
})
