#!/usr/bin/env bash
# Render the platform icon package from the two SVG masters.
#
# Identity and documentation site
# (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 3.
#
# Everything here lands in `design/dist/`, which is **not** tracked: nothing
# outside CI reads it, it is large, and it is byte-reproducible from
# `design/masters/`. Run `./design/build.sh` first — this script reads its
# output and does not regenerate it.
#
# **Each frame is rendered at its own size from the right cut.** The usual
# mistake is to render 1024 once and downsample, which hands a 16px favicon the
# five-plate cut and produces a smear. The rule is one line, `cut_for`, and it
# switches to the three-plate cut at 32px and below.
#
# **Wrappers are synthesised rather than composited.** resvg is a renderer, not
# an image editor, so a padded or ground-bearing variant is built by writing a
# new SVG that insets the master's rects into a transformed group. That also
# keeps every output a single self-contained file with no external references,
# which `<use href="...">` across files would not.
set -euo pipefail

cd "$(dirname "$0")/.."

MASTERS="design/masters"
TOKENS="design/tokens/tokens.json"
# Overridable so `crates/voltaic-cli/tests/design.rs` can verify a fresh render
# without disturbing the working tree.
DIST="${VOLTAIC_DESIGN_DIST:-design/dist}"

for t in resvg oxipng icotool png2icns jq; do
    command -v "$t" >/dev/null || { echo "error: $t not found. Run inside 'devenv shell'." >&2; exit 1; }
done
[ -f "$MASTERS/pile-display-light.svg" ] || { echo "error: run ./design/build.sh first." >&2; exit 1; }

# resvg keeps its own font database and does not read this shell's fontconfig
# dirs, so a `font-family="Archivo"` it cannot resolve becomes
#
#   Warning (in usvg::text:143): No match for '"Archivo"' font-family.
#
# and then renders *nothing* — exit 0, a valid PNG, and no wordmark. Measured
# 2026-11-13: the first run of this script produced three banners with zero ink
# pixels and reported success. So the file is located explicitly and passed in,
# and `render` below treats any resvg warning as fatal.
ARCHIVO=$(fc-list 2>/dev/null | grep -i 'Archivo\[' | head -1 | cut -d: -f1 || true)
if [ -z "${ARCHIVO:-}" ] || [ ! -f "$ARCHIVO" ]; then
    echo "error: Archivo not found. It is pinned in devenv.nix; run inside 'devenv shell'." >&2
    exit 1
fi

role() { jq -r --arg r "$1" --arg t "$2" '.roles[$r][$t]' "$TOKENS"; }
COPPER_L=$(role copper light); ZINC_L=$(role zinc light)
COPPER_D=$(role copper dark);  ZINC_D=$(role zinc dark)
INK_L=$(role ink light);       INK_D=$(role ink dark)
GROUND_L=$(role ground light); GROUND_D=$(role ground dark)

rm -rf "$DIST"
mkdir -p "$DIST"/{web,ios/AppIcon.appiconset,macos,windows/tiles,linux/scalable/apps,social,readme}

# Below 32px the five-plate cut smears; this is the whole optical-size system.
cut_for() { [ "$1" -le 32 ] && echo small || echo display; }

# The master's rects, indented into a group, with currentColor resolved so the
# single-colour cut can be rasterised at all.
rects_of() {
    sed -n 's|^  \(<rect.*/>\)$|    \1|p' "$MASTERS/pile-$1-$2.svg" \
        | sed "s|currentColor|${3:-$INK_L}|g"
}

# wrap <canvas> <bg|none> <radius> <cut> <variant> <fraction> [mono-ink]
wrap() {
    local c=$1 bg=$2 r=$3 cut=$4 variant=$5 frac=$6 ink=${7:-}
    local m o s
    m=$(awk -v c="$c" -v f="$frac" 'BEGIN{printf "%.4f", c*f}')
    o=$(awk -v c="$c" -v m="$m" 'BEGIN{printf "%.4f", (c-m)/2}')
    s=$(awk -v m="$m" 'BEGIN{printf "%.8f", m/64}')
    printf '<svg xmlns="http://www.w3.org/2000/svg" width="%s" height="%s" viewBox="0 0 %s %s">\n' "$c" "$c" "$c" "$c"
    [ "$bg" != "none" ] && printf '  <rect width="%s" height="%s" rx="%s" fill="%s"/>\n' "$c" "$c" "$r" "$bg"
    printf '  <g transform="translate(%s %s) scale(%s)">\n' "$o" "$o" "$s"
    rects_of "$cut" "$variant" "$ink"
    printf '  </g>\n</svg>\n'
}

