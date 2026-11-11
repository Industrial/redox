# -*- coding: utf-8 -*-
"""Two spellings batch 3 needed.

**An array literal in argument position reads as indexing.**
`.trim_end_matches ['\\n', '\\r'] source` is parsed as `trim_end_matches[…]`,
and the index wants a `]` after the first element. Parenthesised it is an
argument. The same shape as plan 69's `(*is_move)`: juxtaposition application
collides with any bracket that also has a postfix meaning, and Redox resolves
towards the postfix.

**A `crate::`-prefixed path cannot open a pattern.** `crate::Severity::Error`
in a `case` arm earns `RDX0101`, which is plan 68's *"a constructor pattern's
path must start uppercase"* met from a new direction — the path here starts
lowercase because it starts with a keyword. Importing the type and matching
`Severity::Error` is both shorter and the form every other ported file uses.
"""
import io

ROOT = '/home/tom/Code/rust/redox/crates/redox-diag/sdx/'

p = ROOT + 'render/anchor_span.sdx'
s = io.open(p, encoding='utf-8').read()
OLD = "  trimmed = .trim_end_matches ['\\n', '\\r'] source"
NEW = ("  -- Parenthesised: an array literal in **argument** position is read as an\n"
       "  -- index on the callee, so the bare form wants a `]` after the first\n"
       "  -- element. Plan 69's `(*is_move)` is the same collision for a prefix\n"
       "  -- operator.\n"
       "  trimmed = .trim_end_matches (['\\n', '\\r']) source")
assert s.count(OLD) == 1, 'anchor: %d' % s.count(OLD)
io.open(p, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
print('anchor_span.sdx: the array is an argument')

p = ROOT + 'render/plain_one.sdx'
s = io.open(p, encoding='utf-8').read()
OLD = """  level = case d.severity of
    crate::Severity::Error -> "error"
    crate::Severity::Warning -> "warning\""""
NEW = """  -- `Severity::…` imported rather than `crate::Severity::…` written inline: a
  -- `crate::`-prefixed path **cannot open a pattern**, which is plan 68's
  -- uppercase-path rule met from a new direction — this path starts with a
  -- keyword.
  level = case d.severity of
    Severity::Error -> "error"
    Severity::Warning -> "warning\""""
assert s.count(OLD) == 1, 'plain: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)
# The import is added only if the body did not already carry it: `write()`
# dedupes by path, but this script edits the file directly and would otherwise
# add a second copy of a line the body supplies.
if 'import crate::Severity' not in s:
    OLD = "import crate::Diagnostic\n"
    NEW = "import crate::Diagnostic\nimport crate::Severity\n"
    assert s.count(OLD) == 1, 'import: %d' % s.count(OLD)
    s = s.replace(OLD, NEW, 1)
io.open(p, 'w', encoding='utf-8').write(s)
print('plain_one.sdx: Severity is imported')
