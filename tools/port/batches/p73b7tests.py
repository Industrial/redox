# -*- coding: utf-8 -*-
"""parse_map's tests, which the first pass dropped.

Worth noting as a method failure rather than a typo: the body replaced
everything from the TODO down, and the TODO sits above the `impl` *and* the
`#[cfg(test)] mod tests` block, so a body that stops at the last function
silently discards the tests. The generated `.rs` must carry everything the
hand-written one did — `tests/port.rs` compares them byte for byte — so a
dropped test module would surface as a diff, but only once the crate joins
`PORTED`.

Nine of redox-diag's files have inline tests and this is the only one where
they were missed, because it is the only one whose body was written in a
separate step from its tests.
"""
import io

p = '/home/tom/Code/rust/redox/crates/redox-diag/sdx/source/map/parse_map.sdx'
s = io.open(p, encoding='utf-8').read().rstrip('\n')

s += '''

import crate::source::map::render_map::render_map

-- | `#[cfg(test)]`: a helper for the tests below, dead code otherwise.
#[cfg(test)]
sample : SourceMap
sample = SourceMap::new "examples/phase1.sdx" (vec! \\-> (Some (Origin { span = (0, 0), src_line = 1, src_col = 1, gen_col = 4 })) \\-> None \\-> (Some (Origin { span = (0, 0), src_line = 2, src_col = 3, gen_col = 5 })))

test "round trips"
  text = render_map (&(sample ()))
  assert_eq! \\-> (.expect "should parse" (parse_map (&text))) \\-> (sample ())

test "rejects a foreign file"
  err = .expect_err "should fail" (parse_map "{\\"version\\":3}\\n")
  assert! \\-> (.contains "does not begin" (&err)) \\-> "#{err}"

test "rejects a future version"
  err = .expect_err "should fail" (parse_map "redox-map 99 a.sdx\\n")
  assert! \\-> (.contains "version 99" (&err)) \\-> "#{err}"

test "rejects a header with no source"
  err = .expect_err "should fail" (parse_map "redox-map 1\\n")
  assert! \\-> (.contains "names no source" (&err)) \\-> "#{err}"

-- | Records are positional, so a gap would shift every line after it. Better
-- | to refuse the file than to resolve the wrong line.
test "rejects records out of order"
  err = .expect_err "should fail" (parse_map "redox-map 1 a.sdx\\n1 1 1 1\\n3 2 1 1\\n")
  assert! \\-> (.contains "must be in order" (&err)) \\-> "#{err}"

test "rejects a malformed record"
  err = .expect_err "should fail" (parse_map "redox-map 1 a.sdx\\n1 2 3\\n")
  assert! \\-> (.contains "3 fields" (&err)) \\-> "#{err}"

test "rejects a zero column"
  err = .expect_err "should fail" (parse_map "redox-map 1 a.sdx\\n1 1 0 1\\n")
  assert! \\-> (.contains "1-based" (&err)) \\-> "#{err}"

test "parses a map with no records"
  map = .expect "should parse" (parse_map "redox-map 1 a.sdx\\n")
  assert! \\-> (.is_empty map)
  assert_eq! \\-> map.source_path \\-> "a.sdx"
'''

io.open(p, 'w', encoding='utf-8').write(s + '\n')
print('parse_map.sdx: eight tests restored')
