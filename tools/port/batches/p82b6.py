# -*- coding: utf-8 -*-
r"""Plan 82 batch 6: twelve more of `voltaic-parser`'s bodies.

**Every `write()` text here is a raw Python string**, where batches 1–5 used
plain ones. Three of these bodies carry a literal backslash into the `.sdx` —
`retired_spelling`'s help text quotes `\->` as *text*, so the file needs `\\->`
— and a plain Python literal needs four backslashes to put two in the file,
which is how batch 5's `rust_hatch` shipped `\\"` where it meant `\"`. A raw
string is exactly the `.sdx` bytes, so there is one layer of escaping to think
about instead of two.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/error', r'''
instance ParseError
  -- | The common case: a token appeared where the grammar wanted another.
  export(crate) expected : impl Into<String> -> Span -> Self
  expected message span =
    ParseError { code = Code::ExpectedToken, message = .into message, span, help = None }

  -- | An error with a code other than [`Code::ExpectedToken`].
  -- |
  -- | `r#with`, because `with` is doc 03 §7.6's keyword. The emitted Rust is
  -- | `fn r#with`, which is the identifier `with` — so `ParseError::with` is
  -- | still what a Rust reader of the `.xrs` sees named, and every call site
  -- | in this crate spells it `ParseError::r#with` for the same reason.
  export(crate) r#with : Code -> impl Into<String> -> Span -> Self
  r#with code message span =
    ParseError { code, message = .into message, span, help = None }

  -- | An error whose **reason** needs saying as well as its remedy.
  export(crate) with_help : Code -> impl Into<String> -> impl Into<String> -> Span -> Self
  with_help code message help span =
    ParseError { code, message = .into message, span, help = Some (.into help) }
''')

write('parse/parser/errors_decl', r'''
export(crate) errors_decl : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<ErrorsDecl, ParseError>
errors_decl p leading start visibility =
  (.expect (&TokenKind::Kw (Keyword::Errors)) "expected `errors`" p)?
  name = (.ident "expected the error type's name" p)?
  (.newline_with_trailing "expected a line break after the error type's name" p)?
  (.expect_with Code::EmptyBody (&TokenKind::Indent) "expected indented error variants" p)?
  mut variants = Vec::new ()
  loop
    .skip_separators p
    if .check (&TokenKind::Dedent) p || .at_eof p
      break
    .push (error_variant p)? variants
  (.expect (&TokenKind::Dedent) "expected the error variants to end" p)?
  if .is_empty variants
    return Err (ParseError::r#with Code::EmptyBody "an `errors` declaration must have at least one variant" (start, (.prev_span p).1))
  Ok (ErrorsDecl { name, variants, visibility, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/local_test', r'''
export(crate) local_test : &mut Parser -> Result<(), ParseError>
local_test p =
  if !.check (&TokenKind::Kw (Keyword::Test)) p
    return Ok ()
  -- `Vec::new ()` for the leading trivia, because the node is dropped — the
  -- one way this differs from `Parser::item`'s call, and sound because
  -- nothing downstream ever sees it.
  keyword = .peek_span p
  -- `_dropped`, where the Rust wrote `let _ = …`: `_` is not a Voltaic pattern,
  -- and the binding is still what satisfies `Result`'s `#[must_use]`. A
  -- `drop (…)` would do it too and draws `clippy::drop_non_drop`.
  _dropped = test_decl p (Vec::new ()) keyword.0
  Err (ParseError::with_help Code::LocalTest "a `test` declaration belongs at module scope, and this one is inside a body" "move it out to the left margin: rustc does not collect a nested `#[test]` — it warns `cannot test inner items` and the test silently never runs (plan 55 §1.5)" keyword)
''')

write('parse/parser/retired_spelling', r'''
export(crate) retired_spelling : &Ident -> Option<ParseError>
-- Every `\->` in this function's strings is **text**, not a lambda arrow: the
-- help quotes the spelling it is recommending. So each one is written `\\->`,
-- and the `#{total}` inside the last is written `\#{total}` — doc 03 §11.1's
-- escape, which exists for exactly this, a message *about* interpolation.
retired_spelling name =
  replacement = case .as_str name.text of
    "assert" -> "`assert! \\-> cond`, or `assert! \\-> a == b` for a comparison"
    "panic" -> "`panic! \\-> \"message\"`"
    "print" -> "`println! \\-> \"\#{x}\"`, or `eprintln!` to write to stderr"
    "todo" -> "`todo!`"
    "unimplemented" -> "`unimplemented!`"
    "unreachable" -> "`unreachable!`"
    _ ->
      return None
  message = format! \-> "`#{name.text}` is not a construct any more — macros are callable, so write #{replacement}"
  Some (ParseError::with_help Code::RetiredSpelling message "`!` marks a macro's name and `\\->` opens each argument, so no bracket is needed: `println! \\-> \"\#{total} rows\"`" name.span)
''')

write('parse/parser/path_or_ident', r'''
export(crate) path_or_ident : &mut Parser -> Result<Expr, ParseError>
path_or_ident p =
  first = (.ident "expected a name" p)?
  if !.check (&TokenKind::ColonColon) p
    return Ok (Expr::Ident first)
  start = first.span.0
  mut text = first.text
  mut tight_through = first.span.1
  while .check (&TokenKind::ColonColon) p
    separator = .peek_span p
    if separator.0 > tight_through
      return Err (ParseError::r#with Code::SpacedPathSeparator "`::` joins a path, so it takes no space before it — write `:` to annotate a type, or close up the `::` to name a path" separator)
    .advance p
    next = (.ident "expected a name after `::`" p)?
    .push_str "::" text
    .push_str (&next.text) text
    tight_through := next.span.1
  Ok (Expr::Path { text, span = (start, (.prev_span p).1) })
''')

write('parse/parser/try_await', r'''
export(crate) try_await : &mut Parser -> Result<Option<Expr>, ParseError>
try_await p =
  start = (.peek_span p).0
  -- The effect-only form: nothing to bind, so nothing to scan past.
  if .check (&TokenKind::DrawArrow) p
    .advance p
    effect = Box::new (parse_expr p 0)?
    return Ok (Some (Expr::Await { pattern = None, value = effect, span = (start, (.prev_span p).1) }))
  if !reaches_a_draw_arrow p
    return Ok None
  pattern = (pattern_alt p)?
  (.expect (&TokenKind::DrawArrow) "expected `<-`" p)?
  value = Box::new (parse_expr p 0)?
  Ok (Some (Expr::Await { pattern = Some pattern, value, span = (start, (.prev_span p).1) }))
''')

write('parse/parser/case_of', r'''
export(crate) case_of : &mut Parser -> Result<Expr, ParseError>
case_of p =
  start = (.peek_span p).0
  (.expect (&TokenKind::Kw (Keyword::Case)) "expected `case`" p)?
  scrutinee = (parse_expr p 0)?
  (.expect (&TokenKind::Kw (Keyword::Of)) "expected `of` after the scrutinee" p)?
  (.expect (&TokenKind::Newline) "expected a line break after `of`" p)?
  (.expect_with Code::EmptyBody (&TokenKind::Indent) "expected indented `case` arms" p)?
  mut arms = Vec::new ()
  loop
    .skip_separators p
    if .check (&TokenKind::Dedent) p || .at_eof p
      break
    .push (case_arm p)? arms
  (.expect (&TokenKind::Dedent) "expected the `case` arms to end" p)?
  if .is_empty arms
    return Err (ParseError::r#with Code::EmptyBody "a `case` must have at least one arm" (start, (.prev_span p).1))
  Ok (Expr::Case { scrutinee = Box::new scrutinee, arms, span = (start, (.prev_span p).1) })
''')

write('parse/parser/lambda', r'''
export(crate) lambda : &mut Parser -> Result<Expr, ParseError>
lambda p =
  start = (.peek_span p).0
  is_move = .eat_keyword Keyword::Move p
  (.expect (&TokenKind::Backslash) "expected `\\` to open a lambda" p)?
  mut params = Vec::new ()
  while !.check (&TokenKind::Arrow) p && !.at_eof p
    name = (.ident "expected a lambda parameter name" p)?
    ty = if .check (&TokenKind::Colon) p
      .advance p
      Some (capture_type p "expected a type after `:`" TypeStop::Greedy)?
    else
      None
    at = name.span
    .push (Param { name, ty, is_mut = false, span = at }) params
  (.expect (&TokenKind::Arrow) "expected `->` after the lambda's parameters" p)?
  body = (parse_expr p 0)?
  span = (start, (.span body).1)
  Ok (Expr::Lambda { params, body = Box::new body, is_move, span })
''')

write('parse/parser/case_arm', r'''
export(crate) case_arm : &mut Parser -> Result<CaseArm, ParseError>
case_arm p =
  start = (.peek_span p).0
  pattern = (pattern_alt p)?
  -- Plan 62 leaf 3. `Err e : VersionError ->` pins the error type at the arm
  -- that handles it. Read **here** and nowhere else: `pattern` cannot take a
  -- type in general, because a typed binding reads its pattern through the
  -- same path, and an arm is the one position where `->` makes the end of the
  -- type unambiguous.
  ty = if .check (&TokenKind::Colon) p
    .advance p
    Some (capture_type p "expected a type after `:` in a `case` pattern" TypeStop::Greedy)?
  else
    None
  -- An arm guard may be a pattern test as of plan 22 — doc 18 §3's fourth
  -- position, *"in a condition or guard"*. It lowers to `if let` inside the
  -- match guard, which is `if_let_guard` and **stable** as of rustc 1.97.1 —
  -- verified before the plan was written, because a plan that promises an
  -- unstable lowering cannot ship (plan 22 §1.5).
  -- `arm_guard`, where the Rust named the local `guard`: `guard` is doc 03
  -- §7.4's keyword, and `test` — the first substitute tried — is §22's. The
  -- field keeps its own name as `r#guard`, which is how `voltaic-ast` declares
  -- it and how `emit::case` already reads it.
  arm_guard = if .check (&TokenKind::Or) p
    .advance p
    Some (condition p)?
  else
    None
  (.expect (&TokenKind::Arrow) "expected `->` after a `case` pattern" p)?
  -- One expression, or — since plan 55 leaf 3 — an indented block, which is
  -- also one. `arm_body` owns the choice; a block body ends on a `Dedent`,
  -- which `just_dedented` lets stand for the line break below.
  body = (arm_body p)?
  (.newline_with_trailing "expected a line break after a `case` arm" p)?
  Ok (CaseArm { pattern, ty, r#guard = arm_guard, body, span = (start, (.prev_span p).1) })
''')

write('parse/parser/visibility', r'''
export(crate) visibility : &mut Parser -> Result<Visibility, ParseError>
visibility p =
  if !.check (&TokenKind::Kw (Keyword::Export)) p
    return Ok Visibility::Private
  .advance p
  if !.check (&TokenKind::LParen) p
    return Ok Visibility::Public
  .advance p
  scope = if .check (&TokenKind::Kw (Keyword::In)) p
    .advance p
    mut path = String::new ()
    while !.check (&TokenKind::RParen) p && !.at_eof p
      .push_str (&token_text (&(.peek p).kind)) path
      .advance p
    if .is_empty path
      return Err (ParseError::expected "expected a module path after `in`" (.peek_span p))
    Visibility::In path
  else
    word = token_text (&(.peek p).kind)
    .advance p
    case .as_str word of
      "crate" -> Visibility::Crate
      "super" -> Visibility::Super
      _ ->
        return Err (ParseError::expected "expected `crate`, `super` or `in` inside `export(…)`" (.prev_span p))
  (.expect (&TokenKind::RParen) "expected `)` to close `export(…)`" p)?
  Ok scope
''')

write('parse/parser/generalise_data', r'''
export(crate) generalise_data : &mut DataDecl -> ()
generalise_data d =
  mut texts : Vec<String> = Vec::new ()
  for c in &d.constructors
    case &c.fields of
      ConstructorFields::Unit -> ()
      ConstructorFields::Tuple ts -> .extend (.map (\t -> .clone t.text) (.iter ts)) texts
      ConstructorFields::Record fs -> .extend (.map (\f -> .clone f.ty.text) (.iter fs)) texts
  refs : Vec<&str> = .collect (.map String::as_str (.iter texts))
  found = type_vars (&refs)
  if .is_empty found
    return ()
  d.generics := .collect (.map (\v -> rust_generic v) (.iter found))
  for c in &mut d.constructors
    types : Vec<&mut String> = case &mut c.fields of
      ConstructorFields::Unit -> Vec::new ()
      ConstructorFields::Tuple ts -> .collect (.map (\t -> &mut t.text) (.iter_mut ts))
      ConstructorFields::Record fs -> .collect (.map (\f -> &mut f.ty.text) (.iter_mut fs))
    for text in types
      for v in &found
        *text := rewritten text v (&rust_generic v)
  ()
''')

write('parse/parser/class_decl', r'''
export(crate) class_decl : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<ClassDecl, ParseError>
class_decl p leading start visibility =
  (.expect (&TokenKind::Kw (Keyword::Class)) "expected `class`" p)?
  supers = (class_supers p)?
  name = (.ident "expected the class's name" p)?
  -- Greedy, and safe to be: a class head is followed by a line break, so
  -- there is nothing else on the line for a name to belong to.
  mut params = Vec::new ()
  while .try_ident p is Some param
    .push param params
  mut ctx = MethodContext::for_class (&params)
  -- Plan 63 leaf 1: members are recorded under this class's name as they are
  -- read, which is what an `instance` later borrows from.
  ctx.recording := Some (.clone name.text)
  members = (class_body p (&ctx))?
  Ok (ClassDecl { name, params, supers, members, visibility, span = (start, (.prev_span p).1), leading })
''')

print('plan 82 batch 6: twelve files')
