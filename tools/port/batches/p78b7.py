# -*- coding: utf-8 -*-
"""Plan 78 batch 7: the lint pass, and an attribute Redox cannot carry.

**An attribute on a record-field initialiser is silently dropped.** The Rust
writes `#[allow(deprecated, reason = …)] deprecated: None` inside a
`DocumentSymbol` literal, and `FieldInit` has no `leading` field — the parser
discards the attribute before the AST exists, so the emitter never sees it and
`redox fmt` cannot print it. That contradicts doc 02 §10.1's *"comments are
trivia, never discarded"*, which attributes joined at plan 18 §1.3, and it is
quiet: the `.sdx` parses, the port reports nothing, and clippy then warns about
a deprecated field the author had explicitly allowed.

Carrying it needs a `leading` on `FieldInit` plus the parser, the emitter and
the formatter — a feature, not a fix, and owed to a successor row. Meanwhile
the allow goes on the **function**, which is a wider scope than the field and
the same effect, and the gap is written down rather than worked around
silently.

Three crate-root allows besides: `no_effect` and `unused_unit` for plan 55
§1.6's spelled `()` — the **seventh** crate — and `single_match`, because Redox
has no `if let`.
"""
import io

ROOT = '/home/tom/Code/rust/redox/'

# --- the allow moves from the field to the function ---------------------
P = ROOT + '.tmp/p78b6.py'
s = io.open(P, encoding='utf-8').read()

# Stated as a destination, not a diff: this batch edits another batch's script,
# so on the second run its before-state is already gone. Plans 73 and 76 each
# recorded that such a batch lands one build late; it also has to tolerate
# having already landed.
for indent in ('          ', '    '):
    drop = indent + '#[allow(deprecated, reason = "the field is required by the struct literal")]\n'
    s = s.replace(drop, '')

OLD = "export(crate) symbol_of : &Item -> &str -> DocumentSymbol"
if OLD not in s:
    # Already on the function, which is the destination.
    assert '#[allow(deprecated' in s, 'the allow is in neither place'
    io.open(P, 'w', encoding='utf-8').write(s)
    print('  p78b6: the allow is already on the function')
    OLD = None
NEW = OLD and ('-- **The allow is on the function, where the Rust had it on each field.**\n'
       '-- An attribute on a record-field initialiser is **dropped**: `FieldInit`\n'
       '-- has no `leading`, so the parser discards it before the AST exists and\n'
       '-- neither the emitter nor `redox fmt` ever sees it. That contradicts doc\n'
       '-- 02 §10.1, which attributes joined at plan 18 §1.3, and it is quiet —\n'
       '-- the `.sdx` parses and clippy then warns about a field the author had\n'
       '-- explicitly allowed. Carrying it is a feature for a successor row; this\n'
       '-- is a wider scope with the same effect.\n'
       '#[allow(deprecated, reason = "the field is required by the struct literal")]\n'
       'export(crate) symbol_of : &Item -> &str -> DocumentSymbol')
if OLD:
    assert s.count(OLD) == 1, 'signature: %d' % s.count(OLD)
    io.open(P, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
    print('  p78b6: the allow moves to the function')

# --- the crate-root allows ----------------------------------------------
P = ROOT + 'crates/redox-lsp/sdx/lib.sdx'
WHY = {
    'no_effect': (
        "-- Plan 55 §1.6 requires a `()`-returning body to **end with `()`** rather\n"
        "-- than implying it, so a body ending in a loop or an assignment cannot\n"
        "-- become `()` by arriving last. clippy reads that `()` as a statement with\n"
        "-- no effect, and names it `unused_unit` where it closes a body.\n"
        "--\n"
        "-- The **seventh** crate to say so. Plan 75 §2.4 called repeating this\n"
        "-- paragraph the wrong answer and plan 76 §6 repeated it anyway; the\n"
        "-- decision is owed either a workspace-level allow or a change to §1.6.\n"),
    'unused_unit': None,
    'single_match': (
        "-- Redox has no `if let`, so *do this if it matches* is a `case` with a\n"
        "-- `_ -> ()` arm — the same gap as `match_like_matches_macro`, one step\n"
        "-- over. `serve`'s notification arm is the one here.\n"),
}

s = io.open(P, encoding='utf-8').read()
prefix = ''
added = 0
for lint in ('no_effect', 'unused_unit', 'single_match'):
    line = '#![allow(clippy::%s)]' % lint
    if line in s:
        continue
    why = WHY[lint]
    prefix += (why + line + '\n\n') if why else (line + '\n\n')
    added += 1
if added:
    io.open(P, 'w', encoding='utf-8').write(prefix + s)
    print('  lib.sdx: %d allow(s) added' % added)
else:
    print('  lib.sdx: the allows are already present')
