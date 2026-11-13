# -*- coding: utf-8 -*-
"""Two composition errors in the batches, fixed at the source.

**Batch 4 and batch 5 both wrote `diagnostic`.** Batch 4 used the elided
`&mut Formatter` because `'_` was still being hoisted; batch 5 restored the
idiomatic `Formatter<'_>` after that was fixed. Running both in order means the
second finds no `-- TODO` marker, falls back to cutting at the first construct
— the file's first line — and replaces the `module` declarations and the whole
`data Diagnostic` with a body that assumed they were still above it.

So batch 4 stops writing `diagnostic` and batch 5 owns it. A file must be
written exactly once.

**Batch 5's `reportable` body re-declared the struct** the translator had
already produced above the TODO, giving two `data Reportable` and `E0428`. The
body keeps only what is owed: the instances.
"""
import io
import re

ROOT = '/home/tom/Code/rust/voltaic/.tmp/'

# ---------------------------------------------------- batch 4 drops diagnostic
p = ROOT + 'p73b4.py'
s = io.open(p, encoding='utf-8').read()
start = s.index("    'diagnostic': '''")
end = s.index("''',\n", start) + len("''',\n")
s = s[:start] + s[end:]
s = s.replace(
    '* **`instance Display Diagnostic`** with `&mut Formatter` and no lifetime, for\n'
    '  the E0637 reason batch 1 recorded.\n'
    '* **`instance Error Diagnostic`** with no members at all.\n',
    '* **`instance From Code DiagnosticCode`**, as above.\n'
    '\n'
    '`diagnostic` itself is batch 5\'s: it was written here first with the\n'
    'elided `&mut Formatter`, and batch 5 rewrote it idiomatically once `\'_`\n'
    'stopped being hoisted. A file written twice is destroyed the second time,\n'
    'because `write()` finds no TODO and cuts at the first construct.\n',
)
io.open(p, 'w', encoding='utf-8').write(s)
print('p73b4.py: diagnostic removed, batch 5 owns it')

# ------------------------------------------- batch 5 stops redeclaring the struct
p = ROOT + 'p73b5.py'
s = io.open(p, encoding='utf-8').read()

struct = re.search(
    r"-- \| A `Diagnostic` in the shape `miette` reports\..*?\n\}\n\n", s, re.S
)
assert struct, 'the Reportable struct block was not found'
s = s[:struct.start()] + s[struct.end():]

s = s.replace(
    "* **An aliased import to break a name collision.**",
    "* **An aliased import to break a name collision.**",
)
s = s.replace(
    "`diagnostic.sdx` goes back to the idiomatic `Formatter<'_>` it was written\nwith, the elision having been a workaround for a defect that is now fixed.",
    "`diagnostic.sdx` goes back to the idiomatic `Formatter<'_>` it was written\n"
    "with, the elision having been a workaround for a defect that is now fixed.\n"
    "\n"
    "Neither body re-declares its type: the translator produced both\n"
    "`data Diagnostic` and `data Reportable` above their `-- TODO` markers, and\n"
    "a body that declares one again is `E0428`.",
)
io.open(p, 'w', encoding='utf-8').write(s)
print('p73b5.py: the Reportable struct is the head\'s')
