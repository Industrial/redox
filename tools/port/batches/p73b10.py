# -*- coding: utf-8 -*-
"""Batch 10: what `moon run :lint` asked for that is this crate's to answer.

Three of plan 73's four lint causes were tooling and are fixed in the
translator, the writer and the emitter. These five are the source's own.

**A test-only import does not belong at module scope.** `BTreeSet` and
`render_map` are each named once, inside a `test` block — which emits into
`mod __rdx_tests`, whose `use super::*;` resolves the parent's import and
leaves it unused in the *library* build. Voltaic has no import inside a `test`
block, so the use site names the path instead: `std::collections::BTreeSet`
directly, and `crate::render_map` via the re-export `lib.sdx` already makes.

**`origins = origins` is a pun.** Doc 03 §20 — a bare field name means the
binding of that name, which is Rust's own shorthand, and clippy asks for it.

**`n := n - 1` has an operator.** `compound_assign.rs` reads `-=`; the
translator produced the long form because the Rust did.

**`(&err)` is the Rust's borrow, not Voltaic's.** `.contains "…" (&err)` emits
`(&err).contains("…")`, which `needless_borrow` rejects — `err` is a `String`
and auto-ref does it. Six tests, one shape.

**The `matches!` allow.** `voltaic-ast` and `voltaic-fmt` both carry it with plan
68's reasoning: `matches!`'s second argument is a **pattern**, so a language
without one writes `case` with a `_ -> false`, and clippy asks for a macro it
cannot call. `voltaic-diag` has two such methods.

---

**Every edit here is stated as a destination, not as a diff**, and that is
load-bearing rather than tidy. `p73build.sh` runs the translator over
`crates/voltaic-diag/src`, which is *generated from this very tree* — so the
second run reads back the first run's output and the "before" text is already
gone. A batch written as `assert count == 1; replace` therefore passes once and
then aborts the build, which is exactly what it did. `ensure` applies the edit
when the before-state is present and checks the after-state when it is not, so
the batch is a fixpoint and the build is reproducible from any starting point.
"""
import io

ROOT = '/home/tom/Code/rust/voltaic/crates/voltaic-diag/sdx/'


def ensure(rel, pairs):
    """Drive `rel` to a state. Each pair is (before, after, count)."""
    p = ROOT + rel
    s = io.open(p, encoding='utf-8').read()
    for before, after, n in pairs:
        have = s.count(before)
        if have:
            assert have == n, '%s: %d of %r (wanted %d)' % (
                rel, have, before[:60], n)
            s = s.replace(before, after)
        else:
            # Already at the destination — the usual case on a second run,
            # because the translator's input is this tree's own output.
            assert after in s or not after, \
                '%s: neither %r nor %r' % (rel, before[:50], after[:50])
    io.open(p, 'w', encoding='utf-8').write(s)
    print('%-28s ok' % rel)


ensure('diagnostic/code.sdx', [
    ('import std::collections::BTreeSet\n', '', 1),
    ('  numbers : BTreeSet<u16> =',
     '  -- The path is written out rather than imported: `BTreeSet` is named\n'
     '  -- only here, inside a `test` block, and a module-scope import that\n'
     '  -- only `mod __rdx_tests` resolves is `unused_imports` in the library\n'
     '  -- build.\n'
     '  numbers : std::collections::BTreeSet<u16> =', 1),
])

ensure('source/map/parse_map.sdx', [
    ('import crate::source::map::render_map::render_map\n', '', 1),
    ('  text = render_map (&(sample ()))',
     '  -- `crate::render_map` via the re-export `lib.sdx` makes, rather than\n'
     '  -- an import: this is the only mention, and it is inside a `test`.\n'
     '  text = crate::render_map (&(sample ()))', 1),
    ('(&err))', 'err)', 6),
])

ensure('source/map.sdx', [
    ('    origins = origins,\n', '    origins,\n', 1),
])

ensure('render/snippet_line.sdx', [
    ('    n := n - 1\n', '    n -= 1\n', 1),
])

# The crate-root allow, above the module docs as `voltaic-ast` has it. Stated as
# a destination for the same reason as everything above: once it is in the
# `.sdx` it reaches the generated Rust, and the translator reads it back.
ALLOW = '#![allow(clippy::match_like_matches_macro)]'
WHY = """-- `matches!`'s second argument is a **pattern**, and Voltaic has no way to pass
-- one — so a method asking *is this one of these variants* is a `case` with a
-- `_ -> false` arm, and `clippy::match_like_matches_macro` fires on every one
-- asking for a macro this language cannot call. Two methods here:
-- `DiagnosticCode::is_voltaic` and `Severity::is_error`.
--
-- The allow states that fact rather than suppressing a defect. `voltaic-ast` and
-- `voltaic-fmt` carry the same one for the same reason — plan 68 found it, and
-- only a row making `matches!` reachable would retire it.
"""
p = ROOT + 'lib.sdx'
s = io.open(p, encoding='utf-8').read()
if ALLOW not in s:
    io.open(p, 'w', encoding='utf-8').write(WHY + ALLOW + '\n\n' + s)
    print('%-28s ok (allow added)' % 'lib.sdx')
else:
    print('%-28s ok (allow present)' % 'lib.sdx')
