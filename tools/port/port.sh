#!/usr/bin/env bash
# Rebuild one crate's `sdx/` from scratch, deterministically.
#
# Generic over the crate, where `p73build.sh` and `p74build.sh` were copies —
# eight crates remain and three of the lessons below were each learned twice
# already.
#
#   CRATE=redox-fuzz PIN=<sha> BATCHES="p75b1 p75b2" bash .tmp/port.sh
#
# **The input is a pinned commit's Rust, not the working tree's.** `src/` holds
# the hand-written Rust until the port is blessed and generated Rust
# afterwards, so a script that rebuilds `sdx/` from `src/` reads its own
# previous output on every run but the first. `PIN` must name the last commit
# that has the hand-written tree — the port's own commit untracks it.
#
# **`git restore --worktree`, not `git checkout`.** `git checkout <sha> --
# <path>` *stages* what it writes, which silently re-tracks a tree the port
# untracked, and `the_generated_trees_are_ignored` then fails on every build
# after the first.
#
# **Every batch states a destination, not a diff.** Applying an edit when the
# before-state is present and checking the after-state when it is not is what
# makes the build a fixpoint.
#
# **A batch that patches another batch lands one build late.** Batches run in
# the order given, so a fix applied to an earlier batch's *script* takes effect
# only on the next run — the build has to be run twice, and the second run is
# the one that counts. Rows 73, 76 and 78 each learned this once.
#
# **And such a batch must tolerate having already run.** Row 78's second run
# aborted on its own `assert`, because the before-state it looked for was the
# edit it had already made. State the destination, not the diff.
#
# **And the order afterwards is build -> bless -> seed-refresh.** The restore
# puts the hand-written Rust back in `src/`, so refreshing the seed before
# blessing builds it from the Rust being replaced — and the clean-clone test
# then compares the seed's correct output against a tree that is not the
# port's. It reads as a version skew and is an ordering mistake.
set -euo pipefail
cd /home/tom/Code/rust/redox

CRATE="${CRATE:?set CRATE}"
PIN="${PIN:-HEAD}"
SDX="crates/$CRATE/sdx"
SRC="crates/$CRATE/src"

git restore --source="$PIN" --worktree -- "$SRC"

rm -rf "$SDX"
for f in $(cd "$SRC" && find . -maxdepth 4 -name '*.rs' | sed 's|^\./||' | sort); do
    python3 tools/port/translate.py "crate=$CRATE" --partial "$f" >/dev/null
done
# **The counts are compared, not just printed.** A file the translator cannot
# read is *skipped*, not reported — plan 79 found `run/command/state.rs` missing
# because a wrapped attribute raised `Fail`, and the only evidence was 23 `.sdx`
# for 24 `.rs` in two numbers nothing compared. `port.rs` would catch it at
# bless time; this catches it here, where the cause is still on screen.
made=$(find "$SDX" -maxdepth 4 -name '*.sdx' | wc -l)
want=$(find "$SRC" -maxdepth 4 -name '*.rs' | wc -l)
echo "translator: $made files"
if [ "$made" -ne "$want" ]; then
    echo "error: $want .rs but $made .sdx — the translator skipped a file." >&2
    exit 1
fi

for b in ${BATCHES:-}; do
    python3 "tools/port/batches/$b.py" >/dev/null || { echo "  $b FAILED"; exit 1; }
done
[ -n "${BATCHES:-}" ] && echo "batches: applied"

# `|| true`: `grep -rl` exits 1 when nothing matches, which under `pipefail`
# would abort at exactly the moment the port is finished.
todo=$( (grep -rl 'TODO plan 68' "$SDX" --include='*.sdx' || true) | wc -l)
echo "remaining TODO: $todo files"

if [ "$todo" -eq 0 ]; then
    ./sdx-fmt.sh >/dev/null
    echo "formatted"
fi

bad=0
for f in $(cd "$SDX" && find . -maxdepth 4 -name '*.sdx' | sed 's|^\./||;s|\.sdx$||' | sort); do
    out=$(cargo run -q -p redox-cli -- build "$SDX/$f.sdx" 2>&1 | sed 's/\x1b\[[0-9;]*m//g' | grep -v '^$' || true)
    if echo "$out" | grep -q 'RDX0[0-9]'; then
        bad=$((bad + 1))
        [ "$bad" -le 3 ] && { echo "### $f"; echo "$out" | head -8; }
    fi
done
echo "files with errors: $bad"
