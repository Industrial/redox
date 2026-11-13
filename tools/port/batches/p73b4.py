# -*- coding: utf-8 -*-
"""voltaic-diag batch 4 — diagnostic, snippet_line, diagnostic_code.

Shapes first met here:

* **`instance From Code DiagnosticCode`** is `impl From<Code> for
  DiagnosticCode`: the **last** name in an instance head is the target and the
  rest form the class with its arguments, which `instance_decl`'s own doc
  states and `.tmp/probe_from.sdx` confirms.
* **`instance From Code DiagnosticCode`**, as above.

`diagnostic` itself is batch 5's: it was written here first with the
elided `&mut Formatter`, and batch 5 rewrote it idiomatically once `'_`
stopped being hoisted. A file written twice is destroyed the second time,
because `write()` finds no TODO and cuts at the first construct.

`snippet_line`'s test inputs drop plan 64's `println! \\-> "#{…}"` for
`println 1`, for the reason `line_col`'s did: `#{…}` is interpolation in any
Voltaic string. The columns are recomputed from the shorter line and the comment
that explains them is rewritten rather than left describing the old numbers.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

BODIES = {

    'render/snippet_line': '''-- | Choose the source line to show under a diagnostic, and where to point.
-- |
-- | Returns `(line number, line text, column)`, all 1-based.
-- |
-- | The subtlety is end-of-file. A file ending in a newline makes the offset
-- | one past the last character resolve to a line that exists but holds no
-- | text, and an "unexpected end of file" error rendered with no snippet at
-- | all is the least useful diagnostic there is. So when the resolved line is
-- | empty and there is a line above it, this backs up and points just past the
-- | end of that line — which is where the reader has to type.
export(crate) snippet_line : &str -> usize -> usize -> Option<(usize, &str, usize)>
snippet_line source line col =
  text = source |> .lines |> .nth (line - 1) |> .unwrap_or ""
  if !(.is_empty text)
    return Some (line, text, col)
  mut n = line
  while n > 1
    n := n - 1
    -- The `?` outside the whole call. `.nth (n - 1)?` puts it **inside the
    -- argument** — `.nth((n - 1)?)` — because a parenthesised argument is
    -- where it binds.
    above = (source |> .lines |> .nth (n - 1))?
    if !(.is_empty above)
      return Some (n, above, .count (.chars above) + 1)
  None

test "keeps a line that has text"
  -- `  println 1` where the Rust wrote plan 64's `println! \\-> "#{1}"`:
  -- `#{…}` is interpolation in any Voltaic string, so that literal cannot be
  -- written directly. What the test asserts is unaffected.
  src = "main : ()\\nmain =\\n  println 1\\n"
  assert_eq! \\-> (snippet_line src 3 3) \\-> (Some (3, "  println 1", 3))

-- | The case that motivates the function: an error at end of file.
test "backs up from the phantom line after a trailing newline"
  src = "main : ()\\nmain =\\n  println 1\\n"
  -- 12, one past the last character of `  println 1`, which is eleven
  -- characters. The Rust's 26 counted plan 64's longer macro call.
  assert_eq! \\-> (snippet_line src 4 1) \\-> (Some (3, "  println 1", 12))

test "skips blank lines on the way up"
  src = "x\\n\\n\\n"
  assert_eq! \\-> (snippet_line src 4 1) \\-> (Some (1, "x", 2))

test "gives up on an empty source"
  assert_eq! \\-> (snippet_line "" 1 1) \\-> None
''',

    'diagnostic/diagnostic_code': '''instance DiagnosticCode
  -- | The code as it is printed.
  export text : &self -> String
  text self = case self of
    DiagnosticCode::Voltaic code -> .text code
    DiagnosticCode::Foreign text -> .clone text

  -- | Whether this code comes from Voltaic's catalogue.
  -- |
  -- | Plan 05's corpus checks read `.stderr` files for `RDX` codes; this is how
  -- | a later `check` UI case can be excluded from that count without the check
  -- | having to pattern-match on the text.
  export is_voltaic : &self -> bool
  is_voltaic self = case self of
    DiagnosticCode::Voltaic _ -> true
    _ -> false

instance From Code DiagnosticCode
  from : Code -> Self
  from code = DiagnosticCode::Voltaic code

test "renders a voltaic code from the catalogue"
  code : DiagnosticCode = .into Code::ExpectedToken
  assert_eq! \\-> (.text code) \\-> "RDX0101"
  assert! \\-> (.is_voltaic code)

test "renders a foreign code untouched"
  code = DiagnosticCode::Foreign (.to_string "E0308")
  assert_eq! \\-> (.text code) \\-> "E0308"
  assert! \\-> !(.is_voltaic code)

-- | A lint name is not numbered, and must survive anyway.
test "carries a foreign code that is not a number"
  code = DiagnosticCode::Foreign (.to_string "unused_variables")
  assert_eq! \\-> (.text code) \\-> "unused_variables"
''',
}

for rel, text in BODIES.items():
    print('%-40s written' % write(rel, text))
