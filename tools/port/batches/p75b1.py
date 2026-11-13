# -*- coding: utf-8 -*-
"""Plan 75 batch 1: `voltaic-fuzz`'s four targets.

The smallest crate in the workspace, and the first whose whole point is
**discarding** a result: three of the four targets assert totality, so the
assertion is that the call returns rather than what it returns.

The parameter is `bytes` and not the Rust's `data`, because **`data` is a
keyword** — it opens a declaration — so it cannot name a parameter. `r#data`
would preserve the spelling and reach the generated Rust as `r#data`; a
private parameter is not worth that, and `bytes` says more. The Rust writes
`let _ = …`, and `Result` is `#[must_use]`, so the discard has to survive the
port — a bare call statement would be `unused_must_use`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('target/lex_source', '''
export lex_source : &[u8] -> ()
lex_source bytes =
  text = String::from_utf8_lossy bytes
  -- The result is deliberately dropped: the assertion is that this returns
  -- rather than panicking. The sweep in `tests/` is what inspects outcomes.
  --
  -- Bound to `_dropped` rather than left as a bare call: `Result` is
  -- `#[must_use]`, so a statement that discards one is `unused_must_use`, and
  -- the underscore prefix is what keeps the binding itself from warning.
  _dropped = voltaic_lexer::lex (&text)
  ()
''')

write('target/parse_program', '''
export parse_program : &[u8] -> ()
parse_program bytes =
  text = String::from_utf8_lossy bytes
  _dropped = voltaic_parser::parse_source (&text)
  ()
''')

write('target/emit_program', '''
export emit_program : &[u8] -> ()
emit_program bytes =
  text = String::from_utf8_lossy bytes
  -- A `case` where the Rust had a let-else: Voltaic has no `let … else`, and an
  -- arm that returns says the same thing.
  program = case voltaic_parser::parse_source (&text) of
    Ok p -> p
    Err _ ->
      return ()
  _dropped = voltaic_emit::emit (&program)
  ()
''')

write('target/roundtrip', '''
export roundtrip : &[u8] -> Option<String>
roundtrip bytes =
  text = String::from_utf8_lossy bytes
  program = (.ok (voltaic_parser::parse_source (&text)))?
  Some (voltaic_emit::emit (&program))
''')

print('plan 75 batch 1: four targets')
