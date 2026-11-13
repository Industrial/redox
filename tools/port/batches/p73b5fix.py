# -*- coding: utf-8 -*-
"""`Debug` has to be imported as a trait, not left to resolve to the macro.

`instance Debug Reportable` emits `impl Debug for Reportable`. With no import,
`Debug` in that position resolves to the **derive macro** of the same name and
rustc answers `E0404: expected trait, found derive macro 'Debug'` — followed by
two `E0277`s, because `miette::Diagnostic` requires `Debug` and the impl that
was meant to supply it did not exist.

`Display` and `Error` needed no equivalent note only because neither has a
same-named macro in the prelude.
"""
import io

p = '/home/tom/Code/rust/voltaic/tools/port/batches/p73b5.py'
s = io.open(p, encoding='utf-8').read()

OLD = "    'render/graphical_one/reportable': '''import crate::Diagnostic"
NEW = (
    "    'render/graphical_one/reportable': '''-- `Debug` as a **trait**, not the\n"
    "-- derive macro of the same name: `instance Debug Reportable` emits\n"
    "-- `impl Debug for Reportable`, and without this import `Debug` resolves to\n"
    "-- the macro — `E0404: expected trait, found derive macro`.\n"
    "import std::fmt::Debug\n"
    "import crate::Diagnostic"
)
assert s.count(OLD) == 1, s.count(OLD)
io.open(p, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
print('p73b5.py: Debug imported as a trait')
