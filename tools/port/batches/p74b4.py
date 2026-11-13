# -*- coding: utf-8 -*-
"""Batch 4: `scan_rust_block` and `split_parts`.

**The lexer has to write `#{` as a literal**, which is the first file in any
port to need it: `split_parts` tests `cur.starts_with("#{")`, and in Voltaic that
spelling opens an interpolation. `\\#` is a valid escape — `scan_escape` maps
`'#' => '#'` — so `"\\#{"` is the two characters, and this crate is the one
place where the language's own syntax collides with the text it is about.

**A tuple-destructuring binding is bound as a pair**, the same way batch 3's
`for` binder had to be: `let (open, close) = …` has no Voltaic spelling, so the
pair is named and its fields read. Three sites here.

**And `end` is renamed `stop` on rebinding.** The Rust writes `let end = end?;`
over its own `Option`; `RDX1003` reports a shadow, and plan 54 leaf 3 made that
a lint on purpose.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lexer/scan_rust_block', '''
export(crate) scan_rust_block : &mut Cursor<'_> -> usize -> Option<String>
scan_rust_block cur base =
  mark = .mark cur
  .eat_spaces cur
  if .peek cur != Some '\\n'
    .reset_to mark cur
    return None

  -- Every line of the body, including the blank ones between its statements.
  -- `end` tracks the last **non-blank** line's newline, so a trailing blank
  -- line is left for the off-side rule rather than swallowed into the block.
  body_start = .pos cur + 1
  mut end = None
  loop
    line_mark = .mark cur
    if .peek cur != Some '\\n'
      break
    .bump cur
    width = .eat_spaces cur
    if .peek cur == Some '\\n'
      -- A blank line belongs to the block only if something deeper follows,
      -- which the next turn of this loop decides.
      continue
    if .is_eof cur || width <= base
      .reset_to line_mark cur
      break
    .eat_while (\\c -> c != '\\n') cur
    end := Some (.pos cur)

  -- `stop`, not `end`: the Rust rebinds `let end = end?` over its own
  -- `Option`, and `RDX1003` reports that shadow — plan 54 leaf 3 made it a
  -- lint deliberately.
  stop = end?
  .reset_to stop cur
  Some (dedent_block (.slice (body_start, stop) cur))
''')

write('lexer/scan_string/split_parts', '''
export(crate) split_parts : &str -> usize -> Result<Vec<StrPart>, LexError>
split_parts body base =
  mut parts = Vec::new ()
  mut text = String::new ()
  mut cur = Cursor::new body

  while !(.is_eof cur)
    -- `"\\#{"`, escaped: in Voltaic that spelling **opens an interpolation**,
    -- and `\\#` is the escape for a literal `#`. This crate is the one place
    -- where the language's own syntax collides with the text it is about.
    if .starts_with "\\#{" cur
      if !(.is_empty text)
        .push (StrPart::Text (std::mem::take (&mut text))) parts

      -- The body ends at the `}` that returns brace depth to zero, so a
      -- struct literal or a closure inside an interpolation stays intact —
      -- and a *string* inside it keeps its braces, which is why this is the
      -- same function the outer scanner uses. Two scanners walking the same
      -- text must agree about where it ends; plan 23 §10 records what it cost
      -- when they did not.
      bounds = (skip_interpolation (&mut cur) base)?
      open = bounds.0
      close = bounds.1

      inner = .unwrap_or "" (.get (open..close) body)
      split = split_format_spec inner
      expr = split.0
      spec = split.1
      .push (StrPart::Interp {
        tokens = (lex_fragment expr (base + open))?,
        spec = .map str::to_string spec,
      }) parts
      continue

    if .peek cur == Some '\\\\'
      .bump cur
      -- `cur` runs over the string's body, so an escape error reports an
      -- offset into that body; rebase it onto the file or the diagnostic
      -- names the wrong line entirely.
      .push (.map_err (\\e -> .offset_by base e) (scan_escape (&mut cur)))? text
      continue

    .push (.expect "not at eof" (.bump cur)) text

  if !(.is_empty text)
    .push (StrPart::Text text) parts
  Ok parts
''')

print('batch 4: two files')
