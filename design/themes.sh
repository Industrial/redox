#!/usr/bin/env bash
# Emit a Voltaic theme for each of the five editor integrations.
#
# Identity and documentation site
# (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 4.
#
# Five editors, five formats, **one palette**. `design/tokens/tokens.json`
# holds the nine roles and the `syntax` block that maps a token class onto one
# of them; everything below is a translation of that into whatever shape the
# editor wants. A colour that is not in the token file is not in any theme, and
# `crates/voltaic-cli/tests/design.rs` asserts it.
#
# The emitted files are tracked, because each one ships inside its editor's
# configuration and is read by that editor rather than by this build — the same
# argument the masters make. `--check` is what keeps them honest.
set -euo pipefail

cd "$(dirname "$0")/.."

TOKENS="design/tokens/tokens.json"
CHECK=0
[ "${1:-}" = "--check" ] && CHECK=1

command -v jq >/dev/null || { echo "error: jq not found. Run inside 'devenv shell'." >&2; exit 1; }

OUT="editors"
if [ "$CHECK" -eq 1 ]; then
    OUT=$(mktemp -d)
    trap 'rm -rf "$OUT"' EXIT
fi

# hex <role> <light|dark>
hex() { jq -r --arg r "$1" --arg t "$2" '.roles[$r][$t]' "$TOKENS"; }
# The role a token class maps to, and its emphasis.
srole() { jq -r --arg k "$1" '.syntax[$k].role' "$TOKENS"; }
sbold() { jq -r --arg k "$1" '.syntax[$k].bold // false' "$TOKENS"; }
sital() { jq -r --arg k "$1" '.syntax[$k].italic // false' "$TOKENS"; }

CLASSES=$(jq -r '.syntax | keys_unsorted[]' "$TOKENS")

# ── VS Code ───────────────────────────────────────────────────────────────
# One file per theme, because `contributes.themes` lists them separately.
vscode() {
    local theme=$1 out=$2 type
    type=$([ "$theme" = light ] && echo vs || echo vs-dark)
    {
        printf '{\n'
        printf '  "$schema": "vscode://schemas/color-theme",\n'
        printf '  "name": "Voltaic %s",\n' "$theme"
        printf '  "type": "%s",\n' "$type"
        printf '  "colors": {\n'
        printf '    "editor.background": "%s",\n' "$(hex ground "$theme")"
        printf '    "editor.foreground": "%s",\n' "$(hex ink "$theme")"
        printf '    "editorLineNumber.foreground": "%s",\n' "$(hex ink-faint "$theme")"
        printf '    "editorLineNumber.activeForeground": "%s",\n' "$(hex copper "$theme")"
        printf '    "editorCursor.foreground": "%s",\n' "$(hex copper "$theme")"
        printf '    "editorIndentGuide.background1": "%s",\n' "$(hex rule "$theme")"
        printf '    "editorIndentGuide.activeBackground1": "%s",\n' "$(hex zinc "$theme")"
        printf '    "editorWidget.background": "%s",\n' "$(hex ground-2 "$theme")"
        printf '    "editorWidget.border": "%s",\n' "$(hex rule "$theme")"
        printf '    "textLink.foreground": "%s",\n' "$(hex arc "$theme")"
        printf '    "focusBorder": "%s"\n' "$(hex arc "$theme")"
        printf '  },\n'
        printf '  "tokenColors": [\n'
        local first=1
        for c in $CLASSES; do
            local scope fg b i settings
            scope=$(jq -r --arg k "$c" '.syntax[$k].scope' "$TOKENS")
            fg=$(hex "$(srole "$c")" "$theme")
            b=$(sbold "$c"); i=$(sital "$c")
            settings="\"foreground\": \"$fg\""
            if [ "$b" = true ] && [ "$i" = true ]; then
                settings="$settings, \"fontStyle\": \"bold italic\""
            elif [ "$b" = true ]; then
                settings="$settings, \"fontStyle\": \"bold\""
            elif [ "$i" = true ]; then
                settings="$settings, \"fontStyle\": \"italic\""
            fi
            [ "$first" -eq 0 ] && printf ',\n'
            first=0
            printf '    { "name": "%s", "scope": "%s", "settings": { %s } }' "$c" "$scope" "$settings"
        done
        printf '\n  ]\n}\n'
    } > "$out"
}
mkdir -p "$OUT/vscode/themes" "$OUT/vscode/icons"
vscode light "$OUT/vscode/themes/voltaic-light.json"
vscode dark "$OUT/vscode/themes/voltaic-dark.json"
cp design/masters/pile-small-light.svg "$OUT/vscode/icons/sdx-light.svg"
cp design/masters/pile-small-dark.svg "$OUT/vscode/icons/sdx-dark.svg"

