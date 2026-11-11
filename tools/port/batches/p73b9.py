# -*- coding: utf-8 -*-
"""redox-diag batch 9 — diagnostic/code, the last file.

The translator handled all 382 lines of the enum; this is the `impl` and the
five tests. New shapes: a `#[cfg(test)]` attribute on an instance member, a
local `const` inside a test body (plan 55 leaf 5), a `{:04}` width spec in an
interpolation, and `(1..=39).contains` on a range value.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

NUMBERS = [
    ('TabIndent', 1, 'Lexical.'), ('InconsistentDedent', 2, None),
    ('UnterminatedString', 3, None), ('UnterminatedRawString', 4, None),
    ('UnterminatedInterpolation', 5, None), ('UnterminatedFence', 6, None),
    ('UnterminatedAttribute', 7, None), ('UnterminatedByteLiteral', 8, None),
    ('UnknownEscape', 9, None), ('MalformedNumber', 10, None),
    ('UnexpectedCharacter', 11, None), ('UnbalancedDelimiter', 12, None),
    ('BadQuote', 13, None),
    ('ExpectedToken', 101, 'Syntactic.'), ('ExpectedExpression', 102, None),
    ('ExpectedName', 103, None), ('ExpectedType', 104, None),
    ('EmptyBody', 105, None), ('ModuleScopeBinding', 106, None),
    ('PatternUntil', 107, None), ('EllipsisRestPattern', 108, None),
    ('NestingTooDeep', 109, None), ('SpacedPathSeparator', 110, None),
    ('NewtypeShape', 111, None), ('LocalTest', 112, None),
    ('UnitTail', 113, None), ('ReservedWord', 114, None),
    ('MethodCallSyntax', 115, None), ('PipelineSection', 116, None),
    ('OptionalChaining', 117, None), ('RetiredSpelling', 118, None),
    ('WhereGroupShape', 119, None),
    ('UnwrapUsed', 1001, "Doc 03 §17's published number, and the first in the lint band."),
    ('CommentAllowPragma', 1002, None), ('ShadowedBinding', 1003, None),
    ('ReturnInCallbackTail', 44, "Doc 03 §17's published assignment, not the next free number."),
]

arms = []
for name, n, note in NUMBERS:
    if note:
        arms.append('    -- %s' % note)
    arms.append('    Code::%s -> %d' % (name, n))
ARMS = '\n'.join(arms)

ALL = '\n'.join('    Code::%s,' % name for name, _, _ in NUMBERS)

BODY = '''instance Code
  -- | The numeric code, which is what users cite and what UI tests assert.
  export number : self -> u16
  number self = case self of
%s

  -- | The rendered form, `RDX0001`.
  export text : self -> String
  text self = format! \\-> "RDX#{(.number self):04}"

  -- | Every code, for the uniqueness and range checks.
  -- A nullary signature is the return type alone; see `lint.sdx`.
  export all : &'static [Code]
  all = &[
%s
  ]

  -- | Whether this code is a lexical one, for the range check.
  #[cfg(test)]
  is_lexical : self -> bool
  is_lexical self = .number self <= 39

-- | Criterion 2: codes are unique.
test "every code is unique"
  numbers : BTreeSet<u16> = Code::all () |> .iter |> .map (\\c -> .number c) |> .collect
  assert_eq! \\-> (.len numbers) \\-> (.len (Code::all ())) \\-> "two codes share a number; a code must never have two meanings"

-- | Criterion 2: codes are inside their declared range.
test "every code is in its declared range"
  for code in Code::all ()
    n = .number code
    if .is_lexical code
      assert! \\-> (.contains (&n) (1..=39)) \\-> "#{code:?} is #{n}, outside RDX0001-0039"
    else
      assert! \\-> (.contains (&n) (40..=99) || .contains (&n) (100..=199) || n >= 1000) \\-> "#{code:?} is #{n}, outside RDX0040-0099, RDX0100-0199 and the RDX1000+ lint band"

-- | Criterion 3: nothing collides with doc 03 §17's published assignments.
-- |
-- | Doc 03 published `RDX0042`-`RDX0063` for surface features belonging to
-- | plans 19-28. A code in that band must be **one §17 published, at the
-- | number §17 gave it** — otherwise one number ships two meanings.
-- |
-- | This test used to assert that *no* code was in the band at all, which is
-- | the opposite of what its own comment said and would have blocked every
-- | plan 19-28 surface code. Plan 30 §1.5 found it by being the first plan to
-- | need one: `RDX0044` is §17's assignment for `return` in a `use` tail, and
-- | using it is the reservation working rather than a collision.
test "every reserved code is one doc 03 published"
  -- Doc 03 §17's table, transcribed. A code in the band and absent here is
  -- a number taken from a plan that has not claimed it yet.
  const PUBLISHED : &[(u16, &str)] = &[(44, "ReturnInCallbackTail")]

  for code in Code::all ()
    n = .number code
    if !(.contains (&n) (40..=99))
      continue
    name = format! \\-> "#{code:?}"
    -- One parameter read with `.0` / `.1`, where the Rust wrote the tuple
    -- pattern `|(num, who)|`: a lambda parameter is a name, not a pattern.
    assert! \\-> (.any (\\p -> p.0 == n && p.1 == name) (.iter PUBLISHED)) \\-> "#{code:?} is #{n}, inside the band doc 03 §17 reserved, and is not the diagnostic §17 assigned that number to"

test "renders as a four digit code"
  assert_eq! \\-> (.text Code::TabIndent) \\-> "RDX0001"
  assert_eq! \\-> (.text Code::ExpectedToken) \\-> "RDX0101"
''' % (ARMS, ALL)

print('%-40s written' % write('diagnostic/code', BODY))
