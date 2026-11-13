# -*- coding: utf-8 -*-
"""voltaic-diag batch 2 — render_map, line_col, caret_width, and the first tests.

The inline `#[cfg(test)] mod tests` blocks become module-scope `test "…"`
declarations, which plan 72's table called *"reachable today, awkwardly"*:
item-level `test` emits `#[cfg(test)] mod __rdx_tests { use super::*; }` and so
sees the file's private items, which is what an inline module was for.

`line_col`'s test input is a **raw** string, because the Rust literal contains
`#{1}` and `#{…}` is interpolation in any Voltaic string, not only a macro
argument.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

BODIES = {
    'source/map/render_map': '''import crate::source::map::SourceMap
import crate::source::map::version::VERSION
import std::fmt::Write

-- | Serialise a map as the `.xrs.map` sidecar.
-- |
-- | Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 4. Text, one record per
-- | generated line:
-- |
-- | ```text
-- | voltaic-map 1 examples/phase1.sdx
-- | 1 1 1 4
-- | 2 2 3 5
-- | 3 -
-- | ```
-- |
-- | `gen_line src_line src_col gen_col`, or `gen_line -` for a synthetic line.
-- | The generated line number is written even though it is the record's index,
-- | because the first thing anyone does with this file is open it next to the
-- | `.xrs` and compare a line — and counting lines in a text editor to find
-- | record 47 is exactly the friction that stops people checking.
export render_map : &SourceMap -> String
render_map map =
  mut out = format! \\-> "voltaic-map #{VERSION} #{map.source_path}\\n"
  last = .len map
  for gen_line in 1..=last
    -- `.origin gen_line map`: the **receiver is the last argument** (plan
    -- 57), so the Rust order reads `gen_line.origin(map)` and rustc says so.
    case .origin gen_line map of
      Some o ->
        _ = writeln! \\-> out \\-> "#{gen_line} #{o.src_line} #{o.src_col} #{o.gen_col}"
      None ->
        _ = writeln! \\-> out \\-> "#{gen_line} -"
  out
''',

    'render/line_col': '''-- | Resolve a byte offset to a 1-based line and column.
-- |
-- | Columns are counted in characters, not bytes, so a diagnostic under a
-- | multi-byte character points at the right place.
export(crate) line_col : &str -> usize -> (usize, usize)
line_col source offset =
  clamped = .min (.len source) offset
  before = &source[0..clamped]
  line = .count (.matches '\\n' before) + 1
  col = before
    |> .rsplit '\\n'
    |> .next
    |> .map_or 1 (\\l -> .count (.chars l) + 1)
  (line, col)

test "resolves a byte offset"
  src = r#"main : ()
main =
  println! \\-> "#{1}"
"#
  assert_eq! \\-> (line_col src 0) \\-> (1, 1)
  -- `main : ()\\n` is 10 bytes, `main =\\n` is 7, so the `p` of `print` is
  -- at 19 — line 3, column 3.
  --
  -- Both numbers moved by one when plan 52 made the type marker `:` rather
  -- than `::`. The slice assertion below is deliberately there to catch
  -- exactly that: an offset derived in a comment is only as trustworthy as
  -- the assertion that re-derives it from the source.
  assert_eq! \\-> (&src[19..20]) \\-> "p"
  assert_eq! \\-> (line_col src 19) \\-> (3, 3)

test "counts columns in characters not bytes"
  -- `é` is two bytes, so a byte-based column would report 3.
  assert_eq! \\-> (line_col "é x" 3) \\-> (1, 3)

test "clamps past the end"
  assert_eq! \\-> (line_col "ab" 99) \\-> (1, 3)
''',

    'render/caret_width': '''import crate::Span

-- | How many `^` to draw under a span.
-- |
-- | Characters, not bytes. A span is a byte range, so subtracting its ends
-- | gives the right number only for ASCII — plan 05's UI corpus caught it
-- | drawing `^^` under a single `§`, because `§` is two bytes. The column the
-- | caret starts at is already counted in characters, so the width has to be
-- | too or the two disagree.
-- |
-- | A span that is empty, or that does not line up with the text, still gets
-- | one caret: a diagnostic with no visible marker is worse than one whose
-- | marker is a character wide.
export(crate) caret_width : &str -> Span -> usize
caret_width source span = source
  -- `(span.0..span.1)` parenthesised: a range binds **looser** than
  -- application, so the bare form reads `(.get span.0)..span.1` and the
  -- pipeline's value then meets it as a call. Same lesson as plan 69's
  -- `(*is_move)`.
  |> .get (span.0..span.1)
  |> .map_or 1 (\\text -> .max 1 (.count (.chars text)))

test "counts characters not bytes"
  -- `§` is two bytes but occupies one column.
  assert_eq! \\-> (caret_width "1 § 2" (2, 4)) \\-> 1

test "spans several characters"
  assert_eq! \\-> (caret_width "print 0x" (6, 8)) \\-> 2

test "always draws at least one"
  assert_eq! \\-> (caret_width "abc" (1, 1)) \\-> 1

-- | A span past the end — an error at end of input — must not panic.
test "survives a span outside the source"
  assert_eq! \\-> (caret_width "ab" (5, 9)) \\-> 1

-- | Nor one that would slice through a character.
test "survives a span off a character boundary"
  assert_eq! \\-> (caret_width "é" (1, 2)) \\-> 1
''',
}

for rel, text in BODIES.items():
    print('%-40s written' % write(rel, text))
