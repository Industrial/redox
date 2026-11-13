# -*- coding: utf-8 -*-
r"""Plan 82 batch 7: twelve more of `voltaic-parser`'s bodies.

`method_context` is the crate's second type-with-an-impl after `parse/error`,
so it takes the same shape: the translator wrote the `data` declaration and an
`instance` block supplies the three associated functions.

Raw Python strings throughout, for the reason batch 6's docstring gives.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/method_context', r'''
instance MethodContext
  -- | A `class`'s context: its parameters are the trait's own generic
  -- | parameters, so each takes its uppercase name — `class Convert t` gives
  -- | `trait Convert<T>`.
  -- |
  -- | Plan 67 §1.6 claimed this could only shrink, since the implementing type
  -- | substitutes to itself under `Self`. It lost one entry and gained a
  -- | dimension: the class variable is gone, and a class may now have more
  -- | than one parameter.
  export(crate) for_class : &[Ident] -> Self
  for_class params =
    vars = params
      |> .iter
      |> .map (\param -> (.clone param.text, rust_generic (&param.text)))
      |> .collect
    MethodContext { vars, recording = None, implementing = None }

  -- | An `instance`'s context. `var` is the target's first type variable, if
  -- | it has one, and `subject` is that variable's Rust spelling — the impl's
  -- | generic parameter, as in `impl<T> Describe for Wrapper<T>`.
  export(crate) for_instance : Option<&str> -> &str -> Self
  for_instance var subject =
    -- `found`, where the Rust's closure parameter shadowed `var`: `RDX1003`
    -- reports a shadow, and the two names are the same value either way.
    vars = var
      |> .map (\found -> (.to_string found, .to_string subject))
      |> .into_iter
      |> .collect
    MethodContext { vars, recording = None, implementing = None }

  -- | The names a method must **not** make generic parameters of its own.
  export(crate) reserved : &self -> Vec<String>
  reserved self =
    -- `pair.0` is the source spelling, where the Rust's closure destructured
    -- the pair: a lambda parameter is a bare name.
    self.vars
      |> .iter
      |> .map (\pair -> .clone pair.0)
      |> .collect
''')

write('parse/parser/reserved_word', r'''
export(crate) reserved_word : &mut Parser -> ParseError
reserved_word p =
  debug_assert! \-> .check (&TokenKind::Kw (Keyword::Do)) p \-> "`do` is the only reserved word with no construct"
  keyword = .peek_span p
  .advance p
  labelled = case (.peek p).kind of
    TokenKind::Label _ -> true
    _ -> false
  if labelled
    .advance p
  -- Only when a block actually follows: `do = 2` is the identifier use, and
  -- there is nothing after the word to read.
  if .check (&TokenKind::Newline) p
    -- `_dropped`, where the Rust wrote `let _ = …`: the block is parsed to
    -- move past it and its value is deliberately thrown away.
    _dropped = block_expr p None keyword.0
    ()
  ParseError::with_help Code::ReservedWord "`do` is a reserved word and names no construct" "an indented block is an expression on its own, so drop the `do` and keep the block — `found = 'search` for the labelled form. The word stays reserved for plan 60's do-notation (plan 55 §1.7)" keyword
''')

write('parse/parser/application', r'''
export(crate) application : &mut Parser -> Result<Expr, ParseError>
application p =
  head = (atom p)?
  -- A head that ended with a `Dedent` closed an indented block, so whatever
  -- follows is on a *later line* and cannot be its argument. Doc 18 §2.3's
  -- `case … of` is the first expression with layout of its own, and without
  -- this `label age = case age of …` swallowed the next item's name as an
  -- argument — the application loop reads tokens, and the token after a
  -- dedent looks like any other.
  if .just_dedented p || !starts_argument (&(.peek p).kind)
    return Ok head
  -- The parenless half of plan 57 leaf 3's retirement — `RDX0115`. §11.2:
  -- `x.f a` is an application over an `Expr::Field`, because there is no
  -- paren-call step above, so `postfix`'s refusal of `x.f (a)` would
  -- otherwise be escaped by dropping the parentheses. A `.` is field access
  -- and a field is not applied; the call is `.f a x`.
  if &head is Expr::Field { name, .. }
    return Err (method_call_syntax (&name.text) (.span head))
  mut args = Vec::new ()
  while starts_argument (&(.peek p).kind)
    .push (atom p)? args
  span = ((.span head).0, (.prev_span p).1)
  Ok (Expr::Call { callee = Box::new head, args, is_dollar = false, span })
''')

write('parse/parser/condition', r'''
export(crate) condition : &mut Parser -> Result<Condition, ParseError>
condition p =
  -- Does the **whole** condition hold a pattern test? This scan runs past
  -- `&&`, unlike the per-operand one below: a chain's pattern test may be any
  -- operand, and `if n > 0 && x is Some v` is as legal as `if x is Some v`.
  if !holds_a_pattern_test p false (&TokenKind::Kw (Keyword::Is))
    -- Plan 60 leaf 2. Every program written before this row spells a pattern
    -- test with `=`, so the diagnostic **is** the migration guide — plan 52's
    -- and plan 59's choice, applied a third time.
    --
    -- Detected with the same scanner the `is` form uses, given the other
    -- marker, so the message cannot describe a shape the detection does not
    -- find. A record literal's `=` sits inside its braces and the depth rule
    -- is what keeps `if p { x = 1 }.valid` an ordinary condition.
    if holds_a_pattern_test p false (&TokenKind::Eq)
      return Err (ParseError::r#with Code::ExpectedToken (.to_string "a pattern test is written `scrutinee is pattern` — plan 60 moved it off `=`, which now means definition and nothing else") (.peek_span p))
    return Ok (Condition::plain (parse_expr p 0)?)
  mut operands = vec! \-> (condition_operand p)?
  while .check (&TokenKind::AndAnd) p
    .advance p
    .push (condition_operand p)? operands
  -- A let-chain joins with `&&` only — **Rust's** restriction, not Voltaic's —
  -- so say that rather than leaving the generic "expected a line break" the
  -- block parser would give.
  if .check (&TokenKind::OrOr) p
    return Err (ParseError::r#with Code::ExpectedToken "a condition holding a pattern test joins with `&&` only: Rust's let-chains do not admit `||`. Split the condition, or use a `case`" (.peek_span p))
  Ok (Condition { operands })
''')

write('parse/parser/error_variant', r'''
export(crate) error_variant : &mut Parser -> Result<ErrorVariant, ParseError>
error_variant p =
  start = (.peek_span p).0
  -- `from` is not a reserved word — doc 18 §7's table does not add one — so
  -- it is matched as the identifier it is. A variant genuinely named `from`
  -- would be lowercase, which doc 18 §5.3 reserves for values, so the
  -- collision is already ruled out by the capitalisation rule.
  is_from = case &(.peek p).kind of
    TokenKind::Ident text | text == "from" -> true
    _ -> false
  -- `read.0` is the name, `read.1` the payload types and `read.2` the `from`
  -- flag, where the Rust destructured the triple.
  read = if is_from
    .advance p
    ty = (capture_type p "expected a type after `from`" TypeStop::Greedy)?
    -- `as` is a reserved word, so it is matched as the keyword it is.
    (.expect (&TokenKind::Kw (Keyword::As)) "expected `as` between the type and the variant's name" p)?
    named = (.ident "expected the variant's name after `as`" p)?
    (named, vec! \-> ty, true)
  else
    named = (.ident "expected a variant name" p)?
    mut payload = Vec::new ()
    while starts_a_field_type (&(.peek p).kind)
      .push (capture_type p "expected a payload type" TypeStop::Atom)? payload
    (named, payload, false)
  message = case &(.peek p).kind of
    TokenKind::Str { raw, .. } -> .clone raw
    _ ->
      return Err (ParseError::expected "expected the variant's `Display` message, as a string literal" (.peek_span p))
  .advance p
  (.newline_with_trailing "expected a line break after the variant" p)?
  Ok (ErrorVariant { name = read.0, fields = read.1, from = read.2, message, span = (start, (.prev_span p).1) })
''')

write('parse/parser/generalise', r'''
export(crate) generalise : &mut Signature -> &[Constraint] -> &[String] -> ()
generalise sig constraints reserved =
  mut texts : Vec<&str> = sig.params
    |> .iter
    |> .filter_map (\param -> .map (\t -> .as_str t.text) (.as_ref param.ty))
    |> .collect
  if &sig.ret is Some ret
    .push (.as_str ret.text) texts
  found : Vec<String> = type_vars (&texts)
    |> .into_iter
    |> .filter (\v -> !.contains v reserved)
    |> .collect
  if .is_empty found
    return ()
  sig.generics := .collect (.map (\v -> rust_generic v) (.iter found))
  sig.bounds := constraints
    |> .iter
    |> .filter (\c -> .contains (&c.var.text) found)
    |> .map (\c -> (rust_generic (&c.var.text), .clone c.trait_name.text))
    |> .collect
  for var in &found
    if .starts_with '\'' var
      continue
    subject = rust_generic var
    for param in &mut sig.params
      if &mut param.ty is Some ty
        ty.text := rewritten (&ty.text) var (&subject)
    if &mut sig.ret is Some ret
      ret.text := rewritten (&ret.text) var (&subject)
  ()
''')

write('parse/parser/macro_call', r'''
export(crate) macro_call : &mut Parser -> Result<Expr, ParseError>
macro_call p =
  head = (path_or_ident p)?
  if !.check (&TokenKind::Bang) p || (.peek p).ws_before
    -- Plan 64 leaf 5. A *bare* name is where all six retired spellings
    -- stood, so this is the one position that has to say so — `println!`
    -- has already been taken by the branch below.
    if &head is Expr::Ident name && retired_spelling name is Some retired
      return Err retired
    return Ok head
  .advance p
  name = case &head of
    Expr::Ident i -> .clone i.text
    Expr::Path { text, .. } -> .clone text
    -- `path_or_ident` returns one of those two and nothing else.
    other ->
      unreachable! \-> "a macro head is a name or a path, not #{other:?}"
  start = (.span head).0
  mut args = Vec::new ()
  while .check (&TokenKind::Backslash) p && .is_some_and (\t -> t.kind == TokenKind::Arrow) (.peek_at 1 p)
    .advance p
    .advance p
    .push (parse_expr p 0)? args
  Ok (Expr::MacroCall { name, args, span = (start, (.prev_span p).1) })
''')

write('parse/parser/unit_tail', r'''
export(crate) unit_tail : &Ident -> &Block -> Result<(), ParseError>
unit_tail name block =
  -- An empty block is `RDX0105`'s, raised by `statements` itself. Nothing to
  -- say about the tail of a body that has none.
  guard .last block.stmts is Some last else
    return Ok ()
  -- `case … of` with a `_ -> false`, where the Rust wrote `matches!`: a
  -- macro's argument is an expression (plan 64 §6) and these are patterns.
  spells_unit = case last.expr of
    Expr::Unit { .. } or Expr::Return { .. } or Expr::Break { .. } or Expr::Continue { .. } -> true
    _ -> false
  if spells_unit
    return Ok ()
  message = format! \-> "`#{name.text}` returns `()`, so its body must end with `()`"
  Err (ParseError::with_help Code::UnitTail message "write `()` as the body's last line — plan 55 §1.6 spells a unit value rather than implying it, so a body ending in a loop, a binding or an assignment cannot become `()` by arriving last" last.span)
''')

write('parse/parser/pattern_app', r'''
import voltaic_ast::Ident

export(crate) pattern_app : &mut Parser -> Result<Pattern, ParseError>
pattern_app p =
  start = (.peek_span p).0
  -- Only an uppercase identifier can head an application. Anything else is a
  -- single pattern, read and returned unchanged — which is the path every
  -- pattern took before this plan.
  guard constructor_path p is Some path else
    return pattern p
  -- Doc 03 §6.3's record pattern — plan 33. A `{` after the path is the
  -- record form, and it is handled here rather than in `pattern` because
  -- `constructor_path` has already consumed an uppercase head by the time
  -- `pattern` would see one. `{` is deliberately not in
  -- `starts_a_pattern_argument`: it opens a field list, not an argument.
  if .check (&TokenKind::LBrace) p
    return record_pattern p path start
  mut args = Vec::new ()
  while starts_a_pattern_argument p
    .push (pattern p)? args
  -- A bare constructor stays a `Binding`, as it was before this plan: the
  -- emitter writes the bare path, and doc 18 §2.4 emits `use T::*;` beside the
  -- enum so the path resolves. Emitting `Close()` for `Close` would be wrong
  -- for a unit variant, so the two cases must stay distinct.
  if .is_empty args
    return Ok (Pattern::Binding (Ident { text = path, span = (start, (.prev_span p).1) }))
  Ok (Pattern::Apply { path, args, span = (start, (.prev_span p).1) })
''')

write('parse/parser/extern_block', r'''
export(crate) extern_block : &mut Parser -> Vec<Trivia> -> usize -> Result<ExternBlock, ParseError>
extern_block p leading start =
  (.expect (&TokenKind::Kw (Keyword::Extern)) "expected `extern`" p)?
  abi = case &(.peek p).kind of
    TokenKind::Str { raw, .. } -> .clone raw
    _ ->
      return Err (ParseError::expected "expected an ABI string after `extern`, as in `extern \"C\"`" (.peek_span p))
  .advance p
  (.newline_with_trailing "expected a line break after the ABI" p)?
  mut decls = Vec::new ()
  (.expect (&TokenKind::Indent) "expected indented `extern` declarations" p)?
  loop
    .skip_separators p
    if .check (&TokenKind::Dedent) p || .at_eof p
      break
    decl_leading = .peek_leading p
    decl_start = (.peek_span p).0
    name = (.ident "expected a foreign function's name" p)?
    (.expect (&TokenKind::Colon) "expected `:` after the foreign function's name" p)?
    spine = (parse_spine p)?
    span = (decl_start, (.prev_span p).1)
    .push (ForeignFn { name, sig = signature_only (&spine) span, span, leading = decl_leading }) decls
    (.newline_with_trailing "expected a line break after the declaration" p)?
  (.expect (&TokenKind::Dedent) "expected the `extern` block to end" p)?
  Ok (ExternBlock { abi, decls, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/parse_expr', r'''
export(crate) parse_expr : &mut Parser -> u8 -> Result<Expr, ParseError>
parse_expr p min =
  -- Doc 18 §2.1: application binds tighter than every operator, so it is the
  -- primary rather than a rung on the precedence ladder. `application` also
  -- absorbs the postfix loop that used to run here, because an argument is a
  -- postfixed primary and the head is one too.
  mut lhs = (application p)?
  loop
    -- Doc 18 §6's ascription, at the loosest binding power there is: it
    -- applies to the whole expression, so it is tried only at the top level
    -- (`min == 0`) and only after the binary loop has taken everything else.
    -- `1 + 2 : u64` therefore ascribes the sum, not the `2`.
    if min == 0 && .check (&TokenKind::Colon) p
      .advance p
      ty = (capture_type p "expected a type after `:`" TypeStop::Greedy)?
      at = ((.span lhs).0, ty.span.1)
      lhs := Expr::Ascribe { inner = Box::new lhs, ty, span = at }
      continue
    -- Doc 18 §2.1's `$`: grouping that runs to the end of the expression, and
    -- the reason a nested application rarely needs brackets. It **lowers to
    -- nothing at all** — `f $ g x` is the call `f(g(x))` — so the call is
    -- built here and no node of its own is carried.
    --
    -- `is_dollar` records the spelling and nothing else. The lowering ignores
    -- it, because `$` has no lowering; the formatter reads it, because a
    -- formatter that reprinted `f $ x` as `f (x)` would rewrite away the one
    -- construct whose purpose is to avoid that parenthesis.
    --
    -- Binding power 1: looser than every operator, tighter than ascription,
    -- which matches Haskell's `infixr 0` for `$` against an even looser `:`.
    if min <= 1 && .check (&TokenKind::Dollar) p
      .advance p
      arg = (parse_expr p 1)?
      at = ((.span lhs).0, (.span arg).1)
      lhs := Expr::Call { callee = Box::new lhs, args = vec! \-> arg, is_dollar = true, span = at }
      continue
    -- `rung.0` is the operator, `rung.1` its left power and `rung.2` its
    -- right, where the Rust destructured the triple in the `let … else`.
    guard binary_power (&(.peek p).kind) is Some rung else
      return Ok lhs
    if rung.1 < min
      return Ok lhs
    .advance p
    rhs = (parse_expr p rung.2)?
    span = ((.span lhs).0, (.span rhs).1)
    lhs := Expr::Binary { op = rung.0, lhs = Box::new lhs, rhs = Box::new rhs, span }
''')

write('parse/parser/newtype_shape', r'''
export(crate) newtype_shape : &DataDecl -> Result<(), ParseError>
newtype_shape decl =
  if !decl.is_newtype
    return Ok ()
  -- The constructor count comes first so `newtype Multi = A | B` is reported
  -- as the enum it would have become, rather than as `A` carrying no field.
  -- The blamed span is the **second** constructor: the first is the one a
  -- `newtype` is allowed, so underlining the lot would accuse it too.
  count = .len decl.constructors
  if count > 1
    message = format! \-> "a `newtype` has exactly one constructor, and this declaration has #{count} — write `data` for a choice between constructors"
    return Err (ParseError::r#with Code::NewtypeShape message decl.constructors[1].span)
  -- `newtype Lone`, with no `=` row at all, parses and emitted `struct Lone;`
  -- — the same wrong shape as a constructor carrying no field, so the two
  -- share a message and differ only in what the caret can point at.
  --
  -- `shape.0` is the blamed span and `shape.1` the field spans, where the
  -- Rust destructured the pair.
  -- The ascription is the Rust's, and load-bearing: `Span` is imported by
  -- this file for it and nothing else, so dropping it made the import unused.
  shape : (Span, Vec<Span>) = case .first decl.constructors of
    None -> (decl.name.span, Vec::new ())
    Some only ->
      spans = case &only.fields of
        ConstructorFields::Unit -> Vec::new ()
        ConstructorFields::Tuple types -> .collect (.map (\t -> t.span) (.iter types))
        -- A one-field record `newtype Tag = Tag { raw : String }` is legal
        -- Haskell and a legal `repr(transparent)` struct, so records are
        -- counted rather than refused.
        ConstructorFields::Record field_decls -> .collect (.map (\f -> f.span) (.iter field_decls))
      (only.span, spans)
  blame = shape.0
  fields = shape.1
  n = .len fields
  case n of
    1 -> Ok ()
    -- rustc is no help here: `#[repr(transparent)] pub struct Unit;` compiles
    -- clean (plan 53 §11.3), so this branch is the only thing that catches a
    -- payload-free `newtype`.
    0 -> Err (ParseError::r#with Code::NewtypeShape "a `newtype` wraps exactly one field, and this declaration wraps none — write `data` for a type with no payload" blame)
    _ ->
      message = format! \-> "a `newtype` wraps exactly one field, and this declaration wraps #{n} — write `data` for a constructor with more than one field"
      Err (ParseError::r#with Code::NewtypeShape message fields[1])
''')

print('plan 82 batch 7: twelve files')
