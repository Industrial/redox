# -*- coding: utf-8 -*-
"""Plan 81 batch 9: four more of `voltaic-emit`'s bodies.

`control_text` is the first ported body with a **let-chain whose first link is
a record pattern**: `if let Expr::If { … } = expr && let Some(text) = …`. Both
links carry over as `is`, which is what plan 68 wrote in `where_cycle.sdx` for
one link and plan 81 batch 1 wrote for a `Some`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/emit_fn', '''
export(crate) emit_fn : &mut Sink<'_> -> &FnDecl -> usize -> ()
emit_fn out f indent =
  -- Leading trivia is the author's comment lines, which belong to no
  -- construct's span; attributing them to the declaration would report a
  -- comment's line for an error in the code below it.
  .unmark out
  emit_trivia out (&f.leading) indent
  .mark f.span out
  pad out indent
  .push_str (&.rust_text f.visibility) out
  if f.sig.is_const
    .push_str "const " out
  -- Doc 03 §14.3's `async fn` — plan 39. Both markers are written if both are
  -- present; Rust has no `const async fn` and rustc is what says so.
  if f.sig.is_async
    .push_str "async " out
  -- Plan 45 leaf 2. The Rust reference's `FunctionQualifiers` are
  -- `const? async? ItemSafety? (extern Abi?)?` — so `unsafe` comes after
  -- `async`, which is the order these three pushes are in. Emitting them in
  -- Rust's own order means two markers together produce valid Rust instead of
  -- a permutation rustc would reject for a reason the author did not write.
  if f.sig.is_unsafe
    .push_str "unsafe " out
  .push_str "fn " out
  .push_str (&f.name.text) out
  sig = signature (&f.sig) indent
  .push_str (&sig) out
  -- A `where` clause ends a line, so the body's brace goes on the next one —
  -- which is what rustfmt does and the only form it leaves alone.
  if .contains '\\n' sig
    .push '\\n' out
    pad out indent
    .push '{' out
  else
    .push_str " {" out
  emit_inline_comment out (.as_ref f.trailing)
  .push '\\n' out
  -- A function that declares a return type ends in a tail expression; one
  -- that does not returns `()` and ends in a statement. See `emit_stmt`.
  --
  -- `emit_clauses` rather than the body directly, because doc 18 §2.3 makes a
  -- function one *or more* equations: several clauses, or one refutable
  -- pattern, lower to a `match` over the parameters.
  emit_clauses out f indent
  -- The closing brace has no source line — the off-side rule means the author
  -- never wrote one. Recording it as synthetic rather than attributing it to
  -- the last statement keeps the map from making a claim it cannot support.
  .unmark out
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/emit_stmts', '''
export(crate) emit_stmts : &mut Sink<'_> -> &[Stmt] -> usize -> bool -> ()
emit_stmts out stmts indent tail =
  -- `split.0` is the first statement and `split.1` the rest, where the Rust
  -- destructured the pair.
  guard .split_first stmts is Some split else
    return ()
  first = split.0
  rest = split.1
  guard &first.expr is Expr::Draw { name, value, .. } else
    last = .is_empty rest
    emit_stmt out first indent (tail && last)
    emit_stmts out rest indent tail
    return ()
  .mark first.span out
  emit_trivia out (&first.leading) indent
  pad out indent
  .push_str (&emit_expr value indent) out
  .push_str "(|" out
  .push_str (&name.text) out
  .push_str "| {\\n" out
  emit_stmts out rest (indent + crate::emit::rust_indent::RUST_INDENT) tail
  pad out indent
  .push_str "})" out
  -- The closing `;` is the draw-arrow statement's own, and is omitted when the
  -- callback's value is the function's value.
  if !tail
    .push ';' out
  .push '\\n' out
  ()
''')

write('emit/control_text', '''
export(crate) control_text : &Expr -> usize -> String
control_text expr indent =
  -- rustfmt collapses a narrow two-armed if-else, so ask before writing the
  -- multi-line form — `one_line_if` is the same decision `two_armed` makes for
  -- a guard ladder.
  if expr is Expr::If { condition, then_block, r#otherwise, .. } && one_line_if condition then_block (.as_ref r#otherwise) indent is Some text
    return text
  mut inner = Sink::new ""
  emit_control (&mut inner) expr indent true
  -- `finished.0` is the text and `finished.1` the map, which this discards.
  finished = .finish "" inner
  .to_string (.trim_end (.trim_start finished.0))
''')

write('emit/member', '''
export(crate) emit_member : &mut Sink<'_> -> &ClassMember -> usize -> ()
emit_member out m indent =
  case m of
    ClassMember::AssocType name ->
      .mark name.span out
      pad out indent
      .push_str (&format! \\-> "type #{name.text};\\n") out
    ClassMember::AssocConst { name, ty, value } ->
      .mark name.span out
      pad out indent
      .push_str (&format! \\-> "const #{name.text}: #{ty.text}") out
      if value is Some v
        .push_str (&format! \\-> " = #{emit_expr v indent}") out
      .push_str ";\\n" out
    ClassMember::Method f ->
      .unmark out
      emit_trivia out (&f.leading) indent
      .mark f.span out
      pad out indent
      sig = signature (&f.sig) indent
      vis = .rust_text f.visibility
      const_marker = if f.sig.is_const
        "const "
      else
        ""
      async_marker = if f.sig.is_async
        "async "
      else
        ""
      .push_str (&format! \\-> "#{vis}#{const_marker}#{async_marker}fn #{f.name.text}#{sig}") out
      if .is_empty f.clauses
        .push_str ";\\n" out
        return ()
      if .contains '\\n' sig
        .push '\\n' out
        pad out indent
      else
        .push ' ' out
      .push_str "{\\n" out
      emit_clauses out f indent
      .unmark out
      pad out indent
      .push_str "}\\n" out
  ()
''')

print('plan 81 batch 9: four files')
