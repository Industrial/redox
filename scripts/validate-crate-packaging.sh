#!/usr/bin/env bash
# Validate that the workspace crates can be packaged for a crates.io publish, and
# print the order they must be published in.
#
# `.github/workflows/publish.yml` runs `$0 --print-order` and publishes in that
# order; `docs/RELEASING.md` describes the flow.
#
# Adapted from Industrial/id_effect's script of the same name, with one
# structural difference. That one carries a hand-written `LEVEL` map and `ORDER`
# array naming all forty of its crates. This workspace states its dependency
# graph in `Cargo.toml` already, and a second hand-written copy of a graph is
# exactly the drift `AGENTS.md` forbids — the rename from Redox to Voltaic would
# have had to touch both. So the order here is **derived**: a topological sort of
# the intra-workspace runtime dependencies, read out of `cargo metadata`.
#
# Checks, in order of cost:
#   1  one version across the workspace, and every publishable crate inherits it
#   2  crates.io's required metadata is present (it rejects a publish without a
#      description)
#   3  runtime intra-workspace deps carry a version; dev-deps carry none. Both
#      directions matter and they are opposite rules — see `check_dependencies`
#   4  no publishable crate depends on an unpublishable one
#   5  the graph is acyclic, and the topological order is stable
#   6  `cargo package` succeeds per crate, and the archive it produces actually
#      contains Rust. Check 6 is the one that matters most here — see below
#
# WHY CHECK 6 EXISTS. Every crate under `crates/` is written in Voltaic: `src/`
# is generated from `sdx/` and is `.gitignore`d. `cargo package` enumerates
# files from git, so without an `include` key it packages **no Rust at all** and
# the published crate is an unbuildable empty shell that uploads without a
# warning. Measured before the `include` keys were added: `cargo package
# -p voltaic-ast --list` emitted `Cargo.toml`, `moon.yml` and `sdx/*.sdx`, and
# nothing else. An `include` switches cargo to a filesystem walk, which does see
# the generated tree. Check 6 is the standing guard on that, and it reads the
# built `.crate` rather than the manifest because the manifest is not the thing
# that can be wrong.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

for tool in cargo jq tar; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: $tool not found. Run inside 'devenv shell', which pins it." >&2
        exit 1
    fi
done

META=$(cargo metadata --no-deps --format-version 1)

