#!/usr/bin/env bash
# Make the built book legible to crawlers and language models.
#
# Identity and documentation site
# (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 8.
#
# Run after `mdbook build`. Four things:
#
#   1. per-page `<link rel="canonical">`, OpenGraph, Twitter card and JSON-LD
#   2. `sitemap.xml` and `robots.txt`
#   3. `llms.txt` (the index) and `llms-full.txt` (the whole book, one file)
#   4. the icon package's web assets, copied in from `design/dist/`
#
# **Post-processing rather than `theme/head.hbs`.** The static tags would fit in
# a handlebars partial, but a canonical URL needs the page's own path with
# `.md` turned into `.html`, and mdBook exposes no helper for that. A partial
# that got the canonical wrong on every page would be worse than no canonical.
#
# **`llms-full.txt` resolves includes itself.** It is built from `src/`, not
# from the HTML, so it carries Markdown rather than stripped tags — but that
# means `{{#include}}` has to be expanded here, or every sample would appear in
# it as the literal directive. That expansion is the one piece of real work in
# this script.
set -euo pipefail

cd "$(dirname "$0")"

SITE_URL=$(sed -n 's/^site-url = "\(.*\)"$/\1/p' book.toml)
ORIGIN="https://industrial.github.io"
BASE="${ORIGIN}${SITE_URL%/}"
HTML="${1:-book/html}"
SRC="src"
DIST="../design/dist/web"
DESC="A source-to-source compiler: .sdx in, ordinary Rust out."

[ -d "$HTML" ] || { echo "error: $HTML does not exist. Run 'mdbook build' first." >&2; exit 1; }

# ── 1. the icon package's web assets ──────────────────────────────────────
if [ -d "$DIST" ]; then
    cp "$DIST/favicon.svg" "$DIST/favicon.ico" "$DIST/apple-touch-icon.png" \
        "$DIST/icon-192.png" "$DIST/icon-512.png" "$DIST/maskable-512.png" \
        "$DIST/monochrome-512.png" "$DIST/manifest.webmanifest" "$HTML/"
    mkdir -p "$HTML/social"
    cp "../design/dist/social/social-preview.png" "$HTML/social/"
else
    echo "warn: $DIST is absent, so the site has mdBook's default favicon." >&2
    echo "      Run ./design/rasterize.sh first." >&2
fi

