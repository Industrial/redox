# -*- coding: utf-8 -*-
"""Plan 81 batch 14: `emit_case` and `emit_section`.

`emit_section`'s Rust matches a **three-element tuple** out of two guarded arms
and then destructures it in an `if let` — `Some((op, hole_on_the_left,
operand))`. Bound once and projected three times, which is the same treatment
every pair in this crate got and the first triple.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/case', '''
export(crate) emit_case : &Expr -> &[CaseArm] -> usize -> String
emit_case scrutinee arms indent =
  arm_pad = .repeat (indent + RUST_INDENT) " "
  close_pad = .repeat indent " "
  head = emit_expr (unparen scrutinee) indent
  mut out = format! \\-> "match #{head} {\\n"
  for arm in arms
    .push_str (&arm_pad) out
    .push_str (&.rust_text arm.pattern) out
    if &arm.r#guard is Some row
      .push_str " if " out
      -- A pattern test in an arm guard emits `if let` inside the match
      -- guard — `if_let_guard`, stable as of rustc 1.97.1 and verified before
      -- plan 22 was written.
      .push_str (&emit_condition row (indent + RUST_INDENT)) out
    .push_str " => " out
    -- Plan 55 leaf 3: an arm body written as an indented block is a block
    -- expression, and a block of one plain expression collapses to
    -- `pat => expr,`. rustfmt *flattens* a one-expression block body, so
    -- emitting the braces would be output rustfmt rewrites — which is
    -- `emit_arm`'s reason for making the same choice for a clause.
    --
    -- A comment on that one line disqualifies it: a collapsed arm has no line
    -- to put one on, and a comment must reach the `.xrs` (doc 02 §10.1).
    -- `sole_expression` answers the rest, and leaves the trivia test to the
    -- caller for exactly this reason.
    body = case &arm.body of
      Expr::Block { label = None, body = block, .. } | .is_some_and (\\stmt -> .is_empty stmt.leading && .is_none stmt.trailing) (.first block.stmts) ->
        .unwrap_or (&arm.body) (.sole_expression block)
      other -> other
    -- Plan 62 leaf 3. An ascription on the pattern becomes a `let` at the top
    -- of the arm body, because rustc takes no type in a `match` pattern — so
    -- the body must become a block even when it was one expression.
    ascribed = .and_then (\\ty -> ascribed_arm (&arm.pattern) (&ty.text)) (.as_ref arm.ty)
    case ascribed of
      Some binding ->
        inner_pad = .repeat (indent + RUST_INDENT + RUST_INDENT) " "
        .push_str "{\\n" out
        .push_str (&inner_pad) out
        .push_str (&binding) out
        .push '\\n' out
        .push_str (&inner_pad) out
        .push_str (&emit_expr (unparen body) (indent + RUST_INDENT + RUST_INDENT)) out
        .push '\\n' out
        .push_str (&arm_pad) out
        .push '}' out
      None -> .push_str (&emit_expr (unparen body) (indent + RUST_INDENT)) out
    -- rustfmt writes **no** trailing comma after an arm whose body is a plain
    -- block, and one after every other body — `arm_comma`, and it reads the
    -- body's node kind rather than its width. A block body is what leaf 3
    -- added, so the comma became a question with it.
    is_block = case body of
      Expr::Block { .. } -> true
      _ -> false
    if !is_block && .is_none arm.ty
      .push ',' out
    .push '\\n' out
  .push_str (&close_pad) out
  .push '}' out
  out
''')

write('emit/section', '''
export(crate) emit_section : &Section -> usize -> String
emit_section section indent =
  param = closure_param 0
  -- Plan 66's composition, and doc 18 §4's *"a section for free"* is free
  -- only for an operator that has a Rust **spelling**. This is the first one
  -- that does not: the arms below would reach `rust_text`'s `None` and write
  -- `/* unsupported */` into the generated file.
  --
  -- A section of composition is also the one section that is not a function
  -- of a *value*: `(>>> g)` is `\\f -> f >>> g`, so it binds a function and
  -- returns the closure `compose` writes over it.
  --
  -- Two things about the shape were measured rather than chosen.
  --
  -- **The parameter needs a type.** A closure parameter that is itself
  -- *called* is `E0282` with no annotation at all — `|__rdx_1| move |__rdx_2|
  -- g(__rdx_1(__rdx_2))` does not compile, and neither does the flat
  -- two-parameter shape plan 56 uses for an under-applied call. `fn(_) -> _`
  -- is the narrowest annotation that does: the arity is composition's own,
  -- and both halves stay inferred. The cost is disclosed — the operand must
  -- be a function *item*, not a capturing closure.
  --
  -- **The inner closure must `move`.** It outlives the parameter it composes,
  -- so without it rustc answers `E0373` and names the fix.
  --
  -- `composed.0`, `.1` and `.2`, where the Rust destructured the triple in an
  -- `if let`. The guard compares, where the Rust wrote `matches!`: a macro's
  -- second argument cannot be a pattern, and a helper beside this function
  -- would be a second construct in the file (doc 06 §1).
  composed = case section of
    Section::Right { op, operand } | *op == BinaryOp::Compose || *op == BinaryOp::ComposeBack -> Some (*op, true, operand)
    Section::Left { op, operand } | *op == BinaryOp::Compose || *op == BinaryOp::ComposeBack -> Some (*op, false, operand)
    _ -> None
  if composed is Some parts
    op = parts.0
    hole_on_the_left = parts.1
    operand = parts.2
    hole = Expr::Ident (Ident { text = .clone param, span = .span operand })
    -- `sides.0` and `sides.1`, where the Rust destructured the pair.
    sides = if hole_on_the_left
      (&hole, .as_ref operand)
    else
      (.as_ref operand, &hole)
    inner = compose op sides.0 sides.1 1 indent
    typed = format! \\-> "#{param}: fn(_) -> _"
    moved = format! \\-> "move #{inner}"
    return closure (&[typed]) (&moved)
  body = case section of
    Section::Field name -> format! \\-> "#{param}.#{name.text}"
    Section::Right { op, operand } ->
      value = emit_expr (unparen operand) indent
      case .compare_method op of
        Some method -> format! \\-> "#{param}.#{method}(&#{value})"
        None ->
          spelling = .unwrap_or "/* unsupported */" (.rust_text op)
          format! \\-> "#{param} #{spelling} #{value}"
    Section::Left { op, operand } ->
      value = emit_expr (unparen operand) indent
      case .and_then redox_ast::BinaryOp::compare_method (.flipped op) of
        Some method -> format! \\-> "#{param}.#{method}(&#{value})"
        None ->
          spelling = .unwrap_or "/* unsupported */" (.rust_text op)
          format! \\-> "#{value} #{spelling} #{param}"
  closure (&[param]) (&body)
''')

print('plan 81 batch 14: two files')
