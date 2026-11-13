# -*- coding: utf-8 -*-
"""Batch 8: `scan_escape`, and one error literal where the Rust had eight.

`scan_escape`'s Rust opens with an `err` closure and calls it from eight
places. A Voltaic lambda parameter is a bare name, so `|message: String, cur:
&Cursor|` has no spelling — and writing the `LexError` literal out eight times,
as `scan_byte` had to, would be the worst version of this file.

So the scan yields `Result<char, String>` and the code and span are attached
**once**, at the end. That is shorter than the Rust and says something the
Rust did not: every failure in this function is the same code at the same span,
and only the message differs. The closure encoded that fact and then let each
caller re-state it.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lexer/scan_escape', '''
export(crate) scan_escape : &mut Cursor -> Result<char, LexError>
scan_escape cur =
  start = .saturating_sub 1 (.pos cur)

  ch = case .bump cur of
    Some c -> c
    None ->
      return Err (LexError {
        code = Code::UnknownEscape,
        help = None,
        message = .to_string "unterminated escape sequence",
        span = (start, .pos cur),
      })

  -- **`Result<char, String>`, and the code and span attached once below.**
  -- The Rust had an `err` closure and eight call sites; a Voltaic lambda
  -- parameter is a bare name, so that closure has no spelling. Carrying just
  -- the message says what the closure encoded and each caller re-stated —
  -- every failure here is the same code at the same span.
  decoded : Result<char, String> = case ch of
    'n' -> Ok '\\n'
    'r' -> Ok '\\r'
    't' -> Ok '\\t'
    '0' -> Ok '\\0'
    '\\\\' -> Ok '\\\\'
    '"' -> Ok '"'
    '\\'' -> Ok '\\''
    '#' -> Ok '#'
    'x' ->
      -- **Exactly two characters, not as many hex digits as follow.** Rust
      -- reads two and stops, so `b"\\\\x7fELF"` is `0x7f` then `E`, `L`, `F`.
      -- Scanning greedily consumed the `E` as a third digit and rejected a
      -- literal rustc accepts — found by plan 23's example, which is the only
      -- place a hex escape has ever been followed by a letter.
      at = .pos cur
      .bump cur
      .bump cur
      text = .slice (at, .pos cur) cur
      if .len text != 2 || !(.all (\\c -> .is_ascii_hexdigit c) (.chars text))
        Err (.to_string "`\\\\x` takes exactly two hexadecimal digits")
      else
        case u8::from_str_radix text 16 of
          -- `char::from`, where the Rust wrote `value as char`: an `as` cast
          -- is graded `UNREACHABLE`, and `char::from` on a `u8` is total.
          Ok v -> Ok (char::from v)
          Err _ -> Err (format! \\-> "`\\\\x#{text}` is not a valid byte escape")
    'u' ->
      if !(.eat "{" cur)
        Err (.to_string "`\\\\u` must be followed by `{`")
      else
        digits = .eat_while (\\c -> .is_ascii_hexdigit c) cur
        text = .to_string (.slice digits cur)
        if !(.eat "}" cur)
          Err (.to_string "unterminated `\\\\u{…}` escape")
        else
          case u32::from_str_radix (&text) 16 of
            Err _ -> Err (format! \\-> "`\\\\u{#{text}}` is not valid hexadecimal")
            Ok value ->
              case char::from_u32 value of
                Some c -> Ok c
                None -> Err (format! \\-> "`\\\\u{#{text}}` is not a Unicode scalar value")
    other -> Err (format! \\-> "unknown escape sequence `\\\\#{other}`")

  case decoded of
    Ok c -> Ok c
    Err message -> Err (LexError {
      code = Code::UnknownEscape,
      help = None,
      message,
      span = (start, .pos cur),
    })

-- | `#[cfg(test)]`: a helper the three tests below share. Voltaic can put
-- | nothing but `#[test] fn`s inside its generated `mod __rdx_tests`, so a
-- | shared helper sits at module scope and the attribute keeps it out of the
-- | library build — plan 73 §2.9.
#[cfg(test)]
decode : &str -> Result<char, String>
decode after_backslash =
  mut cur = Cursor::new after_backslash
  .map_err (\\e -> e.message) (scan_escape (&mut cur))

test "decodes the simple escapes"
  assert_eq! \\-> (decode "n") \\-> (Ok '\\n')
  assert_eq! \\-> (decode "t") \\-> (Ok '\\t')
  assert_eq! \\-> (decode "0") \\-> (Ok '\\0')
  assert_eq! \\-> (decode "\\\\") \\-> (Ok '\\\\')
  assert_eq! \\-> (decode "\\"") \\-> (Ok '"')
  -- `\\#` exists because `#{` is interpolation inside an ordinary string.
  assert_eq! \\-> (decode "#") \\-> (Ok '#')

test "decodes byte and unicode escapes"
  assert_eq! \\-> (decode "x41") \\-> (Ok 'A')
  assert_eq! \\-> (decode "u{1F600}") \\-> (Ok '\\u{1F600}')

test "rejects a malformed escape"
  assert! \\-> (.is_err (decode "q")) \\-> "unknown escape should fail"
  assert! \\-> (.is_err (decode "x4")) \\-> "short byte escape should fail"
  assert! \\-> (.is_err (decode "u41")) \\-> "missing brace should fail"
  assert! \\-> (.is_err (decode "u{D800}")) \\-> "a surrogate is not a scalar value"
''')

print('batch 8: scan_escape')
