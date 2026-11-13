# -*- coding: utf-8 -*-
"""Plan 81 batch 18: `emit` and `emit_control`."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit', '''
export emit : &Program -> String
emit program = (emit_with_map program "" "").0
''')

write('emit/emit_control', '''
export(crate) emit_control : &mut Sink<'_> -> &Expr -> usize -> bool -> ()
emit_control out expr indent tail =
  case expr of
    Expr::If { condition, then_block, r#otherwise, .. } ->
      -- A **tail** `if … else …` is Rust's if-else *expression*, and rustfmt
      -- collapses one whose rendering fits `single_line_if_else_max_width`.
      -- `control_text` has asked that since leaf 2 for an `if` in an
      -- expression position; plan 55 leaf 7 made it a question here too,
      -- because retiring `then` moved
      -- `tier n = if n > 9 then "big" else "small"` out of the expression
      -- path and into tail position. `generated_rust_is_rustfmt_clean` is
      -- what caught it, on the one example that migrated.
      --
      -- `one_line_if` declines on an `else`-less `if`, so `tail` alone is the
      -- whole condition: a non-tail `if` is a statement in Rust too and
      -- rustfmt leaves it spread.
      if tail && one_line_if condition then_block (.as_ref r#otherwise) indent is Some text
        pad out indent
        .push_str (&text) out
        .push '\\n' out
        return ()
      pad out indent
      .push_str (&condition_head "if" condition indent) out
      -- `tail && otherwise.is_some()`: see the note above — an `if` with no
      -- `else` is `()`, so its branch is never the function's value.
      emit_stmts out (&then_block.stmts) (indent + RUST_INDENT) (tail && .is_some r#otherwise)
      guard r#otherwise is Some other else
        .unmark out
        pad out indent
        .push_str "}\\n" out
        return ()
      -- `else if` is an `else` whose block holds one `If`, which is how the
      -- parser builds it — so the chain writes itself and needs no third
      -- node. Recognising it here is what keeps `} else if c {` on one line
      -- rather than opening a nested block rustfmt would flatten.
      .unmark out
      pad out indent
      chained = case .as_slice other.stmts of
        [only] -> case only.expr of
          Expr::If { .. } -> Some only
          _ -> None
        _ -> None
      if chained is Some only
        -- The nested `if` writes its own `if … {`, so its leading pad is
        -- already on this line.
        .push_str "} else " out
        emit_control_inline out (&only.expr) indent tail
        return ()
      .push_str "} else {\\n" out
      emit_stmts out (&other.stmts) (indent + RUST_INDENT) tail
      .unmark out
      pad out indent
      .push_str "}\\n" out
    Expr::While { condition, negated, body, label, .. } ->
      pad out indent
      -- A pattern test can never be negated here — the parser rejects
      -- `until P <- e` outright (plan 22 §1.3) — so `until` keeps the simple
      -- text wrap, and only `while` can need rustfmt's chain breaking.
      if *negated
        text = emit_condition condition indent
        prefix = label_prefix label
        .push_str (&format! \\-> "#{prefix}while !(#{text}) {\\n") out
      else
        .push_str (&label_prefix label) out
        .push_str (&condition_head "while" condition indent) out
      emit_stmts out (&body.stmts) (indent + RUST_INDENT) false
      .unmark out
      pad out indent
      .push_str "}\\n" out
    Expr::For { binder, source, body, label, .. } ->
      pad out indent
      prefix = label_prefix label
      over = emit_expr (unparen source) indent
      .push_str (&format! \\-> "#{prefix}for #{binder.text} in #{over} {\\n") out
      emit_stmts out (&body.stmts) (indent + RUST_INDENT) false
      .unmark out
      pad out indent
      .push_str "}\\n" out
    Expr::Loop { body, label, .. } ->
      pad out indent
      prefix = label_prefix label
      .push_str (&format! \\-> "#{prefix}loop {\\n") out
      emit_stmts out (&body.stmts) (indent + RUST_INDENT) false
      .unmark out
      pad out indent
      .push_str "}\\n" out
    -- Doc 03 §7.1's `guard`, lowering to let-else — plan 22. Its else body is
    -- a statement block like any other, so a `return`, a `break` or a panic
    -- form all reach it, and rustc is what insists one of them diverges.
    Expr::Guard { pattern, source, r#otherwise, .. } ->
      pad out indent
      lowered = .rust_text pattern
      value = emit_expr (unparen source) indent
      .push_str (&format! \\-> "let #{lowered} = #{value} else {\\n") out
      emit_stmts out (&r#otherwise.stmts) (indent + RUST_INDENT) false
      .unmark out
      pad out indent
      -- The `;` is **required**: let-else is a `let` statement, and omitting
      -- it is a parse error rustc reports against this line.
      .push_str "};\\n" out
    Expr::Break { label, value, .. } ->
      pad out indent
      .push_str "break" out
      if label is Some name
        .push_str (&format! \\-> " '#{name}") out
      if value is Some v
        .push_str (&format! \\-> " #{emit_expr (unparen v) indent}") out
      .push_str ";\\n" out
    Expr::Continue { label, .. } ->
      pad out indent
      .push_str "continue" out
      if label is Some name
        .push_str (&format! \\-> " '#{name}") out
      .push_str ";\\n" out
    -- Not reachable, and a hard error rather than a silent fallthrough: the
    -- callers that route here ask `Expr::is_control_flow` first, and one node
    -- kind means the compiler can no longer check that for them.
    _ ->
      unreachable! \\-> "`emit_control` is called only for a control-flow form"
  ()
''')

print('plan 81 batch 18: two files')
