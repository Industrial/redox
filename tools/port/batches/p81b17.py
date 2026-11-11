# -*- coding: utf-8 -*-
"""Plan 81 batch 17: `Sink`, the emitter's output.

Two things worth naming. `Sink` is the first ported type with a **lifetime
parameter on its instance** — `instance (Sink 'a)`, which `redox-lexer`'s
`Cursor` established. And it implements `std::fmt::Write`, which is what lets
a `write!`-based helper change in its parameter type and nowhere else; the
instance is `instance Write (Sink '_)`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/sink', '''
import std::fmt::Write

export(crate) data Sink = Sink {
  -- | The generated Rust so far.
  out : String,
  -- | The source text, needed to resolve a span to a line and column. Borrowed
  -- | rather than copied because the emitter already outlives it.
  source : &'a str,
  -- | The span every line begun from now on is attributed to.
  current : Option<Span>,
  -- | One entry per *completed* line; the line being built is still `current`.
  origins : Vec<Option<Origin>>,
}

-- The trait above is imported rather than written as a path: an `instance`
-- head takes a *name*, not `std::fmt::Write`. And the type's own doc comment
-- is the translated head's — repeating it here would emit it twice.
instance (Sink 'a)
  -- | An empty sink over a source, attributing nothing yet.
  export(crate) new : &'a str -> Self
  new source = Sink { out = String::new (), source, current = None, origins = Vec::new () }

  -- | Attribute everything emitted from here to `span`.
  export(crate) mark : Span -> &mut self -> ()
  mark span self =
    self.current := Some span
    ()

  -- | Stop attributing: the next lines are the emitter's own.
  export(crate) unmark : &mut self -> ()
  unmark self =
    self.current := None
    ()

  -- | Append one character.
  export(crate) push : char -> &mut self -> ()
  push ch self =
    .write_text (&.to_string ch) self
    ()

  -- | Append a string.
  export(crate) push_str : &str -> &mut self -> ()
  push_str s self =
    .write_text s self
    ()

  -- | Remove the last character, un-recording its line if it ended one.
  export(crate) pop : &mut self -> Option<char>
  pop self =
    ch = .pop self.out
    if ch == Some '\\n'
      .pop self.origins
    ch

  -- | Whether the output so far ends with a string.
  export(crate) ends_with : &str -> &self -> bool
  ends_with s self = .ends_with s self.out

  -- | Whether nothing has been emitted yet.
  export(crate) is_empty : &self -> bool
  is_empty self = .is_empty self.out

  -- | Finish, returning the Rust source and its map.
  -- |
  -- | `source_path` is recorded in the map's header; it is the `.sdx` path as
  -- | the caller knows it, which the emitter has no other way to learn.
  export(crate) finish : &str -> mut self -> (String, SourceMap)
  finish source_path self =
    -- A final line with no terminating newline is still a line.
    if !.is_empty self.out && !.ends_with '\\n' self.out
      .close_line self
    (self.out, SourceMap::new source_path self.origins)

  -- | Record the origin of the line just completed.
  close_line : &mut self -> ()
  close_line self =
    origin = .and_then (\\span -> .origin_of span self) self.current
    .push origin self.origins
    ()

  -- | Resolve a span to an [`Origin`], given where the line being built began.
  origin_of : Span -> &self -> Option<Origin>
  origin_of span self =
    if span.0 > .len self.source
      return None
    before = &self.source[0 .. span.0]
    src_line = .count (.matches '\\n' before) + 1
    src_col = .map_or 1 (\\l -> .count (.chars l) + 1) (.next (.rsplit '\\n' before))
    Some (Origin { span, src_line, src_col, gen_col = .gen_col_of_current_line self })

  -- | The column at which the current line's non-blank content begins.
  -- |
  -- | This is the generated half of the column shift: content on the generated
  -- | line starts after the emitter's indentation, and the source's starts
  -- | after the author's, so the difference is what [`Origin::map_col`]
  -- | subtracts.
  gen_col_of_current_line : &self -> usize
  gen_col_of_current_line self =
    line_start = .map_or 0 (\\i -> i + 1) (.rfind '\\n' self.out)
    n = .len self.out
    text = &self.out[line_start .. n]
    trimmed = .trim_start text
    if .is_empty trimmed
      -- Nothing on the line yet: content will begin where the cursor is.
      return .count (.chars text) + 1
    .count (.chars text) - .count (.chars trimmed) + 1

  -- | Append text, closing a line record at every newline it contains.
  write_text : &str -> &mut self -> ()
  write_text s self =
    for part in .split_inclusive '\\n' s
      .push_str part self.out
      if .ends_with '\\n' part
        -- The origin is computed *before* the newline is consumed, so the
        -- column it records is the one for the line that just ended. Pop the
        -- newline, measure, put it back.
        .pop self.out
        .close_line self
        .push '\\n' self.out
    ()

instance Write (Sink '_)
  write_str : &str -> &mut self -> std::fmt::Result
  write_str s self =
    .write_text s self
    Ok ()

test "attributes a line to the current mark"
  -- Doc 18 §2.2's two-line form: the signature occupies bytes 0..16 and
  -- the body's statement starts at 19 on line 3. Plan 64 made that
  -- statement a macro call, so the slice is longer than the `print 1` this
  -- premise used to name.
  --
  -- `\\#{1}` is a literal `#{1}`, so the fixture is Redox source rather than
  -- this file interpolating into it.
  source = "main : ()\\nmain =\\n  println! \\\\-> \\"\\#{1}\\"\\n"
  assert_eq! \\-> &source[19 .. 38] \\-> r##"println! \\-> "#{1}""## \\-> "the premise of this test"
  mut sink = Sink::new source
  -- The signature line and the equation head.
  .mark (0, 16) sink
  .push_str "fn main() {\\n" sink
  -- The macro call.
  .mark (19, 38) sink
  .push_str "    println!();\\n" sink
  .unmark sink
  .push_str "}\\n" sink
  -- `finished.0` is the text and `finished.1` the map.
  finished = .finish "a.sdx" sink
  -- `String::from` and then a comparison, where a brace-bearing string
  -- literal reaching a macro would be brace-doubled by `macro_argument` —
  -- plan 80 §2.4's defect, and `String::from` is the third spelling that
  -- avoids it after `r"…"` and a `char`.
  expected = String::from "fn main() {\\n    println!();\\n}\\n"
  assert_eq! \\-> finished.0 \\-> expected
  assert_eq! \\-> .len finished.1 \\-> 3
  assert_eq! \\-> .resolve 1 1 finished.1 \\-> Some (1, 1)
  -- Generated line 2 column 5 is the body's content; source line 3 column 3
  -- is where the author wrote it. Two source lines collapse into the `fn`
  -- header above it, which is the shift the map exists to carry.
  assert_eq! \\-> .resolve 2 5 finished.1 \\-> Some (3, 3) \\-> "content column aligns"
  assert_eq! \\-> .resolve 3 1 finished.1 \\-> None \\-> "the closing brace is synthetic"

-- | A mark that spans several generated lines maps every one of them.
test "carries a mark across lines"
  mut sink = Sink::new "a\\nb\\n"
  .mark (0, 1) sink
  .push_str "one\\ntwo\\nthree\\n" sink
  finished = .finish "a.sdx" sink
  assert_eq! \\-> .len finished.1 \\-> 3
  for line in 1 ..= 3
    assert_eq! \\-> .resolve line 1 finished.1 \\-> Some (1, 1) \\-> "line #{line}"

test "counts a final line without a newline"
  mut sink = Sink::new "a\\n"
  .mark (0, 1) sink
  .push_str "x" sink
  finished = .finish "a.sdx" sink
  assert_eq! \\-> finished.0 \\-> "x"
  assert_eq! \\-> .len finished.1 \\-> 1

test "write macro records the same way"
  mut sink = Sink::new "ab\\n"
  .mark (0, 2) sink
  _dropped = writeln! \\-> sink \\-> "hello"
  finished = .finish "a.sdx" sink
  assert_eq! \\-> finished.0 \\-> "hello\\n"
  assert_eq! \\-> .len finished.1 \\-> 1

-- | `pop` is how the emitter trims trailing blank lines; the map must shrink
-- | with the text or every record after the trim would be off by one.
test "pop removes the line it removes"
  mut sink = Sink::new "a\\n"
  .mark (0, 1) sink
  .push_str "one\\n\\n" sink
  assert_eq! \\-> .pop sink \\-> Some '\\n'
  finished = .finish "a.sdx" sink
  assert_eq! \\-> finished.0 \\-> "one\\n"
  assert_eq! \\-> .len finished.1 \\-> 1
''')

print('plan 81 batch 17: one file')
