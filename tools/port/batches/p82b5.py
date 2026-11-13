# -*- coding: utf-8 -*-
"""Plan 82 batch 5: twelve more of `voltaic-parser`'s bodies.

`rust_hatch` is the shape where Rust's shadowing is load-bearing and Voltaic's
`RDX1003` forbids it: `let raw = raw.clone()` ends the borrow of `p` by
retiring the name, and a Voltaic port has to end it by *last use* instead — the
clone is the binding's final mention, so `.advance p` is still legal under a
different local name.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/assign_tail', '''
export(crate) assign_tail : &mut Parser -> Expr -> usize -> Result<Expr, ParseError>
assign_tail p target start =
  op = if .check (&TokenKind::ColonEq) p
    .advance p
    None
  else
    -- `chosen`, not a rebinding of `op`: the outer name is what this block
    -- produces, and `RDX1003` reports a shadow.
    guard compound_assign p is Some chosen else
      return Ok target
    Some chosen
  value = Box::new (parse_expr p 0)?
  Ok (Expr::Assign { target = Box::new target, op, value, span = (start, (.prev_span p).1) })
''')

write('parse/parser/const_decl', '''
export(crate) const_decl : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<ConstDecl, ParseError>
const_decl p leading start visibility =
  is_static = .check (&TokenKind::Kw (Keyword::Static)) p
  .advance p
  name = (.ident "expected a name after `const`" p)?
  (.expect (&TokenKind::Colon) "expected `:` and a type — doc 03 §13.4 requires one on a module-scope binding" p)?
  ty = (capture_type p "expected the binding's type" TypeStop::Greedy)?
  (.expect (&TokenKind::Eq) "expected `=` and a value — a module-scope binding must be initialised" p)?
  value = (parse_expr p 0)?
  Ok (ConstDecl { name, ty, value, is_static, visibility, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/rust_hatch', '''
export(crate) rust_hatch : &mut Parser -> Result<(String, Span), ParseError>
rust_hatch p =
  span = .peek_span p
  if &(.peek p).kind is TokenKind::RustBlock code
    -- `block_code`, where the Rust shadowed `code` with its own clone. A
    -- shadow is how the Rust ends the borrow of `p` before `.advance`; here
    -- the clone is simply `code`'s last mention, which ends it the same way.
    block_code = .clone code
    .advance p
    return Ok ((block_code, span))
  guard &(.peek p).kind is TokenKind::Str { raw, .. } else
    return Err (ParseError::expected "expected a `\\"\\"\\"` block of Rust after `rust`, or an indented block on the lines under it" span)
  literal = .clone raw
  .advance p
  -- A single-quoted string is **rejected** rather than accepted as a one-line
  -- hatch: doc 01 §23.1 writes `"""`, a one-line escape hatch is a worse thing
  -- to encourage than a two-line one, and — the deciding reason — a `"…"`
  -- literal has nowhere to put the Rust code's own quotes.
  verbatim_triple (&literal)
    |> .map (\\code -> (code, span))
    |> .ok_or_else (\\-> ParseError::expected "inline Rust needs a `\\"\\"\\"` block or an indented one: a single-quoted string cannot hold Rust's own quotes" span)
''')

write('parse/parser/holds_a_pattern_test', '''
export(crate) holds_a_pattern_test : &Parser -> bool -> &TokenKind -> bool
holds_a_pattern_test p one_operand marker =
  mut depth : i32 = 0
  mut i = 0
  -- A missing token is the end of input, which ends the condition too.
  while .peek_at i p is Some tok
    case &tok.kind of
      kind | depth == 0 && kind == marker ->
        return true
      TokenKind::LParen or TokenKind::LBracket or TokenKind::LBrace -> depth += 1
      TokenKind::RParen or TokenKind::RBracket or TokenKind::RBrace ->
        depth -= 1
        -- A closing delimiter never opened here ends the condition: it
        -- belongs to something enclosing, so there is no marker left to find.
        if depth < 0
          return false
      TokenKind::AndAnd or TokenKind::OrOr | depth == 0 && one_operand ->
        return false
      TokenKind::Kw (Keyword::Else) | depth == 0 ->
        return false
      -- The arm-guard caller's terminator.
      TokenKind::Arrow | depth == 0 ->
        return false
      TokenKind::Newline or TokenKind::Indent or TokenKind::Dedent or TokenKind::Eof ->
        return false
      _ -> ()
    i += 1
  false
