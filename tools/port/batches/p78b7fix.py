# -*- coding: utf-8 -*-
"""`p78b7` edits `p78b6`, so it must be idempotent — the third time this bites.

A batch that patches another batch's script runs once and then finds its
before-state gone. Plans 73 and 76 each recorded it; `port.sh`'s header says
the build has to be run twice. What it did not say is that such a batch must
also **tolerate having already run**, or the second run aborts.

So the edit is stated as a destination: apply it when the attribute is still on
the fields, accept it when it is already on the function, and fail only when it
is in neither place.
"""
import io

P = '/home/tom/Code/rust/voltaic/tools/port/batches/p78b7.py'
s = io.open(P, encoding='utf-8').read()

OLD = """DROP = '          #[allow(deprecated, reason = "the field is required by the struct literal")]\\n'
assert s.count(DROP) == 1, 'inner attribute: %d' % s.count(DROP)
s = s.replace(DROP, '', 1)

DROP = '    #[allow(deprecated, reason = "the field is required by the struct literal")]\\n'
assert s.count(DROP) == 1, 'outer attribute: %d' % s.count(DROP)
s = s.replace(DROP, '', 1)

OLD = "export(crate) symbol_of : &Item -> &str -> DocumentSymbol"
"""
NEW = """# Stated as a destination, not a diff: this batch edits another batch's script,
# so on the second run its before-state is already gone. Plans 73 and 76 each
# recorded that such a batch lands one build late; it also has to tolerate
# having already landed.
for indent in ('          ', '    '):
    drop = indent + '#[allow(deprecated, reason = "the field is required by the struct literal")]\\n'
    s = s.replace(drop, '')

OLD = "export(crate) symbol_of : &Item -> &str -> DocumentSymbol"
if OLD not in s:
    # Already on the function, which is the destination.
    assert '#[allow(deprecated' in s, 'the allow is in neither place'
    io.open(P, 'w', encoding='utf-8').write(s)
    print('  p78b6: the allow is already on the function')
    OLD = None
"""
assert s.count(OLD) == 1, 'drop block: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)

OLD = """NEW = ('-- **The allow is on the function, where the Rust had it on each field.**\\n'"""
NEW = """NEW = OLD and ('-- **The allow is on the function, where the Rust had it on each field.**\\n'"""
assert s.count(OLD) == 1, 'NEW block: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)

OLD = """assert s.count(OLD) == 1, 'signature: %d' % s.count(OLD)
io.open(P, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
print('  p78b6: the allow is on the function')
"""
NEW = """if OLD:
    assert s.count(OLD) == 1, 'signature: %d' % s.count(OLD)
    io.open(P, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
    print('  p78b6: the allow moves to the function')
"""
assert s.count(OLD) == 1, 'write block: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)

io.open(P, 'w', encoding='utf-8').write(s)
print('p78b7: idempotent')

# And the stale claim in p78b6's own docstring.
P = '/home/tom/Code/rust/voltaic/tools/port/batches/p78b6.py'
s = io.open(P, encoding='utf-8').read()
OLD = """Two ascriptions stand in for `collect::<Vec<_>>()`, and the
`#[allow(deprecated, …)]` attribute inside a record literal is carried through
as trivia."""
NEW = """Two ascriptions stand in for `collect::<Vec<_>>()`. The Rust's
`#[allow(deprecated, …)]` sits on each `deprecated:` field, and **Voltaic cannot
carry an attribute there at all** — `FieldInit` has no `leading`, so the parser
discards it before the AST exists. The allow is on the function instead; batch
7 has the finding."""
if OLD in s:
    io.open(P, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
    print('p78b6: the docstring no longer claims the attribute survives')
