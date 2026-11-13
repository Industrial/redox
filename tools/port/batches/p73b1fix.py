# -*- coding: utf-8 -*-
"""Batch 1's bodies re-declared modules the translator had already produced.

Third instance of one mistake, so it is worth stating as a rule rather than
fixed three times: **a body supplies only what the `-- TODO` marker says is
owed.** Everything above the marker is the translator's output and is already
there — the `module` declarations, the `data` declaration, the doc comments.

`render.sdx` got six duplicate `mod` declarations and
`render/graphical_one.sdx` one, each `E0428`. The struct duplication in
`reportable.sdx` was the same error on a different construct, and
`diagnostic.sdx` losing its struct was its mirror image: a body that assumed
*less* was there than actually was.
"""
import io

ROOT = '/home/tom/Code/rust/voltaic/.tmp/'
p = ROOT + 'p73b1.py'
s = io.open(p, encoding='utf-8').read()

OLD = """    'render/graphical_one': '''export(crate) module reportable

import crate::Diagnostic"""
NEW = """    'render/graphical_one': '''import crate::Diagnostic"""
assert s.count(OLD) == 1, 'graphical_one: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)

OLD = """    'render': '''export(crate) module anchor_span
export(crate) module caret_width
export(crate) module graphical_one
export(crate) module line_col
export(crate) module plain_one
export(crate) module snippet_line

import crate::Diagnostic"""
NEW = """    'render': '''import crate::Diagnostic"""
assert s.count(OLD) == 1, 'render: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)

OLD = '''An instance with **no members** also works: `instance Error Thing` emits
`impl Error for Thing {}`.
"""'''
NEW = '''An instance with **no members** also works: `instance Error Thing` emits
`impl Error for Thing {}`.

**A body supplies only what the `-- TODO` marker says is owed.** Everything
above it is the translator's and is already in the file; re-declaring a
`module` or a `data` is `E0428`, and assuming less is there than actually is
loses it. This batch learned the first half the hard way, twice.
"""'''
assert s.count(OLD) == 1, 'doc: %d' % s.count(OLD)
io.open(p, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
print('p73b1.py: the module declarations are the head\'s')
