# -*- coding: utf-8 -*-
"""Batch 2: `Trivia`'s constructors, `trim_comment`, and three scanners.

Two shapes appear here for the first time in this port.

**A let-chain becomes a guarded `case` arm.** `scan_quote` is written
`if let Some(ch) = decoded && cur.peek() == Some('\\'')`, and Voltaic has no
`if let … && …`; a `case` arm with a `|` guard says the same thing, and reads
better for being one construct rather than two glued together.

**A block-valued binding is safe here**, which is worth saying because plan 69
found it swallowing the statement below. The hazard is specifically a following
statement that opens with `.name`: the value goes through the Pratt loop, a
block ends on a `Dedent` that does not stop it, and `postfix` reads the `.` as
field access. `scan_quote`'s binding is followed by a `case`, so it does not
arise — and `crates/voltaic-cli/tests/ui/block_valued_binding_eats_the_next_statement.sdx`
is the fixture that pins the rule to the binding.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('trivia', '''
instance Trivia
  -- | A comment of the given form, with its text and span.
  export(crate) comment : TriviaKind -> String -> Span -> Self
  comment kind text span = Trivia { kind, text, span }

  -- | A run of blank lines, which carries no text of its own.
  export(crate) blank : Span -> Self
  blank span = Trivia {
    kind = TriviaKind::BlankLines,
    text = String::new (),
    span,
  }
''')

write('lexer/trim_comment', '''
export(crate) trim_comment : &str -> String
trim_comment rest = .to_string (.unwrap_or rest (.strip_prefix ' ' rest))

test "strips one leading space and no more"
  assert_eq! \\-> (trim_comment " text") \\-> "text"
  assert_eq! \\-> (trim_comment "text") \\-> "text"
  assert_eq! \\-> (trim_comment "  text") \\-> " text"
  assert_eq! \\-> (trim_comment "") \\-> ""
''')

write('lexer/scan_cstring', '''
export(crate) scan_cstring : &mut Cursor -> Result<TokenKind, LexError>
scan_cstring cur =
  start = .pos cur
  -- the `c`
  .bump cur
  .eat "\\"" cur

  mut text = String::new ()
  while !(.starts_with "\\"" cur)
    if .is_eof cur
      return Err (LexError {
        code = Code::UnterminatedByteLiteral,
        help = None,
        message = .to_string "unterminated C string",
        span = (start, .pos cur),
      })
    if .peek cur == Some '\\\\'
      .bump cur
      .push (scan_escape cur)? text
    else
      .push (.expect "not at eof" (.bump cur)) text
  .eat "\\"" cur
  Ok (TokenKind::CStr text)
''')

write('lexer/scan_raw_string', '''
export(crate) scan_raw_string : &mut Cursor -> Result<TokenKind, LexError>
scan_raw_string cur =
  start = .pos cur
  -- the `r`
  .bump cur

  hash_span = .eat_while (\\c -> c == '#') cur
  hashes = .len (.slice hash_span cur)

  if !(.eat "\\"" cur)
    return Err (LexError {
      code = Code::UnterminatedRawString,
      help = None,
      message = .to_string "a raw string must open with `\\"` after `r` and any `#`",
      span = (start, .pos cur),
    })

  -- `pad` is bound rather than interpolated inline: a nested string literal
  -- inside `#{…}` ends the interpolation early, which `RDX0005` reports as an
  -- unterminated one several lines later.
  pad = .repeat hashes "#"
  closing = format! \\-> "\\"#{pad}"
  text_start = .pos cur
  while !(.starts_with (&closing) cur)
    if .is_none (.bump cur)
      return Err (LexError {
        code = Code::UnterminatedRawString,
        help = None,
        message = format! \\-> "unterminated raw string; expected closing `#{closing}`",
        span = (start, .pos cur),
      })
  text = .to_string (.slice (text_start, .pos cur) cur)
  .eat (&closing) cur
  Ok (TokenKind::RawStr { text, hashes })
''')

write('lexer/scan_quote', '''
export(crate) scan_quote : &mut Cursor -> Result<TokenKind, LexError>
scan_quote cur =
  start = .pos cur
  mark = .mark cur

  -- the `'`
  .bump cur

  -- Try a char literal first: one character (or one escape), then `'`.
  decoded = if .peek cur == Some '\\\\'
    .bump cur
    .ok (scan_escape cur)
  else
    .bump cur

  -- A guarded `case` arm where the Rust had a let-chain: Voltaic has no
  -- `if let … && …`, and one construct reads better than two glued together.
  case decoded of
    Some ch | .peek cur == Some '\\'' ->
      .bump cur
      return Ok (TokenKind::Char ch)
    _ -> ()

  -- Not a char literal: rewind and read a lifetime or label name.
  --
  -- `cur |> .reset_to mark` rather than `.reset_to mark cur`, because the
  -- statement above is a `case`. Plan 69's block-valued-binding finding
  -- **also applies to a `case` in statement position**, which the UI fixture
  -- for it says it does not: an `if`/`else` statement followed by `.name`
  -- really is safe, and a `case` statement is not. So the contrast pins the
  -- rule to the construct, not to the binding.
  cur |> .reset_to mark
  -- the `'`
  .bump cur
  name_span = .eat_while (\\c -> .is_ascii_alphanumeric c || c == '_') cur
  name = .to_string (.slice name_span cur)
  if .is_empty name
    return Err (LexError {
      code = Code::BadQuote,
      help = None,
      message = .to_string "a `'` must open a character literal or name a lifetime or label",
      span = (start, .pos cur),
    })
  Ok (TokenKind::Label name)
''')

print('batch 2: five files')
