# -*- coding: utf-8 -*-
"""Batch 3: `dedent_block` and `scan_byte`.

Two costs worth naming, both consequences of surface facts already recorded
rather than new defects.

**A slice index becomes `.get` plus `.unwrap_or`.** `&line[n..]` has no Voltaic
spelling — indexing with a range is `Index`, and doc 03 grades it unreachable —
so it is `.get (n..) line` with a fallback. The fallback is unreachable by
construction, because `n` is `common.min(line.len())` and `common` counts
leading *spaces*, which are always char boundaries; `""` is the honest value
for a slice that cannot be taken.

**A helper closure that needed a type annotation becomes five copies of its
body.** `scan_byte`'s Rust opens with

```rust
let err = |message: String, cur: &Cursor| LexError { … };
```

and a Voltaic lambda parameter is a bare name — neither a pattern nor an
annotation, which plan 73 §2.11 recorded. Here the annotation is load-bearing
twice over: without it `cur`'s type cannot be inferred from `.pos cur` alone,
and the parameter would also *shadow* the enclosing `cur`, which `RDX1003`
reports. So the literal is written at each of its five sites, which is what the
other three scanners in this crate already do.

The alternative — a nullary `\\->` closure capturing `cur` — is declined for a
different reason: `voltaic fmt` diverges on a nullary lambda in **argument**
position, adding one parenthesis per pass without bound, and the gate runs
`:sdx-format-check`. That defect is owed to a successor row; writing around it
costs nothing here.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lexer/dedent_block', '''
export(crate) dedent_block : &str -> String
dedent_block body =
  lines : Vec<&str> = body |> .lines |> .collect

  common = lines
    |> .iter
    |> .filter (\\l -> !(.is_empty (.trim l)))
    |> .map (\\l -> .len l - .len (.trim_start_matches ' ' l))
    |> .min
    |> .unwrap_or 0

  mut out = String::new ()
  -- **A `for` binder is a bare name**, like a lambda parameter: `for (i, line)
  -- in …` earns `RDX0103: expected a name after "for"`. So the pair is bound and
  -- its fields are read, which is consistent with the lambda rule rather than
  -- a separate restriction to remember.
  for pair in .enumerate (.iter lines)
    i = pair.0
    line = pair.1
    if i > 0
      .push '\\n' out
    if .is_empty (.trim line)
      -- A blank line stays blank rather than keeping stray spaces, which
      -- would otherwise show up as trailing whitespace in generated Rust.
      continue
    -- `.get` and a fallback where the Rust sliced: `&line[n..]` is `Index`
    -- with a range, which doc 03 grades unreachable. The fallback cannot
    -- fire — `n` counts leading spaces, so it is always a char boundary and
    -- never past the end — and `""` is the honest value for a slice that
    -- cannot be taken.
    --
    -- **A closed range, because `(cut..)` would be an operator section.**
    -- Doc 18 §4 writes a section as a parenthesised operator with one side
    -- supplied, so `(cut..)` is `|x| cut .. x` — a closure, which parses and
    -- then meets rustc as a type error. `cut..end` has both sides and is the
    -- same slice.
    cut = .min (.len line) common
    end = .len line
    .push_str (.unwrap_or "" (.get (cut..end) line)) out
  out

test "strips the common prefix and keeps relative indentation"
  body = "    indented\\n      relative\\n    back"
  assert_eq! \\-> (dedent_block body) \\-> "indented\\n  relative\\nback"

test "a blank line does not force the prefix to zero"
  body = "    one\\n\\n    two"
  assert_eq! \\-> (dedent_block body) \\-> "one\\n\\ntwo"

test "leaves an unindented body alone"
  assert_eq! \\-> (dedent_block "a\\nb") \\-> "a\\nb"

test "handles an empty body"
  assert_eq! \\-> (dedent_block "") \\-> ""

test "strips trailing spaces from a whitespace only line"
  assert_eq! \\-> (dedent_block "  a\\n    \\n  b") \\-> "a\\n\\nb"
''')

write('lexer/scan_byte', '''
export(crate) scan_byte : &mut Cursor -> Result<TokenKind, LexError>
scan_byte cur =
  start = .pos cur

  -- the `b`
  .bump cur

  if .eat "'" cur
    ch = if .peek cur == Some '\\\\'
      .bump cur
      (scan_escape cur)?
    else
      case .bump cur of
        Some c -> c
        None ->
          return Err (LexError {
            code = Code::UnterminatedByteLiteral,
            help = None,
            message = .to_string "unterminated byte literal",
            span = (start, .pos cur),
          })
    if !(.eat "'" cur)
      return Err (LexError {
        code = Code::UnterminatedByteLiteral,
        help = None,
        message = .to_string "unterminated byte literal",
        span = (start, .pos cur),
      })
    value = case u8::try_from (u32::from ch) of
      Ok v -> v
      Err _ ->
        return Err (LexError {
          code = Code::UnterminatedByteLiteral,
          help = None,
          message = format! \\-> "`#{ch}` is not a single byte",
          span = (start, .pos cur),
        })
    return Ok (TokenKind::Byte value)

  if .eat "\\"" cur
    mut text = String::new ()
    while !(.starts_with "\\"" cur)
      if .is_eof cur
        return Err (LexError {
          code = Code::UnterminatedByteLiteral,
          help = None,
          message = .to_string "unterminated byte string",
          span = (start, .pos cur),
        })
      if .peek cur == Some '\\\\'
        .bump cur
        .push (scan_escape cur)? text
      else
        .push (.expect "not at eof" (.bump cur)) text
    .eat "\\"" cur
    return Ok (TokenKind::ByteStr text)

  Err (LexError {
    code = Code::UnterminatedByteLiteral,
    help = None,
    message = .to_string "`b` must be followed by `'` or `\\"`",
    span = (start, .pos cur),
  })
''')

print('batch 3: two files')
