# -*- coding: utf-8 -*-
"""Batch 1: the nine smallest files, and the end of `voltaic-lexer`'s `as` casts.

**All five casts in this crate are avoidable, and removing them is better
Rust.** Four are `is_ident_start(c as u8)` / `is_ident_continue(c as u8)` where
`c` is already a `char`, so the two predicates take a `char` — `char` has
`is_ascii_alphabetic` and `is_ascii_alphanumeric` of its own, and the cast was
*truncating*: `'é' as u8` is `0xE9`, which `is_ascii_alphabetic` then rejects
for the right answer by luck rather than by construction. The fifth is
`value as char` in `scan_escape`, which is `char::from value` — total, where
the cast is only total because the value is a `u8`.

So `voltaic-lexer` reaches zero blockers, and `docs/CONFORMANCE.md`'s count
falls by five rather than by being worked around.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lex', '''
export lex : &str -> Result<Vec<Token>, LexError>
lex src = .run (Lexer::new src)
''')

write('lexer/is_ident_start', '''
-- | **Takes a `char`, where the Rust took a `u8`.** Every one of the four call
-- | sites had a `char` and wrote `is_ident_start(c as u8)`, and an `as` cast is
-- | graded `UNREACHABLE` in `docs/GRAMMAR.md` — but the cast was also
-- | *truncating*, so `'é' as u8` is `0xE9` and the right answer came out by
-- | luck. `char` has the predicate directly.
export(crate) is_ident_start : char -> bool
is_ident_start c = .is_ascii_alphabetic c || c == '_'
''')

write('lexer/is_ident_continue', '''
-- | Takes a `char`, for the reason `is_ident_start` records.
export(crate) is_ident_continue : char -> bool
is_ident_continue c = .is_ascii_alphanumeric c || c == '_'
''')

write('verbatim_triple', '''
export verbatim_triple : &str -> Option<String>
verbatim_triple raw =
  -- Two bindings rather than a chain: `?` binds **inside** a parenthesised
  -- argument, which plan 73 §2.11 paid for, so `.strip_suffix d (.strip_prefix
  -- d raw)?` puts the `?` on the inner call's argument instead of its result.
  opened = (.strip_prefix "\\"\\"\\"" raw)?
  body = (.strip_suffix "\\"\\"\\"" opened)?
  Some (dedent_triple_body body)
''')

write('verbatim_triple/dedent_triple_body', '''
export(crate) dedent_triple_body : &str -> String
dedent_triple_body body =
  inner = .unwrap_or body (.strip_prefix '\\n' body)
  trimmed = .trim_end_matches ' ' inner
  dedent_block (.unwrap_or trimmed (.strip_suffix '\\n' trimmed))
''')

write('lexer/scan_raw_ident', '''
export(crate) scan_raw_ident : &mut Cursor -> Result<TokenKind, LexError>
scan_raw_ident cur =
  start = .pos cur
  -- the `r`
  .bump cur
  -- the `#`
  .bump cur
  -- `is_ident_continue` by name, not wrapped in a lambda: it takes a `char`
  -- now, which is exactly what `eat_while` hands its predicate.
  .eat_while is_ident_continue cur
  Ok (TokenKind::Ident (.to_string (.slice (start, .pos cur) cur)))
''')

write('trivia/kind', '''
instance TriviaKind
  -- | Whether this trivium carries documentation, which the emitter renders as
  -- | `///` or `//!` rather than `//`.
  export is_doc : &self -> bool
  is_doc self = case self of
    DocComment or InnerDocComment -> true
    _ -> false
''')

write('lexer/lex_fragment', '''
export(crate) lex_fragment : &str -> usize -> Result<Vec<Token>, LexError>
lex_fragment src base =
  mut toks = (.map_err (\\e -> .offset_by base e) (.run (Lexer::new src)))?
  .retain (\\t -> !(.is_structural t.kind) && t.kind != TokenKind::Eof) toks
  for t in .iter_mut toks
    t.span := (t.span.0 + base, t.span.1 + base)
    for trivium in .iter_mut t.leading
      trivium.span := (trivium.span.0 + base, trivium.span.1 + base)
  Ok toks
''')

write('lex/error', '''
instance LexError
  -- | Shift this error's span onto the original file.
  -- |
  -- | A scanner run over a sub-slice — a string's body, an interpolation —
  -- | reports offsets into that slice. The caller knows where the slice began
  -- | and is the only one who can correct them, which is the same rebasing
  -- | `crate::lexer::lex_fragment::lex_fragment` does for tokens.
  -- |
  -- | `mut self` is **Rust's fourth receiver form**, and plan 67's list of
  -- | three did not have it: `receiver_text` stripped `&mut ` and `&` and then
  -- | required `self`, so `mut self` stayed an ordinary parameter and emitted
  -- | `fn offset_by(base: usize, self: mut self)`. Plan 74 found it here.
  export(crate) offset_by : usize -> mut self -> Self
  offset_by base self =
    self.span := (self.span.0 + base, self.span.1 + base)
    self
''')

print('batch 1: nine files')
