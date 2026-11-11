# -*- coding: utf-8 -*-
"""Plan 80 batch 11: `remap`, the last of `redox-driver`'s 43 owed bodies."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('remap', '''
export remap : &RustcMessage -> &SourceMap -> &str -> &str -> &str -> &str -> Diagnostic
remap message map sdx_path sdx_source xrs_path xrs_source =
  -- rustc gives most diagnostics a code and some — a plain lint, say — none.
  -- Falling back to the level keeps the rendered output from printing an empty
  -- `error[]`, and is still rustc's own word rather than one of Redox's.
  code = DiagnosticCode::Foreign (.unwrap_or_else (\\-> .clone message.level) (.clone message.code))

  -- rustc's level, now that plan 38 gives it somewhere to go. Before the
  -- severity existed, a rustc warning rendered as `error[warning]` — the
  -- comment above says the fallback exists to avoid an empty `error[]`, and
  -- the result was a worse lie than the one it prevented.
  --
  -- Plan 38 §1.2 said this path was unaffected, and §10 records why it was
  -- changed anyway: once a severity exists, rendering a warning as an error is
  -- a defect one would be *choosing* to leave. It changes only the rendering —
  -- `check` already does not fail on a rustc warning.
  severity = if message.level == "warning"
    redox_diag::Severity::Warning
  else
    redox_diag::Severity::Error

  case .origin message.line map of
    Some origin ->
      shifted = .map_col message.col origin
      Diagnostic { code, severity, message = .clone message.message, span = narrow origin.span sdx_source origin.src_line shifted, help = None, path = .to_string sdx_path, source = .to_string sdx_source }
    None ->
      offset = generated_offset xrs_source message.line message.col
      note = format! \\-> "this is on a line Redox generated, so it has no place in your source; see #{xrs_path}:#{message.line}:#{message.col}"
      Diagnostic { code, severity, message = .clone message.message, span = (offset, offset + 1), help = Some note, path = .to_string xrs_path, source = .to_string xrs_source }

#[cfg(test)]
const SDX : &str = "add = (a: i32, b: i32): i32 ->\\n  a + \\"x\\"\\n"

#[cfg(test)]
const XRS : &str = "fn add(a: i32, b: i32) -> i32 {\\n    a + \\"x\\"\\n}\\n"

-- | `  a + "x"` spans bytes 33..40.
#[cfg(test)]
map_of : SourceMap
map_of = SourceMap::new "p.sdx" (vec! \\-> (Some (redox_diag::Origin { span = (0, 30), src_line = 1, src_col = 1, gen_col = 1 })) \\-> (Some (redox_diag::Origin { span = (33, 40), src_line = 2, src_col = 3, gen_col = 5 })) \\-> None)

#[cfg(test)]
message_at : usize -> usize -> RustcMessage
message_at line col = RustcMessage { path = .to_string "p.xrs", line, col, level = .to_string "error", code = Some (.to_string "E0277"), message = .to_string "cannot add `&str` to `i32`" }

#[cfg(test)]
run : &RustcMessage -> redox_diag::Diagnostic
run m = remap m (&map_of ()) "p.sdx" SDX "p.xrs" XRS

-- | Case 1, and the measured one: rustc's `.xrs:2:7` is the `+`, and the
-- | `.sdx`'s `+` is at line 2 column 5.
test "narrows to the exact character on a verbatim line"
  d = run (&message_at 2 7)
  assert_eq! \\-> d.path \\-> "p.sdx"
  assert_eq! \\-> &SDX[d.span.0 .. d.span.1] \\-> "+"
  assert_eq! \\-> .text d.code \\-> "E0277"
  assert_eq! \\-> d.message \\-> "cannot add `&str` to `i32`" \\-> "rustc's message passes through"
  assert! \\-> .is_none d.help

-- | Case 2: a column past the source line underlines the whole statement
-- | rather than clamping to a plausible-looking character.
test "underlines the whole span when the column misses"
  d = run (&message_at 2 99)
  assert_eq! \\-> d.span \\-> (33, 40)
  assert_eq! \\-> &SDX[d.span.0 .. d.span.1] \\-> "a + \\"x\\""

-- | Case 3: a synthetic line is reported against the `.xrs`, with a note.
test "reports a synthetic line against the generated file"
  d = run (&message_at 3 1)
  assert_eq! \\-> d.path \\-> "p.xrs"
  assert_eq! \\-> d.source \\-> XRS
  help = .expect "a synthetic line should explain itself" d.help
  assert! \\-> .contains "Redox generated" help \\-> "#{help}"
  assert! \\-> .contains "p.xrs:3:1" help \\-> "#{help}"

-- | A line past the map's end is not relocated either.
test "reports an unmapped line against the generated file"
  d = run (&message_at 99 1)
  assert_eq! \\-> d.path \\-> "p.xrs"

-- | A diagnostic rustc gave no code for still carries its level, so the
-- | rendered output never prints an empty `error[]`.
test "falls back to the level when rustc gave no code"
  mut m = message_at 2 7
  m.code := None
  m.level := .to_string "warning"
  assert_eq! \\-> .text (run (&m)).code \\-> "warning"
''')

print('plan 80 batch 11: one file')
