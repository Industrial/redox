# -*- coding: utf-8 -*-
r"""Plan 82 batch 11: three more of `redox-parser`'s largest bodies.

`prefix` hits the `@`-over-an-alternation defect twice more —
`kw @ (Async | Unsafe | Try)` and `kw @ (True | False)` — and both split into
one arm per keyword, which is behaviour-identical and needs no binding at all.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('parse/parser/class_member', r'''
export(crate) class_member : &mut Parser -> &MethodContext -> Result<ClassMember, ParseError>
class_member p ctx =
  -- **Peeked before anything is consumed**, which plan 68 found the hard way:
  -- this was read further down, after `visibility(p)` had taken the `export`
  -- the trivia was attached to, so every doc comment on a member with a
  -- visibility marker was silently discarded. `FnDecl` has carried a
  -- `leading` field all along and `emit_member` has always printed it — the
  -- capture was one token too late.
  --
  -- The same shape as plan 68's fix to `constructor`, where the doc comment
  -- sits above the `=` or `|` rather than above the name. Peek first.
  leading = .peek_leading p
  -- **And the span, for the same reason.** `start` used to be read further
  -- down, after `visibility(p)` had taken the `export` — which the emitter
  -- never minded, because it prints `decl.visibility`, and which broke the
  -- *formatter*: `signature_line` rebuilds a signature from the source slice
  -- at `decl.span.0`, so a span beginning at the name produced a signature
  -- with no visibility, and `redox fmt` wrote that back as canonical. Every
  -- member with a marker lost it — `pub fn` became `fn`.
  --
  -- Plan 68 moved `peek_leading` above `visibility` for the trivia and left
  -- this one behind. Plan 73 found it by putting `redox fmt --check` in the
  -- gate, which is the only thing that reads these members back.
  start = (.peek_span p).0
  -- A trait's members take their visibility from the trait, and an inherent
  -- `impl`'s may each have their own — so the marker is read here and rustc
  -- rejects it where it is meaningless, which is the division of labour
  -- AGENTS.md states.
  member_visibility = (visibility p)?
  if .check (&TokenKind::Kw (Keyword::Type)) p
    .advance p
    assoc = (.ident "expected a name after `type`" p)?
    (.newline_with_trailing "expected a line break after the associated type" p)?
    return Ok (ClassMember::AssocType assoc)
  if .check (&TokenKind::Kw (Keyword::Const)) p
    .advance p
    assoc = (.ident "expected a name after `const`" p)?
    (.expect (&TokenKind::Colon) "expected `:` after the constant's name" p)?
    ty = (capture_type p "expected the constant's type" TypeStop::Greedy)?
    -- A `class` declares the constant and an `instance` supplies it, so the
    -- value is optional here and rustc decides which was required.
    value = if .check (&TokenKind::Eq) p
      .advance p
      Some (parse_expr p 0)?
    else
      None
    (.newline_with_trailing "expected a line break after the associated constant" p)?
    return Ok (ClassMember::AssocConst { name = assoc, ty, value })
  -- The head's variables belong to the enclosing declaration, not to this
  -- method, so they are kept out of the method's generic list.
  reserved = .reserved ctx
  -- Plan 63. An `instance` member may omit its signature, and the class
  -- supplies it. The discriminator is the one doc 18 §2.3 already uses: a
  -- name followed by `:` opens a signature line, and a name not followed by
  -- one opens an equation.
  --
  -- Leaf 0's blocker is why this is read here rather than inside
  -- `Parser::function`: that function's clause loop treats any `Ident` not
  -- followed by `:` as a *continuation equation of the member it is already
  -- reading*, so an elided member collides with the multi-clause grammar and
  -- the name-mismatch guard fires — `RDX0103: this equation defines `x`, but
  -- the signature above it is for `y``. Reading the name here, and branching
  -- before the loop starts, is what keeps the two apart.
  -- `start` is **not** taken here: it is captured at the top of this function,
  -- before `visibility(p)`, so the span covers the member's `export`. The
  -- comment there says what went wrong when it was not.
  -- The name is **peeked**, not consumed, and that is load-bearing rather
  -- than fussy: `Parser::equations`'s loop reads the name once per equation,
  -- because doc 18 §2.3 makes every equation repeat it. A member with a
  -- signature line spells its name twice — once on the line, once on the
  -- equation — and an elided one spells it once. So consuming it here would
  -- leave the loop looking at `self` where it expects `cents_of`, which is
  -- exactly the `RDX0103: expected a member's name` the first attempt earned.
  guard &(.peek p).kind is TokenKind::Ident peeked else
    return Err (ParseError::r#with Code::ExpectedName (.to_string "expected a member's name") (.peek_span p))
  name = Ident { text = .clone peeked, span = .peek_span p }
  has_signature = .is_some_and (\t -> t.kind == TokenKind::Colon) (.peek_at 1 p)
  head = if has_signature
    -- `declared` and `stated`, where the Rust shadowed `name` and `head`.
    declared = (.ident "expected a member's name" p)?
    stated = (.signature_head (&declared) p)?
    -- Collect and never transform — plan 61's rule. A `class` records what it
    -- states so an `instance` can borrow it; nothing is rewritten either way.
    if &ctx.recording is Some recording_class
      .record_head recording_class (&declared.text) (&stated) p
    stated
  else
    borrowed = ctx.implementing
      |> .as_deref
      |> .and_then (\implemented -> .borrowed_head implemented (&name.text) p)
    -- Nothing to borrow: no class, a class this file has not declared, or a
    -- class declared *after* this instance. §10 states that last one as the
    -- row's narrowing rather than hiding it, and the diagnostic says which
    -- of the three it was by naming what it looked for.
    guard borrowed is Some found else
      message = case .as_deref ctx.implementing of
        Some implemented -> format! \-> "`#{name.text}` has no signature here, and `#{implemented}` declares none for it earlier in this file — write the signature, or declare the class above the instance"
        None -> format! \-> "`#{name.text}` needs a signature: an inherent `instance` has no class to take one from"
      return Err (ParseError::r#with Code::ExpectedToken message name.span)
    found
  mut decl = (.equations name leading start (&head) (&reserved) true p)?
  decl.visibility := member_visibility
  receiver (&mut decl.sig)
  -- `pair.0` is the source spelling and `pair.1` its Rust one, where the Rust
  -- destructured the pair in the `for` binder.
  for pair in &ctx.vars
    substitute_var (&mut decl.sig) (&pair.0) (&pair.1)
  Ok (ClassMember::Method decl)
''')

write('parse/parser/control_flow', r'''
export(crate) control_flow : &mut Parser -> Result<Option<Expr>, ParseError>
control_flow p =
  start = (.peek_span p).0
  -- `'name` before a loop keyword. Doc 03 §7.2: *"labels apply equally to
  -- loops, so `break 'outer` and `continue 'outer` reach an enclosing loop."*
  label = case &(.peek p).kind of
    TokenKind::Label text ->
      owned = .clone text
      .advance p
      Some owned
    _ -> None
  if .check (&TokenKind::Kw (Keyword::While)) p || .check (&TokenKind::Kw (Keyword::Until)) p
    negated = .check (&TokenKind::Kw (Keyword::Until)) p
    keyword_span = .peek_span p
    .advance p
    -- `cond`, where the Rust shadowed the `condition` function it calls.
    cond = (condition p)?
    -- `until` lowers to `while !(c)`, and a pattern test cannot be negated
    -- that way: `while !(let Some(v) = x)` is not Rust, and the nearest Rust
    -- — `while !matches!(x, Some(_))` — binds nothing, so it cannot be what a
    -- binding construct meant. Plan 22 §1.3: report it rather than lower it
    -- to something plausible.
    reject = negated && .has_pattern cond
    body = (.statement_block "expected an indented loop body" p)?
    -- Reported **after** the body is read, so recovery resumes at the next
    -- statement rather than inside a block it never consumed. Returning early
    -- produced a second, cascading "expected an expression" against the loop's
    -- own first line — one true error reported twice reads as two defects.
    if reject
      return Err (ParseError::r#with Code::PatternUntil "`until` cannot take a pattern test: it lowers to `while !(…)`, and a pattern test cannot be negated. Write `while` with the opposite pattern, or a `loop` with a `break`" keyword_span)
    return Ok (Some (Expr::While { condition = cond, negated, body, label, span = (start, (.prev_span p).1) }))
  if .check (&TokenKind::Kw (Keyword::For)) p
    .advance p
    binder = (.ident "expected a name after `for`" p)?
    -- Doc 30 §3.2: Rust's own `in`, which doc 18 §3's table had deleted in
    -- favour of `<-` and which costs nothing to restore. `<-` now means
    -- `await` and nothing else.
    (.expect (&TokenKind::Kw (Keyword::In)) "expected `in` after the binder — doc 30 §3.2 writes a `for` with Rust's own word" p)?
    source = Box::new (parse_expr p 0)?
    body = (.statement_block "expected an indented loop body" p)?
    return Ok (Some (Expr::For { binder, source, body, label, span = (start, (.prev_span p).1) }))
  if .check (&TokenKind::Kw (Keyword::Loop)) p
    .advance p
    body = (.statement_block "expected an indented loop body" p)?
    return Ok (Some (Expr::Loop { body, label, span = (start, (.prev_span p).1) }))
  -- A label on a **block** — plan 55 leaf 7, and Rust's own `'name: { … }`.
  -- §1.7 keeps it as the one capability `do` carried that nothing else
  -- provides: how a value leaves a loop that is not a function's tail. The
  -- line ends after the label, which is what tells it from the four loop
  -- forms above.
  if .is_some label && .check (&TokenKind::Newline) p
    return .map Some (block_expr p label start)
  -- A label followed by neither is a mistake worth reporting here: the
  -- alternative is rewinding and letting `'name` be read as an expression,
  -- which it is only when a block follows.
  if label is Some named
    message = format! \-> "expected `while`, `until`, `for`, `loop` or an indented block after the label `'#{named}`"
    return Err (ParseError::r#with Code::ExpectedToken message (.peek_span p))
  if .check (&TokenKind::Kw (Keyword::Break)) p
    .advance p
    break_label = case &(.peek p).kind of
      TokenKind::Label text ->
        owned = .clone text
        .advance p
        Some owned
      _ -> None
    -- A value, if the line has not ended. `break` and `break v` differ by
    -- whether anything follows, which is the same test the `return` statement
    -- plan 01 shipped already makes.
    value = if .at_statement_end p
      None
    else
      Some (Box::new (parse_expr p 0)?)
    return Ok (Some (Expr::Break { label = break_label, value, span = (start, (.prev_span p).1) }))
  if .check (&TokenKind::Kw (Keyword::Continue)) p
    .advance p
    continue_label = case &(.peek p).kind of
      TokenKind::Label text ->
        owned = .clone text
        .advance p
        Some owned
      _ -> None
    return Ok (Some (Expr::Continue { label = continue_label, span = (start, (.prev_span p).1) }))
  if .check (&TokenKind::Kw (Keyword::If)) p
    return .map Some (.if_statement p)
  -- Doc 03 §7.1's `guard`, with doc 18 §3.1's operator. The keyword has had a
  -- lexer entry since plan 02 and no parser until plan 22.
  if .check (&TokenKind::Kw (Keyword::Guard)) p
    .advance p
    -- Plan 60: scrutinee first here too — `guard slot is Some n else`.
    source = Box::new (parse_expr p 0)?
    (.expect (&TokenKind::Kw (Keyword::Is)) "expected `is` before a `guard` pattern" p)?
    pattern = (pattern_alt p)?
    (.expect (&TokenKind::Kw (Keyword::Else)) "expected `else` — a `guard` without one is a binding, and let-else requires it" p)?
    -- `fallback`, where the Rust named the local `otherwise`: that is doc 03
    -- §7.5's keyword. The field keeps its own name as `r#otherwise`.
    fallback = (.statement_block "expected an indented `else` body after `guard`" p)?
    return Ok (Some (Expr::Guard { pattern, source, r#otherwise = fallback, span = (start, (.prev_span p).1) }))
  Ok None
''')

write('parse/parser/prefix', r'''
import redox_ast::Ident

export(crate) prefix : &mut Parser -> Result<Expr, ParseError>
prefix p =
  start = (.peek_span p).0
  if unary_op p is Some op
    operand = (parse_expr p PREFIX_POWER)?
    span = (start, (.span operand).1)
    return Ok (Expr::Unary { op, operand = Box::new operand, span })
  case .clone ((.peek p).kind) of
    TokenKind::Int text or TokenKind::Float text ->
      span = .peek_span p
      .advance p
      Ok (Expr::Num { text, span })
    -- A string with no interpolation keeps `raw` — passed through verbatim,
    -- which is what plan 02 carried `raw` for. One holding an interpolation
    -- becomes `Expr::StrInterp` instead, so every existing path is untouched
    -- (plan 23's criterion 6).
    --
    -- A `"""` block arrives with its dedent already computed and `raw`
    -- holding triple-quoted source, which is not Rust at all — so it takes
    -- the dedented value and the emitter escapes it. Plan 23 §1.3.
    TokenKind::Str { parts, raw } ->
      span = .peek_span p
      .advance p
      -- A `for` with a `case`, where the Rust wrote `any(|part| matches!(…))`:
      -- a `case` needs a line of its own, so it cannot be a one-line lambda
      -- body.
      mut interpolates = false
      for part in &parts
        case part of
          StrPart::Interp { .. } -> interpolates := true
          _ -> ()
      if interpolates
        return interpolated_string (&parts) span
      if .starts_with "\"\"\"" raw
        mut joined = String::new ()
        for part in &parts
          case part of
            StrPart::Text text -> .push_str (.as_str text) joined
            StrPart::Interp { .. } ->
              unreachable! \-> "checked above"
        body = rust_string_body (&joined)
        return Ok (Expr::StrLit { value = format! \-> "\"#{body}\"", span })
      Ok (Expr::StrLit { value = raw, span })
    -- Doc 03 §11.2's four literal kinds, *"all pure passthrough"* — plan 23.
    -- Rust's own spellings, which is why one node carries all of them: nothing
    -- downstream needs to tell them apart, and none may be escaped,
    -- brace-doubled or used as a format template.
    TokenKind::RawStr { text, hashes } ->
      span = .peek_span p
      .advance p
      pad = .repeat hashes "#"
      Ok (Expr::RawLit { text = format! \-> "r#{pad}\"#{text}\"#{pad}", span })
    TokenKind::ByteStr text ->
      span = .peek_span p
      .advance p
      body = rust_string_body (&text)
      Ok (Expr::RawLit { text = format! \-> "b\"#{body}\"", span })
    TokenKind::CStr text ->
      span = .peek_span p
      .advance p
      body = rust_string_body (&text)
      Ok (Expr::RawLit { text = format! \-> "c\"#{body}\"", span })
    TokenKind::Byte value ->
      span = .peek_span p
      .advance p
      Ok (Expr::RawLit { text = byte_literal value, span })
    -- Doc 03 §14.3's `async` block, §12.4's `try` block and Rust's own
    -- `unsafe` — plans 39, 28 and 45 leaf 2, and one arm since plan 55 leaf
    -- 7 made them **modifiers on a block** (§1.7) rather than three
    -- constructs of one shape. `modified_block` has the argument.
    --
    -- Three arms, where the Rust bound `kw @ (Async | Unsafe | Try)`: an `@`
    -- over an alternation emits without the parentheses rustc wants, and each
    -- arm already knows which keyword it matched.
    TokenKind::Kw (Keyword::Async) -> modified_block p Keyword::Async start
    TokenKind::Kw (Keyword::Unsafe) -> modified_block p Keyword::Unsafe start
    TokenKind::Kw (Keyword::Try) -> modified_block p Keyword::Try start
    TokenKind::Char value ->
      span = .peek_span p
      .advance p
      Ok (Expr::Char { value, span })
    -- Two arms, for the reason the three above give.
    TokenKind::Kw (Keyword::True) ->
      span = .peek_span p
      .advance p
      Ok (Expr::Bool { value = true, span })
    TokenKind::Kw (Keyword::False) ->
      span = .peek_span p
      .advance p
      Ok (Expr::Bool { value = false, span })
    -- `@field` → `self.field`, `@@name` → `Self::name` (doc 01 §16). Legal
    -- only in a method body; plan 25 is where that is checked.
    TokenKind::At ->
      .advance p
      name = (.ident "expected a field name after `@`" p)?
      span = (start, name.span.1)
      Ok (Expr::SelfField { name, span })
    TokenKind::AtAt ->
      .advance p
      name = (.ident "expected a name after `@@`" p)?
      span = (start, name.span.1)
      Ok (Expr::SelfType { name, span })
    -- `[` opens a list literal or doc 18 §3.2's comprehension; `bracketed`
    -- tells them apart by what follows the first expression.
    TokenKind::LBracket -> bracketed p
    TokenKind::LParen ->
      .advance p
      -- Plan 65 leaf 1 gave `(` a layout context, so a group whose content
      -- opens on the next line arrives with a `Newline` and an `Indent` in
      -- front of it. Grouping reads none of that — the construct *inside*
      -- the group reads its own layout, which is the whole point of the row.
      .skip_layout p
      -- `()` — the unit value. Doc 18 §7 replaces the `!` zero-argument
      -- suffix with `f ()`, so this is what makes a no-argument call
      -- writable: juxtaposition needs something to juxtapose.
      if .check (&TokenKind::RParen) p
        .advance p
        return Ok (Expr::Unit { span = (start, (.prev_span p).1) })
      -- Doc 18 §4's sections. Tried before the grouping, because `(> 0)` is
      -- not an expression at all — and `section` puts the cursor back when
      -- what follows turns out to be an ordinary group.
      sectioned = (section p start)?
      if sectioned is Some found
        return Ok found
      inner = (parse_expr p 0)?
      .skip_layout p
      -- A tuple, told from a group by its **comma** — plan 33, and the same
      -- rule the tuple *pattern* uses one position over.
      if .check (&TokenKind::Comma) p
        mut items = vec! \-> inner
        while .check (&TokenKind::Comma) p
          .advance p
          .skip_layout p
          if .check (&TokenKind::RParen) p
            break
          .push (parse_expr p 0)? items
          .skip_layout p
        (.expect (&TokenKind::RParen) "expected `)` to close the tuple" p)?
        return Ok (Expr::Tuple { items, span = (start, (.prev_span p).1) })
      (.expect (&TokenKind::RParen) "expected `)` to close the group" p)?
      Ok (Expr::Paren { inner = Box::new inner, span = (start, (.prev_span p).1) })
    -- Doc 18 §4's lambda, and `move \x -> …` with it: `move` can only open
    -- a closure here, so one token of lookahead settles it.
    -- Plan 64 leaf 4: the escape hatch in expression position. §11.5 found
    -- plan 45's `rust """…"""` working as a **top-level item only** — here it
    -- was `RDX0102` — and §1.4's own sample is an expression binding, so the
    -- row owed both this arm and the layout spelling `rust_hatch` reads.
    TokenKind::Kw (Keyword::Rust) ->
      .advance p
      -- `hatched.0` is the code, where the Rust destructured and discarded
      -- the span.
      hatched = (rust_hatch p)?
      Ok (Expr::RustHatch { code = hatched.0, span = (start, (.prev_span p).1) })
    TokenKind::Backslash -> lambda p
    TokenKind::Kw (Keyword::Move) -> lambda p
    -- Doc 01 §8's conditional — plan 20, and one production since plan 55
    -- leaf 7 retired `then`. Leaf 2 chose between two spellings here with a
    -- depth-zero scan for the keyword; with one spelling left there is
    -- nothing to choose, and `if` as a **value** is the same parse as `if` as
    -- a statement. `stmt` asks `control_flow` first, so one only reaches here
    -- when it is being used as a value — which plan 65 made writable inside
    -- brackets, and which is what paid for the retirement.
    TokenKind::Kw (Keyword::If) -> .if_statement p
    -- `do` is reserved and names no construct — `RDX0114`, plan 55 §1.7.
    -- A block needs no keyword in front of it, so the `Newline` arm below is
    -- where the old `do` block went.
    TokenKind::Kw (Keyword::Do) -> Err (reserved_word p)
    -- An indented block, as a value: `inner =` with the block on the lines
    -- under it. Plan 55 leaf 7, and the production `do` used to hold.
    --
    -- A `Newline` here is always this: `parse_expr` is reached with the
    -- cursor on one only when the construct that wanted an expression found
    -- the line ended instead, and a block is what that means now. The old
    -- answer was `RDX0102` "expected an expression".
    TokenKind::Newline -> block_expr p None start
    -- `'name` on a bare block — Rust's own labelled block, and the one
    -- capability `do` carried that nothing else provides (§1.7): how a value
    -- leaves a loop that is not a function's tail.
    --
    -- A label before a *loop* keyword is `control_flow`'s, which is reached
    -- from `stmt` before any expression. So a label arriving here opens a
    -- block or is a mistake, and `statement_block` says which.
    TokenKind::Label name ->
      .advance p
      block_expr p (Some name) start
    -- Doc 18 §2.3's `case e of`. An expression, not a statement: its value is
    -- the matching arm's, which is what lets it be a clause body, an argument
    -- or the right-hand side of a binding with no extra form.
    TokenKind::Kw (Keyword::Case) -> case_of p
    -- `self` and `Self` are expressions, because doc 18 §2.5 drops `@name`
    -- and `@@name` in favour of `self.name` and `Self::name`. They lex as
    -- reserved words, so they need naming here — an `Ident` arm would never
    -- see them.
    TokenKind::Kw (Keyword::SelfValue) ->
      span = .peek_span p
      .advance p
      Ok (Expr::Ident (Ident { text = .to_string "self", span }))
    TokenKind::Kw (Keyword::SelfType) ->
      span = .peek_span p
      .advance p
      Ok (Expr::Ident (Ident { text = .to_string "Self", span }))
    -- `.name` — a method as a function, plan 57. The receiver is the **last**
    -- argument, so this node is the function and `application` supplies the
    -- rest: `.cmp y x` is `x.cmp(y)` and `.trim raw` is `raw.trim()`. Applied
    -- to nothing it is a value awaiting its receiver, which is what retires
    -- `(|> name)`.
    --
    -- Reachable only in **prefix** position, and that is the whole
    -- selector-versus-method rule: a `.` *after* an expression is consumed by
    -- `postfix` as a field access, which is `.`'s one remaining job, so this
    -- row needs no adjacency rule and §6's hazard is a parenthesis rather
    -- than a space.
    TokenKind::Dot ->
      .advance p
      name = (.ident "expected a method name after `.`" p)?
      span = (start, name.span.1)
      Ok (Expr::MethodRef { name, span })
    -- A name, or plan 64's macro invocation when a `!` is glued to it.
    -- `unary_op` is tried at the top of this function, so a `!` reaching
    -- `macro_call` is always *after* an identifier and never before one.
    TokenKind::Ident _ -> macro_call p
    _ ->
      Err (ParseError::r#with Code::ExpectedExpression "expected an expression" (.peek_span p))
''')

print('plan 82 batch 11: three files')
