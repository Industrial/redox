# -*- coding: utf-8 -*-
"""Batch 9: `scan_number` and `scan_operator`.

Two shapes new to this port, both consequences of the bare-binder rule rather
than new restrictions.

**A tuple binding becomes a named pair**, for the third time: `let (name,
valid): (&str, fn(char) -> bool) = match radix { … }`. The ascription is
load-bearing and survives — a Redox binding takes one — so only the
destructuring moves.

**`for (text, kind) in OPERATOR_TABLE` becomes `for entry in …`** with
`entry.0` and `entry.1`, which is the `for`-binder rule batch 3 recorded.
`scan_operator`'s ordering test nests two of them and reads `outer.1.0`, a
field of a field; that is the least pleasant line in the port so far and it is
still only a notation.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lexer/scan_number', '''
export(crate) scan_number : &mut Cursor -> Result<TokenKind, LexError>
scan_number cur =
  start = .pos cur

  -- A radix prefix forbids a fractional part, so handle it first and return.
  if .peek cur == Some '0'
    radix = .unwrap_or '\\0' (.peek_at 1 cur)
    -- The ascription is the Rust's and is load-bearing: the arms are three
    -- different closures, and `fn(char) -> bool` is what unifies them. Only
    -- the tuple *destructuring* had to go — a Redox binding takes one name.
    pair : (&str, fn(char) -> bool) = case radix of
      'x' or 'X' -> ("hexadecimal", \\c -> .is_ascii_hexdigit c)
      -- `.contains` rather than two comparisons: clippy asks for the range to
      -- say it, and it reads better.
      'o' or 'O' -> ("octal", \\c -> .contains (&c) ('0'..='7'))
      'b' or 'B' -> ("binary", \\c -> c == '0' || c == '1')
      _ -> ("", \\_c -> false)
    name = pair.0
    valid = pair.1
    if !(.is_empty name)
      .bump cur
      .bump cur
      digits = .eat_while (\\c -> valid c || c == '_') cur
      if .all (\\c -> c == '_') (.chars (.slice digits cur))
        return Err (LexError {
          code = Code::MalformedNumber,
          help = None,
          message = format! \\-> "#{name} literal has no digits",
          span = (start, .pos cur),
        })
      -- A suffix such as `u8` follows the digits directly.
      .eat_while (\\c -> .is_ascii_alphanumeric c || c == '_') cur
      return Ok (TokenKind::Int (.to_string (.slice (start, .pos cur) cur)))

  .eat_while (\\c -> .is_ascii_digit c || c == '_') cur

  mut is_float = false

  -- A `.` is a fractional point only when a digit follows it; `1..10` is a
  -- range and `1.method()` is a method call, and both must stay intact.
  if .peek cur == Some '.' && .is_some_and (\\c -> .is_ascii_digit c) (.peek_at 1 cur)
    is_float := true
    .bump cur
    .eat_while (\\c -> .is_ascii_digit c || c == '_') cur

  exponent = case .peek cur of
    Some 'e' or Some 'E' -> true
    _ -> false
  if exponent && .is_some_and (\\c -> .is_ascii_digit c || c == '+' || c == '-') (.peek_at 1 cur)
    is_float := true
    .bump cur
    signed = case .peek cur of
      Some '+' or Some '-' -> true
      _ -> false
    if signed
      .bump cur
    digits = .eat_while (\\c -> .is_ascii_digit c || c == '_') cur
    if .is_empty (.slice digits cur)
      return Err (LexError {
        code = Code::MalformedNumber,
        help = None,
        message = .to_string "exponent has no digits",
        span = (start, .pos cur),
      })

  .eat_while (\\c -> .is_ascii_alphanumeric c || c == '_') cur

  text = .to_string (.slice (start, .pos cur) cur)
  if is_float
    return Ok (TokenKind::Float text)
  Ok (TokenKind::Int text)

-- | `#[cfg(test)]`: the helper every test below shares. Redox can put nothing
-- | but `#[test] fn`s inside its generated `mod __rdx_tests` — plan 73 §2.9.
#[cfg(test)]
scan : &str -> (TokenKind, usize)
scan src =
  mut cur = Cursor::new src
  kind = .expect "should scan" (scan_number (&mut cur))
  (kind, .pos cur)

test "scans decimal integers with separators"
  one = scan "1_000_000"
  assert_eq! \\-> one.0 \\-> (TokenKind::Int (.into "1_000_000"))
  two = scan "42"
  assert_eq! \\-> two.0 \\-> (TokenKind::Int (.into "42"))

test "scans radix prefixes"
  hex = scan "0xDEAD_beef"
  assert_eq! \\-> hex.0 \\-> (TokenKind::Int (.into "0xDEAD_beef"))
  oct = scan "0o755"
  assert_eq! \\-> oct.0 \\-> (TokenKind::Int (.into "0o755"))
  bin = scan "0b1010"
  assert_eq! \\-> bin.0 \\-> (TokenKind::Int (.into "0b1010"))

test "scans floats with exponents"
  plain = scan "2.5"
  assert_eq! \\-> plain.0 \\-> (TokenKind::Float (.into "2.5"))
  signed = scan "2.5e-3"
  assert_eq! \\-> signed.0 \\-> (TokenKind::Float (.into "2.5e-3"))
  upper = scan "1E10"
  assert_eq! \\-> upper.0 \\-> (TokenKind::Float (.into "1E10"))

test "carries a type suffix verbatim"
  int = scan "42u8"
  assert_eq! \\-> int.0 \\-> (TokenKind::Int (.into "42u8"))
  float = scan "1.5f64"
  assert_eq! \\-> float.0 \\-> (TokenKind::Float (.into "1.5f64"))

-- | `1..10` is a range, not a malformed float: the `.` is only a fractional
-- | point when a digit follows it.
test "leaves a range operator alone"
  ranged = scan "1..10"
  assert_eq! \\-> ranged.0 \\-> (TokenKind::Int (.into "1"))
  assert_eq! \\-> ranged.1 \\-> 1 \\-> "the cursor stops before `..`"

-- | `1.method()` likewise.
test "leaves a method call alone"
  called = scan "1.abs()"
  assert_eq! \\-> called.0 \\-> (TokenKind::Int (.into "1"))
  assert_eq! \\-> called.1 \\-> 1

test "rejects a radix prefix with no digits"
  mut cur = Cursor::new "0x"
  assert! \\-> (.is_err (scan_number (&mut cur)))
''')
write('lexer/scan_operator', '''
export(crate) scan_operator : &mut Cursor -> Option<TokenKind>
scan_operator cur =
  -- `for entry in …` and `entry.0`/`entry.1`, because a `for` binder is a
  -- bare name — the rule a lambda parameter follows too.
  for entry in OPERATOR_TABLE
    if .eat entry.0 cur
      return Some (.clone entry.1)
  None

-- | `#[cfg(test)]`: the helper every test below shares. Redox can put nothing
-- | but `#[test] fn`s inside its generated `mod __rdx_tests` — plan 73 §2.9.
#[cfg(test)]
scan : &str -> (Option<TokenKind>, usize)
scan src =
  mut cur = Cursor::new src
  kind = scan_operator (&mut cur)
  (kind, .pos cur)

-- | The property the table's order exists for. Every prefix pair is checked
-- | mechanically rather than by eye, so adding a token in the wrong place
-- | fails here instead of silently truncating a longer one.
test "longer operators are tried before their prefixes"
  -- `entry = outer.1` rather than `outer.1.0`: a nested tuple index lexes
  -- `1.0` as a **float**, so the chain needs a binding to break it. Rust has
  -- the same wart and spells the way out `(outer.1).0`.
  for outer in .enumerate (.iter OPERATOR_TABLE)
    i = outer.0
    left = outer.1
    text = left.0
    for inner in .enumerate (.iter OPERATOR_TABLE)
      j = inner.0
      right = inner.1
      -- No `*`: the Rust destructured to a `&&str`, and field access on the
      -- bound pair auto-dereferences to `&str` already. Dereferencing again
      -- gives an unsized `str`.
      other = right.0
      if i != j && .starts_with other text && .len text > .len other
        assert! \\-> (i < j) \\-> "`#{text}` must be listed before its prefix `#{other}`"

test "scans the three character operators whole"
  assert_eq! \\-> (scan "...") \\-> ((Some TokenKind::DotDotDot), 3)
  assert_eq! \\-> (scan "..=") \\-> ((Some TokenKind::DotDotEq), 3)
  assert_eq! \\-> (scan "??=") \\-> ((Some TokenKind::CoalesceEq), 3)
  assert_eq! \\-> (scan ">>>") \\-> ((Some TokenKind::Compose), 3)
  assert_eq! \\-> (scan "<<<") \\-> ((Some TokenKind::ComposeBack), 3)

-- | Plan 66 criterion 1, and the half the ordering test cannot state.
-- |
-- | `longer_operators_are_tried_before_their_prefixes` proves the table's
-- | order; this proves what the order *buys*, in the one direction a reader
-- | of plan 58 §1.3 will want checked — the shifts are untouched. A `>>>`
-- | listed after `>>` scans two characters here and silently truncates,
-- | which is the failure both tests exist to rule out.
test "the shifts keep their own spellings"
  assert_eq! \\-> (scan ">> 1") \\-> ((Some TokenKind::Shr), 2)
  assert_eq! \\-> (scan "<< 1") \\-> ((Some TokenKind::Shl), 2)
  assert_eq! \\-> (scan ">") \\-> ((Some TokenKind::Gt), 1)
  assert_eq! \\-> (scan "<") \\-> ((Some TokenKind::Lt), 1)

-- | Doc 03 §4.2 corrects doc 01 §21: `..` is exclusive and `..=` inclusive,
-- | matching Rust. Both must lex as themselves, never as two `Dot`s.
test "scans both range operators"
  assert_eq! \\-> (scan "..") \\-> ((Some TokenKind::DotDot), 2)
  assert_eq! \\-> (scan "..=10") \\-> ((Some TokenKind::DotDotEq), 3)

test "distinguishes the dot family"
  assert_eq! \\-> (scan ".+") \\-> ((Some TokenKind::DotPlus), 2)
  assert_eq! \\-> (scan ".(") \\-> ((Some TokenKind::DotParen), 2)
  assert_eq! \\-> (scan ".x") \\-> ((Some TokenKind::Dot), 1)

test "distinguishes the question family"
  assert_eq! \\-> (scan "??") \\-> ((Some TokenKind::Coalesce), 2)
  -- `?.` is **two** tokens since plan 57 deleted optional chaining.
  assert_eq! \\-> (scan "?.") \\-> ((Some TokenKind::Question), 1)
  assert_eq! \\-> (scan "?") \\-> ((Some TokenKind::Question), 1)

test "distinguishes the at sigils"
  assert_eq! \\-> (scan "@@") \\-> ((Some TokenKind::AtAt), 2)
  assert_eq! \\-> (scan "@x") \\-> ((Some TokenKind::At), 1)

test "returns none for a non operator"
  assert_eq! \\-> (scan "abc") \\-> (None, 0)
''')

print('batch 9: two files')
