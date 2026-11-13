#!/usr/bin/env bash
# Emit every Voltaic identity asset from two source files.
#
# Identity and documentation site
# (`.cursor/plans/voltaic-identity-and-site.plan.md`) waves 2 and 3.
#
# **Two sources, and nothing else.** `design/geometry/pile.json` holds the
# mark's geometry once, `design/tokens/tokens.json` holds the palette once, and
# everything below is derived. A hex literal anywhere else in the repository is
# a defect, which `crates/voltaic-cli/tests/design.rs` asserts.
#
# **Why the masters carry literal colours rather than `var(--m1)`.** The
# obvious design is one SVG per cut with CSS custom properties, recoloured per
# consumer. Measured 2026-11-13: `resvg` 0.48.0 does not resolve them —
#
#   Warning (in usvg::parser::style:159): Failed to parse fill value:
#   'var(--m1)'. Fallback to black.
#
# — so every rasterised frame would have come out black, and the `:root`
# selector is skipped on top of that. Browsers do support both, which is why
# `favicon.svg` is the one output that keeps a `prefers-color-scheme` block:
# its consumer is a browser, and it is never rasterised. Everything resvg
# touches is literal.
#
# `--check` regenerates into a temporary directory and diffs, so CI can prove
# the committed assets match their sources instead of trusting that someone
# re-ran this. Same split as `sdx-fmt.sh`.
set -euo pipefail

cd "$(dirname "$0")/.."

TOKENS="design/tokens/tokens.json"
GEOM="design/geometry/pile.json"
OUT="design/masters"
CHECK=0
[ "${1:-}" = "--check" ] && CHECK=1

for f in "$TOKENS" "$GEOM"; do
    [ -f "$f" ] || { echo "error: $f is missing." >&2; exit 1; }
done
command -v jq >/dev/null || { echo "error: jq not found. Run inside 'devenv shell'." >&2; exit 1; }

if [ "$CHECK" -eq 1 ]; then
    OUT="$(mktemp -d)"
    trap 'rm -rf "$OUT"' EXIT
fi
mkdir -p "$OUT" design/tokens

# A role's hex for one theme: role "copper" "light" -> #A85C32
role() { jq -r --arg r "$1" --arg t "$2" '.roles[$r][$t]' "$TOKENS"; }

# The syntax block, read the same way `design/themes.sh` reads it. Duplicated
# across the two scripts rather than sourced from a third, because four
# one-line `jq` wrappers are less to go wrong than a shared library, and both
# scripts are checked against the same token file anyway.
srole() { jq -r --arg k "$1" '.syntax[$k].role' "$TOKENS"; }
sbold() { jq -r --arg k "$1" '.syntax[$k].bold // false' "$TOKENS"; }
sital() { jq -r --arg k "$1" '.syntax[$k].italic // false' "$TOKENS"; }
CLASSES=$(jq -r '.syntax | keys_unsorted[]' "$TOKENS")

M1_ROLE=$(jq -r '.mark.m1' "$TOKENS")
M2_ROLE=$(jq -r '.mark.m2' "$TOKENS")
MONO_ROLE=$(jq -r '.mark.mono' "$TOKENS")

# The rects of one cut, coloured by two literal values.
rects() {
    jq -r --arg cut "$1" --arg m1 "$2" --arg m2 "$3" '
      .cuts[$cut].rects[]
      | "  <rect x=\"\(.x)\" y=\"\(.y)\" width=\"\(.w)\" height=\"\(.h)\" rx=\"\(.rx)\" fill=\"\(if .role == "m1" then $m1 else $m2 end)\"/>"
    ' "$GEOM"
}

svg_open() {
    printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64" role="img" aria-label="Voltaic">\n'
    printf '  <title>Voltaic</title>\n'
}

# pile-<cut>-<variant>.svg for the three variants of each cut.
for cut in display small; do
    for variant in light dark mono; do
        case "$variant" in
            light) m1=$(role "$M1_ROLE" light); m2=$(role "$M2_ROLE" light) ;;
            dark)  m1=$(role "$M1_ROLE" dark);  m2=$(role "$M2_ROLE" dark) ;;
                # `currentColor` rather than a hex: the single-colour cut is for
                # embedding in HTML and editor chrome, where it should inherit. It
                # is never handed to resvg, so resvg's lack of support is moot.
            mono)  m1="currentColor"; m2="currentColor" ;;
        esac
        {
            svg_open
            rects "$cut" "$m1" "$m2"
            printf '</svg>\n'
        } > "$OUT/pile-$cut-$variant.svg"
    done
done

