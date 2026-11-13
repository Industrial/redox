#!/usr/bin/env bash
# Publish one crate to crates.io (`.github/workflows/publish.yml`).
#
# Safe to rerun after a partial publish: a version already in the crates.io index
# is skipped before cargo runs, so a workflow re-run resumes at the first crate
# that did not make it rather than failing on the ones that did.
#
# Taken from Industrial/id_effect's script of the same name, which is already
# generic over the package. The one thing worth knowing that is specific to this
# repository: the `.crate` being uploaded contains Rust that was *generated* from
# `.sdx` by `./bootstrap.sh`, so the digest check below is load-bearing in a way
# it is not there. If the seed's output is not byte-reproducible, two jobs
# packaging "the same tree" produce different bytes and this is what catches it.
# `publish.yml`'s `determinism` step exists to fail earlier and more legibly.
#
# Usage: publish-crate.sh <package> [extra cargo publish args...]
# Env:
#   VERSION               the version being published (required)
#   DRY_RUN=true          pass --dry-run to cargo publish
#   PLAN_ONLY=true        print "publish" or "skip" from the crates.io index and
#                         exit (read-only); a crate never published is planned as
#                         a new crate
#   SHA256SUMS            optional sha256sum file of the attested .crate files:
#                         the .crate that cargo publish uploaded must match its
#                         line
#   CRATES_IO_TOKEN       the crates.io token (the CARGO_REGISTRY_TOKEN secret);
#                         handed to cargo on stdin by `cargo login`, never on the
#                         command line, where it would reach the process list
set -euo pipefail

pkg=$1
shift
version=${VERSION:?set VERSION to the version being published}

# Sparse-index path of a crate.
# https://doc.rust-lang.org/cargo/reference/registry-index.html
index_url() {
    local name=${1,,}
    case ${#name} in
        1) echo "https://index.crates.io/1/$name" ;;
        2) echo "https://index.crates.io/2/$name" ;;
        3) echo "https://index.crates.io/3/${name:0:1}/$name" ;;
        *) echo "https://index.crates.io/${name:0:2}/${name:2:2}/$name" ;;
    esac
}

# 0 when $pkg $version is in the index. A crate never published is a 404, which
# curl -f turns into a non-zero exit: "not yet", not an error.
published() {
    local body
    body=$(curl -fsS --retry 3 --retry-delay 5 "$(index_url "$pkg")" 2>/dev/null) || return 1
    grep -qF "\"vers\":\"$version\"" <<<"$body"
}

# 0 when the crate has no version on crates.io at all.
new_crate() {
    local status
    status=$(curl -sS -o /dev/null -w '%{http_code}' --retry 3 --retry-delay 5 "$(index_url "$pkg")")
    [ "$status" = "404" ]
}

if published; then
    echo "skip    $pkg $version (already on crates.io)"
    exit 0
fi
if [ "${PLAN_ONLY:-false}" = "true" ]; then
    if new_crate; then
        echo "publish $pkg $version (new crate)"
        exit 0
    fi
    echo "publish $pkg $version"
    exit 0
fi

# `--allow-dirty` is not optional here, and it is not papering over a dirty
# checkout. The files the manifests tell cargo to include — every crate's
# generated `src/` — are untracked by design, and cargo counts an untracked file
# it is about to package as an uncommitted change. Measured on a clean checkout
# at a fresh commit: "error: 5 files in the working directory contain changes
# that were not yet committed into git", naming voltaic-diag's five generated
# files.
#
# What the flag gives up is the refusal to package genuine uncommitted edits.
# publish.yml restores that separately, with a step asserting
# `git status --porcelain` is empty before this script runs — that command omits
# ignored files, so it sees real dirt and not the generated trees.
args=(--package "$pkg" --no-verify --allow-dirty "$@")
if [ "${DRY_RUN:-false}" = "true" ]; then
    args+=(--dry-run)
else
    : "${CRATES_IO_TOKEN:?set CRATES_IO_TOKEN (the CARGO_REGISTRY_TOKEN secret) for a real publish}"
    cargo login <<<"$CRATES_IO_TOKEN"
fi

set +e
output=$(cargo publish "${args[@]}" 2>&1)
code=$?
set -e
echo "$output"
if [ "$code" -ne 0 ]; then
    if grep -qiE "already (uploaded|exists)" <<<"$output"; then
        echo "$pkg already on crates.io — skipping"
        exit 0
    fi
    if grep -qiE "required permissions|403|not authorized|trusted publish" <<<"$output"; then
        echo "::error::$pkg publish denied — the CARGO_REGISTRY_TOKEN secret (environment crates-io) may not publish it"
        exit 1
    fi
    if grep -qiE "readme .* does not appear to exist" <<<"$output"; then
        echo "::error::$pkg missing readme declared in Cargo.toml"
        exit 1
    fi
    # crates.io rate-limits new-crate publishes, and the first release here
    # creates eight crates in one run, which is exactly the shape that trips it.
    # Nothing to fix and nothing lost: every crate already uploaded is skipped on
    # the next run, so re-running the workflow resumes at the first one missing.
    # Said plainly, because the raw message is a 429 with a retry-after and reads
    # like a broken token.
    if grep -qiE "429|too many requests|rate limit" <<<"$output"; then
        echo "::error::$pkg hit a crates.io rate limit. This is not a configuration problem."
        echo "::error::Re-run the publish job: crates already on crates.io are skipped, so it resumes here."
        exit 1
    fi
    if grep -qiE "no matching package named|didn't match|failed to select a version for the requirement" <<<"$output"; then
        echo "::error::$pkg publish blocked — a dependency is not on crates.io yet."
        echo "::error::Check the order from scripts/validate-crate-packaging.sh --print-order, and that dev-deps are path-only."
        exit 1
    fi
    exit "$code"
fi

# The bytes cargo uploaded must be the attested ones. cargo publish leaves them
# in target/package/tmp-crate/ (current cargo) or target/package/ (older cargo).
if [ -n "${SHA256SUMS:-}" ]; then
    file="$pkg-$version.crate"
    dir="${CARGO_TARGET_DIR:-target}/package"
    uploaded="$dir/tmp-crate/$file"
    if [ ! -f "$uploaded" ]; then
        uploaded="$dir/$file"
    fi
    expected=$(awk -v f="$file" '$2 == f || $2 == "*" f { print $1 }' "$SHA256SUMS")
    actual=$(sha256sum "$uploaded" | cut -d' ' -f1)
    if [ "$expected" != "$actual" ]; then
        echo "::error::$file sha256 $actual is not the attested ${expected:-<missing>}"
        echo "::error::If the digests differ, the generated Rust is not byte-reproducible between jobs."
        echo "::error::See docs/RELEASING.md, 'Reproducibility and the seed'. Bump the patch version rather than re-running."
        exit 1
    fi
    echo "$file matches the attested sha256 $actual"
fi

# cargo publish waits for the index itself; poll as well so a dependent crate
# published moments later never races it.
if [ "${DRY_RUN:-false}" != "true" ]; then
    for _ in $(seq 1 30); do
        if published; then
            exit 0
        fi
        sleep 10
    done
    echo "::error::$pkg $version is not in the crates.io index after 5 minutes"
    exit 1
fi