# render <out.png> <width> <height> <svg> — the single path to resvg.
#
# **A resvg warning is a failure here.** It exits 0 on an unmatched font and on
# an unparseable fill, having silently dropped what it could not draw, so the
# only way to notice is to read stderr and refuse it. The first version of this
# script redirected it to /dev/null and shipped three empty banners.
render() {
    local out=$1 w=$2 h=$3 src=$4 err
    err=$(resvg --use-font-file "$ARCHIVO" --width "$w" --height "$h" "$src" "$out" 2>&1 >/dev/null) || {
        echo "error: resvg failed on $src" >&2; echo "$err" >&2; exit 1
    }
    if [ -n "$err" ]; then
        echo "error: resvg warned while drawing $src, so the output is incomplete:" >&2
        echo "$err" >&2
        exit 1
    fi
    oxipng --quiet --opt 4 --strip safe "$out" >/dev/null 2>&1 || true
}

# png <out> <size> <svg> — square convenience wrapper.
png() { render "$1" "$2" "$2" "$3"; }

tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

# ── web ───────────────────────────────────────────────────────────────────
cp "$MASTERS/favicon.svg" "$DIST/web/favicon.svg"
for s in 16 32 48; do
    wrap "$s" none 0 "$(cut_for "$s")" light 1.0 > "$tmp/fav-$s.svg"
    png "$tmp/fav-$s.png" "$s" "$tmp/fav-$s.svg"
done
icotool -c -o "$DIST/web/favicon.ico" "$tmp/fav-16.png" "$tmp/fav-32.png" "$tmp/fav-48.png"

# Apple wants no transparency and no pre-rounding on the touch icon.
wrap 180 "$GROUND_L" 0 display light 0.64 > "$tmp/touch.svg"
png "$DIST/web/apple-touch-icon.png" 180 "$tmp/touch.svg"

for s in 192 512; do
    wrap "$s" "$GROUND_L" 0 display light 0.68 > "$tmp/icon-$s.svg"
    png "$DIST/web/icon-$s.png" "$s" "$tmp/icon-$s.svg"
done
# Maskable: Android crops to a circle inscribed in the inner 80%, so the mark
# gets 56% of the canvas rather than 68% — anything larger loses its ends.
wrap 512 "$GROUND_L" 0 display light 0.56 > "$tmp/maskable.svg"
png "$DIST/web/maskable-512.png" 512 "$tmp/maskable.svg"
# Monochrome: one ink on transparency, for Android's themed icons.
wrap 512 none 0 display mono 0.56 "$INK_L" > "$tmp/monochrome.svg"
png "$DIST/web/monochrome-512.png" 512 "$tmp/monochrome.svg"

jq -n --arg copper "$COPPER_L" --arg ground "$GROUND_L" '{
  name: "Voltaic", short_name: "Voltaic",
  description: "A source-to-source compiler: .sdx in, ordinary Rust out.",
  start_url: "/voltaic/", display: "standalone",
  background_color: $ground, theme_color: $copper,
  icons: [
    { src: "icon-192.png", sizes: "192x192", type: "image/png" },
    { src: "icon-512.png", sizes: "512x512", type: "image/png" },
    { src: "maskable-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
    { src: "monochrome-512.png", sizes: "512x512", type: "image/png", purpose: "monochrome" }
  ]
}' > "$DIST/web/manifest.webmanifest"

# ── iOS ───────────────────────────────────────────────────────────────────
# One 1024 master; Xcode generates the rest and applies the squircle itself.
wrap 1024 "$GROUND_L" 0 display light 0.66 > "$tmp/ios.svg"
png "$DIST/ios/AppIcon.appiconset/icon-1024.png" 1024 "$tmp/ios.svg"
wrap 1024 "$GROUND_D" 0 display dark 0.66 > "$tmp/ios-dark.svg"
png "$DIST/ios/AppIcon.appiconset/icon-1024-dark.png" 1024 "$tmp/ios-dark.svg"
wrap 1024 none 0 display mono 0.66 "$INK_L" > "$tmp/ios-tinted.svg"
png "$DIST/ios/AppIcon.appiconset/icon-1024-tinted.png" 1024 "$tmp/ios-tinted.svg"
jq -n '{
  images: [
    { filename: "icon-1024.png", idiom: "universal", platform: "ios", size: "1024x1024" },
    { filename: "icon-1024-dark.png", idiom: "universal", platform: "ios", size: "1024x1024",
      appearances: [ { appearance: "luminosity", value: "dark" } ] },
    { filename: "icon-1024-tinted.png", idiom: "universal", platform: "ios", size: "1024x1024",
      appearances: [ { appearance: "luminosity", value: "tinted" } ] }
  ],
  info: { author: "xcode", version: 1 }
}' > "$DIST/ios/AppIcon.appiconset/Contents.json"

# ── macOS ─────────────────────────────────────────────────────────────────
# Pre-rounded, because an .icns is drawn as given. 22.4% is the squircle
# approximation; macOS also insets the art inside its grid, hence 0.56.
icns_args=()
for s in 16 32 64 128 256 512 1024; do
    r=$(awk -v s="$s" 'BEGIN{printf "%.2f", s*0.224}')
    wrap "$s" "$GROUND_L" "$r" "$(cut_for "$s")" light 0.56 > "$tmp/mac-$s.svg"
    png "$tmp/mac-$s.png" "$s" "$tmp/mac-$s.svg"
    icns_args+=("$tmp/mac-$s.png")
