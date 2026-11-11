# -*- coding: utf-8 -*-
"""Batch 6: `OPERATOR_TABLE` and `TokenKind`'s two predicates.

**The first `const` slice in any port.** The ported corpus has exactly one
`const` so far — `PIPELINE_WIDTH : usize = 60` — so a `&[(&str, TokenKind)]`
table is new ground, and `&[…]` after `=` is the spelling that matches the
Rust. Plan 73's finding about a list literal needing parentheses is about
*argument* position, where `[` also opens an index; after `=` there is no
second reading.

**Two more `matches!` methods**, so this crate needs the crate-root
`match_like_matches_macro` allow that `redox-ast`, `redox-fmt` and `redox-diag`
all carry: `matches!`'s second argument is a pattern, and Redox cannot pass one.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lexer/operator_table', '''
export(crate) const OPERATOR_TABLE : &[(&str, TokenKind)] = &[
  -- Three characters.
  ("...", TokenKind::DotDotDot),
  ("..=", TokenKind::DotDotEq),
  ("??=", TokenKind::CoalesceEq),
  (">>>", TokenKind::Compose),
  ("<<<", TokenKind::ComposeBack),
  -- Two characters.
  ("..", TokenKind::DotDot),
  (".(", TokenKind::DotParen),
  (".+", TokenKind::DotPlus),
  (".-", TokenKind::DotMinus),
  (".*", TokenKind::DotStar),
  ("./", TokenKind::DotSlash),
  (".%", TokenKind::DotPercent),
  ("??", TokenKind::Coalesce),
  ("|>", TokenKind::Pipe),
  ("&:", TokenKind::AmpColon),
  ("@@", TokenKind::AtAt),
  ("::", TokenKind::ColonColon),
  (":=", TokenKind::ColonEq),
  ("->", TokenKind::Arrow),
  ("<-", TokenKind::DrawArrow),
  ("=>", TokenKind::FatArrow),
  ("==", TokenKind::EqEq),
  ("!=", TokenKind::Ne),
  ("<=", TokenKind::Le),
  (">=", TokenKind::Ge),
  ("&&", TokenKind::AndAnd),
  ("||", TokenKind::OrOr),
  ("+=", TokenKind::PlusEq),
  ("-=", TokenKind::MinusEq),
  ("*=", TokenKind::StarEq),
  ("/=", TokenKind::SlashEq),
  ("%=", TokenKind::PercentEq),
  ("<<", TokenKind::Shl),
  (">>", TokenKind::Shr),
  -- One character.
  ("(", TokenKind::LParen),
  (")", TokenKind::RParen),
  ("[", TokenKind::LBracket),
  ("]", TokenKind::RBracket),
  ("{", TokenKind::LBrace),
  ("}", TokenKind::RBrace),
  ("@", TokenKind::At),
  ("?", TokenKind::Question),
  (":", TokenKind::Colon),
  ("\\\\", TokenKind::Backslash),
  ("$", TokenKind::Dollar),
  (",", TokenKind::Comma),
  (".", TokenKind::Dot),
  (";", TokenKind::Semi),
  ("=", TokenKind::Eq),
  ("+", TokenKind::Plus),
  ("-", TokenKind::Minus),
  ("*", TokenKind::Star),
  ("/", TokenKind::Slash),
  ("%", TokenKind::Percent),
  ("<", TokenKind::Lt),
  (">", TokenKind::Gt),
  ("!", TokenKind::Bang),
  ("&", TokenKind::Amp),
  ("|", TokenKind::Or),
  ("^", TokenKind::Caret),
  ("~", TokenKind::Tilde),
]
''')

write('token/kind', '''
instance TokenKind
  -- | Whether this token is structural — emitted at a position the parser does
  -- | not read trivia from.
  -- |
  -- | Structural tokens never claim trivia. Letting a `Dedent` absorb a pending
  -- | comment loses it silently, which is how a comment between two top-level
  -- | items, or at end of file, used to disappear. `Eof` counts as
  -- | non-structural so a trailing comment has somewhere to land.
  export(crate) is_structural : &self -> bool
  is_structural self = case self of
    Indent or Dedent or Newline -> true
    _ -> false

  -- | Whether a line **ending** in this token continues onto the next.
  -- |
  -- | Plan 18 removed `Attr`. An attribute is **trivia**, not a token: it
  -- | belongs to the declaration below it, which is what `Trivia` already means,
  -- | and a token nothing produced would have been a variant every exhaustive
  -- | match had to carry for nothing.
  -- |
  -- | Plan 02 leaf 2: doc 01 §4.2 says "newlines terminate statements unless an
  -- | expression is continued" without saying when. A line ending in a binary
  -- | operator or a comma is continued.
  -- |
  -- | `=`, `->`, `=>` and `:` are deliberately **not** here, and that is not an
  -- | oversight: each of them ends a header whose body follows on an *indented*
  -- | line, so suppressing the newline would swallow the `Indent` the parser
  -- | needs. `main : ()` / `main =` is the case that proved it.
  -- |
  -- | `>`, `>>` and `>>>` are not here either, for a different reason: they
  -- | close a generic argument list far more often than they dangle off a
  -- | comparison — or, for the third, off a composition.
  -- | `wide = Some 1 : Option<u8>` swallowed the whole next line into the
  -- | type's verbatim text, because `capture_type` ends a type at a newline and
  -- | there was no newline to find. A comparison genuinely split after `>` is
  -- | writable with parentheses; a type ending in `>` is not avoidable at all,
  -- | and doc 18 §5.6 deleted the chained comparison that made the dangling
  -- | form attractive in the first place.
  -- |
  -- | `*` left the set for the same reason, one plan later:
  -- | `import crate::models::*` is doc 03 §13.1's glob, a `*` is the last
  -- | token of every one of them, and the import path swallowed the following
  -- | declaration whole. A multiplication split after `*` is writable with
  -- | parentheses; a glob import is not avoidable at all. Twice is a pattern
  -- | worth naming — **a token that ends a construct should not continue a
  -- | line**, however operator-like it looks.
  -- |
  -- | That rule is what splits plan 66's pair. `<<<` ends no construct, so it
  -- | continues; `>>>` closes `Vec<Vec<Vec<i32>>>`, so it does not, and a
  -- | forward chain split across lines leads its continuation line instead —
  -- | which is what `Lexer::next_line_continues` admits and is the house
  -- | style for `|>` already.
  export(crate) continues_line : &self -> bool
  continues_line self = case self of
    Pipe or ComposeBack or Comma or Plus or Minus or Slash or Percent or EqEq or Ne or Lt or Le or Ge or AndAnd or OrOr or Amp or Or or Caret or Shl or Coalesce or DotDot or DotDotEq -> true
    _ -> false
''')

print('batch 6: two files')
