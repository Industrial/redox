# -*- coding: utf-8 -*-
"""Write a ported body into `crates/$CRATE/sdx/<rel>.sdx`, merging imports.

Generic over the crate, where `p73write.py` and `p74write.py` were copies. The
crate comes from the `CRATE` environment variable, which `.tmp/port.sh` sets.

The head — doc comments, `import`s, `module` declarations, attributes — is what
the partial translator produced and is kept; everything from the `-- TODO`
marker down, or from the first construct if there is none, is replaced by the
hand-written text.

**Imports are merged, not appended, and keyed by path.** A body legitimately
needs imports the original did not, and `import X` and `import X as _` are the
same import spelled two ways — a set keyed on the line keeps both, which is
`E0252`. The plain spelling wins where both appear.

**Cut at the TODO when there is one.** It is the translator saying exactly
where it stopped, so everything above it is real output to keep — which
matters for a file whose type translated cleanly and only whose `impl` blocks
are owed. Cutting at the first construct would throw the type away.

**A comment at the end of the head documents the body's first item**, so the
trailing blanks come off and no separator goes back: a blank line between a doc
comment and its construct is `empty_line_after_doc_comment`.

Usage: `from write import write; write('lexer/cursor', text)`.
"""
import io
import os
import re

CRATE = os.environ.get('CRATE')
assert CRATE, 'set CRATE'
ROOT = '/home/tom/Code/rust/redox/crates/%s/sdx/' % CRATE

# The first line that begins a construct rather than describing one.
CONSTRUCT = re.compile(
    r'^(export(\(crate\)|\(super\))? )?'
    r'(data |newtype |type |instance |errors |class |const |module |test )'
    r'|^(export(\(crate\)|\(super\))? )?[A-Za-z_][A-Za-z0-9_]* : '
    r'|^-- TODO plan 68'
)

IMPORT = re.compile(r'^(export )?import ')


def write(rel, text):
    p = ROOT + rel + '.sdx'
    lines = io.open(p, encoding='utf-8').read().splitlines()
    todo = next(
        (i for i, l in enumerate(lines) if l.startswith('-- TODO plan 68')), None
    )
    cut = (
        todo
        if todo is not None
        else next((i for i, l in enumerate(lines) if CONSTRUCT.match(l)), None)
    )
    assert cut is not None, 'no construct or TODO line in %s' % p

    head = lines[:cut]
    body = text.lstrip('\n').splitlines()

    by_path = {}
    for l in head + body:
        if not IMPORT.match(l):
            continue
        path = l.split(' as ')[0].rstrip()
        if path not in by_path or ' as ' not in l:
            by_path[path] = l
    imports = sorted(by_path.values())
    head = [l for l in head if not IMPORT.match(l)]
    body = [l for l in body if not IMPORT.match(l)]

    while head and not head[0].strip():
        head.pop(0)
    while body and not body[0].strip():
        body.pop(0)

    # The head's trailing comment block is an orphan once the body supplies its
    # own. Blanks come off first: the `-- TODO` marker sits one blank line below
    # the comment it follows, so a loop that tests for a comment exits at once.
    if body and body[0].lstrip().startswith('--'):
        while head and not head[-1].strip():
            head.pop()
        while head and head[-1].lstrip().startswith('--'):
            head.pop()
        while head and not head[-1].strip():
            head.pop()

    # **A leading `-- !` block stays first.** Rust requires `//!` to precede
    # every item in a file, so hoisting the imports above it emits
    # `E0753: expected outer doc comment`. Plan 76 found it on
    # `redox-bench/main.sdx` — the first ported file whose head carries module
    # docs *and* imports, which is why three crates went by without it.
    module_doc = []
    at = 0
    while at < len(head) and (
        head[at].lstrip().startswith('-- !') or not head[at].strip()
    ):
        at += 1
    if any(l.lstrip().startswith('-- !') for l in head[:at]):
        module_doc = head[:at]
        head = head[at:]
        while module_doc and not module_doc[-1].strip():
            module_doc.pop()

    out = []
    if module_doc:
        out += module_doc + ['']
    if imports:
        out += imports + ['']
    if any(l.strip() for l in head):
        while head and not head[-1].strip():
            head.pop()
        out += head
        if out and out[-1].strip() and not out[-1].lstrip().startswith('--'):
            out.append('')
    out += body

    io.open(p, 'w', encoding='utf-8').write('\n'.join(out).rstrip('\n') + '\n')
    return rel
