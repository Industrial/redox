# -*- coding: utf-8 -*-
r"""Plan 82 batch 8: eight more of `voltaic-parser`'s bodies.

Two of these carry inline `#[cfg(test)]` modules — `token_text`'s one test and
`ungrouped`'s five — which become `test "…"` declarations. `ungrouped` also
holds the crate's only `|=`: `compound_assign` reads the five arithmetic forms
and no bitwise one, so the or-assignment is written out as `x := x || y`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/token_text', r'''
export(crate) token_text : &TokenKind -> String
token_text kind = case kind of
  TokenKind::Ident t -> .clone t
  TokenKind::Kw kw -> .to_string (.text kw)
  TokenKind::Int t or TokenKind::Float t -> .clone t
  -- `char`'s and `u8`'s `Debug` spellings *are* Rust source syntax, escapes
  -- included, which is why re-spelling a literal needs no escape table of
  -- its own. Reached from a literal pattern rather than from a type: doc 18
  -- §2.3's `case 'a' or 'e' -> …` was emitting `Char('a')`, the debug-arm
  -- marker, because the literal arms came through here and only the type
  -- tokens had spellings.
  TokenKind::Char c -> format! \-> "#{c:?}"
  TokenKind::Byte b ->
    -- `value` and `ch` bound out, where the Rust wrote `char::from(*b)`
    -- inline: `*` does not open an argument — `starts_argument` has no
    -- `Star` — so a dereference reaches a call through a binding.
    value = *b
    ch = char::from value
    format! \-> "b#{ch:?}"
  TokenKind::Str { raw, .. } -> .clone raw
  TokenKind::Label name -> format! \-> "'#{name}"
  TokenKind::ColonColon -> .to_string "::"
  TokenKind::Lt -> .to_string "<"
  TokenKind::Gt -> .to_string ">"
  TokenKind::Shr -> .to_string ">>"
  -- `Vec<Vec<Vec<i32>>>` closes as one token since plan 66, and a captured
  -- type is re-emitted from these spellings — without this arm the type's
  -- text ends in `Compose`, which is the debug marker below doing its job
  -- on a token that does belong inside a type.
  TokenKind::Compose -> .to_string ">>>"
  TokenKind::LParen -> .to_string "("
  TokenKind::RParen -> .to_string ")"
  TokenKind::LBracket -> .to_string "["
  TokenKind::RBracket -> .to_string "]"
  -- Doc 03 §13.1's brace-nested import trees — `{self, Read, Write}`. The
  -- braces reached the debug fallback and emitted `LBraceself`, which is
  -- what that fallback is for: a visible marker rather than a silent wrong
  -- answer.
  TokenKind::LBrace -> .to_string "{"
  TokenKind::RBrace -> .to_string "}"
  TokenKind::Comma -> .to_string ","
  TokenKind::Semi -> .to_string ";"
  TokenKind::Amp -> .to_string "&"
  TokenKind::Star -> .to_string "*"
  TokenKind::Plus -> .to_string "+"
  TokenKind::Arrow -> .to_string "->"
  TokenKind::Colon -> .to_string ":"
  -- **An associated-type binding** — `Iterator<Item = T>`,
  -- `Future<Output = T>`. `=` is a type terminator at depth zero, where it
  -- is a binding's `=` rather than a type's, so `capture_type` only ever
  -- reaches here inside `<…>`.
  --
  -- Without this arm the debug fallback spelled it `Eq` and the generated
  -- Rust read `dyn Iterator<Item Eq miette::LabeledSpan>`, which rustc
  -- rejects. That is the fallback working as its comment below describes,
  -- and it is how `LBrace` and `Compose` were found too.
  TokenKind::Eq -> .to_string "="
  TokenKind::Bang -> .to_string "!"
  TokenKind::Dot -> .to_string "."
  -- A token with no business inside a type reaches here only if the scan's
  -- terminator set is wrong, so the debug spelling is a visible marker
  -- rather than a silent wrong answer.
  other -> format! \-> "#{other:?}"

-- `Keyword` is qualified rather than imported: it is wanted by this test and
-- by nothing at module scope, and an import used only under `#[cfg(test)]` is
-- an `unused_imports` error in an ordinary build.
test "respells the tokens a type can contain"
  assert_eq! \-> token_text (&TokenKind::Ident (.into "Vec")) \-> "Vec"
  assert_eq! \-> token_text (&TokenKind::Lt) \-> "<"
  assert_eq! \-> token_text (&TokenKind::Shr) \-> ">>"
  assert_eq! \-> token_text (&TokenKind::Compose) \-> ">>>"
  assert_eq! \-> token_text (&TokenKind::Amp) \-> "&"
  assert_eq! \-> token_text (&TokenKind::Kw (voltaic_lexer::Keyword::Mut)) \-> "mut"
  assert_eq! \-> token_text (&TokenKind::Label (.into "a")) \-> "'a"
''')

write('parse/parser/receiver', r'''
export(crate) receiver : &mut Signature -> ()
receiver sig =
  guard .position (\param -> .is_some_and (\ty -> .is_some (receiver_text (&ty.text))) (.as_ref param.ty)) (.iter sig.params) is Some at else
    return ()
  text = sig.params[at].ty
    |> .as_ref
    |> .and_then (\ty -> receiver_text (&ty.text))
    |> .expect "the position just matched"
  span = sig.params[at].span
  -- The whole receiver lives in the name, with no type, which is the shape
  -- `Param` already had for `self` — the emitter writes a parameter's name
  -- and then its type only if there is one.
  sig.params[at] := Param { name = Ident { text, span }, ty = None, is_mut = false, span }
  -- A **generated** name — doc 18 §2.3's `__rdx_aN`, which a required method
  -- or a testing pattern gets instead of a binder — is numbered by its
  -- position in the *Rust* signature rather than in the spine. That is what
  -- makes `scaled : &self -> f64 -> Self` and `scaled : f64 -> &self -> Self`
  -- byte-identical with no equation to supply a name: both give
  -- `fn scaled(&self, __rdx_a1: f64)`. Nothing existing moves, because a
  -- receiver written first already numbered this way.
  if at > 0
    -- `n` bound first: an open-ended range has no spelling, so `0 .. n` is
    -- how `(0..sig.params.len())` is written.
    n = .len sig.params
    mut order : Vec<usize> = .collect (0 .. n)
    .remove at order
    .insert 0 at order
    -- `pair.0` is the Rust position and `pair.1` the spine position, where
    -- the Rust destructured the pair in the `for` binder.
    for pair in .enumerate (.into_iter order)
      rust_at = pair.0
      from = pair.1
      name = &mut sig.params[from].name.text
      if .starts_with "__rdx_a" name
        *name := format! \-> "__rdx_a#{rust_at}"
  ()
''')

write('parse/parser/record_pattern', r'''
export(crate) record_pattern : &mut Parser -> String -> usize -> Result<Pattern, ParseError>
record_pattern p head start =
  -- A `::`-joined path, for `Event::Click { x }`. `::` is a path separator and
  -- nothing else — plan 52 made `:` the type marker.
  mut path = head
  while .check (&TokenKind::ColonColon) p
    .advance p
    segment = (.ident "expected a name after `::`" p)?
    .push_str "::" path
    .push_str (&segment.text) path
  (.expect (&TokenKind::LBrace) "expected `{` to open a record pattern" p)?
  mut fields = Vec::new ()
  mut rest = false
  -- Plan 65 leaf 1: `{` opens a layout context, so a field list spanning
  -- lines carries layout tokens this loop has no use for.
  .skip_layout p
  while !.check (&TokenKind::RBrace) p
    -- `..` ends the field list — *"and other fields"*. Rust requires it last,
    -- so nothing follows it here either.
    if .check (&TokenKind::DotDot) p
      .advance p
      rest := true
      break
    name = (.ident "expected a field name in a record pattern" p)?
    -- No `=` is doc 03 §6.3's *"field pun"*: the field binds under its own
    -- name, which is Rust's own shorthand and needs no spelling.
    bound = if .check (&TokenKind::Eq) p
      .advance p
      Some (pattern_alt p)?
    else
      None
    .push (RecordPatternField { name, pattern = bound }) fields
    .skip_layout p
    if !.check (&TokenKind::Comma) p
      break
    .advance p
    .skip_layout p
  (.expect (&TokenKind::RBrace) "expected `}` to close the record pattern" p)?
  Ok (Pattern::Record { path, fields, rest, span = (start, (.prev_span p).1) })
''')

write('parse/parser/import_path', r'''
import voltaic_lexer::Keyword

export(crate) import_path : &mut Parser -> Result<String, ParseError>
-- `Keyword` is imported, where the Rust qualified it as `voltaic_lexer::Keyword`
-- in all three `matches!`: those became `case` expressions, and a pattern
-- path's first segment has to be uppercase.
import_path p =
  start = .peek_span p
  mut depth : i32 = 0
  mut text = String::new ()
  loop
    kind = &(.peek p).kind
    ends = case kind of
      TokenKind::Newline or TokenKind::Indent or TokenKind::Dedent or TokenKind::Eof or TokenKind::Kw (Keyword::As) -> true
      _ -> false
    if depth == 0 && ends
      break
    case kind of
      TokenKind::LBrace -> depth += 1
      TokenKind::RBrace -> depth -= 1
      _ -> ()
    -- **`as` inside a tree takes its spaces.** The scan breaks on `as` at
    -- depth *zero*, so the only one that reaches here is inside a brace group
    -- — `{Notification as NotificationTrait}` — and Rust needs the spaces
    -- around it. The note below is true of doc 03 §13.1's five forms and not
    -- of a renamed item inside a tree.
    --
    -- Plan 78 found it porting `voltaic-lsp`: the path is one string in the AST,
    -- so `{NotificationasNotificationTrait}` came out of *both* the emitter
    -- and the formatter — Rust that does not compile, and a `voltaic fmt` that
    -- rewrote the source to match it.
    renames = case kind of
      TokenKind::Kw (Keyword::As) -> true
      _ -> false
    if renames
      .push_str " as " text
      .advance p
      continue
    -- No spaces are inserted otherwise. A path has none at depth zero, and
    -- inside a tree Rust's own style is `{self, Read, Write}` — one space
    -- after each comma, which is what the comma arm writes.
    .push_str (&token_text kind) text
    separates = case kind of
      TokenKind::Comma -> true
      _ -> false
    if separates
      .push ' ' text
    .advance p
  if .is_empty text
    return Err (ParseError::r#with Code::ExpectedName "expected a module path after `import`" start)
  Ok text
''')

write('parse/parser/type_vars', r'''
export(crate) type_vars : &[&str] -> Vec<String>
type_vars types =
  mut lifetimes : Vec<String> = Vec::new ()
  mut vars : Vec<String> = Vec::new ()
  for text in types
    chars : Vec<char> = .collect (.chars text)
    mut i = 0
    n = .len chars
    while i < n
      if !is_word chars[i]
        i += 1
        continue
      start = i
      while i < n && is_word chars[i]
        i += 1
      word : String = .collect (.iter chars[start .. i])
      is_lifetime = start > 0 && chars[start - 1] == '\''
      -- **A path segment is not a type variable.** `fmt::Result` has a
      -- lowercase `fmt` and `fmt` is a *module*; making it a parameter emitted
      -- `fn write_ok<FMT>(…) -> FMT::Result`, which rustc rejects with
      -- "associated type `Result` not found". Doc 18 §5.3's rule is about a
      -- type variable, and a variable is never part of a path — so an
      -- identifier adjacent to `::` on either side is skipped.
      after_colons = start >= 2 && chars[start - 2] == ':' && chars[start - 1] == ':'
      before_colons = .get i chars == Some (&':') && .get (i + 1) chars == Some (&':')
      if after_colons || before_colons
        continue
      if is_lifetime
        -- `'static` needs no declaration and `'_` must not have one: it is
        -- the anonymous lifetime, and `fn f<'_>(…)` is `E0637`. See the
        -- doc comment's exclusion list.
        if word != "static" && word != "_"
          named = format! \-> "'#{word}"
          if !.contains (&named) lifetimes
            .push named lifetimes
        continue
      -- `case … of` with a `_ -> false`, where the Rust wrote `matches!`.
      is_type_word = case .as_str word of
        "dyn" or "impl" or "mut" or "as" -> true
        _ -> false
      -- `.chars |> .next |> .is_some_and char::is_lowercase`, where the Rust
      -- wrote `starts_with(|c: char| c.is_lowercase())`. A lambda parameter
      -- takes no annotation in Voltaic, and here the annotation is
      -- **load-bearing**: `str::starts_with` is generic over `Pattern`, so a
      -- bare closure has nothing to infer its parameter from. An empty word
      -- answers `false` either way, which is the one case the two spellings
      -- could have differed on.
      lowercase_head = .is_some_and char::is_lowercase (.next (.chars word))
      -- One line, where the Rust wrapped the condition over five: a
      -- continuation is a line *ending* in an operator, so a `||` opening a
      -- line is `RDX0102`.
      --
      -- `word == "self"` is plan 67's receiver marker, and not a type at all
      -- — see the doc comment's fifth exclusion for why it is not in
      -- `primitive_type`.
      excluded = !lowercase_head || primitive_type (&word) || word == "self" || is_type_word
      if excluded
        continue
      if !.contains (&word) vars
        .push word vars
  .extend vars lifetimes
  lifetimes
''')

write('parse/parser/where_group', r'''
export(crate) where_group : &[Stmt] -> &[Pattern] -> Result<(), ParseError>
where_group entries params =
  bound : Vec<String> = .collect (.flat_map Pattern::binders (.iter params))
  for entry in entries
    case &entry.expr of
      -- A function entry is a declaration and is order-independent; nothing
      -- below applies to it.
      Expr::Item { .. } ->
        continue
      Expr::Binding b ->
        for name in .binders b.pattern
          if .contains (&name) bound
            message = format! \-> "`#{name}` is already a parameter of this equation, and a `where` entry may not shadow one"
            return Err (ParseError::with_help Code::WhereGroupShape message "a `where` group is sorted into dependency order, so a name that means the parameter in one entry and the local in another would depend on where the sort put them — rename the local" b.span)
      other ->
        return Err (ParseError::with_help Code::WhereGroupShape "a `where` entry defines a name, so it is a binding or a function declaration and nothing else" "an expression evaluated for its effect belongs in the body: a `where` group is a set of definitions and is emitted in dependency order, so an entry with an effect has no position to be emitted at" (.span other))
  if where_cycle entries is Some names
    joined = .join ", " names
    message = format! \-> "these `where` bindings depend on each other in a cycle: #{joined}"
    blame = entries
      |> .iter
      |> .find (\e -> .is_some_and (\binder -> .contains (&binder) names) (.where_entry_name e))
      |> .map_or ((0, 0)) (\e -> e.span)
    return Err (ParseError::with_help Code::WhereGroupShape message "Rust `let` bindings are evaluated in order, so a cycle among values has no order to emit. Haskell accepts this and diverges; give one of them a parameter, which makes it a nested function and lets the cycle be recursion" blame)
  Ok ()
''')

write('parse/parser/bracketed', r'''
export(crate) bracketed : &mut Parser -> Result<Expr, ParseError>
bracketed p =
  start = (.peek_span p).0
  (.expect (&TokenKind::LBracket) "expected `[`" p)?
  -- Plan 65 leaf 1 gave `[` a layout context, so the lines inside one carry
  -- `Newline`, `Indent` and `Dedent`. A list's structure is its commas, and
  -- nothing here reads layout — so every boundary skips it.
  .skip_layout p
  if .check (&TokenKind::RBracket) p
    .advance p
    return Ok (Expr::List { items = Vec::new (), span = (start, (.prev_span p).1) })
  -- Above bitwise or, so a bare `|` is left for the qualifier list rather
  -- than absorbed as an operator — doc 18 §5.4's rule, applied to the third
  -- place the bar is structural.
  first = (parse_expr p (above_bitor ()))?
  .skip_layout p
  if .check (&TokenKind::Or) p
    .advance p
    .skip_layout p
    quals = (qualifiers p)?
    .skip_layout p
    (.expect (&TokenKind::RBracket) "expected `]` to close the comprehension" p)?
    return Ok (Expr::Comprehension { element = Box::new first, qualifiers = quals, span = (start, (.prev_span p).1) })
  -- Doc 03 §10.4's array repeat — plan 29. A `;` after the first element
  -- means repeat; a `,` means list. One token of lookahead, no backtracking,
  -- and the comprehension's `|` was already ruled out above.
  if .check (&TokenKind::Semi) p
    .advance p
    .skip_layout p
    count = (parse_expr p 0)?
    .skip_layout p
    (.expect (&TokenKind::RBracket) "expected `]` to close the array repeat" p)?
    return Ok (Expr::ArrayRepeat { value = Box::new first, count = Box::new count, span = (start, (.prev_span p).1) })
  mut items = vec! \-> first
  while .check (&TokenKind::Comma) p
    .advance p
    .skip_layout p
    if .check (&TokenKind::RBracket) p
      break
    .push (parse_expr p 0)? items
    .skip_layout p
  (.expect (&TokenKind::RBracket) "expected `]` to close the list" p)?
  Ok (Expr::List { items, span = (start, (.prev_span p).1) })
''')

write('parse/parser/ungrouped', r'''
export(crate) ungrouped : &str -> String
ungrouped text =
  guard .and_then (\t -> .strip_suffix ')' t) (.strip_prefix '(' text) is Some inner else
    return .to_string text
  -- One group, not two side by side: `(a) -> (b)` must not lose its arrow.
  chars : Vec<char> = .collect (.chars inner)
  mut depth : i32 = 0
  mut top_level_arrow = false
  mut top_level_comma = false
  mut i = 0
  n = .len chars
  while i < n
    -- **`->` is one token, and its `>` is not a closing angle bracket.**
    -- Counting it as one took the depth negative on the very shape this
    -- exists for, so the strip never fired.
    if chars[i] == '-' && .get (i + 1) chars == Some (&'>')
      -- `:=` over an `||`, where the Rust wrote `|=`: `compound_assign` reads
      -- the five arithmetic forms and no bitwise one, so an or-assignment is
      -- spelled out.
      top_level_arrow := top_level_arrow || depth == 0
      i += 2
      continue
    case chars[i] of
      '(' or '[' or '<' -> depth += 1
      ')' or ']' or '>' -> depth -= 1
      -- **A top-level comma means a tuple**, and a tuple keeps its
      -- parentheses however its elements are spelled. That is not the same
      -- test as the arrow: `(&str, fn(char) -> bool)` has both, and the first
      -- version of this stripped it — which the very next file to use one
      -- caught, as `expected = in a binding`.
      ',' | depth == 0 -> top_level_comma := true
      _ -> ()
    -- A closer with no opener means the outer parentheses did not match each
    -- other — `(a) -> (b)` is two groups, not one — so the text stands.
    if depth < 0
      return .to_string text
    i += 1
  if depth != 0 || !top_level_arrow || top_level_comma
    return .to_string text
  .to_string inner

test "a function types grouping goes"
  assert_eq! \-> ungrouped "(impl Fn(char) -> bool)" \-> "impl Fn(char) -> bool"

-- | Each of these is also a single balanced group, and none has a top-level
-- | arrow — which is the whole of why the test is the arrow and not the
-- | brackets.
test "a tuple a unit and a trait object keep theirs"
  assert_eq! \-> ungrouped "(i32, u32)" \-> "(i32, u32)"
  assert_eq! \-> ungrouped "()" \-> "()"
  assert_eq! \-> ungrouped "(dyn Display)" \-> "(dyn Display)"

-- | **A tuple whose element is a function type has both marks**, so the
-- | comma has to be tested as well as the arrow. `scan_number`'s
-- | `(&str, fn(char) -> bool)` is the type that caught the first version.
test "a tuple holding a function type keeps its parentheses"
  assert_eq! \-> ungrouped "(&str, fn(char) -> bool)" \-> "(&str, fn(char) -> bool)"

-- | Two groups side by side are not one group, so the arrow between them is
-- | the type's own and the text is left alone.
test "two groups are not one"
  assert_eq! \-> ungrouped "(a) -> (b)" \-> "(a) -> (b)"

test "an unparenthesised type is untouched"
  assert_eq! \-> ungrouped "Vec<Vec<u8>>" \-> "Vec<Vec<u8>>"
  assert_eq! \-> ungrouped "&'a str" \-> "&'a str"
''')

print('plan 82 batch 8: eight files')
