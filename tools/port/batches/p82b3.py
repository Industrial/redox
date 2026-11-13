# -*- coding: utf-8 -*-
"""Plan 82 batch 3: twelve more of `voltaic-parser`'s bodies."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('parse/parser/range_or_literal', '''
export(crate) range_or_literal : &mut Parser -> String -> Span -> Pattern
range_or_literal p lo span =
  inclusive = case &(.peek p).kind of
    TokenKind::DotDotEq -> true
    TokenKind::DotDot -> false
    _ ->
      return Pattern::Literal { text = lo, span }
  .advance p
  hi = token_text (&(.peek p).kind)
  end = (.peek_span p).1
  .advance p
  Pattern::Range { lo, hi, inclusive, span = (span.0, end) }
''')

write('parse/parser/rewritten', '''
export(crate) rewritten : &str -> &str -> &str -> String
rewritten text var subject =
  mut out = String::with_capacity (.len text)
  mut word = String::new ()
  for ch in .chars text
    if .is_alphanumeric ch || ch == '_'
      .push ch word
      continue
    flush_word (&mut out) (&mut word) var subject
    .push ch out
  flush_word (&mut out) (&mut word) var subject
  out
''')

write('parse/parser/block_expr', '''
export(crate) block_expr : &mut Parser -> Option<String> -> usize -> Result<Expr, ParseError>
block_expr p label start =
  body = (.statement_block "expected an indented block" p)?
  Ok (Expr::Block { label, body, span = (start, (.prev_span p).1) })
''')

write('parse/parser/expr_from_tokens', '''
export(crate) expr_from_tokens : &[Token] -> Span -> Result<Expr, ParseError>
expr_from_tokens tokens span =
  mut toks = .to_vec tokens
  .push (Token { kind = TokenKind::Eof, span, ws_before = false, leading = Vec::new (), trailing = None }) toks
  mut p = Parser::new (&toks)
  expr = (parse_expr (&mut p) 0)?
  -- An interpolation holds **one** expression. Leftover tokens mean the body
  -- was something else — `#{a b c}` with no application, say — and silently
  -- dropping them would emit a format argument the author did not write.
  if !.at_eof p
    return Err (ParseError::r#with voltaic_diag::Code::ExpectedToken "an interpolation holds one expression; `}` was expected here" (.peek_span p))
  Ok expr
''')

write('parse/parser/parse_spine', '''
export(crate) parse_spine : &mut Parser -> Result<Vec<TypeExpr>, ParseError>
parse_spine p =
  mut spine = vec! \\-> (capture_type p "expected a type after `::`" TypeStop::Greedy)?
  while .check (&TokenKind::Arrow) p
    .advance p
    .push (capture_type p "expected a type after `->`" TypeStop::Greedy)? spine
  Ok spine
''')

write('parse/parser/arm_body', '''
export(crate) arm_body : &mut Parser -> Result<Expr, ParseError>
arm_body p =
  start = (.peek_span p).0
  value = (parse_expr p 0)?
  assign_tail p value start
''')

write('parse/parser/method_call_syntax', '''
export(crate) method_call_syntax : &str -> Span -> ParseError
method_call_syntax name span =
  help = format! \\-> "a method is a function whose last argument is its receiver, so write `.#{name} args recv` or `recv |> .#{name} args`. `.` is field access and nothing else (plan 57 §1.1)"
  ParseError::with_help Code::MethodCallSyntax "method-call syntax is retired" help span
''')

write('parse/parser/return_in_tail', '''
export(crate) return_in_tail : &Block -> Vec<ParseError>
return_in_tail block =
  mut found = Vec::new ()
  scan_run (&block.stmts) false (&mut found)
  found
''')

write('parse/parser/starts_a_declaration', '''
export(crate) starts_a_declaration : &Parser -> bool
starts_a_declaration p = case (.peek p).kind of
  TokenKind::Kw (Keyword::Data or Keyword::Newtype or Keyword::Union or Keyword::Type or Keyword::Class or Keyword::Instance or Keyword::Errors or Keyword::Import or Keyword::Const or Keyword::Static) -> true
  _ -> false
''')

write('parse/parser/class_body', '''
export(crate) class_body : &mut Parser -> &MethodContext -> Result<Vec<ClassMember>, ParseError>
class_body p ctx =
  mut members = Vec::new ()
  opens = .check (&TokenKind::Newline) p && .is_some_and (\\t -> t.kind == TokenKind::Indent) (.peek_at 1 p)
  if !opens
    return Ok members
  .advance p
  .advance p
  loop
    .skip_separators p
    if .check (&TokenKind::Dedent) p || .at_eof p
      break
    .push (class_member p ctx)? members
  (.expect (&TokenKind::Dedent) "expected the body to end" p)?
  Ok members
''')

write('parse/parser/interpolated_string', '''
export(crate) interpolated_string : &[StrPart] -> Span -> Result<Expr, ParseError>
interpolated_string parts span =
  mut out = Vec::with_capacity (.len parts)
  for part in parts
    case part of
      -- Escaped **here**, not in the emitter. The lexer decodes escapes, so
      -- the text has to be re-escaped for Rust somewhere, and doing it in the
      -- parser means one implementation serves the `"""` block, the byte and
      -- C-string literals and this — rather than the same function living in
      -- two crates that cannot call each other.
      StrPart::Text text -> .push (StrInterpPart::Text (rust_string_body text)) out
      -- The first bad interpolation ends the string, as the first bad
      -- anything ends any other expression. The parser's batch recovery
      -- happens at item boundaries, which is where it belongs.
      StrPart::Interp { tokens, spec } ->
        .push (StrInterpPart::Interp { expr = (expr_from_tokens tokens span)?, spec = .clone spec }) out
  Ok (Expr::StrInterp { parts = out, span })
''')

write('parse/parser/scan_run', '''
export(crate) scan_run : &[Stmt] -> bool -> &mut Vec<ParseError> -> ()
scan_run stmts in_tail problems =
  -- `i` and `stmt` bound from the pair: a `for` binder is a bare name.
  for pair in .enumerate (.iter stmts)
    i = pair.0
    stmt = pair.1
    if &stmt.expr is Expr::Draw { .. }
      -- Everything after a `with` is its closure's body, and the `with`
      -- itself ends this run — so the rest is scanned as a tail and nothing
      -- here follows it.
      --
      -- `stmts[from .. n]` with `n` bound first: an open range has no
      -- spelling.
      from = i + 1
      n = .len stmts
      scan_run (&stmts[from .. n]) true problems
      return ()
    if in_tail && &stmt.expr is Expr::Return { .. }
      .push (ParseError::r#with Code::ReturnInCallbackTail "`return` here returns from the closure `with` generates, not from the enclosing function — doc 03 §7.6. Use `?` to propagate, or restructure so the value is the callback's" stmt.span) problems
    -- Descend into the blocks that stay inside this closure. Every one of
    -- these is a statement list in the same scope, so a `return` in it leaves
    -- the same closure.
    for nested in .statement_blocks stmt.expr
      scan_run (&nested.stmts) in_tail problems
  ()
''')

print('plan 82 batch 3: twelve files')
