# -*- coding: utf-8 -*-
r"""Plan 82 batch 10: three of `voltaic-parser`'s largest bodies.

`pattern` is where plan 81's owed defect comes due: the Rust binds
`kind @ (Int | Float | Char | Byte | True | False)`, and `Pattern::rust_text`
emits an `@` over an alternation without the parentheses rustc needs — `E0408`.
So the arm re-reads the token with `token_text` instead of binding it, which is
the same value by a different route and steps around the defect rather than
into it. The row stays owed.

`signature_from` replaces a `map` over a **nested** tuple with a `for` loop:
`|(i, (pattern, ty))|` has no Voltaic spelling, and projecting it would want
`entry.1.0`, where `1.0` lexes as a float.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/signature_from', r'''
import voltaic_ast::Ident

export(crate) signature_from : &[TypeExpr] -> &[Pattern] -> Span -> Result<Signature, ParseError>
signature_from spine bound span =
  -- `split.0` is the return type and `split.1` the parameter types, where the
  -- Rust destructured the pair. The second is `arrow_types` rather than
  -- `params`, which is the name of the field built from it.
  split = .expect "parse_spine returns at least one type" (.split_last spine)
  ret = split.0
  arrow_types = split.1
  arrows = .len arrow_types
  names = .len bound
  if arrows != names
    detail = if names < arrows
      format! \-> "the signature takes #{arrows}, but this equation binds #{names}"
    else
      format! \-> "this equation binds #{names}, but the signature takes #{arrows}"
    message = format! \-> "the signature and the equation disagree about arity: #{detail}"
    return Err (ParseError::r#with Code::ExpectedName message span)
  -- A `for` loop, where the Rust mapped over the zipped iterator: its closure
  -- parameter was `(i, (pattern, ty))`, a **nested** tuple, and a lambda
  -- parameter is a bare name. Projecting the pair instead would want
  -- `entry.1.0`, and `1.0` lexes as a float — the wart `scan_operator`'s own
  -- test records.
  mut params = Vec::new ()
  for entry in .enumerate (.zip arrow_types (.iter bound))
    i = entry.0
    row = entry.1
    binder = row.0
    ty = row.1
    -- A pattern that binds lends its name to the parameter; one that
    -- *tests* cannot, so the parameter gets a generated name and the
    -- pattern moves into the `match` the emitter builds. That is doc 18
    -- §2.3's `__rdx_a0`, and the reason the generated-identifier prefix
    -- exists.
    name = case binder of
      Pattern::Binding ident -> .clone ident
      _ -> Ident { text = format! \-> "__rdx_a#{i}", span = .span binder }
    .push (Param { name, ty = Some (.clone ty), is_mut = false, span = .span binder }) params
  Ok (Signature { params, ret = Some (.clone ret), generics = Vec::new (), is_const = false, is_async = false, is_unsafe = false, bounds = Vec::new (), span })

-- | `#[cfg(test)]`: a type expression at a dummy span. Voltaic can put nothing
-- | but `#[test] fn`s inside its generated `mod __rdx_tests` — plan 73 §2.9.
#[cfg(test)]
ty : &str -> TypeExpr
ty text = TypeExpr { text = .to_string text, span = (0, 0) }

-- | `#[cfg(test)]`: a binding pattern, which is what an ordinary parameter
-- | name is.
#[cfg(test)]
id : &str -> Pattern
id text = Pattern::Binding (Ident { text = .to_string text, span = (0, 0) })

test "pairs the spine with the binders"
  sig = .expect "should build" (signature_from (&[ty "i32", ty "i32", ty "i32"]) (&[id "a", id "b"]) (0, 0))
  assert_eq! \-> .len sig.params \-> 2
  assert_eq! \-> sig.params[0].name.text \-> "a"
  assert_eq! \-> (.expect "a type" (.as_ref sig.params[0].ty)).text \-> "i32"
  assert_eq! \-> (.expect "a return type" sig.ret).text \-> "i32"

-- | `main : ()` keeps its `()` in the tree — plan 55 leaf 4. The emitter is
-- | what declines to write `-> ()`, so `returns_unit` is the question a
-- | reader asks rather than `ret.is_some()`.
test "a unit return is recorded and not discarded"
  sig = .expect "should build" (signature_from (&[ty "()"]) (&[]) (0, 0))
  assert! \-> .is_empty sig.params
  assert_eq! \-> (.expect "a return type" (.as_ref sig.ret)).text \-> "()"
  assert! \-> .returns_unit sig

test "too few binders is an error naming both counts"
  err = .expect_err "should fail" (signature_from (&[ty "i32", ty "i32", ty "i32"]) (&[id "a"]) (0, 0))
  assert! \-> .contains "arity" err.message \-> "#{err.message}"
  assert! \-> .contains "takes 2" err.message \-> "#{err.message}"
  assert! \-> .contains "binds 1" err.message \-> "#{err.message}"

test "too many binders is also an error"
  err = .expect_err "should fail" (signature_from (&[ty "i32"]) (&[id "a"]) (0, 0))
  assert! \-> .contains "binds 1" err.message \-> "#{err.message}"
  assert! \-> .contains "takes 0" err.message \-> "#{err.message}"

-- | A non-unit zero-parameter function keeps its return type.
test "a zero parameter function can still return something"
  sig = .expect "should build" (signature_from (&[ty "u32"]) (&[]) (0, 0))
  assert! \-> .is_empty sig.params
  assert_eq! \-> (.expect "a return type" (.as_ref sig.ret)).text \-> "u32"
  assert! \-> !.returns_unit sig
''')

write('parse/parser/pattern', r'''
import voltaic_ast::Ident

export(crate) pattern : &mut Parser -> Result<Pattern, ParseError>
pattern p =
  span = .peek_span p
  case &(.peek p).kind of
    TokenKind::Ident text | text == "_" ->
      .advance p
      Ok (Pattern::Wildcard span)
    -- An identifier is a binding, unless `@` follows (an as-pattern, doc 03
    -- §6.4) or a `{` does (a record pattern, §6.3). A constructor path with a
    -- `{` is read here too, because `pattern_app` hands an uppercase head
    -- back only when no argument follows and `{` is not an argument.
    TokenKind::Ident _ ->
      name = (.ident "expected a pattern" p)?
      if .check (&TokenKind::At) p
        .advance p
        -- `name @ ..` is a **named rest** and `name @ pattern` is an
        -- as-pattern. Rust admits the first only in a slice, and one node for
        -- it keeps the parser from building what rustc would reject.
        if .check (&TokenKind::DotDot) p
          .advance p
          return Ok (Pattern::NamedRest { name, span = (span.0, (.prev_span p).1) })
        inner = (pattern_app p)?
        return Ok (Pattern::As { name, inner = Box::new inner, span = (span.0, (.prev_span p).1) })
      if .check (&TokenKind::LBrace) p
        return record_pattern p name.text span.0
      Ok (Pattern::Binding name)
    -- `..` alone — doc 03 §6.5's rest, spelled Rust's way because doc 18 §7
    -- removed `...` from the language. Plan 33 §1.3 and §1.4: the meaning is
    -- positional, and this arm is reached only where a *pattern* was expected,
    -- which is never where a range's second operand would be.
    TokenKind::DotDot ->
      .advance p
      Ok (Pattern::Rest span)
    -- `[a, .., z]` — doc 03 §6.4's slice pattern.
    TokenKind::LBracket ->
      .advance p
      items = (pattern_series p (&TokenKind::RBracket))?
      (.expect (&TokenKind::RBracket) "expected `]` to close the slice pattern" p)?
      Ok (Pattern::Slice { items, span = (span.0, (.prev_span p).1) })
    -- Doc 03 §6.5's `...rest` left the language with doc 18 §7, and an author
    -- following §6.5 wrote the published spelling — so the message names the
    -- correction rather than the token, as `RDX0106` does for doc 01 §4.3.
    TokenKind::DotDotDot ->
      Err (ParseError::r#with voltaic_diag::Code::EllipsisRestPattern "`...` left the language with doc 18 §7 — write a rest pattern Rust's way: `..` to discard, or `rest @ ..` to bind" span)
    -- A parenthesised group, which is how a constructor application reaches
    -- an equation's *row* — `depth (Meters d) = d`. Juxtaposition in a row is
    -- ambiguous without it, exactly as in Haskell, so the parentheses are
    -- required there and optional nowhere. Plan 22; see `pattern_app`.
    --
    -- A tuple pattern — `(a, b)` — is **not** this: it needs a comma, and it
    -- is plan 33's along with the record and slice forms.
    TokenKind::LParen ->
      .advance p
      -- `()` is the unit pattern, which is a tuple of nothing.
      if .check (&TokenKind::RParen) p
        .advance p
        return Ok (Pattern::Tuple { items = Vec::new (), span = (span.0, (.prev_span p).1) })
      inner = (pattern_alt p)?
      -- **The comma is the whole difference** between a group and a tuple —
      -- plan 33. A group holds one pattern; a tuple holds a comma.
      if .check (&TokenKind::Comma) p
        .advance p
        mut items = vec! \-> inner
        .extend (pattern_series p (&TokenKind::RParen))? items
        (.expect (&TokenKind::RParen) "expected `)` to close the tuple pattern" p)?
        return Ok (Pattern::Tuple { items, span = (span.0, (.prev_span p).1) })
      (.expect (&TokenKind::RParen) "expected `)` to close a parenthesised pattern" p)?
      Ok inner
    -- `self` is a reserved word, not an identifier, so an equation's binder
    -- row needs it named here. Doc 18 §2.5's methods are written
    -- `describe self = …`, and the binder is dropped by `receiver` — it is
    -- the *type* that decides the receiver, and the name is what the body
    -- refers to.
    TokenKind::Kw (Keyword::SelfValue) ->
      .advance p
      Ok (Pattern::Binding (Ident { text = .to_string "self", span }))
    -- A literal **tests**, and is carried as verbatim source text — the same
    -- discipline doc 03 §14.1 applies to types, so Voltaic never has to agree
    -- with rustc about what a literal means.
    --
    -- The token is re-read with `token_text` rather than bound, where the
    -- Rust wrote `kind @ (…)`: `Pattern::rust_text` emits an `@` over an
    -- alternation without the parentheses rustc wants, which is `E0408` and
    -- an owed row of its own.
    TokenKind::Int _ or TokenKind::Float _ or TokenKind::Char _ or TokenKind::Byte _ or TokenKind::Kw (Keyword::True or Keyword::False) ->
      text = token_text (&(.peek p).kind)
      .advance p
      Ok (range_or_literal p text span)
    TokenKind::Str { raw, .. } ->
      text = .clone raw
      .advance p
      Ok (Pattern::Literal { text, span })
    _ ->
      Err (ParseError::r#with voltaic_diag::Code::ExpectedName "expected a parameter name, a literal pattern, a range, `_`, `(`, `[`, or `=`" span)
''')

write('parse/parser/capture_type', r'''
import voltaic_lexer::Keyword

export(crate) capture_type : &mut Parser -> &str -> TypeStop -> Result<TypeExpr, ParseError>
-- `Keyword` is imported, where the Rust qualified it as `voltaic_lexer::Keyword`
-- inside three `matches!`: those became `case` expressions, and a pattern
-- path's first segment has to be uppercase.
capture_type p message stop =
  start = (.peek_span p).0
  mut depth : i32 = 0
  mut text = String::new ()
  -- Whether the `(` of the group just closed followed an identifier, which
  -- is what makes it `Fn(…)` rather than a unit or a tuple.
  mut group_followed_a_name = false
  mut prev_was_a_name = false
  mut closed_a_named_group = false
  mut prev_kind = TokenKind::Eof
  loop
    kind = &(.peek p).kind
    -- A juxtaposed payload list reads one type per item, so a whitespace
    -- boundary between something that can *end* a type and something that
    -- can *begin* one ends this atom. Without it `Invalid String u32` is a
    -- single field spelled `String u32`.
    --
    -- One line, where the Rust wrapped the condition: a continuation is a
    -- line *ending* in an operator, so a `&&` opening a line is `RDX0102`.
    if depth == 0 && stop == TypeStop::Atom && !.is_empty text && (.peek p).ws_before && can_end_a_type (&prev_kind)
      break
    -- **`Eof` ends the loop at any depth**, and this is a termination proof
    -- rather than a convenience. Every other condition below is guarded by
    -- `depth == 0`, and `p.advance()` at `Eof` does not advance — so a depth
    -- that is wrong by even one turns this loop into a hang that grows `text`
    -- until the process is killed. That happened: see the stray-closer note
    -- on the match below.
    at_eof = case kind of
      TokenKind::Eof -> true
      _ -> false
    if at_eof
      break
    if depth == 0
      ends = case kind of
        TokenKind::Comma or TokenKind::RParen or TokenKind::Eq -> true
        -- Doc 18 §2.4: a record's closing brace, and the bar between two
        -- constructors.
        TokenKind::RBrace or TokenKind::Or or TokenKind::Pipe -> true
        -- Doc 03 §12.5: an `errors` row's `Display` message follows its
        -- payload types with no separator, which is what lets the rows read
        -- as a table. A string literal cannot occur in a Rust type, so
        -- ending here costs no type form.
        TokenKind::Str { .. } or TokenKind::RawStr { .. } -> true
        -- `as` never occurs inside a Rust type — it is a cast operator — and
        -- doc 03 §12.5's `from io::Error as Io` needs the type to end there.
        TokenKind::Kw (Keyword::As) -> true
        -- The refinement: `Fn(i32) -> i32` keeps going; `: i32 ->` stops.
        -- The `(` has to have followed a *name* for that to apply — `()` is
        -- the unit type, and `() -> Option<T>` is a signature spine whose
        -- first type is unit, not a function type. Testing only for a
        -- preceding `)` swallowed the arrow and the return type into the
        -- unit, and reported the arity of a zero-parameter function as zero
        -- when the equation bound one.
        TokenKind::Arrow -> !closed_a_named_group
        TokenKind::Newline or TokenKind::Indent or TokenKind::Dedent or TokenKind::Eof -> true
        -- **A closer with no opener ends the type**, for the same reason
        -- `RParen` and `RBrace` two arms up do: at depth zero there is
        -- nothing for it to close, so it belongs to whatever encloses this
        -- type and the caller's own `expect` should be the one to complain
        -- about it.
        --
        -- Without this, `>` fell through to the match below and took `depth`
        -- to **-1**, which disabled every condition in this block
        -- permanently — a 28-byte input (`n : i32 !> i32 = 1`) hung the
        -- compiler until the OOM killer stopped it. No valid Rust type has a
        -- depth-zero closer: in `Vec<u8>` the `<` has already raised the
        -- depth before the `>` arrives, and `[u8; 4]` works the same way.
        --
        -- `>>>` is here for the same reason, and plan 66 is what adds it:
        -- `Vec<Vec<Vec<i32>>>` closes as one token, so a type ending in a
        -- triple closer reaches depth zero on it.
        TokenKind::Gt or TokenKind::Shr or TokenKind::Compose or TokenKind::RBracket -> true
        _ -> false
      if ends
        break
    case kind of
      TokenKind::LParen ->
        if depth == 0
          group_followed_a_name := prev_was_a_name
        depth += 1
      TokenKind::Lt or TokenKind::LBracket or TokenKind::LBrace -> depth += 1
      -- Reached only at `depth > 0` — every one of these is a depth-zero
      -- terminator above, which is what keeps `depth` from going negative and
      -- is the invariant the `Eof` guard no longer has to rely on.
      TokenKind::Gt or TokenKind::RParen or TokenKind::RBracket or TokenKind::RBrace -> depth -= 1
      -- `>>` closes two generic argument lists, as in `Vec<Vec<u8>>`.
      TokenKind::Shr -> depth -= 2
      -- And `>>>` closes three, as in `Vec<Vec<Vec<i32>>>` — plan 66. The
      -- **opening** side needs nothing: an identifier separates the angles of
      -- a nested generic, so that type opens as three `Lt` tokens.
      TokenKind::Compose -> depth -= 3
      _ -> ()
    is_rparen = case kind of
      TokenKind::RParen -> true
      _ -> false
    closed_a_named_group := is_rparen && group_followed_a_name
    prev_was_a_name := case kind of
      TokenKind::Ident _ or TokenKind::Kw (Keyword::Fn) -> true
      _ -> false
    if !.is_empty text && (.peek p).ws_before
      .push ' ' text
    .push_str (&token_text kind) text
    prev_kind := .clone kind
    .advance p
  if .is_empty text
    return Err (ParseError::r#with Code::ExpectedType message (.peek_span p))
  Ok (TypeExpr { text = ungrouped (&text), span = (start, (.prev_span p).1) })
''')

print('plan 82 batch 10: three files')
