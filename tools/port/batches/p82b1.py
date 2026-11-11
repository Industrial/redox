# -*- coding: utf-8 -*-
"""Plan 82 batch 1: twenty of `redox-parser`'s 112 owed bodies — the small ones.

`unary_op`'s `Amp` arm returns from inside a `case`, which carries over; the
`matches!` predicates all become a `case` with a `_ -> false`, which is plan
64 §6's gap met for the twelfth and thirteenth time.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('parse/precedence/prefix_power', '''
export(crate) const PREFIX_POWER : u8 = 25
''')

write('parse/parser/above_and_and', '''
export(crate) const ABOVE_AND_AND : u8 = 10
''')

write('parse/max_nesting', '''
export const MAX_NESTING : usize = 16
''')

write('parse/parser/is_word', '''
export(crate) is_word : char -> bool
is_word ch = .is_alphanumeric ch || ch == '_'
''')

write('parse/source', '''
export parse_source : &str -> Result<Program, ParseFailure>
parse_source src =
  toks = (.map_err ParseFailure::Lex (redox_lexer::lex src))?
  .map_err ParseFailure::Parse (parse (&toks) (.len src))
''')

write('parse/parser/walk_member_for_tails', '''
import redox_ast::ClassMember
import redox_ast::Item

export(crate) walk_member_for_tails : &ClassMember -> &mut Vec<ParseError> -> ()
-- `problems`, where the Rust named the parameter `errors`: `errors` is doc 03
-- §14's declaration keyword. It is the name this crate's Rust reaches for
-- most often, so every such parameter is `problems` and every such local is
-- `found`.
--
-- **The types are imported, where the Rust qualified them.** A pattern path's
-- first segment has to be **uppercase** — doc 18's case convention is what
-- tells a constructor path from a binder — so `redox_ast::ClassMember::Method`
-- is not a pattern Redox can write, and `ClassMember::Method` is.
walk_member_for_tails member problems =
  if member is ClassMember::Method decl
    walk_item_for_tails (&Item::Fn (.clone decl)) problems
  ()
''')

write('parse/parser/rust_generic', '''
export(crate) rust_generic : &str -> String
rust_generic name =
  if .starts_with '\\'' name
    return .to_string name
  .to_uppercase name
''')

write('parse/parser/flush_word', '''
export(crate) flush_word : &mut String -> &mut String -> &str -> &str -> ()
flush_word out word var subject =
  if .is_empty word
    return ()
  if word == var
    .push_str subject out
  else
    .push_str word out
  .clear word
  ()
''')

write('parse/parser/binary_op', '''
export(crate) binary_op : &mut Parser -> Option<BinaryOp>
binary_op p =
  -- `found.0` is the operator, where the Rust destructured and discarded the
  -- two powers.
  found = (binary_power (&(.peek p).kind))?
  .advance p
  Some found.0
''')

write('parse/parser/byte_literal', '''
export(crate) byte_literal : u8 -> String
byte_literal value = case value of
  -- The three a byte-literal body cannot hold, plus the printable range.
  b'\\\\' -> .to_string "b'\\\\\\\\'"
  b'\\'' -> .to_string "b'\\\\''"
  0x20 ..= 0x7E -> format! \\-> "b'#{char::from value}'"
  _ -> format! \\-> "b'\\\\x#{value:02X}'"
''')

write('parse/parser/starts_a_field_type', '''
export(crate) starts_a_field_type : &TokenKind -> bool
starts_a_field_type kind = case kind of
  TokenKind::Ident _ or TokenKind::Amp or TokenKind::Star or TokenKind::LParen or TokenKind::LBracket -> true
  TokenKind::Kw (Keyword::SelfType or Keyword::Dyn or Keyword::Impl) -> true
  _ -> false
''')

write('parse/precedence/above_bitor', '''
export(crate) above_bitor : u8
above_bitor =
  -- `found.1` is the left power, where the Rust destructured and discarded
  -- the other two.
  found = .expect "`|` is in the precedence table" (binary_power (&TokenKind::Or))
  found.1 + 1
''')

write('parse/parser/opens_a_section', '''
export(crate) opens_a_section : &TokenKind -> bool
opens_a_section kind = case kind of
  TokenKind::Minus or TokenKind::Amp or TokenKind::Star or TokenKind::Bang -> false
  _ -> true
''')

write('parse/parser/starts_a_pattern_argument', '''
export(crate) starts_a_pattern_argument : &Parser -> bool
starts_a_pattern_argument p = case &(.peek p).kind of
  TokenKind::Ident _ or TokenKind::Int _ or TokenKind::Float _ or TokenKind::Char _ or TokenKind::Byte _ or TokenKind::Str { .. } or TokenKind::LParen -> true
  TokenKind::Kw (Keyword::True) or TokenKind::Kw (Keyword::False) or TokenKind::Kw (Keyword::SelfValue) -> true
  _ -> false
''')

write('parse/parser/check_callback_tails', '''
export(crate) check_callback_tails : &Program -> Vec<ParseError>
check_callback_tails program =
  mut found = Vec::new ()
  for item in &program.items
    walk_item_for_tails item (&mut found)
  found
''')

write('parse/parser/unary_op', '''
export(crate) unary_op : &mut Parser -> Option<UnaryOp>
unary_op p =
  op = case (.peek p).kind of
    TokenKind::Minus -> UnaryOp::Neg
    TokenKind::Bang -> UnaryOp::Not
    TokenKind::Star -> UnaryOp::Deref
    TokenKind::Amp ->
      .advance p
      if .eat_keyword Keyword::Mut p
        return Some UnaryOp::RefMut
      return Some UnaryOp::Ref
    _ ->
      return None
  .advance p
  Some op
''')

write('parse/nesting_depth', '''
export(crate) nesting_depth : &[Token] -> usize
nesting_depth toks =
  mut depth : usize = 0
  mut deepest : usize = 0
  for token in toks
    case token.kind of
      TokenKind::LParen or TokenKind::LBracket or TokenKind::LBrace ->
        depth += 1
        deepest := .max depth deepest
      TokenKind::RParen or TokenKind::RBracket or TokenKind::RBrace ->
        depth := .saturating_sub 1 depth
      _ -> ()
  deepest
''')

write('parse/parser/atom', '''
export(crate) atom : &mut Parser -> Result<Expr, ParseError>
atom p =
  mut expr = (prefix p)?
  loop
    case (postfix p expr)? of
      Ok grown -> expr := grown
      Err unchanged ->
        return Ok unchanged
''')

write('parse/parser/can_end_a_type', '''
export(crate) can_end_a_type : &TokenKind -> bool
can_end_a_type kind = case kind of
  -- A triple closer ends a type exactly as a double does — plan 66. It is
  -- here rather than left out because `>>>` is a token this row *adds*:
  -- a new closer absent from the set that answers "can a type end here"
  -- is the shape of the `Shl` gap plan 66 §6 declines, and the cheap
  -- moment to not have it is now.
  TokenKind::Ident _ or TokenKind::Gt or TokenKind::Shr or TokenKind::Compose or TokenKind::RParen or TokenKind::RBracket -> true
  TokenKind::Kw (Keyword::SelfType) -> true
  _ -> false
''')

write('parse/parser/patterns', '''
export(crate) patterns : &mut Parser -> Result<Vec<Pattern>, ParseError>
patterns p =
  mut found = Vec::new ()
  while !.at_eof p && !.check (&TokenKind::Eq) p && !.check (&TokenKind::Newline) p
    .push (pattern p)? found
  Ok found
''')

print('plan 82 batch 1: twenty files')
