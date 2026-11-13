# -*- coding: utf-8 -*-
"""Plan 81 batch 11: four more of `voltaic-emit`'s bodies.

Three shapes meet their limits here:

* **`vec![false; n]`** — the repeat form of `vec!`. A Voltaic macro argument is
  an expression and `[x; n]` is `Expr::ArrayRepeat`, an *array*; `Vec::new`
  plus `.resize` is the same thing with a spelling.
* **A tuple pattern as a closure parameter** — `.filter(|(_, e)| …)`. A lambda
  parameter is a bare name (doc 69 §10.4), so the pair is projected.
* **A `let` inside a match guard** — `Expr::Call { … } if let
  Expr::MethodRef { … } = unparen(callee)`. The two `Call` arms fold into one
  with an inner `case`, which is where the binding can live.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/record', '''
export(crate) emit_record : &Expr -> &[FieldInit] -> usize -> String
emit_record head fields indent =
  inner_pad = .repeat (indent + RUST_INDENT) " "
  close_pad = .repeat indent " "
  if !names_a_constructor head
    -- `p { x = 99 }` — the update lowering. Multi-line, because rustfmt
    -- expands a block of statements unconditionally.
    base = emit_expr (unparen head) (indent + RUST_INDENT)
    mut updated = format! \\-> "{\\n#{inner_pad}let mut __rdx_r = #{base};\\n"
    for f in fields
      value = field_value f (indent + RUST_INDENT)
      .push_str (&format! \\-> "#{inner_pad}__rdx_r.#{f.name.text} = #{value};\\n") updated
    .push_str (&format! \\-> "#{inner_pad}__rdx_r\\n#{close_pad}}") updated
    return updated
  head_text = emit_expr (unparen head) indent
  if .is_empty fields
    return head_text
  texts : Vec<String> = fields |> .iter |> .map (\\f -> one_field f indent) |> .collect
  joined = .join ", " texts
  if .len joined <= STRUCT_LIT_WIDTH && !.contains '\\n' joined
    return format! \\-> "#{head_text} { #{joined} }"
  mut text = format! \\-> "#{head_text} {\\n"
  for f in fields
    one = one_field f (indent + RUST_INDENT)
    .push_str (&format! \\-> "#{inner_pad}#{one},\\n") text
  .push_str (&format! \\-> "#{close_pad}}") text
  text
''')

write('emit/guards', '''
export(crate) emit_guards : &mut Sink<'_> -> &[Guard] -> usize -> bool -> bool -> ()
emit_guards out guards indent tail joinable =
  -- A two-armed ladder narrow enough goes on one line, because that is what
  -- rustfmt does to it — see `SINGLE_LINE_IF_ELSE_MAX_WIDTH`. An `else if`
  -- in the middle disowns the rule, which is why doc 18 §2.3's three-row
  -- `tier` example was unaffected and a two-row ladder found it.
  --
  -- `joinable` is the second half of the rule, and it was found the same
  -- way: rustfmt joins an `if`/`else` that is a **function body's** tail and
  -- expands the identical expression inside a `match` arm. Only the caller
  -- knows which position this is, so only the caller can say.
  if joinable && two_armed guards indent is Some one_line
    pad out indent
    .push_str (&one_line) out
    .push '\\n' out
    return ()
  -- `i` and `row` bound from the pair: a `for` binder is a bare name. `row`
  -- rather than `guard`, which is doc 03 §7.1's keyword.
  for pair in .enumerate (.iter guards)
    i = pair.0
    row = pair.1
    case (&row.condition, i) of
      (Some cond, 0) ->
        pad out indent
        head = emit_expr cond indent
        .push_str (&format! \\-> "if #{head} {\\n") out
      (Some cond, _) ->
        pad out indent
        head = emit_expr cond indent
        .push_str (&format! \\-> "} else if #{head} {\\n") out
      -- `otherwise` in first position is a ladder of one arm, which is not a
      -- conditional at all — emit the body with no wrapper.
      (None, 0) ->
        emit_stmts out (&row.body.stmts) indent tail
        return ()
      (None, _) ->
        pad out indent
        .push_str "} else {\\n" out
    -- Every arm is a tail position, for the same reason a `match` arm is.
    emit_stmts out (&row.body.stmts) (indent + RUST_INDENT) tail
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/where_order', '''
export(crate) where_order : &[Stmt] -> Vec<usize>
where_order entries =
  n = .len entries
  -- `pair.0` and `pair.1`, where the Rust's closure destructured `|(_, e)|`:
  -- a lambda parameter is a bare name.
  bindings : Vec<usize> = entries
    |> .iter
    |> .enumerate
    |> .filter (\\pair ->
    case pair.1.expr of
      Expr::Binding _ -> true
      _ -> false)
    |> .map (\\pair -> pair.0)
    |> .collect
  if .len bindings < 2
    return .collect (0 .. n)
  graph = where_edges entries
  -- `Vec::new` plus `.resize`, where the Rust wrote `vec![false; n]`: the
  -- repeat form of `vec!` takes a length after a `;`, and a Voltaic macro
  -- argument is an expression.
  mut done : Vec<bool> = Vec::new ()
  .resize n false done
  mut visiting : Vec<bool> = Vec::new ()
  .resize n false visiting
  mut sorted = Vec::with_capacity (.len bindings)
  -- `i` is the reference, where the Rust wrote `for &i in …`.
  for i in &bindings
    visit (*i) (&graph) (&mut done) (&mut visiting) (&mut sorted)
  -- Each non-binding entry keeps its slot; the bindings fill the rest in
  -- sorted order.
  --
  -- `queued`, not a rebinding of `sorted`: `RDX1003` reports a shadow.
  mut queued = .into_iter sorted
  (0 .. n)
    |> .map (\\i ->
    case entries[i].expr of
      Expr::Binding _ -> .unwrap_or i (.next queued)
      _ -> i)
    |> .collect
''')

write('emit/pipeline', '''
export(crate) pipeline : &Expr -> &Expr -> usize -> String
pipeline lhs rhs indent = case unparen rhs of
  -- `a |> .name args` and `a |> .name` — plan 57's method-as-function,
  -- matched **before** the two arms below. §11.1 is the trap: the bare-name
  -- arm appends the piped value as an argument, so a zero-argument
  -- `x |> .name` falling through to it would lower to a call of a closure.
  --
  -- `receiver` rather than `emit_expr` for the left operand here, and only
  -- here: `1 .. 3 |> .count` is `(1..3).count()`, not `1..3.count()`, which
  -- is a question a method call asks and a function call does not.
  Expr::MethodRef { name, .. } -> format! \\-> "#{receiver lhs indent}.#{name.text}()"
  -- One `Call` arm with an inner `case`, where the Rust wrote two and told
  -- them apart with `if let` **inside the guard**: a guard asks a question
  -- and does not bind.
  Expr::Call { callee, args, .. } -> case unparen callee of
    Expr::MethodRef { name, .. } ->
      recv = receiver lhs indent
      listed = expr_list args indent
      format! \\-> "#{recv}.#{name.text}(#{listed})"
    -- `a |> f x y` — the piped value joins the argument list, last.
    other ->
      mut supplied : Vec<String> = args |> .iter |> .map (\\a -> emit_expr a indent) |> .collect
      .push (emit_expr lhs indent) supplied
      joined = .join ", " supplied
      format! \\-> "#{callable other indent}(#{joined})"
  -- `a |> f` — reverse application proper, and the case the row exists for.
  other -> format! \\-> "#{callable other indent}(#{emit_expr lhs indent})"
''')

print('plan 81 batch 11: four files')
