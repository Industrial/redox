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

  # uv venv at `.devenv/state/venv` — Serena MCP (`scripts/serena-mcp-wrapper.sh`);
  # see pyproject.toml [dependency-groups].
  cursor.features.languages-python-uv = {
    enable = true;
    syncArguments = [
      "--no-install-project"
      "--group"
      "serena"
    ];
  };

  cursor.features.git-hooks-moon = {
    enable = true;
    # Preserve this repo's Moon gate composition (not the shared ci-* defaults).
    # :check / :build / :test are blaze's. blaze's verdict is its own front
    # end's and not yet all of rustc's, so the push gate adds :cargo-check and
    # :cargo-test — rustc is still the last word before anything leaves here.
    preCommitTargets = ":format :check :lint :test";
    prePushTargets = ":format :check :lint :build :test :cargo-check :cargo-test :audit :check-docs";
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
  ];

  # Project-only shell wiring (shared features handle moon-sync, prek install).
  enterShell = ''
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
