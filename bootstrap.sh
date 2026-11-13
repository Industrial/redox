#!/usr/bin/env bash
# Regenerate the Rust for the crates written in Voltaic.
#
# Plan 72 (`docs/66 - 72 - The Seed Compiler.md`) leaf 5.
#
# `crates/voltaic-ast` and `crates/voltaic-fmt` are written in Voltaic: the `.sdx`
# under each one's `sdx/` is the source, and the `.rs` under its `src/` is
# generated and **not tracked**. So a clean clone has no `voltaic-ast` crate at
# all, and nothing that could build the compiler which would generate it.
#
# `seed/voltaic1.wasm` breaks that cycle. Run this before the first build:
#
#     ./bootstrap.sh
#     cargo build
#
# `seed/SEED.md` is the blob's provenance and says when to refresh it.
#
# **Two steps, and the second is not optional.** The seed emits for rustfmt's
# defaults; this repository sets `tab_spaces = 2`, so the committed form is the
# formatted one. `skip_children` keeps rustfmt from chasing `mod` declarations
# whose files may not exist yet, which is the state this script is fixing.
set -euo pipefail

cd "$(dirname "$0")"

SEED="seed/voltaic1.wasm"
# The crates whose `src/` this regenerates, from `seed/PORTED` — the single
# copy of that list. The array that used to be here named two crates when three
# were ported, so a clean clone silently left the third's `src/` empty, and the
# test that proves every generated tree is ignored read an equally stale list of
# its own. `crates/voltaic-cli/tests/support.rs` and
# `crates/voltaic-seed/tests/bootstrap.rs` read the same file.
mapfile -t PORTED < <(sed -e 's/#.*//' seed/PORTED | awk 'NF { print $1 }')
if [ "${#PORTED[@]}" -eq 0 ]; then
    echo "error: seed/PORTED names no crates." >&2
    exit 1
fi

if [ ! -f "$SEED" ]; then
    echo "error: $SEED is missing. It is tracked; see seed/SEED.md." >&2
    exit 1
fi

for tool in wasmtime rustfmt; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: $tool not found. Run inside 'devenv shell', which pins it." >&2
        exit 1
    fi
done

for crate in "${PORTED[@]}"; do
    src="crates/$crate/sdx"
    out="crates/$crate/src"
    if [ ! -d "$src" ]; then
        echo "error: $src is missing, so $crate has no source to generate from." >&2
        exit 1
    fi

    # `--dir .` is the whole filesystem interface: WASI grants no access without
    # a preopen, and the paths below must therefore be **relative** to it. An
    # absolute path reaches nothing, which the seed now reports rather than
    # exiting 0 on an empty tree (plan 72 §11.8).
    wasmtime run --dir . "$SEED" "$src" "$out"

    # One file at a time, so a `mod` declaration pointing at a sibling that is
    # still being written cannot fail the run.
    find "$out" -name '*.rs' -exec rustfmt --edition 2024 --config skip_children=true {} +

    echo "bootstrap: $crate regenerated from $src"
done

echo "bootstrap: done. 'cargo build' will now work."