# `publish = false` reaches metadata as an empty array; a publishable crate as
# null. Anything else is an explicit registry allow-list, which this repository
# does not use.
mapfile -t PUBLISHABLE < <(jq -r '
    .packages[] | select(.publish == null) | .name
' <<<"$META" | sort)
mapfile -t ALL_MEMBERS < <(jq -r '.packages[].name' <<<"$META" | sort)

if [ "${#PUBLISHABLE[@]}" -eq 0 ]; then
    echo "error: no publishable crates found. Did every manifest gain publish = false?" >&2
    exit 1
fi

is_member() {
    local needle=$1 m
    for m in "${ALL_MEMBERS[@]}"; do [ "$m" = "$needle" ] && return 0; done
    return 1
}

is_publishable() {
    local needle=$1 m
    for m in "${PUBLISHABLE[@]}"; do [ "$m" = "$needle" ] && return 0; done
    return 1
}

# ── Check 1: one version, inherited everywhere ────────────────────────────────
mapfile -t VERSIONS < <(jq -r '.packages[] | select(.publish == null) | .version' <<<"$META" | sort -u)
if [ "${#VERSIONS[@]}" -ne 1 ]; then
    echo "ERROR: publishable crates are at ${#VERSIONS[@]} different versions: ${VERSIONS[*]}"
    echo "  → Every publishable crate must carry version.workspace = true so one bump"
    echo "    propagates atomically. docs/RELEASING.md step 1."
    exit 1
fi
WS_VERSION="${VERSIONS[0]}"

# ── The dependency graph, and checks 2-4 ──────────────────────────────────────
# Edges as "dependency dependent", one per line, for the topological sort.
EDGES=""
failed=0

check_dependencies() {
    local crate=$1 deps dep kind req
    # kind is null for a normal dependency, "dev" or "build" otherwise.
    deps=$(jq -r --arg c "$crate" '
        .packages[] | select(.name == $c) | .dependencies[]
        | "\(.name)\t\(.kind // "normal")\t\(.req)"
    ' <<<"$META")

    while IFS=$'\t' read -r dep kind req; do
        [ -n "$dep" ] || continue
        is_member "$dep" || continue

        if [ "$kind" = "dev" ]; then
            # The opposite rule, and it is not symmetry for its own sake:
            # `cargo publish` resolves a dev-dependency's version against
            # crates.io even when it also has a path, so a versioned dev-dep
            # makes a crate's publish wait for a sibling it does not need in
            # order to build. Path-only reaches metadata as req "*".
            if [ "$req" != "*" ]; then
                echo "ERROR: $crate [dev-dependencies] pins $dep at '$req'"
                echo "  → Dev-deps on workspace crates must be path-only (no version):"
                echo "    $dep = { path = \"../$dep\" }"
                failed=1
            fi
            continue
        fi

        # A runtime or build dependency. It must carry the workspace version,
        # because cargo refuses to publish a path dependency with no version.
        if [ "$req" = "*" ]; then
            echo "ERROR: $crate depends on $dep with a path and no version"
            echo "  → cargo publish refuses that. Use the workspace dependency:"
            echo "    $dep.workspace = true"
            failed=1
        elif [ "$req" != "^$WS_VERSION" ]; then
            echo "ERROR: $crate pins $dep at '$req', but the workspace is at $WS_VERSION"
            echo "  → [workspace.dependencies] in the root Cargo.toml has drifted from"
            echo "    [workspace.package] version. cargo cannot inherit a version there,"
            echo "    so the two copies are kept honest by this check. Update both."
            failed=1
        fi

        if ! is_publishable "$dep"; then
            echo "ERROR: $crate is publishable but depends on $dep, which is publish = false"
            echo "  → crates.io could never resolve $dep. Publish it, or drop the dependency."
            failed=1
            continue
        fi

        EDGES+="$dep $crate"$'\n'
    done <<<"$deps"
}

for crate in "${PUBLISHABLE[@]}"; do
    # ── Check 2: crates.io's required metadata ────────────────────────────────
    missing=$(jq -r --arg c "$crate" '
        .packages[] | select(.name == $c)
        | [ (if (.description // "") == "" then "description" else empty end),
            (if (.repository  // "") == "" then "repository"  else empty end),
            (if (.license     // "") == "" then "license"     else empty end) ]
        | join(", ")
    ' <<<"$META")
    if [ -n "$missing" ]; then
        echo "ERROR: $crate is missing required metadata: $missing"
        echo "  → crates.io rejects a publish with no description. repository and"
        echo "    license are inherited: add 'repository.workspace = true'."
        failed=1
    fi

    # ── Checks 3 and 4 ───────────────────────────────────────────────────────
    check_dependencies "$crate"
done

# ── Check 5: topological sort (Kahn), alphabetical within a rank for stability ─
declare -A INDEGREE=()
declare -A DEPENDENTS=()
for crate in "${PUBLISHABLE[@]}"; do INDEGREE["$crate"]=0; done
while read -r dep dependent; do
    [ -n "$dep" ] || continue
    DEPENDENTS["$dep"]+="$dependent "
    INDEGREE["$dependent"]=$((${INDEGREE["$dependent"]} + 1))
done <<<"$(printf '%s' "$EDGES" | sort -u)"

declare -a ORDER=()
while true; do
    # Every crate whose dependencies are all already placed, taken in
    # alphabetical order so the sequence is reproducible run to run.
    mapfile -t ready < <(
        for crate in "${PUBLISHABLE[@]}"; do
            [ "${INDEGREE["$crate"]}" -eq 0 ] && echo "$crate"
        done | sort
    )
    [ "${#ready[@]}" -eq 0 ] && break
    for crate in "${ready[@]}"; do
        ORDER+=("$crate")
        INDEGREE["$crate"]=-1
        for dependent in ${DEPENDENTS["$crate"]:-}; do
            INDEGREE["$dependent"]=$((${INDEGREE["$dependent"]} - 1))
        done
    done
done

if [ "${#ORDER[@]}" -ne "${#PUBLISHABLE[@]}" ]; then
    echo "ERROR: the intra-workspace dependency graph has a cycle."
    echo "  → Placed ${#ORDER[@]} of ${#PUBLISHABLE[@]} crates. Not placed:"
    for crate in "${PUBLISHABLE[@]}"; do
        [ "${INDEGREE["$crate"]}" -ge 0 ] && echo "      $crate"
    done
    exit 1
fi

if [[ "${1:-}" == "--print-order" ]]; then
    if [ "$failed" -ne 0 ]; then
        echo "error: the publish order is not trustworthy while the checks above fail." >&2
        exit 1
    fi
    printf '%s\n' "${ORDER[@]}"
    exit 0
fi

# ── Check 6: the archive really contains Rust ────────────────────────────────
# **One invocation, every crate.** Packaging crate by crate does not work before
# the first release: cargo resolves each sibling against the crates.io index, so
# a crate whose dependency is not published yet fails with "no matching package
# named `voltaic-diag` found, location searched: crates.io index". Given every
# `--package` at once, cargo resolves the siblings from the packages it is
# building in the same run. id_effect answers the same problem with a temporary
# `CARGO_HOME` holding a `[patch.crates-io]` that redirects each crate to its
# path; one invocation needs no patch, and is also exactly what publish.yml's
# build job runs, so this validates the real path rather than a proxy for it.
#
# **Verification is left ON**, which is the other half of why one invocation
# matters. Without `--no-verify`, cargo extracts each archive it just built into
# `target/package/<crate>-<version>/`, stands the siblings up in a
# `target/package/tmp-registry`, and *compiles the extracted tree*. That is the
# only check that proves the bytes we are about to upload build at all, as
# opposed to merely packaging — and the empty-shell failure this script's header
# describes packages perfectly cleanly. id_effect passes `--no-verify` here
# because its publish path attests the bytes instead; this repository needs the
# compile, because its `src/` is generated rather than written, and it is cheap:
# 17s for all eight crates, measured.
#
# `--fast` skips it for local iteration on the manifest checks above.
#
# `--allow-dirty` is REQUIRED, not a convenience for developers with edits in
# flight. The files the manifests include — every crate's generated `src/` — are
# untracked by design, and cargo counts an untracked file it is about to package
# as an uncommitted change. It refuses without the flag even on a spotless
# checkout at a fresh commit.
package_args=()
for crate in "${ORDER[@]}"; do package_args+=(--package "$crate"); done
if [[ "${1:-}" == "--fast" ]]; then
    package_args+=(--no-verify)
    echo "note: --fast given, skipping the compile of each packaged crate."
fi
if ! cargo package --allow-dirty --quiet "${package_args[@]}" 2>/tmp/validate-pkg.err; then
    echo "ERROR: cargo package failed"
    sed 's/^/  /' /tmp/validate-pkg.err
    exit 1
fi

for crate in "${ORDER[@]}"; do
    archive="target/package/$crate-$WS_VERSION.crate"
    if [ ! -f "$archive" ]; then
        echo "ERROR: cargo package reported success but $archive does not exist"
        failed=1
        continue
    fi

    # Listed once into a variable, and matched with a here-string rather than a
    # pipe. `tar tzf … | grep -q` is wrong under `set -o pipefail`: grep exits on
    # its first match, tar takes SIGPIPE, and the pipeline reports failure for an
    # archive that was in fact correct. It bit only the two largest archives
    # here — the small ones won the race — which is exactly the intermittent
    # shape that gets written off as flaky.
    listing=$(tar tzf "$archive")

    if ! grep -qE "/src/.*\.rs$" <<<"$listing"; then
        echo "ERROR: $archive contains no src/**/*.rs — the published crate would not build"
        echo "  → crates/$crate/src/ is generated and .gitignore'd, so cargo's git file"
        echo "    listing skips it. The manifest needs:"
        echo "      include = [\"src/**/*.rs\", \"sdx/**/*.sdx\", \"Cargo.toml\"]"
        echo "  → See the header of this script, and docs/RELEASING.md."
        failed=1
        continue
    fi

    # A `.rs.map` is a source map, useful in the repository and noise on
    # crates.io. The `*.rs` glob excludes them; this catches an `include` widened
    # to `src/**` by someone who did not know that.
    if grep -qE "\.rs\.map$" <<<"$listing"; then
        echo "ERROR: $archive ships .rs.map source maps"
        echo "  → Narrow the include back to \"src/**/*.rs\"."
        failed=1
    fi
done

if [ "$failed" -ne 0 ]; then
    exit 1
fi

echo "All ${#ORDER[@]} publishable crates pass packaging validation, at $WS_VERSION."
echo "Publish order: ${ORDER[*]}"
