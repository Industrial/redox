# -*- coding: utf-8 -*-
"""Plan 75 batch 2: the two allows plan 55 §1.6 obliges every ported crate to carry.

`no_effect` and `unused_unit` are the same `()`, named differently by clippy
depending on where it sits. §1.6 requires a `()`-returning body to **end with
`()`** rather than implying it — deliberately, so a body ending in a loop, a
binding or an assignment cannot become `()` by arriving last — and clippy reads
that `()` as a statement with no effect.

This is the fifth crate to state it, which is the point at which it stops being
a per-crate note: §1.6 and clippy disagree by design, and a successor row owes
the decision either a workspace-level allow or a change to §1.6.
"""
import io

P = '/home/tom/Code/rust/redox/crates/redox-fuzz/sdx/lib.sdx'

WHY = (
    "-- Plan 55 §1.6 requires a `()`-returning body to **end with `()`** rather than\n"
    "-- implying it — deliberately, so a body ending in a loop, a binding or an\n"
    "-- assignment cannot become `()` by arriving last. clippy reads that `()` as a\n"
    "-- statement with no effect, and names it `unused_unit` where it closes a body.\n"
    "-- The two rules disagree by design, and §1.6's is the one with an argument\n"
    "-- behind it.\n"
    "--\n"
    "-- The fifth crate to say so. A successor row owes the decision either a\n"
    "-- workspace-level allow or a change to §1.6.\n"
)

s = io.open(P, encoding='utf-8').read()
prefix = ''
for lint in ('no_effect', 'unused_unit'):
    line = '#![allow(clippy::%s)]' % lint
    if line not in s:
        prefix += line + '\n'
if prefix:
    io.open(P, 'w', encoding='utf-8').write(WHY + prefix + '\n' + s)
    print('lib.sdx: %d allow(s) added' % prefix.count('#!['))
else:
    print('lib.sdx: the allows are already present')
