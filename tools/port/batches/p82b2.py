# -*- coding: utf-8 -*-
"""Plan 82 batch 2: twelve more of `voltaic-parser`'s bodies."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/pattern_alt', '''
export(crate) pattern_alt : &mut Parser -> Result<Pattern, ParseError>
pattern_alt p =
  first = (pattern_app p)?
  if !.check (&TokenKind::Kw Keyword::Or) p
    return Ok first
  mut alts = vec! \\-> first
  while .eat_keyword Keyword::Or p
    .push (pattern_app p)? alts
  Ok (Pattern::Or alts)
''')

write('parse/parser/field_init', '''
export(crate) field_init : &mut Parser -> Result<FieldInit, ParseError>
field_init p =
  start = (.peek_span p).0
  name = (.ident "expected a field name" p)?
  value = if .check (&TokenKind::Eq) p
    .advance p
    Some (parse_expr p 0)?
  else
    None
  Ok (FieldInit { name, value, span = (start, (.prev_span p).1) })
''')

write('parse/parser/pipeline_section', '''
export(crate) pipeline_section : &str -> Span -> ParseError
pipeline_section name span =
  help = format! \\-> "write `.#{name}` — a method is a function awaiting its receiver, and it lowers to the closure this spelled. `(.#{name})` still selects a *field* (plan 57 §1.3)"
  ParseError::with_help Code::PipelineSection "the `(|> name)` method reference is retired" help span
''')

write('parse/parser/compound_assign', '''
export(crate) compound_assign : &mut Parser -> Option<BinaryOp>
compound_assign p =
  op = case (.peek p).kind of
    TokenKind::PlusEq -> BinaryOp::Add
    TokenKind::MinusEq -> BinaryOp::Sub
    TokenKind::StarEq -> BinaryOp::Mul
    TokenKind::SlashEq -> BinaryOp::Div
    TokenKind::PercentEq -> BinaryOp::Rem
    _ ->
      return None
  .advance p
  Some op
''')

write('parse/parser/reaches_a_draw_arrow', '''
export(crate) reaches_a_draw_arrow : &Parser -> bool
reaches_a_draw_arrow p =
  mut depth : i32 = 0
  mut i = 0
  while .peek_at i p is Some tok
    case &tok.kind of
      TokenKind::DrawArrow | depth == 0 ->
        return true
      TokenKind::LParen or TokenKind::LBracket or TokenKind::LBrace -> depth += 1
      TokenKind::RParen or TokenKind::RBracket or TokenKind::RBrace ->
        depth -= 1
        if depth < 0
          return false
      -- An `=` at depth zero means this is a binding, not an await-bind, and
      -- the binding parser gets it. Stopping here rather than at the end of
      -- the line is what keeps `x = f y <- z` from being read as either.
      TokenKind::Eq | depth == 0 ->
        return false
      TokenKind::Newline or TokenKind::Indent or TokenKind::Dedent or TokenKind::Eof ->
        return false
      _ -> ()
    i += 1
  false
''')

write('parse/parser/condition_operand', '''
export(crate) condition_operand : &mut Parser -> Result<ConditionOperand, ParseError>
condition_operand p =
  -- This scan stops at `&&`, because the question here is whether *this*
  -- operand is a pattern test and not whether a later one is.
  if !holds_a_pattern_test p true (&TokenKind::Kw Keyword::Is)
    return Ok (ConditionOperand::Expr (parse_expr p ABOVE_AND_AND)?)
  -- Plan 60: **scrutinee first.** `slot is Some n`, which is the order a
  -- reader thinks in and the order Rust, C# and Kotlin all use. Doc 30 §7
  -- decision 3 chose `=` partly to avoid reversing the operands; this row
  -- takes the reversal as the improvement rather than the price.
  --
  -- The node is unchanged, so `emit_condition` and every lowering are
  -- untouched — only the parse order and the token moved.
  source = (parse_expr p ABOVE_AND_AND)?
  (.expect (&TokenKind::Kw Keyword::Is) "expected `is` before the pattern" p)?
  Ok (ConditionOperand::Pattern { pattern = (pattern_alt p)?, source })
''')

write('parse/parser/constructor', '''
export(crate) constructor : &mut Parser -> Vec<voltaic_lexer::Trivia> -> Result<Constructor, ParseError>
constructor p leading =
  start = (.peek_span p).0
  name = (.ident "expected a constructor name" p)?
  fields = (constructor_fields p)?
  Ok (Constructor { name, fields, leading, span = (start, (.prev_span p).1) })
''')

write('parse/parser/inline_rust', '''
export(crate) inline_rust : &mut Parser -> Vec<Trivia> -> usize -> Result<InlineRust, ParseError>
inline_rust p leading start =
  (.expect (&TokenKind::Kw Keyword::Rust) "expected `rust`" p)?
  -- `hatched.0` is the code, where the Rust destructured and discarded the
  -- span.
  hatched = (rust_hatch p)?
  Ok (InlineRust { code = hatched.0, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/type_alias', '''
export(crate) type_alias : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<TypeAlias, ParseError>
type_alias p leading start visibility =
  (.expect (&TokenKind::Kw Keyword::Type) "expected `type`" p)?
  name = (.ident "expected a name after `type`" p)?
  (.expect (&TokenKind::Eq) "expected `=` after the alias's name" p)?
  ty = (capture_type p "expected a type" TypeStop::Greedy)?
  Ok (TypeAlias { name, ty, visibility, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/constructor_path', '''
export(crate) constructor_path : &mut Parser -> Option<String>
constructor_path p =
  guard &(.peek p).kind is TokenKind::Ident first else
    return None
  if !.is_some_and char::is_uppercase (.next (.chars first))
    return None
  mut text = .clone first
  .advance p
  -- `::` is a path separator and nothing else — plan 52 retired doc 18 §5.2's
  -- spaced-versus-tight rule when `:` became the type marker.
  while .check (&TokenKind::ColonColon) p
    .advance p
    -- A malformed path is left for `pattern` to report against the token
    -- that is actually wrong.
    guard &(.peek p).kind is TokenKind::Ident next else
      break
    .push_str "::" text
    .push_str next text
    .advance p
  Some text
''')

write('parse/parser/field_decl', '''
export(crate) field_decl : &mut Parser -> Result<FieldDecl, ParseError>
field_decl p =
  start = (.peek_span p).0
  -- Plan 48 leaf 2. Read before the visibility marker, as an item's trivia
  -- is: a `--- doc` on a field attaches to the first token of the field, and
  -- that is the marker when there is one.
  leading = .peek_leading p
  -- §9.6: "Per-field and per-variant visibility use the same markers inside
  -- a declaration body."
  --
  -- `marker`, not a rebinding of the imported `visibility`: `RDX1003` reports
  -- a shadow.
  marker = (visibility p)?
  name = (.ident "expected a field name" p)?
  (.expect (&TokenKind::Colon) "expected `:` after the field name" p)?
  ty = (capture_type p "expected a field type" TypeStop::Greedy)?
  Ok (FieldDecl { name, ty, visibility = marker, leading, span = (start, (.prev_span p).1) })
''')

write('parse/parser/pattern_series', '''
export(crate) pattern_series : &mut Parser -> &TokenKind -> Result<Vec<Pattern>, ParseError>
pattern_series p close =
  mut items = Vec::new ()
  -- Plan 65 leaf 1: the bracket this sits inside opens a layout context, so
  -- its lines carry layout tokens the series itself has no use for.
  .skip_layout p
  while !.check close p
    .push (pattern_alt p)? items
    .skip_layout p
    if !.check (&TokenKind::Comma) p
      break
    .advance p
    .skip_layout p
  Ok items
''')

print('plan 82 batch 2: twelve files')
