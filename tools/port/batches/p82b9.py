# -*- coding: utf-8 -*-
r"""Plan 82 batch 9: five of `redox-parser`'s larger bodies.

`binary_power` brings the second `#[cfg(test)]` helper pattern — a private
signature under a `#[cfg(test)]` attribute, as `redox-lexer`'s `scan_operator`
already does — and one import that has to stay a *path*: the test wants
`PREFIX_POWER`, nothing at module scope does, and an import used only under
`#[cfg(test)]` is an `unused_imports` error in an ordinary build.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('parse/parser/instance_decl', r'''
export(crate) instance_decl : &mut Parser -> Vec<Trivia> -> usize -> Result<InstanceDecl, ParseError>
-- `limits`, where the Rust named the local `constraints`: that is the name of
-- the function it calls, and `RDX1003` reports the shadow. The field keeps
-- its own name.
instance_decl p leading start =
  (.expect (&TokenKind::Kw (Keyword::Instance)) "expected `instance`" p)?
  limits = (constraints p)?
  -- Parentheses on the *first* type mean it is the target and there is no
  -- class — nothing else would be worth grouping there.
  grouped = .check (&TokenKind::LParen) p
  first = (type_app p)?
  -- `split.0` is the class and `split.1` the target, where the Rust
  -- destructured the pair.
  split = if grouped
    (None, first)
  else if .check (&TokenKind::LParen) p
    -- `instance Describe (Wrapper t)`, and `instance Convert f64 (Wrapper t)`
    -- — the whole juxtaposition before the group is the class.
    (Some first, (type_app p)?)
  else if .is_empty first.args
    (None, first)
  else
    -- `instance Show Point` and `instance Convert f64 Meters` — juxtaposition
    -- put the target in the argument list, and it is the **last** one.
    mut args = .clone first.args
    head = .expect "the argument list is not empty" (.pop args)
    class_end = .map_or first.head.span.1 (\a -> a.span.1) (.last args)
    class_app = TypeApp { head = .clone first.head, args, span = (first.head.span.0, class_end) }
    target_app = TypeApp { head = .clone head, args = Vec::new (), span = head.span }
    (Some class_app, target_app)
  -- `implemented`, where the Rust named the local `class`: `class` is doc 03
  -- §15's keyword. The field keeps its own name as `r#class`, which is how
  -- `redox-ast` declares it and how `emit::instance` already reads it.
  implemented = split.0
  target = split.1
  var = .map (\v -> .to_string v) (.first (.vars target))
  -- Inside an `instance` a target's type variable is the impl's generic
  -- parameter, so it is that parameter's uppercase name — doc 18 §5.3.
  subject = .map_or_else String::new TypeApp::rust_name (.as_deref var)
  mut ctx = MethodContext::for_instance (.as_deref var) (&subject)
  -- Plan 63 leaf 1: the class this instance implements is where an elided
  -- member's signature comes from. `None` for an inherent `instance`, which
  -- has no class and so cannot elide — the diagnostic in `class_member` says
  -- exactly that.
  ctx.implementing := .map (\c -> .clone c.head.text) (.as_ref implemented)
  members = (class_body p (&ctx))?
  Ok (InstanceDecl { r#class = implemented, target, constraints = limits, members, span = (start, (.prev_span p).1), leading })
''')

write('parse/parser/section', r'''
export(crate) section : &mut Parser -> usize -> Result<Option<Expr>, ParseError>
section p start =
  mark = .position p
  -- `(.name)` — GHC's record dot.
  --
  -- **A `)` immediately after the name is the discriminator**, and since plan
  -- 57 it is doing real work: a bare `.name` is a method, so `(.age)` closes
  -- and `(.cmp y)` does not. This used to `expect` the `)` and propagate,
  -- which made `(.cmp 1)` a hard `RDX0101` where its three siblings below
  -- rewind; it rewinds too now, and the parenthesis is an ordinary group
  -- around the method application inside it.
  --
  -- The lookahead is an `if` over a `case`, where the Rust wrote
  -- `is_some_and(|t| matches!(…))`: a `case` needs a line of its own, so it
  -- cannot be a one-line lambda body.
  names_a_field = if .peek_at 1 p is Some next
    case next.kind of
      TokenKind::Ident _ -> true
      _ -> false
  else
    false
  if .check (&TokenKind::Dot) p && names_a_field
    .advance p
    name = (.ident "expected a field name after `.`" p)?
    if .check (&TokenKind::RParen) p
      .advance p
      return Ok (Some (Expr::Section { section = Section::Field name, span = (start, (.prev_span p).1) }))
    .rewind mark p
    return Ok None
  -- `(|> name)` — retired by plan 57, which builds its replacement. Kept as
  -- an arm rather than deleted, so the spelling names `.name` instead of
  -- reaching `prefix` and failing as *"expected an expression"*.
  if .check (&TokenKind::Pipe) p && .peek_at 1 p is Some token && &token.kind is TokenKind::Ident named
    end = token.span.1
    return Err (pipeline_section named (start, end))
  -- A right section: the operator leads. The four operators that are also
  -- *prefix* operators are excluded, so `(- 1)`, `(&x)`, `(*p)` and
  -- `(!flag)` keep their prefix reading — see `opens_a_section`.
  if opens_a_section (&(.peek p).kind)
    if binary_op p is Some leading_op
      operand = (parse_expr p 0)?
      (.expect (&TokenKind::RParen) "expected `)` to close the section" p)?
      return Ok (Some (Expr::Section { section = Section::Right { op = leading_op, operand = Box::new operand }, span = (start, (.prev_span p).1) }))
    .rewind mark p
  -- A left section: the operator trails, so the operand is read first and
  -- the read is abandoned if no `op )` follows.
  guard atom p is Ok operand else
    .rewind mark p
    return Ok None
  if binary_op p is Some trailing_op && .check (&TokenKind::RParen) p
    .advance p
    return Ok (Some (Expr::Section { section = Section::Left { op = trailing_op, operand = Box::new operand }, span = (start, (.prev_span p).1) }))
  .rewind mark p
  Ok None
''')

write('parse/parser/postfix', r'''
export(crate) postfix : &mut Parser -> Expr -> Result<Result<Expr, Expr>, ParseError>
postfix p lhs =
  -- **A postfix step cannot cross a dedent**, which is the same rule
  -- `application` states for an argument and for the same reason: an
  -- expression with layout of its own — doc 18 §2.3's `case … of`, an `if`,
  -- a `guard` ladder — ends on a `Dedent`, and whatever follows is on a later
  -- line and belongs to the next statement.
  --
  -- Without this, a `case` statement followed by a line opening with `.` —
  -- which is how every method-call statement in Redox opens — read as a field
  -- access on the `case`, and `application` then refused it as `RDX0115`:
  --
  --     case hit of
  --       true -> ()
  --       false -> ()
  --     .push_str "z" (&mut out)
  --
  -- `application` could not help, because it guards its *argument* loop and
  -- this runs inside `atom`, below it. Plan 80 found it on the first
  -- `redox-driver` body that needed an `if` in expression position; the `if`
  -- *statement* form happened to be safe, which is why nine ports went by.
  if .just_dedented p
    return Ok (Err lhs)
  start = (.span lhs).0
  -- **No paren-call step.** Doc 18 §2.1 makes parentheses grouping only:
  -- `f(x)`, `f (x)` and `f x` are one expression, and `application` builds
  -- the call from juxtaposition. A postfix call here would take the
  -- parenthesised group as an argument list and leave any following argument
  -- to form a *second* call — `map (\n -> n * n) xs` emitted
  -- `map(|n| n * n)(xs)` until this was removed.
  --
  -- And a method call is not a postfix step at all since plan 57: a method is
  -- a function whose last argument is its receiver, so `.name` is read in
  -- *prefix* position and the receiver arrives through `application`.

  -- `e?` — Rust's try operator. A postfix step like any other, and the
  -- reason doc 03 §12.5's generated `From` impl has a caller.
  if .check (&TokenKind::Question) p
    -- `?.` is retired — plan 57 leaf 4, `RDX0117`. The token is gone, so this
    -- is an **adjacency** check rather than a branch: `x? .name` is two
    -- ordinary postfix steps and stays legal, and only the glued spelling is
    -- refused, because it used to mean something else and would otherwise
    -- silently become a different program that compiles.
    glued = .is_some_and (\t -> t.kind == TokenKind::Dot && !t.ws_before) (.peek_at 1 p)
    if glued
      at = .peek_span p
      return Err (optional_chaining (at.0, at.1 + 1))
    .advance p
    return Ok (Ok (Expr::Try { inner = Box::new lhs, span = (start, (.prev_span p).1) }))
  if .check (&TokenKind::LBracket) p
    .advance p
    .skip_layout p
    index = (parse_expr p 0)?
    .skip_layout p
    (.expect (&TokenKind::RBracket) "expected `]` to close the index" p)?
    return Ok (Ok (Expr::Index { recv = Box::new lhs, index = Box::new index, span = (start, (.prev_span p).1) }))
  -- Doc 18 §2.4's record literal. A `{` after an expression is unambiguous:
  -- Redox has no brace-delimited block in expression position — layout does
  -- that job — so nothing else a `{` could open is in the grammar.
  if .check (&TokenKind::LBrace) p
    return Ok (Ok (record_literal p lhs)?)
  if .check (&TokenKind::Dot) p
    .advance p
    -- `m.0` — a tuple field. Doc 18 §2.4's `newtype Meters = Meters f64`
    -- lowers to a tuple struct, and a tuple struct whose field cannot be read
    -- is a declaration with no use. The index is an `Ident` carrying digits
    -- rather than a variant of its own, because `Expr::Field` already holds
    -- exactly "a name after a dot" and the emitter writes it through
    -- unchanged.
    if &(.peek p).kind is TokenKind::Int digits
      index_name = Ident { text = .clone digits, span = .peek_span p }
      .advance p
      return Ok (Ok (Expr::Field { recv = Box::new lhs, name = index_name, span = (start, (.prev_span p).1) }))
    name = (.ident "expected a field name or a tuple index after `.`" p)?
    -- A `(` **adjacent** to the name was Rust's method-call syntax, which plan
    -- 57 leaf 3 retires — `RDX0115`. Half of the retirement is here and half
    -- is in `application`, because §11.2 measured **two** AST shapes for one
    -- form: this built `Expr::MethodCall`, and the parenless `x.f a` is an
    -- application over the `Expr::Field` built below. Refusing only this one
    -- would leave `x.f (a)` re-parsing as the other and emitting the
    -- identical `x.f(a)` silently.
    --
    -- **Adjacency is what separates the two halves, and testing for any `(`
    -- at all refused a legal program.** `.contains p.name (&text)` is the
    -- ordinary receiver-last call, and its `(` opens the *enclosing*
    -- application's next argument rather than a call on the field — but this
    -- runs inside `atom`, which cannot see that there is an enclosing
    -- application. Plan 79 found it in `run/usage.sdx`, where the formatter
    -- then round-tripped the parentheses off a working program.
    --
    -- A *spaced* `x.f (a)` is still refused, and by the other half: it
    -- becomes the `Expr::Field` below, and `application` refuses a field as
    -- the head of an application. The two halves now split on whitespace,
    -- which is exactly how the two spellings differ.
    if .check (&TokenKind::LParen) p && !(.peek p).ws_before
      return Err (method_call_syntax (&name.text) (start, name.span.1))
    return Ok (Ok (Expr::Field { recv = Box::new lhs, name, span = (start, (.prev_span p).1) }))
  Ok (Err lhs)
''')

write('parse/precedence/binary_power', r'''
export(crate) binary_power : &TokenKind -> Option<(BinaryOp, u8, u8)>
binary_power kind =
  -- Loosest first, so the table still reads in precedence order even though
  -- the numbers now live on the operator.
  op = case kind of
    TokenKind::Pipe -> BinaryOp::Pipeline
    TokenKind::Coalesce -> BinaryOp::Coalesce
    -- Doc 03 §4.5 places `catch` *"at `??`'s level"* — plan 28. Being a
    -- keyword and not in `starts_argument`'s allow-list is what makes
    -- `parse s catch 0` need no parentheses, which §12.2 claims and plan 28
    -- §1.4 verified rather than assumed.
    TokenKind::Kw (Keyword::Catch) -> BinaryOp::Catch
    TokenKind::DotDot -> BinaryOp::Range
    TokenKind::DotDotEq -> BinaryOp::RangeInclusive
    TokenKind::OrOr -> BinaryOp::Or
    TokenKind::AndAnd -> BinaryOp::And
    TokenKind::EqEq -> BinaryOp::Eq
    TokenKind::Ne -> BinaryOp::Ne
    TokenKind::Lt -> BinaryOp::Lt
    TokenKind::Gt -> BinaryOp::Gt
    TokenKind::Le -> BinaryOp::Le
    TokenKind::Ge -> BinaryOp::Ge
    TokenKind::Or -> BinaryOp::BitOr
    TokenKind::Caret -> BinaryOp::BitXor
    TokenKind::Amp -> BinaryOp::BitAnd
    TokenKind::Shl -> BinaryOp::Shl
    TokenKind::Shr -> BinaryOp::Shr
    TokenKind::Plus -> BinaryOp::Add
    TokenKind::Minus -> BinaryOp::Sub
    TokenKind::Star -> BinaryOp::Mul
    TokenKind::Slash -> BinaryOp::Div
    TokenKind::Percent -> BinaryOp::Rem
    -- Plan 66, the tightest rung: composition binds above `*` so that
    -- `xs |> f >>> g` is `(f >>> g) xs`.
    TokenKind::Compose -> BinaryOp::Compose
    TokenKind::ComposeBack -> BinaryOp::ComposeBack
    _ ->
      return None
  left = .power op
  Some (op, left, left + 1)

-- | `#[cfg(test)]`: the helper every test below shares. Redox can put nothing
-- | but `#[test] fn`s inside its generated `mod __rdx_tests` — plan 73 §2.9.
#[cfg(test)]
power : TokenKind -> u8
power kind = (.expect "should have a precedence" (binary_power (&kind))).1

-- | Rust's ordering, asserted rather than assumed.
test "precedence follows rust"
  assert! \-> power TokenKind::Star > power TokenKind::Plus
  assert! \-> power TokenKind::Plus > power TokenKind::Shl
  assert! \-> power TokenKind::Shl > power TokenKind::Amp
  assert! \-> power TokenKind::Amp > power TokenKind::Caret
  assert! \-> power TokenKind::Caret > power TokenKind::Or
  assert! \-> power TokenKind::Or > power TokenKind::EqEq
  assert! \-> power TokenKind::EqEq > power TokenKind::AndAnd
  assert! \-> power TokenKind::AndAnd > power TokenKind::OrOr

-- | Doc 03 §4.5's placement for the operators Rust does not have.
test "the new operators sit where doc 03 places them"
  assert! \-> power TokenKind::Coalesce < power TokenKind::OrOr
  assert! \-> power TokenKind::Pipe < power TokenKind::Coalesce

-- `PREFIX_POWER` is reached by its path rather than imported: nothing at
-- module scope wants it, and an import used only under `#[cfg(test)]` is an
-- `unused_imports` error in an ordinary build.
test "unary and postfix bind tighter than any binary"
  assert! \-> crate::parse::precedence::prefix_power::PREFIX_POWER > power TokenKind::Star

-- `POSTFIX_POWER` was asserted here against `PREFIX_POWER` until doc 18 §2.1
-- made application the primary: postfix is now consumed inside
-- `application::atom` rather than ranked on the precedence ladder, so the
-- constant has no value to compare and was deleted with the ladder rung.

test "binary operators are left associative"
  for kind in [TokenKind::Plus, TokenKind::Star, TokenKind::Pipe]
    -- `rung.1` is the left power and `rung.2` the right, where the Rust
    -- destructured the triple.
    rung = .expect "should have a precedence" (binary_power (&kind))
    assert_eq! \-> rung.2 \-> rung.1 + 1 \-> "#{kind:?} should be left-associative"

test "a non operator has no precedence"
  assert! \-> .is_none (binary_power (&TokenKind::Ident (.into "x")))
  assert! \-> .is_none (binary_power (&TokenKind::Newline))

-- | The powers this file reports are the operator's own, not a second copy.
-- |
-- | This is the assertion that the move was a move. If anyone re-introduces
-- | a literal here, it fails.
test "the powers come from the operator"
  for kind in [TokenKind::Pipe, TokenKind::Coalesce, TokenKind::OrOr, TokenKind::EqEq, TokenKind::Plus, TokenKind::Percent]
    rung = .expect "should have a precedence" (binary_power (&kind))
    assert_eq! \-> rung.1 \-> .power rung.0 \-> "#{kind:?} reports a power of its own"
    assert_eq! \-> rung.2 \-> .power rung.0 + 1

test "maps tokens to their operators"
  assert_eq! \-> .map (\pair -> pair.0) (binary_power (&TokenKind::Plus)) \-> Some BinaryOp::Add
  assert_eq! \-> .map (\pair -> pair.0) (binary_power (&TokenKind::DotDot)) \-> Some BinaryOp::Range \-> "`..` is exclusive (doc 03 §4.2)"
''')

write('parse/parser/data_decl', r'''
export(crate) data_decl : &mut Parser -> Vec<Trivia> -> usize -> Visibility -> Result<DataDecl, ParseError>
data_decl p leading start visibility =
  is_newtype = .check (&TokenKind::Kw (Keyword::Newtype)) p
  -- Plan 45 leaf 4. The third keyword this one function serves, and the
  -- reason it reads the keyword rather than being told which it was.
  is_union = .check (&TokenKind::Kw (Keyword::Union)) p
  .advance p
  name = (.ident "expected a type name after `data`" p)?
  mut constructors = Vec::new ()
  mut derives = Vec::new ()
  mut depth : usize = 0
  -- The `=` row, the `|` rows and the `deriving` clause may each sit on the
  -- declaration's own line or on an indented one, in any combination.
  loop
    -- Inside an indented block a line break only separates rows. *Outside*
    -- one it terminates the declaration, and must be left for the caller —
    -- consuming it here is what made `newtype Meters = Meters f64` report
    -- "expected a line break after the declaration" at the *next* item.
    if depth > 0
      .skip_separators p
    opens_a_block = .check (&TokenKind::Newline) p && .is_some_and (\t -> t.kind == TokenKind::Indent) (.peek_at 1 p)
    if opens_a_block
      .advance p
      .advance p
      depth += 1
      continue
    if .check (&TokenKind::Eq) p || .check (&TokenKind::Or) p
      -- **Before the separator is eaten**, which is plan 68's finding: a
      -- constructor's doc comment goes on its own line above the variant, so
      -- the trivia attaches to the `=` or `|` that opens that line and
      -- `peek_leading` at the *name* looks one token too late.
      --
      -- `row_leading`, where the Rust shadowed the `leading` parameter.
      row_leading = .peek_leading p
      .advance p
      .push (constructor p row_leading)? constructors
      continue
    if .check (&TokenKind::Kw (Keyword::Deriving)) p
      derives := (r#deriving p)?
      continue
    break
  for _closer in 0 .. depth
    .skip_separators p
    (.expect (&TokenKind::Dedent) "expected the declaration to end" p)?
  mut decl = DataDecl { name, generics = Vec::new (), constructors, derives, is_newtype, is_union, visibility, span = (start, (.prev_span p).1), leading }
  -- Doc 18 §5.3's generic list, from the field types. After the declaration
  -- is assembled because it reads every constructor, and before
  -- `newtype_shape` only because that check does not care either way.
  generalise_data (&mut decl)
  -- Plan 53 leaf 2. On the finished declaration, because `is_newtype`'s
  -- narrowing is a fact about the constructor *list* — the field count alone
  -- would miss `newtype Multi = A | B`, which emitted an enum.
  (newtype_shape (&decl))?
  Ok decl
''')

print('plan 82 batch 9: five files')