''')

write('parse/parser/class_supers', '''
export(crate) class_supers : &mut Parser -> Result<Vec<Ident>, ParseError>
class_supers p =
  mark = .position p
  mut found = Vec::new ()
  -- Haskell parenthesises a list of two or more; one needs no parentheses.
  parenthesised = .check (&TokenKind::LParen) p
  if parenthesised
    .advance p
  while .try_ident p is Some name
    .push name found
    if !.check (&TokenKind::Comma) p
      break
    .advance p
  if parenthesised && .check (&TokenKind::RParen) p
    .advance p
  if .is_empty found || !.check (&TokenKind::FatArrow) p
    .rewind mark p
    return Ok (Vec::new ())
  .advance p
  Ok found
''')

write('parse/parser/type_app', '''
export(crate) type_app : &mut Parser -> Result<TypeApp, ParseError>
type_app p =
  start = (.peek_span p).0
  grouped = .check (&TokenKind::LParen) p
  if grouped
    .advance p
  head = (.ident "expected a type name" p)?
  mut args = Vec::new ()
  loop
    if .try_ident p is Some named
      .push named args
      continue
    -- **A lifetime argument.** `'a` lexes as a `Label` and not an `Ident` —
    -- `scan_quote` reads `'name` as a lifetime or block label — so
    -- `instance (Cursor 'a)` was "expected `)` to close the type". The `'` is
    -- put back, because that is the name everything downstream compares and
    -- prints: `TypeApp::is_lifetime`, `is_var` and `rust_name` all read it.
    if &(.peek p).kind is TokenKind::Label name
      arg = Ident { text = format! \\-> "'#{name}", span = .peek_span p }
      .advance p
      .push arg args
      continue
    break
  if grouped
    (.expect (&TokenKind::RParen) "expected `)` to close the type" p)?
  Ok (TypeApp { head, args, span = (start, (.prev_span p).1) })
''')

write('parse/parser/constraints', '''
export(crate) constraints : &mut Parser -> Result<Vec<Constraint>, ParseError>
constraints p =
  mark = .position p
  mut found = Vec::new ()
  -- Haskell parenthesises a list of two or more; one needs no parentheses.
  parenthesised = .check (&TokenKind::LParen) p
  if parenthesised
    .advance p
  loop
    start = (.peek_span p).0
    guard .try_ident p is Some trait_name else
      break
    guard .try_ident p is Some var else
      break
    .push (Constraint { trait_name, var, span = (start, (.prev_span p).1) }) found
    if !.check (&TokenKind::Comma) p
      break
    .advance p
  if parenthesised && .check (&TokenKind::RParen) p
    .advance p
  if .is_empty found || !.check (&TokenKind::FatArrow) p
    .rewind mark p
    return Ok (Vec::new ())
  .advance p
  Ok found
''')

write('parse/parser/receiver_text', '''
export(crate) receiver_text : &str -> Option<String>
receiver_text ty =
  -- `split.0` is the prefix and `split.1` the bare type, where the Rust
  -- destructured the pair.
  split = if .strip_prefix "&mut " ty is Some rest
    ("&mut ", rest)
  else if .strip_prefix '&' ty is Some rest
    ("&", rest)
  else if .strip_prefix "mut " ty is Some rest
    -- **Rust's fourth receiver form**, and the one plan 67's list of three
    -- missed: a by-value receiver the method may mutate. Plan 74 found it
    -- porting `voltaic-lexer`, whose `LexError::offset_by` is written that way.
    --
    -- Without this the type matched no receiver, stayed an ordinary
    -- parameter, and emitted `fn offset_by(base: usize, self: mut self)` —
    -- not Rust, and reported by rustc rather than by Voltaic, which is the
    -- worse of the two failures.
    --
    -- After `&mut `, because that prefix also contains `mut `.
    ("mut ", rest)
  else
    ("", ty)
  if .trim split.1 != "self"
    return None
  prefix = split.0
  Some (format! \\-> "#{prefix}self")
''')

write('parse/parser/where_entry_is_a_function', '''
export(crate) where_entry_is_a_function : &Parser -> bool
where_entry_is_a_function p =
  named = case (.peek p).kind of
    TokenKind::Ident _ -> true
    _ -> false
  if !named
    return false
  -- A missing second token takes the last arm, which is what the Rust's
  -- `Option` match did with its `_`.
  guard .peek_at 1 p is Some next else
    return .reaches_a_binding_eq p
  case &next.kind of
    -- A signature line, or plan 52's ascription — told apart by whether a
    -- depth-zero `=` follows, which is the one question that separates them.
    TokenKind::Colon -> !.reaches_a_binding_eq p
    -- `name = …` is a binding and `name { … } = …` is a destructuring one;
    -- neither can be a function.
    TokenKind::Eq or TokenKind::LBrace -> false
    -- A name followed by anything else, with a depth-zero `=` still to come:
    -- the tokens between them are the equation's pattern row.
    _ -> .reaches_a_binding_eq p
