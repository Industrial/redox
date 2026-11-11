# -*- coding: utf-8 -*-
"""Batch 5: `skip_interpolation` and `split_format_spec`.

Both are byte-level loops, and both pay the same two costs batch 3 and batch 4
already named — an annotated helper closure becomes repetition, and an index
becomes `.get` with a fallback. Two things are new.

**A let-else becomes a `case` with a `return` arm.** `let Some(ch) = cur.bump()
else { return Err(…) }` is Rust's shape for *bind or leave*; Redox's `guard … is
… else` is the analogue, but the arm here also needs the error value, so a
`case` whose `None` arm returns says it in one construct.

**Indexing a byte slice is `.get` plus `.copied` plus a fallback**, where the
Rust writes `bytes[i]`. The loop's own `i < bytes.len()` makes the fallback
unreachable; `0` is the value a byte that cannot be read would have, and it
matches none of the arms.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lexer/skip_interpolation', '''
export(crate) skip_interpolation : &mut Cursor -> usize -> Result<(usize, usize), LexError>
skip_interpolation cur base =
  start = .pos cur
  .eat "\\#{" cur
  body_start = .pos cur

  -- The Rust's `unterminated` closure is written out at each of its three
  -- sites: a Redox lambda parameter is a bare name, so `|cur: &Cursor|` has no
  -- spelling, and the parameter would shadow the enclosing `cur` besides.
  mut depth : i32 = 1
  while depth > 0
    ch = case .bump cur of
      Some c -> c
      None ->
        return Err (LexError {
          code = Code::UnterminatedInterpolation,
          help = None,
          message = .to_string "unterminated interpolation; expected `}`",
          span = (base + start, base + .pos cur),
        })
    case ch of
      '{' -> depth += 1
      '}' -> depth -= 1
      -- A string literal's contents are text, braces included. Escapes are
      -- honoured so that `"\\""` does not end the string early.
      '"' ->
        loop
          case .bump cur of
            None ->
              return Err (LexError {
                code = Code::UnterminatedInterpolation,
                help = None,
                message = .to_string "unterminated interpolation; expected `}`",
                span = (base + start, base + .pos cur),
              })
            Some '\\\\' ->
              .bump cur
              ()
            -- `break` on its own line: it is not accepted as an **inline**
            -- arm body — `Some '"' -> break` is `RDX0102: expected an
            -- expression` — so it is written as the arm's block.
            Some '"' ->
              break
            Some _ -> ()
      _ -> ()
  -- Less the closing `}`, which is not part of the body.
  Ok ((body_start, .pos cur - 1))
''')

write('lexer/split_format_spec', '''
export(crate) split_format_spec : &str -> (&str, Option<&str>)
split_format_spec body =
  bytes = .as_bytes body
  mut depth : i32 = 0
  mut i : usize = 0

  while i < .len bytes
    -- `.get` and `.copied` where the Rust wrote `bytes[i]`: indexing is
    -- `Index`, which doc 03 grades unreachable. The loop's own bound makes the
    -- fallback unreachable, and `0` matches none of the arms below.
    byte = .unwrap_or 0 (.copied (.get i bytes))
    case byte of
      b'(' or b'[' or b'{' -> depth += 1
      b')' or b']' or b'}' -> depth -= 1
      b':' | depth == 0 ->
        -- Skip `::` entirely — both colons, so the second is not re-examined.
        if .get (i + 1) bytes == Some (&b':')
          i += 2
          continue
        -- A `:` cannot be a spec separator if it closes a `::` we are mid-way
        -- through, which `i += 2` above already prevents.
        -- **Closed ranges, because a parenthesised half-open range is an
        -- operator section.** `(..i)` emits `|__rdx_1| __rdx_1 .. i`, which
        -- parses and then fails in rustc; `0..i` and `(i + 1)..len` are the
        -- same two slices with both sides supplied.
        len = .len body
        head = .unwrap_or "" (.get (0..i) body)
        tail = .unwrap_or "" (.get ((i + 1)..len) body)
        return (head, Some tail)
      _ -> ()
    i += 1
  (body, None)

test "finds a plain spec"
  assert_eq! \\-> (split_format_spec "amount:.2") \\-> ("amount", Some ".2")
  assert_eq! \\-> (split_format_spec "name:>8") \\-> ("name", Some ">8")
  assert_eq! \\-> (split_format_spec "cfg:?") \\-> ("cfg", Some "?")

test "returns no spec when there is none"
  assert_eq! \\-> (split_format_spec "name") \\-> ("name", None)
  assert_eq! \\-> (split_format_spec "a + b") \\-> ("a + b", None)

-- | The `::` guard.
test "keeps a path intact"
  assert_eq! \\-> (split_format_spec "HashMap::new()") \\-> ("HashMap::new()", None)
  assert_eq! \\-> (split_format_spec "String::from(x):>4") \\-> ("String::from(x)", Some ">4")

-- | The depth guard.
test "keeps a nested colon intact"
  assert_eq! \\-> (split_format_spec "Point{x: 1}") \\-> ("Point{x: 1}", None)
  assert_eq! \\-> (split_format_spec "map[k]") \\-> ("map[k]", None)
  assert_eq! \\-> (split_format_spec "Point{x: 1}:?") \\-> ("Point{x: 1}", Some "?")

-- | The first depth-zero colon wins, not the last.
test "takes the first eligible colon"
  assert_eq! \\-> (split_format_spec "a:>8:x") \\-> ("a", Some ">8:x")
''')

print('batch 5: two files')
