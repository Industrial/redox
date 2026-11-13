# -*- coding: utf-8 -*-
"""Batch 10: `scan_string` and `scan_comment`.

`scan_comment`'s Rust binds `body` from each `strip_prefix` and then discards
it with `let _ = body;` — the marker's presence is the whole answer, so this
asks `.is_some` and drops both the binding and the discard.

`scan_string` is the first file to need `cur |> .eat delim` twice: the
statement above each is a **block-valued binding**, and a following `.name`
statement is read as field access on the block. Plan 69's finding, and the one
shape in this port that recurs often enough to be worth a habit — if the line
above binds a block, pipe.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lexer/scan_comment', '''
export(crate) scan_comment : &str -> Option<(TriviaKind, usize)>
scan_comment rest =
  after = (.strip_prefix "--" rest)?

  -- `-- |` and `-- !` are the doc forms. Checked before the generic rule
  -- because the marker is what makes them comments at all.
  --
  -- `.is_some` rather than the Rust's `if let Some(body) = …` followed by
  -- `let _ = body;`: the marker's presence is the whole answer, so the
  -- binding and the discard that cancelled it both go.
  if .is_some (.strip_prefix " |" after)
    return Some ((TriviaKind::DocComment, 4))
  if .is_some (.strip_prefix " !" after)
    return Some ((TriviaKind::InnerDocComment, 4))

  -- A plain line comment needs whitespace or the end of the line after the
  -- dashes. A third dash is still a comment — `---` is a divider people write
  -- — so an all-dash run counts as whitespace-terminated.
  opens = case .next (.chars after) of
    None -> true
    Some c -> .is_whitespace c || c == '-'
  -- Piped: the line above binds a block, and a `.name` statement after one is
  -- read as field access on it.
  opens |> .then_some ((TriviaKind::LineComment, 2))

test "a dash dash space opens a line comment"
  assert_eq! \\-> (scan_comment "-- hello") \\-> (Some ((TriviaKind::LineComment, 2)))

test "a bare dash dash at end of line opens one"
  assert_eq! \\-> (scan_comment "--") \\-> (Some ((TriviaKind::LineComment, 2)))
  assert_eq! \\-> (scan_comment "--\\n") \\-> (Some ((TriviaKind::LineComment, 2)))

-- | A divider is a comment, not an operator.
test "a run of dashes opens one"
  assert_eq! \\-> (scan_comment "-----") \\-> (Some ((TriviaKind::LineComment, 2)))

test "the doc markers are recognised"
  assert_eq! \\-> (scan_comment "-- | outer") \\-> (Some ((TriviaKind::DocComment, 4)))
  assert_eq! \\-> (scan_comment "-- ! inner") \\-> (Some ((TriviaKind::InnerDocComment, 4)))

-- | The hazard doc 18 §5.1 names, taken in the strict reading: this is
-- | arithmetic, not a comment.
test "dashes against a digit are not a comment"
  assert_eq! \\-> (scan_comment "--1") \\-> None

test "dashes against an identifier are not a comment"
  assert_eq! \\-> (scan_comment "--x") \\-> None

test "a single dash is not a comment"
  assert_eq! \\-> (scan_comment "-") \\-> None
  assert_eq! \\-> (scan_comment "- 1") \\-> None
  assert_eq! \\-> (scan_comment "-> x") \\-> None

test "text that is not dashes is not a comment"
  assert_eq! \\-> (scan_comment "hello") \\-> None
  assert_eq! \\-> (scan_comment "") \\-> None
''')

write('lexer/scan_string', '''
export(crate) scan_string : &mut Cursor -> Result<TokenKind, LexError>
scan_string cur =
  start = .pos cur

  triple = .starts_with "\\"\\"\\"" cur
  delim = if triple
    "\\"\\"\\""
  else
    "\\""
  -- Piped, because the line above binds a **block**: a following `.name`
  -- statement is read as field access on it, which is `RDX0115`.
  cur |> .eat delim

  -- Collect the raw body, stopping at the closing delimiter. Escapes are not
  -- decoded here — the body is split into parts first, so an escaped `\\#`
  -- cannot open an interpolation.
  body_start = .pos cur
  loop
    if .is_eof cur
      return Err (LexError {
        code = Code::UnterminatedString,
        help = None,
        message = format! \\-> "unterminated string literal; expected closing #{delim}",
        span = (start, .pos cur),
      })
    if .starts_with delim cur
      break
    -- An interpolation's body is skipped whole, because a string literal
    -- inside it must not be mistaken for this string's closing quote —
    -- `"outer #{String::from("inner")} done"`. Plan 23 found this claim
    -- false after plan 02 asserted it; `skip_interpolation` says why.
    if .starts_with "\\#{" cur
      (skip_interpolation cur 0)?
      continue
    if .peek cur == Some '\\\\'
      .bump cur
      .bump cur
      continue
    if !triple && .peek cur == Some '\\n'
      -- Names both the likely cause (a missing closing quote) and the
      -- alternative, because a reader who meant to span lines needs the
      -- second half and a reader who forgot a quote needs the first.
      return Err (LexError {
        code = Code::UnterminatedString,
        help = None,
        message = .to_string "unterminated string literal: a single-quoted string may not span lines — use \\"\\"\\" for a multi-line string",
        span = (start, .pos cur),
      })
    .bump cur
  body_end = .pos cur
  .eat delim cur

  raw = .to_string (.slice (start, .pos cur) cur)
  body = .slice (body_start, body_end) cur

  -- A `"""` block drops its delimiter lines and loses its common indentation
  -- (doc 01 §7.2), before any part splitting. The rule lives in
  -- `verbatim_triple` because plan 45's escape hatch needs the same body
  -- *undecoded*, and two copies of this would be free to disagree about what a
  -- block's body is.
  owned = if triple
    crate::verbatim_triple::dedent_triple_body::dedent_triple_body body
  else
    .to_string body

  Ok (TokenKind::Str {
    parts = (split_parts (&owned) body_start)?,
    raw,
  })
''')

print('batch 10: two files')
