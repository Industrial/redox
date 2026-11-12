{
  description = "Redox — a source-to-source compiler: .sdx in, .xrs containing ordinary Rust out";

  # One input, deliberately. `devenv.nix` owns the development shell and the
  # pinned toolchain; this flake exists so that *other* flakes can consume the
  # binaries — `~/.dotfiles` builds `redox-lsp` and the VSCode/Cursor extension
  # from it (plan 71 leaves 1 and 3). A `rust-overlay` input would be a second
  # toolchain pin disagreeing with devenv's, for no gain: nixpkgs' rustc is
  # past the 1.85 that edition 2024 needs.
  #
  # A consumer is expected to `follows` this, as `~/.dotfiles` does for every
  # other flake it pulls in.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

    # The version every crate inherits, read from the workspace manifest rather
    # than written here — two copies of a version is one copy too many.
    version =
      (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
  in {
    packages = forAllSystems (pkgs: let
      # Both binaries come from one derivation. The workspace is eleven crates
      # that share almost all of their dependency graph, so two derivations
      # would compile `redox-ast`, `redox-parser` and `redox-emit` twice.
      redox = pkgs.rustPlatform.buildRustPackage {
        pname = "redox";
        inherit version;

        src = pkgs.lib.cleanSource ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        cargoBuildFlags = ["--package" "redox-cli" "--package" "redox-lsp" "--package" "redox-mcp"];

        # `crates/redox-cli/build.rs` runs a candidate `rustc --version` and
        # bakes the first path that answers like real rustc into `REDOX_RUSTC`
        # (see AGENTS.md — neither `PATH` nor `$RUSTC` is trustworthy in this
        # project's environments). Inside the sandbox that resolves to the
        # store rustc below, and because the path is baked into the binary as a
        # string Nix finds it when it scans the output and keeps it as a
        # runtime reference. That is what makes `redox check` work after a
        # garbage collection.
        nativeBuildInputs = [pkgs.rustc pkgs.wasmtime pkgs.rustfmt];

        # **A checkout has no Rust to compile.** Plan 72 untracked every
        # `crates/*/src/`, so `git archive HEAD` — which is what a
        # `github:Industrial/redox` flake input is served as — contains zero
        # files under any crate's `src/`. Verified rather than assumed:
        # `git archive HEAD | tar -t | grep -c '^crates/.*/src/'` is 0, while
        # `bootstrap.sh` and `seed/` are present.
        #
        # So the derivation has to do what `AGENTS.md` tells a human to do
        # before the first build: run the seed over every `.sdx`. `wasmtime`
        # runs `seed/redox1.wasm` and `rustfmt` formats what it writes; both
        # are `bootstrap.sh`'s own declared tool requirements.
        #
        # This ran locally for a year without being noticed because
        # `cleanSource ./.` picks up the *working tree*, which has a
        # bootstrapped `src/` on any developer's machine. The tarball does not.
        # `HOME` is `/homeless-shelter` in the sandbox and wasmtime wants a
        # writable cache directory, so it gets one.
        preBuild = ''
          export HOME="$NIX_BUILD_TOP/home"
          mkdir -p "$HOME"
          patchShebangs bootstrap.sh
          ./bootstrap.sh
        '';

        # The suite is not runnable here, and saying so is better than a
        # `checkPhase` that quietly skips most of it: `editors.rs` wants
        # `cursor`, `nvim`, `node` and `npm`, `watch.rs` wants a filesystem
        # watcher and twenty seconds, and `status.rs` reads the git tree. Those
        # gates are `moon run :test`'s, on a developer's machine, per AGENTS.md.
        doCheck = false;

        meta = {
          description = "Redox compiler, language server and MCP server";
          homepage = "https://github.com/Industrial/redox";
          license = pkgs.lib.licenses.cc-by-sa-40;
          mainProgram = "redox";
        };
      };
    in {
      inherit redox;
      default = redox;

      # `redox-lsp` and `redox-mcp` are the same derivation under the names a
      # consumer asks for. `~/.dotfiles` wants the editor pair by name, and an
      # alias is cheaper than three builds of one dependency graph.
      redox-lsp = redox;
      redox-mcp = redox;
    });

    formatter = forAllSystems (pkgs: pkgs.alejandra);
  };
}
