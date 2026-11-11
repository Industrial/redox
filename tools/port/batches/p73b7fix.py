# -*- coding: utf-8 -*-
"""Two spellings parse_map needed.

A **nullary** lambda is `\\-> e`, not `\\() -> e` — the same form plan 64 gives a
macro argument, which is exactly what a nullary lambda is. `\\()` asks for a
parameter named `()` and earns `RDX0103`.

That edit is now a no-op: `p73b7.py` stopped using a nullary lambda at all,
because one in argument position makes `redox fmt` grow a parenthesis per pass
without bound. It stays here, tolerated rather than asserted, so the record of
the spelling survives in the batch that discovered it.

And `version` was rebound rather than reassigned, which plan 54's `RDX1003`
reports: a bare `=` binds a second name. Two names say it better here anyway —
the text as parsed, and the number it yielded.
"""
import io

p = '/home/tom/Code/rust/redox/crates/redox-diag/sdx/source/map/parse_map.sdx'
s = io.open(p, encoding='utf-8').read()

OLD = '''  raw = (.ok_or "the map header has no version" (.next parts))?
  version : Result<u32, _> = .parse raw
  version = .map_err (\\_ -> .to_string "the map header's version is not a number") version?
'''
NEW = '''  raw = (.ok_or "the map header has no version" (.next parts))?
  -- Two names where the Rust chained: `.parse::<u32>()` has no turbofish in
  -- Redox, so the type arrives by ascription on a binding, and rebinding the
  -- same name would be `RDX1003` rather than a reassignment.
  parsed_version : Result<u32, _> = .parse raw
  version = (.map_err (\\_ -> .to_string "the map header's version is not a number") parsed_version)?
'''
assert s.count(OLD) == 1, 'version: %d' % s.count(OLD)
s = s.replace(OLD, NEW, 1)

# A no-op since the nullary lambda left; kept for the record, not asserted.
OLD = '''      |> .ok_or_else (\\() -> format! \\-> "record #{expected} does not begin with a line number")?'''
NEW = '''      |> .ok_or_else (\\-> format! \\-> "record #{expected} does not begin with a line number")?'''
if s.count(OLD) == 1:
    s = s.replace(OLD, NEW, 1)

io.open(p, 'w', encoding='utf-8').write(s)
print('parse_map.sdx: a second name for the parsed version')
