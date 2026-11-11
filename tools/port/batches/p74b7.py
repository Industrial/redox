# -*- coding: utf-8 -*-
"""Batch 7: `Cursor`, which is where the slice-index cost concentrates.

Nine of its twelve methods slice — `&self.src[self.pos..]`, `&self.src[span.0..span.1]`
— and a range index has no Redox spelling, so each becomes `.get` with a `""`
fallback. Every one of those fallbacks is unreachable: `pos` only ever advances
by `char::len_utf8`, and a `Span` is only ever produced by `eat_while` from
positions this type handed out, so both are char boundaries by construction.
That is the argument the `""` is standing in for, and it is worth writing down
once rather than nine times.

**`eat_while` takes `impl Fn`, where the Rust took `mut pred: impl FnMut`.**
Not a workaround: every one of its nine callers passes a closure that captures
nothing or captures immutably, so `FnMut` plus the `mut` binding was never
needed. The Redox signature parenthesises the bound — `(impl Fn(char) -> bool)
-> &mut self -> Span` — because a type is kept as source text and the arrow
inside `Fn(char) -> bool` would otherwise split the signature.

**The instance carries the lifetime**: `instance (Cursor 'a)` for Rust's
`impl<'a> Cursor<'a>`, which is the first generic instance in any port.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lexer/cursor', '''
export(crate) data Cursor = Cursor {
  -- | The whole source text the offsets index into.
  src : &'a str,
  -- | Byte offset of the next unconsumed character.
  pos : usize,
}

instance (Cursor 'a)
  -- | A cursor at the start of a source.
  export(crate) new : &'a str -> Self
  new src = Cursor { src, pos = 0 }

  -- | The byte offset of the next unconsumed character.
  export(crate) pos : &self -> usize
  pos self = self.pos

  -- | Whether the whole source has been consumed.
  export(crate) is_eof : &self -> bool
  is_eof self = self.pos >= .len self.src

  -- | The next character without consuming it.
  -- |
  -- | `.get` and a `""` fallback where the Rust sliced: a range index is
  -- | `Index`, which doc 03 grades unreachable. **Every such fallback in this
  -- | file is unreachable** — `pos` only advances by `char::len_utf8` and a
  -- | `Span` only ever comes from `eat_while`, so both are char boundaries by
  -- | construction.
  export(crate) peek : &self -> Option<char>
  peek self = .next (.chars (.unwrap_or "" (.get (self.pos..(.len self.src)) self.src)))

  -- | The character `n` positions ahead without consuming anything.
  export(crate) peek_at : usize -> &self -> Option<char>
  peek_at n self = .nth n (.chars (.unwrap_or "" (.get (self.pos..(.len self.src)) self.src)))

  -- | Whether the text at the cursor begins with `s`.
  export(crate) starts_with : &str -> &self -> bool
  starts_with s self = .starts_with s (.unwrap_or "" (.get (self.pos..(.len self.src)) self.src))

  -- | The unconsumed text, for a scanner that matches on more than one
  -- | character at a time.
  -- |
  -- | Doc 18 §5.1's comment rule needs it: whether `--` opens a comment
  -- | depends on the character *after* the dashes, so the decision cannot be
  -- | made with `starts_with` alone without enumerating every terminator.
  export(crate) rest : &self -> &'a str
  rest self = .unwrap_or "" (.get (self.pos..(.len self.src)) self.src)

  -- | Consume and return the next character.
  export(crate) bump : &mut self -> Option<char>
  bump self =
    ch = (.peek self)?
    self.pos += .len_utf8 ch
    Some ch

  -- | Consume `s` if it is at the cursor, reporting whether it was.
  export(crate) eat : &str -> &mut self -> bool
  eat s self =
    if !(.starts_with s self)
      return false
    for _step in (0..(.count (.chars s)))
      .bump self
    true

  -- | Consume characters while `pred` holds, returning the consumed span.
  -- |
  -- | `impl Fn`, where the Rust took `mut pred: impl FnMut`: every one of the
  -- | nine callers passes a closure that captures nothing or captures
  -- | immutably, so the stronger bound was never needed. The bound is
  -- | parenthesised because a type is source text here, and the arrow inside
  -- | `Fn(char) -> bool` would otherwise split the signature.
  export(crate) eat_while : (impl Fn(char) -> bool) -> &mut self -> Span
  eat_while pred self =
    start = self.pos
    while .is_some_and (&pred) (.peek self)
      .bump self
    (start, self.pos)

  -- | Consume spaces, reporting how many. Does not cross a line boundary.
  export(crate) eat_spaces : &mut self -> usize
  eat_spaces self =
    mut n = 0
    while .peek self == Some ' '
      .bump self
      n += 1
    n

  -- | The source text a span covers.
  export(crate) slice : Span -> &self -> &'a str
  slice span self = .unwrap_or "" (.get (span.0..span.1) self.src)

  -- | Rewind to a previously recorded position.
  -- |
  -- | Used only for bounded lookahead — deciding whether `'` opens a character
  -- | literal or a label — where scanning forward and backing out is clearer
  -- | than a hand-rolled multi-character peek.
  export(crate) reset_to : usize -> &mut self -> ()
  reset_to at self =
    self.pos := at
    ()

  -- | The state `Cursor::reset_to` consumes.
  export(crate) mark : &self -> usize
  mark self = self.pos

test "eat matches only an exact prefix"
  mut c = Cursor::new "...x"
  assert! \\-> !(.eat "..=" c)
  assert! \\-> (.eat "..." c)
  assert_eq! \\-> (.peek c) \\-> (Some 'x')

test "eat while returns the consumed span"
  mut c = Cursor::new "abc123"
  span = .eat_while (\\ch -> .is_ascii_alphabetic ch) c
  assert_eq! \\-> span \\-> (0, 3)
  assert_eq! \\-> (.slice span c) \\-> "abc"

test "mark and reset restore the position"
  mut c = Cursor::new "a\\nb"
  mark = .mark c
  c |> .bump
  c |> .bump
  assert_eq! \\-> (.pos c) \\-> 2
  c |> .reset_to mark
  assert_eq! \\-> (.pos c) \\-> 0

-- | A multi-byte character advances the offset by its encoded width, which is
-- | what keeps a span a valid slice range.
test "bump advances by the encoded width"
  mut c = Cursor::new "é x"
  assert_eq! \\-> (.bump c) \\-> (Some 'é')
  assert_eq! \\-> (.pos c) \\-> 2
''')

print('batch 7: cursor')