# favicon.svg — the SMALL cut, because a tab icon is 16px and the five-plate
# cut smears there. The one output with a media query, per the header.
{
    svg_open
    printf '  <style>\n'
    printf '    .m1 { fill: %s }\n' "$(role "$M1_ROLE" light)"
    printf '    .m2 { fill: %s }\n' "$(role "$M2_ROLE" light)"
    printf '    @media (prefers-color-scheme: dark) {\n'
    printf '      .m1 { fill: %s }\n' "$(role "$M1_ROLE" dark)"
    printf '      .m2 { fill: %s }\n' "$(role "$M2_ROLE" dark)"
    printf '    }\n'
    printf '  </style>\n'
    jq -r '
      .cuts.small.rects[]
      | "  <rect class=\"\(.role)\" x=\"\(.x)\" y=\"\(.y)\" width=\"\(.w)\" height=\"\(.h)\" rx=\"\(.rx)\"/>"
    ' "$GEOM"
    printf '</svg>\n'
} > "$OUT/favicon.svg"

# tokens.css and tokens.toml — the same palette, for the two consumers that
# cannot read JSON at the point of use.
TOKENS_CSS="$OUT/../tokens/tokens.css"
TOKENS_TOML="$OUT/../tokens/tokens.toml"
# The book is a second consumer of the same palette, and mdBook reads its extra
# stylesheets from inside the book directory — so the tokens are emitted there
# too rather than copied by a build step that could be forgotten.
BOOK_CSS="www/theme/tokens.css"
[ "$CHECK" -eq 1 ] && { TOKENS_CSS="$OUT/tokens.css"; TOKENS_TOML="$OUT/tokens.toml"; BOOK_CSS="$OUT/book-tokens.css"; }

{
    printf '/* Generated by design/build.sh from design/tokens/tokens.json. Do not edit. */\n'
    printf ':root {\n'
    jq -r '.roles | to_entries[] | "  --voltaic-\(.key): \(.value.light);"' "$TOKENS"
    printf '}\n'
    printf '@media (prefers-color-scheme: dark) {\n'
    printf '  :root:not([data-theme="light"]) {\n'
    jq -r '.roles | to_entries[] | "    --voltaic-\(.key): \(.value.dark);"' "$TOKENS"
    printf '  }\n}\n'
    printf ':root[data-theme="dark"] {\n'
    jq -r '.roles | to_entries[] | "  --voltaic-\(.key): \(.value.dark);"' "$TOKENS"
    printf '}\n'
} > "$TOKENS_CSS"

[ -d "$(dirname "$BOOK_CSS")" ] && cp "$TOKENS_CSS" "$BOOK_CSS"

# The book's syntax colours. `tools/mdbook-voltaic` emits a `tok-<class>` span
# per token for the ten classes in the `syntax` block, and this is where those
# classes get their colour — from the same roles the five editor themes read,
# so the book and an editor cannot disagree about what a keyword looks like.
#
# mdBook scopes its palette on `html.<theme>`: `light` is Voltaic light and
# `navy` is Voltaic dark, which is the remap `www/theme/voltaic.css` makes.
SYNTAX_CSS="www/theme/syntax.css"
[ "$CHECK" -eq 1 ] && SYNTAX_CSS="$OUT/syntax.css"
{
    printf '/* Generated by design/build.sh from design/tokens/tokens.json. Do not edit. */\n'
    for theme in light dark; do
        if [ "$theme" = light ]; then
            printf 'html.light, html:not(.js) {\n'
        else
            printf 'html.navy {\n'
        fi
        for class in $CLASSES; do
            printf '  --tok-%s: %s;\n' "$class" "$(role "$(srole "$class")" "$theme")"
        done
        printf '}\n'
    done
    for class in $CLASSES; do
        printf '.tok-%s { color: var(--tok-%s);' "$class" "$class"
        [ "$(sbold "$class")" = true ] && printf ' font-weight: 600;'
        [ "$(sital "$class")" = true ] && printf ' font-style: italic;'
        printf ' }\n'
    done
} > "$SYNTAX_CSS"

{
    printf '# Generated by design/build.sh from design/tokens/tokens.json. Do not edit.\n'
    jq -r '.roles | to_entries[] | "\n[\(.key)]\nlight = \"\(.value.light)\"\ndark = \"\(.value.dark)\""' "$TOKENS"
} > "$TOKENS_TOML"

if [ "$CHECK" -eq 1 ]; then
    fail=0
    for f in "$OUT"/*.svg; do
        b=$(basename "$f")
        if ! diff -q "$f" "design/masters/$b" >/dev/null 2>&1; then
            echo "design: design/masters/$b is not what design/build.sh emits." >&2
            fail=1
        fi
    done
    for pair in "tokens.css:design/tokens/tokens.css" "tokens.toml:design/tokens/tokens.toml" "book-tokens.css:www/theme/tokens.css" "syntax.css:www/theme/syntax.css"; do
        if ! diff -q "$OUT/${pair%%:*}" "${pair##*:}" >/dev/null 2>&1; then
            echo "design: ${pair##*:} is not what design/build.sh emits." >&2
            fail=1
        fi
    done
    [ "$fail" -eq 0 ] && echo "design: every generated asset matches its sources."
    exit "$fail"
fi

echo "design: emitted $(ls design/masters/*.svg | wc -l) masters, tokens.css and tokens.toml."
