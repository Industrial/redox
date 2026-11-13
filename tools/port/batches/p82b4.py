# -*- coding: utf-8 -*-
"""Plan 82 batch 4: twelve more of `voltaic-parser`'s bodies.

`rust_string_body` holds this crate's `as` cast — `c as u32` — which is
`UNREACHABLE` in `docs/GRAMMAR.md`. `u32::from c` is exact for a `char` and
not merely a fallible substitute, which is a better position than
`voltaic-driver`'s `u64::try_from` was in.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/modified_block', '''
export(crate) modified_block : &mut Parser -> Keyword -> usize -> Result<Expr, ParseError>
modified_block p kw start =
  .advance p
  word = .text kw
  body = (.statement_block (&format! \\-> "expected an indented `#{word}` body") p)?
  span = (start, (.prev_span p).1)
  case kw of
    Keyword::Async -> Ok (Expr::AsyncBlock { body, span })
    Keyword::Unsafe -> Ok (Expr::UnsafeBlock { body, span })
    Keyword::Try -> Ok (Expr::TryBlock { body, span })
    _ ->
      unreachable! \\-> "only `try`, `async` and `unsafe` modify a block"
''')

write('parse/parser/substitute_var', '''
export(crate) substitute_var : &mut Signature -> &str -> &str -> ()
substitute_var sig var subject =
  -- The receiver has already been rewritten and carries no type, so this
  -- walks what is left.
  for param in &mut sig.params
    if &mut param.ty is Some ty
      ty.text := rewritten (&ty.text) var subject
  if &mut sig.ret is Some ret
    ret.text := rewritten (&ret.text) var subject
  ()
''')

write('parse/parser/import_decl', '''
export(crate) import_decl : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<ImportDecl, ParseError>
import_decl p leading start visibility =
  (.expect (&TokenKind::Kw Keyword::Import) "expected `import`" p)?
  path = (import_path p)?
  alias = if .check (&TokenKind::Kw Keyword::As) p
    .advance p
    Some (.ident "expected a name after `as`" p)?
  else
    None
  Ok (ImportDecl { path, alias, visibility, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/test_decl', '''
export(crate) test_decl : &mut Parser -> Vec<Trivia> -> usize -> Result<TestDecl, ParseError>
test_decl p leading start =
  (.expect (&TokenKind::Kw Keyword::Test) "expected `test`" p)?
  description = case &(.peek p).kind of
    TokenKind::Str { raw, .. } -> .clone raw
    _ ->
      return Err (ParseError::expected "expected the test's description, as a string literal" (.peek_span p))
  .advance p
  (.newline_with_trailing "expected a line break after the description" p)?
  body = (.indented_block Code::EmptyBody "expected an indented test body" p)?
  Ok (TestDecl { description, body, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/optional_chaining', '''
export(crate) optional_chaining : Span -> ParseError
optional_chaining span =
  ParseError::with_help Code::OptionalChaining "`?.` is retired" "write the `?` run its own lowering always was — `try` with `Some (&u.as_ref ()? .profile.as_ref ()? .name)` — and keep the `&`, because a field selector moves (plan 57 §11.4). `x? .name`, with a space, is `?` and then a field" span
''')

write('parse/parser/rust_string_body', '''
export(crate) rust_string_body : &str -> String
rust_string_body text =
  mut out = String::with_capacity (.len text)
  for ch in .chars text
    case ch of
      '\\\\' -> .push_str "\\\\\\\\" out
      '"' -> .push_str "\\\\\\"" out
      '\\n' -> .push_str "\\\\n" out
      '\\r' -> .push_str "\\\\r" out
      '\\t' -> .push_str "\\\\t" out
      '\\0' -> .push_str "\\\\0" out
      -- Every other ASCII control character, `0x7F` included. Found by plan
      -- 23's example: `b"\\\\x7fELF"` decoded to a literal `DEL` byte and was
      -- re-emitted as one.
      --
      -- `u32::from`, where the Rust wrote `c as u32`: an `as` cast is
      -- `UNREACHABLE` in `docs/GRAMMAR.md`, and `From<char> for u32` is
      -- exact rather than a fallible substitute.
      c | .is_ascii_control c || c == '\\u{7f}' ->
        point = u32::from c
        .push_str (&format! \\-> "\\\\x#{point:02X}") out
      _ -> .push ch out
  out
''')

write('parse/parser/signature_only', '''
export(crate) signature_only : &[TypeExpr] -> Span -> Signature
signature_only spine span =
  -- `split.0` is the return type and `split.1` the parameters, where the Rust
  -- destructured the pair.
  split = .expect "parse_spine returns at least one type" (.split_last spine)
  params : Vec<Param> = split.1
    |> .iter
    |> .enumerate
    |> .map (\\pair ->
    i = pair.0
    ty = pair.1
    Param { name = Ident { text = format! \\-> "__rdx_a#{i}", span = ty.span }, ty = Some (.clone ty), is_mut = false, span = ty.span })
    |> .collect
  -- A unit return type is recorded here and omitted at emission, for the
  -- reason `signature_from` gives.
  Signature { params, ret = Some (.clone split.0), generics = Vec::new (), is_const = false, is_async = false, is_unsafe = false, bounds = Vec::new (), span }
''')

write('parse/parser/starts_argument', '''
export(crate) starts_argument : &TokenKind -> bool
starts_argument kind = case kind of
  TokenKind::Int _ or TokenKind::Float _ or TokenKind::Str { .. } or TokenKind::RawStr { .. } or TokenKind::ByteStr _ or TokenKind::CStr _ or TokenKind::Char _ or TokenKind::Byte _ or TokenKind::Ident _ or TokenKind::LParen or TokenKind::At or TokenKind::AtAt -> true
  TokenKind::Kw (Keyword::True or Keyword::False or Keyword::SelfValue or Keyword::SelfType) -> true
  _ -> false
''')

write('parse/parser/constructor_fields', '''
export(crate) constructor_fields : &mut Parser -> Result<ConstructorFields, ParseError>
constructor_fields p =
  if .check (&TokenKind::LBrace) p
    .advance p
    mut fields = Vec::new ()
    loop
      .skip_layout p
      if .check (&TokenKind::RBrace) p
        break
      .push (field_decl p)? fields
      .skip_layout p
      if !.check (&TokenKind::Comma) p
        break
      .advance p
    .skip_layout p
    (.expect (&TokenKind::RBrace) "expected `}` to close the record" p)?
    return Ok (ConstructorFields::Record fields)
  mut types = Vec::new ()
  while starts_a_field_type (&(.peek p).kind)
    .push (capture_type p "expected a field type" TypeStop::Atom)? types
  if .is_empty types
    return Ok ConstructorFields::Unit
  Ok (ConstructorFields::Tuple types)
''')

write('parse/parser/record_literal', '''
export(crate) record_literal : &mut Parser -> Expr -> Result<Expr, ParseError>
record_literal p head =
  start = (.span head).0
  (.expect (&TokenKind::LBrace) "expected `{` to open a record" p)?
  mut fields = Vec::new ()
  loop
    .skip_layout p
    if .check (&TokenKind::RBrace) p
      break
    .push (field_init p)? fields
    .skip_layout p
    if !.check (&TokenKind::Comma) p
      break
    .advance p
  .skip_layout p
  (.expect (&TokenKind::RBrace) "expected `}` to close the record" p)?
  Ok (Expr::Record { head = Box::new head, fields, span = (start, (.prev_span p).1) })
''')

write('parse', '''
export parse : &[Token] -> usize -> Result<Program, Vec<ParseError>>
parse toks src_len =
  deepest = nesting_depth toks
  if deepest > MAX_NESTING
    -- Reported at the start of the file rather than at the offending bracket.
    -- Finding *which* bracket crossed the line would mean another scan for a
    -- span nobody can act on differently — the answer is always "this file
    -- nests too deeply", and it is almost always generated input rather than
    -- something a person wrote.
    message = format! \\-> "this input nests brackets #{deepest} deep, and the limit is #{MAX_NESTING}: deeper than that overflows the parser's stack"
    return Err (vec! \\-> (ParseError { code = Code::NestingTooDeep, message, span = (0, 0), help = None }))
  .run src_len (Parser::new toks)
''')

write('parse/parser/qualifiers', '''
export(crate) qualifiers : &mut Parser -> Result<Vec<Qualifier>, ParseError>
qualifiers p =
  mut found = Vec::new ()
  loop
    -- Doc 30 §3.2: the generator takes `in`, not `<-`. `<-` means `await` and
    -- nothing else, so the token doc 18 §3.2 borrowed from Haskell's
    -- comprehension is given up — §4.2 records that as the real loss, taken to
    -- free the token for the position where it is more faithful.
    named = case (.peek p).kind of
      TokenKind::Ident _ -> true
      _ -> false
    is_draw = named && .is_some_and (\\t -> t.kind == TokenKind::Kw Keyword::In) (.peek_at 1 p)
    if is_draw
      name = (.ident "expected a name before `in`" p)?
      .advance p
      source = (parse_expr p 0)?
      .push (Qualifier::Draw { name, source }) found
    else
      .push (Qualifier::Guard (parse_expr p 0)?) found
    .skip_layout p
    if !.check (&TokenKind::Comma) p
      break
    .advance p
    .skip_layout p
  Ok found
''')

print('plan 82 batch 4: twelve files')
