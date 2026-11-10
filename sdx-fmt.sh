#!/usr/bin/env bash
# Format — or `--check` — every `.sdx` that is meant to be canonical.
#
# Plan 73. `redox fmt` existed from plan 34 and was in **no** gate: `moon run
# :format` is treefmt, which has no `.sdx` formatter at all, and the only
# `fmt --check` among the moon tasks is `cargo fmt --check` for Rust. So the
# project's own formatter had never been run over the project's own `.sdx`
# except by hand. Two things followed: `crates/redox-ast/sdx` had 50 of 53
# files non-canonical, and a formatter bug that **deleted a member's
# visibility** survived unnoticed (plan 73 §10).
#
# **One definition of the corpus, used twice.** `moon run :sdx-format` and
# `:sdx-format-check` both call this script, because two task bodies would be
# two exclusion lists free to drift — the one-predicate-two-positions defect
# plan 34 §10.31 found five times and plans 61, 69 and 70 each found again.
#
#     ./sdx-fmt.sh            # format in place
#     ./sdx-fmt.sh --check    # fail if anything is not canonical
set -euo pipefail

cd "$(dirname "$0")"

# **What is excluded, and why.** Measured rather than assumed: every tracked
# `.sdx` was run through `fmt --check` and classified, which gave 181
# canonical, 59 that do not parse, and 5 that parse but are deliberately
# unformatted. Every one of the 64 falls in this list, and the list contains
# nothing else.
#
# - `crates/redox-cli/tests/ui/` (48) — diagnostics fixtures. Most are
#   *supposed* not to parse; that is what their `.stderr` records.
# - `tests/unreachable/` (5) — constructs the grammar does not reach, kept as
#   witnesses that it does not reach them.
# - `crates/redox-fmt/tests/fmt/` (7) — the formatter's **own** fixture pairs:
#   an input beside its `.canonical.sdx` expectation. Formatting the inputs
#   would delete the test.
# - `crates/redox-mcp/tests/fixtures/` (2) — includes `unformatted.sdx`, whose
#   whole purpose is to be unformatted so the MCP format tool has something to
#   change.
# - `examples/lexical.sdx` — the one example about **lexing**, containing
#   constructs the parser rejects such as `[0, ...rest, 9]`. Plan 64 §2
#   already names it as the exception.
# - `crates/redox-cli/tests/check/syntax.sdx` — a deliberate parse error. The
#   other nine files in that directory are canonical and are **not** excluded,
#   which is why this is a file and not a directory.
EXCLUDE='^(crates/redox-cli/tests/ui|tests/unreachable|crates/redox-fmt/tests/fmt|crates/redox-mcp/tests/fixtures)/|^examples/lexical\.sdx$|^crates/redox-cli/tests/check/syntax\.sdx$'

mapfile -t FILES < <(git ls-files '*.sdx' | grep -vE "$EXCLUDE")

if [ "${#FILES[@]}" -eq 0 ]; then
    echo "error: no .sdx files matched; the exclusion pattern or the corpus moved." >&2
    exit 1
fi

# A floor, so a broken glob cannot pass by formatting nothing. The corpus was
# 181 files at plan 73; this guards the order of magnitude, not the count.
if [ "${#FILES[@]}" -lt 100 ]; then
    echo "error: only ${#FILES[@]} .sdx files matched, expected well over 100." >&2
    exit 1
fi

exec cargo run -q -p redox-cli -- fmt "$@" "${FILES[@]}"