done
if ! png2icns "$DIST/macos/voltaic.icns" "${icns_args[@]}" >/dev/null 2>&1; then
    echo "error: png2icns rejected one or more sizes, so the .icns is partial." >&2
    exit 1
fi

# ── Windows ───────────────────────────────────────────────────────────────
ico_args=()
for s in 16 24 32 48 64 256; do
    wrap "$s" none 0 "$(cut_for "$s")" light 1.0 > "$tmp/win-$s.svg"
    png "$tmp/win-$s.png" "$s" "$tmp/win-$s.svg"
    ico_args+=("$tmp/win-$s.png")
done
icotool -c -o "$DIST/windows/voltaic.ico" "${ico_args[@]}"
# MSIX tiles are square-cornered and full-bleed on the brand colour.
for spec in "Square44x44:44" "Square71x71:71" "Square150x150:150" "Square310x310:310" "StoreLogo:50"; do
    name=${spec%%:*}; s=${spec##*:}
    wrap "$s" "$COPPER_L" 0 "$(cut_for "$s")" light 0.62 > "$tmp/tile.svg"
    png "$DIST/windows/tiles/$name.png" "$s" "$tmp/tile.svg"
done

# ── Linux ─────────────────────────────────────────────────────────────────
for s in 16 24 32 48 64 128 256 512; do
    mkdir -p "$DIST/linux/${s}x${s}/apps"
    wrap "$s" none 0 "$(cut_for "$s")" light 1.0 > "$tmp/lin-$s.svg"
    png "$DIST/linux/${s}x${s}/apps/voltaic.png" "$s" "$tmp/lin-$s.svg"
done
cp "$MASTERS/pile-display-light.svg" "$DIST/linux/scalable/apps/voltaic.svg"

# ── social preview and README headers ─────────────────────────────────────
# Archivo is pinned in devenv.nix precisely so this wordmark is the identity's
# and not fontconfig's default.
banner() {
    local w=$1 h=$2 bg=$3 ink=$4 cut=$5 variant=$6 out=$7 mono=${8:-}
    local mark=$(awk -v h="$h" 'BEGIN{printf "%.2f", h*0.42}')
    local mx=$(awk -v h="$h" 'BEGIN{printf "%.2f", h*0.30}')
    local my=$(awk -v h="$h" -v m="$mark" 'BEGIN{printf "%.2f", (h-m)/2}')
    local s=$(awk -v m="$mark" 'BEGIN{printf "%.8f", m/64}')
    local tx=$(awk -v mx="$mx" -v m="$mark" 'BEGIN{printf "%.2f", mx+m+m*0.42}')
    local fs=$(awk -v m="$mark" 'BEGIN{printf "%.2f", m*0.40}')
    local ty=$(awk -v h="$h" -v f="$fs" 'BEGIN{printf "%.2f", h/2 + f*0.355}')
    local sy=$(awk -v h="$h" -v f="$fs" 'BEGIN{printf "%.2f", h/2 + f*1.30}')
    {
        printf '<svg xmlns="http://www.w3.org/2000/svg" width="%s" height="%s" viewBox="0 0 %s %s">\n' "$w" "$h" "$w" "$h"
        printf '  <rect width="%s" height="%s" fill="%s"/>\n' "$w" "$h" "$bg"
        printf '  <g transform="translate(%s %s) scale(%s)">\n' "$mx" "$my" "$s"
        rects_of "$cut" "$variant" "$mono"
        printf '  </g>\n'
        printf '  <text x="%s" y="%s" font-family="Archivo" font-size="%s" font-weight="800" letter-spacing="%.2f" fill="%s">VOLTAIC</text>\n' \
            "$tx" "$ty" "$fs" "$(awk -v f="$fs" 'BEGIN{printf "%.2f", f*0.14}')" "$ink"
        printf '  <text x="%s" y="%s" font-family="Archivo" font-size="%.2f" font-weight="500" fill="%s" opacity="0.72">.sdx in, ordinary Rust out.</text>\n' \
            "$tx" "$sy" "$(awk -v f="$fs" 'BEGIN{printf "%.2f", f*0.30}')" "$ink"
        printf '</svg>\n'
    } > "$tmp/banner.svg"
    render "$out" "$w" "$h" "$tmp/banner.svg"
}

banner 1280 640 "$GROUND_L" "$INK_L" display light "$DIST/social/social-preview.png"
banner 1280 320 "$GROUND_L" "$INK_L" display light "$DIST/readme/header-light.png"
banner 1280 320 "$GROUND_D" "$INK_D" display dark  "$DIST/readme/header-dark.png"

echo "rasterize: $(find "$DIST" -type f | wc -l) files in $DIST"
