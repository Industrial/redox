# -*- coding: utf-8 -*-
"""redox-diag batch 3 — origin, plain_one, anchor_span."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

BODIES = {
    'source/origin': '''instance Origin
  -- | Map a column on the generated line back to a source column.
  -- |
  -- | Floored at 1: a position *left* of where the construct begins — inside
  -- | the emitter's own indentation — has no source column, and reporting the
  -- | construct's first column is better than reporting zero or wrapping.
  export map_col : &self -> usize -> usize
  map_col self gen_col = self.src_col + .saturating_sub self.gen_col gen_col

-- | Source `  a + b` at line 2 column 3; generated `    a + b` at column 5.
-- |
-- | `#[cfg(test)]` because a `test` declaration emits its own
-- | `mod __rdx_tests` and gives a helper no home inside it, so at module
-- | scope this is dead code in a non-test build.
#[cfg(test)]
origin : Origin
origin = Origin { span = (10, 15), src_line = 2, src_col = 3, gen_col = 5 }

test "shifts a column by the indentation difference"
  assert_eq! \\-> (.map_col 5 (origin ())) \\-> 3 \\-> "the first character"
  assert_eq! \\-> (.map_col 9 (origin ())) \\-> 7 \\-> "four characters in"

test "floors at the construct's first column"
  assert_eq! \\-> (.map_col 1 (origin ())) \\-> 3
  assert_eq! \\-> (.map_col 4 (origin ())) \\-> 3
''',

    'render/anchor_span': '''import crate::Span

-- | Move a span onto a character that can actually be drawn under.
-- |
-- | A parse error at end of input carries a span starting at `source.len()`,
-- | and `miette` renders such a label as nothing at all: the snippet box
-- | appears with no `┬` and no `here`. Rather than widen the label — which
-- | would underline the wrong text — the span is walked back to the last
-- | character that is not a line ending, so "unexpected end of input" points at
-- | the last thing the author wrote.
-- |
-- | A span already inside the text is returned untouched, so this costs nothing
-- | in the common case.
export(crate) anchor_span : &str -> Span -> Span
anchor_span source span =
  if span.0 < .len source
    return span
  trimmed = .trim_end_matches ['\\n', '\\r'] source
  case .next_back (.char_indices trimmed) of
    Some pair ->
      i = pair.0
      c = pair.1
      (i, i + .len_utf8 c)
    -- An empty source has nothing to point at; leave the span alone and let
    -- the renderer print a box with no label.
    None -> span

test "leaves a span inside the text alone"
  assert_eq! \\-> (anchor_span "abc\\n" (1, 2)) \\-> (1, 2)

test "walks back past a trailing newline"
  src = "ab\\n"
  assert_eq! \\-> (anchor_span src (3, 3)) \\-> (1, 2)
  assert_eq! \\-> (&src[1..2]) \\-> "b" \\-> "the anchored span covers the last character"

test "respects character boundaries"
  -- `é` is two bytes; a naive `len - 1` would slice mid-character.
  src = "é\\n"
  assert_eq! \\-> (anchor_span src (3, 3)) \\-> (0, 2)
  assert_eq! \\-> (&src[0..2]) \\-> "é"

test "gives up on an empty source"
  assert_eq! \\-> (anchor_span "" (0, 0)) \\-> (0, 0)
  assert_eq! \\-> (anchor_span "\\n\\n" (2, 2)) \\-> (2, 2)
''',

    'render/plain_one': '''import crate::Diagnostic
import crate::render::caret_width::caret_width
import crate::render::line_col::line_col
import crate::render::snippet_line::snippet_line
import std::fmt::Write

-- | Render one diagnostic without colour or width dependence.
-- |
-- | Deliberately close to doc 01 §29's own example output, so the plain form is
-- | recognisable rather than a second dialect:
-- |
-- | ```text
-- | error[RDX0101]: expected `->` after the signature
-- |   --> bad_arrow.sdx:1:15
-- |   |
-- | 1 | add = (a: i32)
-- |   |               ^
-- | ```
-- |
-- | The first word is the **severity**, not a constant: plan 38's `RDX1001` is
-- | a warning, and this renderer said `error[RDX1001]` until it stopped being
-- | one. A hardcoded word is the kind of thing that stays right for thirty-odd
-- | codes and is wrong for the thirty-first.
export(crate) plain_one : &Diagnostic -> String
plain_one d =
  at = line_col (&d.source) d.span.0
  line = at.0
  col = at.1
  level = case d.severity of
    crate::Severity::Error -> "error"
    crate::Severity::Warning -> "warning"
  mut out = format! \\-> "#{level}[#{(.text d.code)}]: #{d.message}\\n"
  _ = writeln! \\-> out \\-> "  --> #{d.path}:#{line}:#{col}"

  if snippet_line (&d.source) line col is Some found
    shown = found.0
    text = found.1
    caret = found.2
    gutter = .repeat (.len (.to_string shown)) " "
    _ = writeln! \\-> out \\-> "#{gutter} |"
    -- A literal tab in the snippet would put the caret line out of step with
    -- the source line, which matters most for the one diagnostic that is
    -- *about* a tab. Expanding to a single space keeps the two aligned.
    _ = writeln! \\-> out \\-> "#{shown} | #{(.replace '\\t' " " text)}"
    pad = .repeat (caret - 1) " "
    carets = .repeat (caret_width (&d.source) d.span) "^"
    _ = writeln! \\-> out \\-> "#{gutter} | #{pad}#{carets}"
  if &d.help is Some help
    _ = writeln! \\-> out \\-> "  = help: #{help}"
  out
''',
}

for rel, text in BODIES.items():
    print('%-40s written' % write(rel, text))