# ── Zed ───────────────────────────────────────────────────────────────────
# One family file carrying both appearances, which is Zed's shape.
mkdir -p "$OUT/zed/themes"
{
    printf '{\n'
    printf '  "$schema": "https://zed.dev/schema/themes/v0.2.0.json",\n'
    printf '  "name": "Voltaic",\n'
    printf '  "author": "Voltaic",\n'
    printf '  "themes": [\n'
    zfirst=1
    for theme in light dark; do
        [ "$zfirst" -eq 0 ] && printf ',\n'
        zfirst=0
        printf '    {\n'
        printf '      "name": "Voltaic %s",\n' "$theme"
        printf '      "appearance": "%s",\n' "$theme"
        printf '      "style": {\n'
        printf '        "background": "%s",\n' "$(hex ground "$theme")"
        printf '        "editor.background": "%s",\n' "$(hex ground "$theme")"
        printf '        "editor.foreground": "%s",\n' "$(hex ink "$theme")"
        printf '        "editor.line_number": "%s",\n' "$(hex ink-faint "$theme")"
        printf '        "editor.active_line_number": "%s",\n' "$(hex copper "$theme")"
        printf '        "border": "%s",\n' "$(hex rule "$theme")"
        printf '        "text": "%s",\n' "$(hex ink "$theme")"
        printf '        "text.muted": "%s",\n' "$(hex ink-mute "$theme")"
        printf '        "link_text.hover": "%s",\n' "$(hex arc "$theme")"
        printf '        "syntax": {\n'
        sfirst=1
        for c in $CLASSES; do
            [ "$sfirst" -eq 0 ] && printf ',\n'
            sfirst=0
            printf '          "%s": { "color": "%s"' "$c" "$(hex "$(srole "$c")" "$theme")"
            [ "$(sbold "$c")" = true ] && printf ', "font_weight": 700'
            [ "$(sital "$c")" = true ] && printf ', "font_style": "italic"'
            printf ' }'
        done
        printf '\n        }\n'
        printf '      }\n'
        printf '    }'
    done
    printf '\n  ]\n}\n'
} > "$OUT/zed/themes/voltaic.json"

# ── Helix ─────────────────────────────────────────────────────────────────
mkdir -p "$OUT/helix/themes"
{
    printf '# Voltaic for Helix. Generated by design/themes.sh — do not edit.\n'
    printf '#\n'
    printf '# Copy to ~/.config/helix/themes/voltaic.toml and set `theme = "voltaic"`.\n'
    printf '# Helix has one theme at a time rather than a light/dark pair, so this is\n'
    printf '# the dark colourway; swap the generator argument for the light one.\n\n'
    for c in $CLASSES; do
        printf '"%s" = { fg = "%s"' "$c" "$(hex "$(srole "$c")" dark)"
        mods=""
        [ "$(sbold "$c")" = true ] && mods="\"bold\""
        [ "$(sital "$c")" = true ] && mods="${mods:+$mods, }\"italic\""
        [ -n "$mods" ] && printf ', modifiers = [%s]' "$mods"
        printf ' }\n'
    done
    printf '\n"ui.background" = { bg = "%s" }\n' "$(hex ground dark)"
    printf '"ui.text" = { fg = "%s" }\n' "$(hex ink dark)"
    printf '"ui.linenr" = { fg = "%s" }\n' "$(hex ink-faint dark)"
    printf '"ui.linenr.selected" = { fg = "%s" }\n' "$(hex copper dark)"
    printf '"ui.cursor" = { fg = "%s", modifiers = ["reversed"] }\n' "$(hex copper dark)"
    printf '"ui.virtual.indent-guide" = { fg = "%s" }\n' "$(hex rule dark)"
    printf '\n[palette]\n'
    jq -r '.roles | to_entries[] | "\(.key) = \"\(.value.dark)\""' "$TOKENS" \
        | sed 's/^/# /'
} > "$OUT/helix/themes/voltaic.toml"

