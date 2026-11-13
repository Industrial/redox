# -*- coding: utf-8 -*-
"""Plan 81 batch 6: seven more, and one fix to batch 4.

`one_line_if`'s Rust names a parameter `otherwise`, which is doc 03's keyword
for a guard ladder's last row — the ninth keyword collision of the port, and
the first on a *parameter*.

The fix to batch 4: `compose_chain` was given a second function in its own
file, which `one_construct_per_file.rs` would have reported. The `case` goes
inline instead.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

# --- fix to batch 4 ------------------------------------------------------
write('emit/compose_chain', '''
export(crate) compose_chain : BinaryOp -> &'a Expr -> &'a Expr -> &mut Vec<&'a Expr> -> ()
compose_chain op lhs rhs out =
  -- `swap` inline rather than a helper beside this function: doc 06 §1 is one
  -- construct per file. The `case` is the Rust's `matches!`, whose second
  -- argument is a pattern.
  swap = case op of
    BinaryOp::ComposeBack -> true
    _ -> false
  -- `sides.0` and `sides.1`, where the Rust destructured the pair.
  sides = if swap
    (rhs, lhs)
  else
    (lhs, rhs)
  -- A guard rather than `inner @ (A or B)`, because the emitter writes an
  -- as-pattern over an alternation **without parentheses** — `inner @ A | B`,
  -- which rustc refuses as `E0408: variable inner is not bound in all
  -- patterns`. Plan 81 §2 records it; the row is owed.
  for side in [sides.0, sides.1]
    bare = unparen side
    composed = case bare of
      Expr::Binary { op, lhs, rhs, .. } | *op == BinaryOp::Compose || *op == BinaryOp::ComposeBack ->
        Some (*op, lhs, rhs)
      _ -> None
    case composed of
      Some parts -> compose_chain parts.0 parts.1 parts.2 out
      None -> .push bare out
  ()
''')

write('emit/module', '''
export(crate) emit_module : &mut Sink<'_> -> &ModuleDecl -> usize -> ()
emit_module out m indent =
  .unmark out
  emit_trivia out (&m.leading) indent
  .mark m.span out
  pad out indent
  vis = .rust_text m.visibility
  .push_str (&format! \\-> "#{vis}mod #{m.name.text}") out
  guard &m.items is Some items else
    .push_str ";\\n" out
    return ()
  .push_str " {\\n" out
  emit_items out items (indent + RUST_INDENT)
  .unmark out
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/try_block', '''
export(crate) try_block : &Block -> usize -> String
try_block body indent =
  -- A nested `Sink`, for the reason `emit_block` records: this returns text, and
  -- `emit_stmts` writes into a sink. So a `try` block's statements are
  -- attributed to the binding that holds it rather than individually — the
  -- same coarseness, closed by the same change `emit_block`'s comment names.
  mut inner = Sink::new ""
  emit_stmts (&mut inner) (&body.stmts) (indent + RUST_INDENT) true
  -- `finished.0` is the text and `finished.1` the map, which this discards.
  finished = .finish "" inner
  mut out = .to_string "(|| {\\n"
  .push_str (&finished.0) out
  pad_to (&mut out) indent
  .push_str "})()" out
  out
''')

write('emit/arm', '''
export(crate) emit_arm : &mut Sink<'_> -> &Clause -> &str -> usize -> bool -> ()
emit_arm out clause arm indent tail =
  -- A clause with `where` locals is never collapsed to `pat => expr,`: the
  -- locals need statements, and an arm with statements needs its braces.
  collapsed = .filter (\\_ -> .is_empty clause.wheres) (one_tail_expression clause tail)
  if collapsed is Some stmt
    .mark stmt.span out
    pad out indent
    .push_str arm out
    .push_str " => " out
    .push_str (&emit_expr (unparen (&stmt.expr)) indent) out
    .push ',' out
    emit_inline_comment out (.as_ref stmt.trailing)
    .push '\\n' out
    return ()
  pad out indent
  .push_str arm out
  .push_str " => {\\n" out
  emit_where_locals out (&clause.wheres) (indent + RUST_INDENT)
  -- Every arm is a tail position: a `match` arm's value is the arm's value,
  -- whether or not the function returns anything.
  -- A `match` arm's body, where rustfmt expands a two-armed `if`/`else` that
  -- it would have joined in a function body.
  emit_body out (&clause.body) (indent + RUST_INDENT) tail false
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/catch_fallback', '''
export(crate) catch_fallback : &Expr -> &Expr -> usize -> String
catch_fallback subject fallback indent =
  arm_pad = .repeat (indent + RUST_INDENT) " "
  close_pad = .repeat indent " "
  -- `parts.0` is the binding and `parts.1` the value, where the Rust
  -- destructured the pair.
  parts = case unparen fallback of
    Expr::Lambda { params, body, .. } | .len params == 1 ->
      (.clone params[0].name.text, emit_expr (unparen body) (indent + RUST_INDENT))
    other -> (.to_string "_", emit_expr other (indent + RUST_INDENT))
  binding = parts.0
  value = parts.1
  scrutinee = emit_expr (unparen subject) indent
  format! \\-> "match #{scrutinee} {\\n#{arm_pad}Ok(__rdx_v) => __rdx_v,\\n#{arm_pad}Err(#{binding}) => #{value},\\n#{close_pad}}"
''')

write('emit/one_line_if', '''
export(crate) one_line_if : &Condition -> &Block -> Option<&Block> -> usize -> Option<String>
-- `else_block`, where the Rust named the parameter `otherwise`: that is doc
-- 03 §7's keyword for a guard ladder's last row.
one_line_if condition then_block else_block indent =
  other = else_block?
  then_expr = (.sole_expression then_block)?
  else_expr = (.sole_expression other)?
  if .is_statement_form then_expr || .is_statement_form else_expr
    return None
  head = emit_condition condition indent
  a = emit_expr (unparen then_expr) indent
  b = emit_expr (unparen else_expr) indent
  text = format! \\-> "if #{head} { #{a} } else { #{b} }"
  if .len text > SINGLE_LINE_IF_ELSE_MAX_WIDTH || .contains '\\n' text
    return None
  Some text
''')

write('emit/error_message', '''
export(crate) error_message : &str -> (String, Vec<String>)
error_message raw =
  text = .trim_matches '"' raw
  -- `template`, where the Rust named the local `format`: the name of the
  -- macro one line down is not a good name for a `String`.
  mut template = String::with_capacity (.len text)
  mut args = Vec::new ()
  chars : Vec<char> = .collect (.chars text)
  mut i = 0
  while i < .len chars
    if chars[i] == '#' && .get (i + 1) chars == Some (&'{')
      mut j = i + 2
      mut digits = String::new ()
      while j < .len chars && .is_ascii_digit chars[j]
        .push chars[j] digits
        j += 1
      if !.is_empty digits && .get j chars == Some (&'}')
        .push_str "{}" template
        .push (format! \\-> "_#{digits}") args
        i := j + 1
        continue
    -- A brace the author wrote prints as a brace, not as a hole.
    if chars[i] == '{' || chars[i] == '}'
      .push chars[i] template
    .push chars[i] template
    i += 1
  (template, args)
''')

write('emit/extern_block', '''
export(crate) emit_extern_block : &mut Sink<'_> -> &ExternBlock -> usize -> ()
emit_extern_block out e indent =
  .mark e.span out
  emit_trivia out (&e.leading) indent
  pad out indent
  .push_str (&format! \\-> "unsafe extern #{e.abi} {\\n") out
  -- `pad` counts spaces, not levels — a declaration sits one four-space step
  -- inside the block.
  inner = indent + 4
  for decl in &e.decls
    .mark decl.span out
    emit_trivia out (&decl.leading) inner
    pad out inner
    sig = signature (&decl.sig) inner
    .push_str (&format! \\-> "fn #{decl.name.text}#{sig};\\n") out
  .unmark out
  pad out indent
  .push_str "}\\n" out
  ()
''')

print('plan 81 batch 6: eight files')
