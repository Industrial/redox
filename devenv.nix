{
  inputs,
  pkgs,
  ...
}: {
  imports = [./.cursor/nix];

  name = "project-template";

  # Shared modules from `.cursor/nix` — defaults are off; enable what this repo needs.
  # blaze is the default Rust build/check/run/test command (moon.yml drives it).
  # It ships its own toolchain, so languages-rust and packages-rust-dev stay on:
  # rust-analyzer, clippy, rustfmt and the cargo-* helpers are still cargo's.
  cursor.features.program-blaze.enable = true;
  cursor.features.program-moon.enable = true;
  cursor.features.program-lean-ctx.enable = true;
  cursor.features.program-roam-code.enable = true;
  cursor.features.dotenv.enable = true;
  cursor.features.packages-base.enable = true;
  cursor.features.packages-rust-dev.enable = true;
  cursor.features.packages-formatters.enable = true;
  cursor.features.env-python-ld.enable = true;
  cursor.features.languages-javascript.enable = true;

  # Agent harness — the `.cursor` toolchain that `.claude` agents/skills and
  # `.mcp.json` expect on PATH (mirrors idclear/monorepo). Paperclip stays
  # monorepo-only: it needs its own Postgres and process supervision.
  cursor.features.program-claude-code.enable = true;
  cursor.features.program-maestro.enable = true;
  cursor.features.program-assay.enable = true;
  cursor.features.program-context7.enable = true;
  cursor.features.program-omniroute.enable = true;
  cursor.features.program-hermes.enable = true;

  cursor.features.languages-rust = {
    enable = true;
    channel = "stable";
  };

  # Plan 72 (`docs/66 - 72 - The Seed Compiler.md`) leaf 2 — the bootstrap
  # seed's target. `crates/redox-seed` is compiled to `wasm32-wasip1` and
  # checked in as `redox1.wasm`, so a crate written in Redox need not also
  # commit the Rust it generates.
  #
  # Set **here** and not in `.cursor/nix/features/languages-rust.nix`, which
  # assigns `targets = []`. That is not a conflict: `targets` is a list
  # option, the module system merges list definitions, and the harness file is
  # shared with every other project that imports `.cursor/nix` — so a
  # project-only target belongs in the project's own file. This is the same
  # split `devenv.yaml` already states: shared input defaults there, project
  # pins here.
  #
  # `wasip1` rather than `wasip2` or `unknown-unknown`: the seed needs real
  # files and nothing else, preview1 is what `wasmtime` runs with no component
  # tooling, and a browser build (`unknown-unknown`) is the playground's
  # target and a different row (plan 72 §6).
  languages.rust.targets = ["wasm32-wasip1"];

  cursor.features.git-hooks-moon = {
    enable = true;
    # Preserve this repo's Moon gate composition (not the shared ci-* defaults).
    # :check / :build / :test are cargo's. blaze's verdict is its own front
    # end's, so moon and the gate run cargo and blaze stays the by-hand inner
    # loop — see .moon/tasks/tag-crate.yml for the measurements behind that.
    # :check and :test are therefore already rustc's last word, and the
    # :cargo-check / :cargo-test duplicates they used to need are gone.
    preCommitTargets = ":format :check :lint :test";
    prePushTargets = ":format :check :lint :build :test :audit";
  };
  cursor.features.git-hooks-prek.enable = true;

  # Project-only env (shared soft defaults cover CARGO_TERM_COLOR / Moon / nextest).
  env = {
    RUST_BACKTRACE = "1";
    RUSTC_WRAPPER = "sccache";
  };

  # Project-only packages (beads removed upstream in favor of Maestro).
  packages = [
    inputs.definitively.packages.${pkgs.stdenv.hostPlatform.system}.definitively
    # Plan 41 (`docs/36 - 41 - Tree-sitter Grammar.md`) leaf 1. A generated
    # parser is an artefact of its generator, so an unpinned `tree-sitter`
    # means the committed parser cannot be regenerated identically on another
    # machine. The system PATH happened to carry the same 0.26.11 when the
    # plan was written, which is exactly the kind of coincidence a pin exists
    # to stop relying on.
    pkgs.tree-sitter
    # Plan 72 leaf 2. `wasmtime` runs the seed on a clean clone; `binaryen`
    # is `wasm-opt`, which leaf 1's size criterion needs to be comparable at
    # all — Zig's 2.4 MiB `zig1.wasm` is the figure *after* `wasm-opt -Oz`,
    # so a number measured without it would be measuring something else.
    pkgs.wasmtime
    pkgs.binaryen
  ];

  # Project-only shell wiring (shared features handle moon-sync, prek install).
  enterShell = ''
    # Keep build temporaries off tmpfs, which is RAM.
    #
    # TMPDIR arrives unset in this shell, so blaze, cargo and rustc wrote
    # their scratch objects to /tmp — a 2.0 GB tmpfs found 90% full with
    # 216 MB free, 1.25 GB of it Claude Code session scratchpads under
    # /tmp/claude-1000. blaze then failed mid-build with
    #
    #   the daemon's worker ended (exit status: 1: Error: backend: writing
    #   the object: No space left on device (os error 28))
    #
    # which reads like a code defect and is not one. Worse, because tmpfs is
    # RAM, every object written there also counted against the memory that was
    # OOM-killing blaze-engine.
    #
    # This is exported here rather than set in `env` above because devenv
    # clears TMPDIR after applying `env` — setting it there leaves it empty,
    # and the mkdir below then fails on an empty directory name.
    #
    # It must stay OUTSIDE the repository. `crates/redox-cli/tests/support.rs`
    # formats generated Rust in `std::env::temp_dir()` precisely because
    # nothing there inherits this repo's `rustfmt.toml` (tab_spaces = 2) —
    # Redox's output uses rustfmt's default four. Pointing TMPDIR at
    # `$DEVENV_STATE/tmp` put it back inside the tree and
    # `generated_rust_is_rustfmt_clean` failed on every example, which is the
    # same false finding plan 04's probe recorded. ~/.cache is on /home, which
    # has 630 GB free, and is nobody's Cargo workspace.
    export TMPDIR="$HOME/.cache/redox/tmp"
    mkdir -p "$TMPDIR"

    # Hermes: project-local home + seed templates (never overwrite secrets).
    export HERMES_HOME="$PWD/.hermes"
    mkdir -p "$HERMES_HOME"
    for _f in config.yaml SOUL.md; do
      if [ ! -f "$HERMES_HOME/$_f" ] && [ -f ".cursor/nix/features/hermes-agent/templates/$_f" ]; then
        cp ".cursor/nix/features/hermes-agent/templates/$_f" "$HERMES_HOME/$_f"
      fi
    done
    if [ ! -f "$HERMES_HOME/.env" ] && [ -f .cursor/nix/features/hermes-agent/templates/.env.example ]; then
      cp .cursor/nix/features/hermes-agent/templates/.env.example "$HERMES_HOME/.env"
    fi
  '';
}