# ── Neovim ────────────────────────────────────────────────────────────────
mkdir -p "$OUT/neovim/lua/voltaic"
{
    printf -- '-- Voltaic for Neovim. Generated by design/themes.sh - do not edit.\n'
    printf -- '--\n'
    printf -- '--   require("voltaic.theme").setup()        -- follows background\n'
    printf -- '--   require("voltaic.theme").setup("light")\n'
    printf -- '--\n'
    printf -- '-- Links Treesitter capture groups rather than legacy syntax groups, so\n'
    printf -- '-- it colours whatever tree-sitter-voltaic reports.\n\n'
    printf 'local M = {}\n\n'
    printf 'M.palette = {\n'
    for theme in light dark; do
        printf '  %s = {\n' "$theme"
        jq -r --arg t "$theme" '.roles | to_entries[] | "    [\"\(.key)\"] = \"\(.value[$t])\","' "$TOKENS"
        printf '  },\n'
    done
    printf '}\n\n'
    printf 'M.groups = {\n'
    for c in $CLASSES; do
        printf '  { capture = "@%s", role = "%s", bold = %s, italic = %s },\n' \
            "$(jq -r --arg k "$c" '.syntax[$k].nvim' "$TOKENS")" \
            "$(srole "$c")" "$(sbold "$c")" "$(sital "$c")"
    done
    printf '}\n\n'
    cat <<'LUA'
function M.setup(variant)
  variant = variant or (vim.o.background == "light" and "light" or "dark")
  local p = M.palette[variant]
  vim.o.background = variant
  vim.cmd("highlight clear")
  vim.g.colors_name = "voltaic"

  vim.api.nvim_set_hl(0, "Normal", { fg = p["ink"], bg = p["ground"] })
  vim.api.nvim_set_hl(0, "LineNr", { fg = p["ink-faint"] })
  vim.api.nvim_set_hl(0, "CursorLineNr", { fg = p["copper"], bold = true })
  vim.api.nvim_set_hl(0, "Cursor", { fg = p["ground"], bg = p["copper"] })
  vim.api.nvim_set_hl(0, "Visual", { bg = p["ground-2"] })
  vim.api.nvim_set_hl(0, "VertSplit", { fg = p["rule"] })
  vim.api.nvim_set_hl(0, "Pmenu", { fg = p["ink"], bg = p["ground-2"] })
  vim.api.nvim_set_hl(0, "PmenuSel", { fg = p["ground"], bg = p["copper"] })

  for _, g in ipairs(M.groups) do
    vim.api.nvim_set_hl(0, g.capture, {
      fg = p[g.role],
      bold = g.bold or nil,
      italic = g.italic or nil,
    })
  end
end

return M
LUA
} > "$OUT/neovim/lua/voltaic/theme.lua"

# ── Vim ───────────────────────────────────────────────────────────────────
mkdir -p "$OUT/vim/colors"
{
    printf '" Voltaic for Vim. Generated by design/themes.sh - do not edit.\n'
    printf '"\n'
    printf '" :colorscheme voltaic\n'
    printf '"\n'
    printf '" 24-bit only. Vim has no equivalent of the token palette at 16 colours,\n'
    printf '" and approximating it would be inventing colours the identity does not\n'
    printf '" have - so this requires termguicolors and says so rather than degrading.\n\n'
    printf 'if !has("termguicolors")\n'
    printf '  echohl WarningMsg\n'
    printf '  echom "voltaic: needs termguicolors (set termguicolors)"\n'
    printf '  echohl None\n'
    printf '  finish\n'
    printf 'endif\n\n'
    printf 'hi clear\n'
    printf 'if exists("syntax_on") | syntax reset | endif\n'
    printf 'let g:colors_name = "voltaic"\n\n'
    printf 'if &background ==# "light"\n'
    for theme in light dark; do
        [ "$theme" = dark ] && printf 'else\n'
        jq -r --arg t "$theme" '.roles | to_entries[] | "  let s:\(.key|gsub("-";"_")) = \"\(.value[$t])\""' "$TOKENS"
    done
    printf 'endif\n\n'
    printf 'exe "hi Normal guifg=" . s:ink . " guibg=" . s:ground\n'
    printf 'exe "hi LineNr guifg=" . s:ink_faint\n'
    printf 'exe "hi CursorLineNr guifg=" . s:copper . " gui=bold"\n'
    printf 'exe "hi VertSplit guifg=" . s:rule\n'
    printf 'exe "hi Visual guibg=" . s:ground_2\n'
    printf 'exe "hi Pmenu guifg=" . s:ink . " guibg=" . s:ground_2\n\n'
    for c in $CLASSES; do
        g=$(jq -r --arg k "$c" '.syntax[$k].vim' "$TOKENS")
        r=$(srole "$c" | tr '-' '_')
        gui=""
        [ "$(sbold "$c")" = true ] && gui="bold"
        [ "$(sital "$c")" = true ] && gui="${gui:+$gui,}italic"
        if [ -n "$gui" ]; then
            printf 'exe "hi %s guifg=" . s:%s . " gui=%s"\n' "$g" "$r" "$gui"
        else
            printf 'exe "hi %s guifg=" . s:%s\n' "$g" "$r"
        fi
    done
} > "$OUT/vim/colors/voltaic.vim"

if [ "$CHECK" -eq 1 ]; then
    fail=0
    while IFS= read -r -d '' f; do
        rel=${f#"$OUT"/}
        if ! diff -q "$f" "editors/$rel" >/dev/null 2>&1; then
            echo "themes: editors/$rel is not what design/themes.sh emits." >&2
            fail=1
        fi
    done < <(find "$OUT" -type f -print0)
    [ "$fail" -eq 0 ] && echo "themes: every editor theme matches the token file."
    exit "$fail"
fi

echo "themes: emitted themes for vscode, zed, helix, neovim and vim."