# ── 2. per-page head injection ────────────────────────────────────────────
# `title` is read out of the page rather than passed in, so a renamed chapter
# cannot end up with a stale OpenGraph title.
pages=0
while IFS= read -r -d '' page; do
    rel=${page#"$HTML"/}
    url="$BASE/$rel"
    title=$(sed -n 's|.*<title>\(.*\)</title>.*|\1|p' "$page" | head -1)
    [ -z "$title" ] && title="Voltaic"
    md="${rel%.html}.md"

    # One temporary file, one substitution: injecting line by line with sed -i
    # over a 40 KB page is slower than rewriting it once.
    {
        printf '<link rel="canonical" href="%s">\n' "$url"
        printf '<meta property="og:type" content="article">\n'
        printf '<meta property="og:site_name" content="Voltaic">\n'
        printf '<meta property="og:title" content="%s">\n' "$title"
        printf '<meta property="og:description" content="%s">\n' "$DESC"
        printf '<meta property="og:url" content="%s">\n' "$url"
        printf '<meta property="og:image" content="%s/social/social-preview.png">\n' "$BASE"
        printf '<meta name="twitter:card" content="summary_large_image">\n'
        printf '<meta name="twitter:title" content="%s">\n' "$title"
        printf '<meta name="twitter:description" content="%s">\n' "$DESC"
        printf '<meta name="twitter:image" content="%s/social/social-preview.png">\n' "$BASE"
        printf '<link rel="manifest" href="%s/manifest.webmanifest">\n' "$BASE"
        printf '<link rel="apple-touch-icon" href="%s/apple-touch-icon.png">\n' "$BASE"
        printf '<link rel="alternate" type="text/markdown" href="%s/%s">\n' "$BASE" "$md"
        printf '<script type="application/ld+json">'
        printf '{"@context":"https://schema.org","@type":"TechArticle",'
        printf '"headline":"%s","description":"%s","url":"%s",' "$title" "$DESC" "$url"
        printf '"inLanguage":"en","isPartOf":{"@type":"Book","name":"Voltaic"},'
        printf '"about":{"@type":"ComputerLanguage","name":"Voltaic"},'
        printf '"license":"https://opensource.org/licenses/MIT"}'
        printf '</script>\n'
    } > /tmp/vlt-head.$$

    python3 - "$page" /tmp/vlt-head.$$ <<'PY'
import io, sys
page, head = sys.argv[1], sys.argv[2]
body = io.open(page, encoding="utf-8").read()
inject = io.open(head, encoding="utf-8").read()
if "rel=\"canonical\"" in body:
    sys.exit(0)
i = body.lower().find("</head>")
if i < 0:
    sys.exit(0)
io.open(page, "w", encoding="utf-8").write(body[:i] + inject + body[i:])
PY
    rm -f /tmp/vlt-head.$$
    pages=$((pages + 1))
done < <(find "$HTML" -name '*.html' -not -name '404.html' -not -name 'print.html' -print0)

# ── 3. the Markdown sources, served beside the HTML ───────────────────────
# So the `text/markdown` alternate above resolves, and a reader that wants the
# text without the chrome can take exactly one page.
while IFS= read -r -d '' md; do
    rel=${md#"$SRC"/}
    mkdir -p "$HTML/$(dirname "$rel")"
    cp "$md" "$HTML/$rel"
done < <(find "$SRC" -name '*.md' -print0)

# ── 4. sitemap and robots ─────────────────────────────────────────────────
{
    printf '<?xml version="1.0" encoding="UTF-8"?>\n'
    printf '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
    find "$HTML" -name '*.html' -not -name '404.html' -not -name 'print.html' -print0 \
        | sort -z | while IFS= read -r -d '' p; do
        printf '  <url><loc>%s/%s</loc></url>\n' "$BASE" "${p#"$HTML"/}"
    done
    printf '</urlset>\n'
} > "$HTML/sitemap.xml"

{
    printf 'User-agent: *\n'
    printf 'Allow: /\n\n'
    printf '# Everything here is meant to be read, by anyone and anything.\n'
    printf 'Sitemap: %s/sitemap.xml\n' "$BASE"
    printf '\n# The whole book as one Markdown file, for language models:\n'
    printf '# %s/llms-full.txt\n' "$BASE"
} > "$HTML/robots.txt"

# ── 5. llms.txt and llms-full.txt ─────────────────────────────────────────
# Chapter order comes from SUMMARY.md, which is the one place it is written.
chapters=$(grep -oE '\]\([^)]+\.md\)' "$SRC/SUMMARY.md" | sed 's|^](||; s|)$||')

{
    printf '# Voltaic\n\n'
    printf '> %s\n\n' "$DESC"
    printf 'Voltaic has no type checker, no borrow checker and no exhaustiveness\n'
    printf 'checker, because Rust has all three. It changes the surface, not the\n'
    printf 'semantics. Every code sample in this documentation is included from the\n'
    printf 'repository'"'"'s `examples/` corpus, which CI compiles and runs against a\n'
    printf 'recorded expectation.\n\n'
    printf '## Documentation\n\n'
    for c in $chapters; do
        t=$(sed -n 's/^# \(.*\)$/\1/p' "$SRC/$c" 2>/dev/null | head -1)
        [ -z "$t" ] && t="$c"
        printf -- '- [%s](%s/%s)\n' "$t" "$BASE" "${c%.md}.html"
    done
    printf '\n## Full text\n\n'
    printf -- '- [The whole book, one Markdown file](%s/llms-full.txt)\n' "$BASE"
    printf '\n## Source\n\n'
    printf -- '- [Repository](https://github.com/Industrial/voltaic)\n'
    printf -- '- [Specification](https://github.com/Industrial/voltaic/blob/main/docs/03%%20-%%20Voltaic%%20Surface%%20Design.md)\n'
} > "$HTML/llms.txt"

# `{{#include}}` is expanded here, or every sample would read as a directive.
expand() {
    python3 - "$1" <<'PY'
import io, os, re, sys
page = sys.argv[1]
base = os.path.dirname(page)
body = io.open(page, encoding="utf-8").read()


def sub(m):
    target = m.group(1).strip().split(":")[0]
    path = os.path.normpath(os.path.join(base, target))
    try:
        return io.open(path, encoding="utf-8").read().rstrip("\n")
    except OSError:
        return "(sample unavailable: %s)" % target


sys.stdout.write(re.sub(r"\{\{#include ([^}]+)\}\}", sub, body))
PY
}

{
    printf '# Voltaic — the complete documentation\n\n'
    printf '> %s\n\n' "$DESC"
    printf 'This file is every chapter of the Voltaic book concatenated in reading\n'
    printf 'order, with all code samples expanded in place. It carries no navigation\n'
    printf 'and no duplication. Generated from the repository; the specification at\n'
    printf 'docs/03 remains the authority where the two disagree.\n\n'
    printf -- '---\n\n'
    for c in $chapters; do
        [ -f "$SRC/$c" ] || continue
        expand "$SRC/$c"
        printf '\n\n---\n\n'
    done
} > "$HTML/llms-full.txt"

echo "machine: $pages pages annotated; sitemap, robots, llms.txt and llms-full.txt written"
echo "machine: llms-full.txt is $(wc -c < "$HTML/llms-full.txt") bytes over $(echo "$chapters" | wc -w) chapters"