''')

write('parse/parser/deriving', '''
-- `r#deriving`, because `deriving` is doc 03 §9.5's keyword: a declaration
-- cannot be named one, and the translator already writes `module r#deriving`
-- for the same reason. `data_decl`'s call site is spelled the same way; both
-- emit the identifier `deriving`, so the generated Rust is unchanged.
export(crate) r#deriving : &mut Parser -> Result<Vec<Ident>, ParseError>
r#deriving p =
  if !.check (&TokenKind::Kw (Keyword::Deriving)) p
    return Ok (Vec::new ())
  .advance p
  (.expect (&TokenKind::LParen) "expected `(` after `deriving`" p)?
  mut traits = Vec::new ()
  loop
    .skip_layout p
    if .check (&TokenKind::RParen) p
      break
    -- **A path, not just a name.** `#[derive(serde::Deserialize)]` is
    -- ordinary Rust, and this clause emits straight into that attribute — it
    -- simply could not read one. Nine of `voltaic-mcp`'s nineteen files use the
    -- qualified form, and plan 77 is what found it.
    --
    -- The segments are joined into the `Ident`'s text, which `emit::data`
    -- writes verbatim, so nothing downstream has to know.
    mut name = (.ident "expected a trait name" p)?
    while .check (&TokenKind::ColonColon) p
      .advance p
      segment = (.ident "expected a name after `::`" p)?
      .push_str "::" name.text
      .push_str (&segment.text) name.text
      name.span.1 := segment.span.1
    .push name traits
    .skip_layout p
    if !.check (&TokenKind::Comma) p
      break
    .advance p
  (.expect (&TokenKind::RParen) "expected `)` to close the `deriving` list" p)?
  Ok traits
''')

write('parse/parser/module_decl', '''
export(crate) module_decl : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<ModuleDecl, ParseError>
module_decl p leading start visibility =
  (.expect (&TokenKind::Kw (Keyword::Module)) "expected `module`" p)?
  name = (.ident "expected the module's name" p)?
  -- A body is a line break then an indent. Without one this is `mod name;`,
  -- which loads a sibling file — a different declaration, not an empty one,
  -- which is why `items` is an `Option`.
  has_body = .check (&TokenKind::Newline) p && .is_some_and (\\t -> t.kind == TokenKind::Indent) (.peek_at 1 p)
  if !has_body
    return Ok (ModuleDecl { name, items = None, visibility, span = (start, (.prev_span p).1), leading })
  .advance p
  .advance p
  items = (.items_until_dedent p)?
  Ok (ModuleDecl { name, items = Some items, visibility, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/walk_item_for_tails', '''
import voltaic_ast::Block

export(crate) walk_item_for_tails : &Item -> &mut Vec<ParseError> -> ()
-- `problems`, where the Rust named the parameter `errors`: `errors` is doc 03
-- §14's declaration keyword, so the whole crate uses `problems` for the
-- parameter and `found` for the local.
--
-- `Block` is imported, where the Rust qualified it as `voltaic_ast::Block` — a
-- record literal's head reads as a path either way, and the import matches
-- what `return_in_tail`'s own signature already names.
walk_item_for_tails item problems =
  case item of
    Item::Fn decl ->
      for clause in &decl.clauses
        if .plain clause.body is Some block
          .extend (return_in_tail block) problems
        -- A guarded clause's bodies are expressions, not blocks, so a draw
        -- arrow cannot appear in one — doc 18 §2.3's guard list takes an
        -- expression per arm.
        --
        -- `where` bindings are statements of the clause's own scope, and a
        -- draw arrow in one would flatten the rest of that scope, so they are
        -- walked as a run.
        if !.is_empty clause.wheres
          .extend (return_in_tail (&Block { stmts = .clone clause.wheres, span = clause.span })) problems
    Item::Class decl ->
      for member in &decl.members
        walk_member_for_tails member problems
    Item::Instance decl ->
      for member in &decl.members
        walk_member_for_tails member problems
    Item::Test decl -> .extend (return_in_tail (&decl.body)) problems
    -- The one item that contains items, so the one that recurses.
    Item::Module decl ->
      for inner in .flatten (.iter decl.items)
        walk_item_for_tails inner problems
    -- Inline Rust is never parsed, so it holds no callback tail to check
    -- — plan 45 leaf 1.
    Item::Data _ or Item::Type _ or Item::Errors _ or Item::Import _ or Item::Const _ or Item::InlineRust _ or Item::Extern _ -> ()
  ()
''')

print('plan 82 batch 5: twelve files')
