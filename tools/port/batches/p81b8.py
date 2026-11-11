# -*- coding: utf-8 -*-
"""Plan 81 batch 8: five more of `redox-emit`'s bodies.

`nest` destructures `((name, source), rest)` in one pattern — a pair inside a
pair — which is the deepest tuple binding the port has met. Bound once and
projected twice.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/wraps_as_receiver', '''
export(crate) wraps_as_receiver : &Expr -> bool
wraps_as_receiver e =
  -- A pipeline is the exception among binary expressions: its *lowering* is
  -- already a method call, so it is postfix-able and parenthesising it turns
  -- `numbers.iter().map(f).sum()` into `((numbers.iter()).map(f)).sum()`.
  -- Caught by `a_chain_nests_left_to_right`, which is what that test is for.
  --
  -- A guard rather than a field pattern: a record pattern's field may be
  -- matched against a *single-segment* path, and `redox_ast::BinaryOp::Pipeline`
  -- is three.
  is_pipeline = case e of
    Expr::Binary { op, .. } | *op == redox_ast::BinaryOp::Pipeline -> true
    _ -> false
  if is_pipeline
    return false
  -- `?` is itself postfix, so a receiver that is one needs no parentheses:
  -- `f()?.len()` chains. `.await` was here too until doc 30 deleted it — an
  -- await is a statement now, so it never appears as a receiver.
  if e is Expr::Try { .. }
    return false
  -- Plan 56. An **under-applied** call is a closure once emitted, and a
  -- closure needs its parentheses in front of a `.` or a `(` exactly as a
  -- written one does. The variant alone cannot say so — `add 1 2` is the same
  -- node and emits a call — so this is the one entry in the list that has to
  -- consult the arity table. §11.5 is the defect it closes, on the side a
  -- synthesized closure reaches: `wraps_as_receiver` already listed
  -- `Expr::Lambda` and `Expr::Section`, and nothing listed the node that
  -- *becomes* one.
  if e is Expr::Call { callee, args, .. }
    return .is_some (missing_arity callee args)
  -- `Expr::MethodRef` is plan 57's entry: a method applied to nothing is a
  -- closure awaiting its receiver, so it needs its parentheses in front of a
  -- `.` or a `(` exactly as a section does — the same entry `Section` has,
  -- for the same reason, on the node that replaced `(|> name)`.
  case e of
    Expr::Binary { .. } or Expr::Unary { .. } or Expr::Ascribe { .. } or Expr::Lambda { .. } or Expr::Case { .. } or Expr::Record { .. } or Expr::Section { .. } or Expr::MethodRef { .. } or Expr::Comprehension { .. } -> true
    _ -> false
''')

write('emit/condition_head', '''
export(crate) condition_head : &str -> &Condition -> usize -> String
condition_head keyword condition indent =
  if !breaks condition
    joined = emit_condition condition indent
    return format! \\-> "#{keyword} #{joined} {\\n"
  inner = .repeat (indent + RUST_INDENT) " "
  mut out = format! \\-> "#{keyword} "
  -- `i` and `operand` bound from the pair: a `for` binder is a bare name.
  for pair in .enumerate (.iter condition.operands)
    i = pair.0
    operand = pair.1
    if i > 0
      .push_str (&format! \\-> "\\n#{inner}&& ") out
    one = emit_condition (&Condition { operands = vec! \\-> (.clone operand) }) indent
    .push_str (&one) out
  outer = .repeat indent " "
  .push_str (&format! \\-> "\\n#{outer}{\\n") out
  out
''')

write('emit/nest', '''
export(crate) nest : &[(&str, &Expr)] -> &[String] -> &str -> usize -> bool -> (String, Vec<String>)
nest draws guards item base nested =
  -- `head.0` is the first pair and `head.1` the rest; `first.0` and `first.1`
  -- are the name and source inside it. The Rust destructured all four in one
  -- pattern — `((name, source), rest)` — and Redox has no such binding.
  head = .expect "the caller checked that there is at least one generator" (.split_first draws)
  first = head.0
  rest = head.1
  name = first.0
  source = first.1
  -- `move` is needed exactly when an inner closure captures an outer
  -- generator's binder, which is exactly when there is more than one.
  mv = if nested
    "move "
  else
    ""
  if .is_empty rest
    last = if .is_empty guards
      format! \\-> ".map(#{mv}|#{name}| #{item})"
    else
      joined = .join " && " guards
      format! \\-> ".filter_map(#{mv}|#{name}| if #{joined} { Some(#{item}) } else { None })"
    return (receiver source base, vec! \\-> (.to_string ".into_iter()") \\-> last)
  inner_base = base + RUST_INDENT * 2
  -- `built.0` is the head and `built.1` the segments.
  built = nest rest guards item inner_base nested
  inner = chain (&built.0) (&built.1) inner_base
  -- rustfmt braces a closure body that spans lines, and leaves a one-line
  -- body bare.
  arg = if .contains '\\n' inner
    mut text = format! \\-> "move |#{name}| {\\n"
    pad_to (&mut text) inner_base
    .push_str (&inner) text
    .push '\\n' text
    pad_to (&mut text) (base + RUST_INDENT)
    .push '}' text
    text
  else
    format! \\-> "move |#{name}| #{inner}"
  flat = format! \\-> ".flat_map(#{arg})"
  (receiver source base, vec! \\-> (.to_string ".into_iter()") \\-> flat)
''')

write('emit/unparen', '''
export(crate) unparen : &Expr -> &Expr
unparen expr = case expr of
  Expr::Paren { inner, .. } -> unparen inner
  other -> other

-- | `#[cfg(test)]`: an identifier expression, for the tests below.
#[cfg(test)]
ident : &str -> Expr
-- `redox_ast::Ident` qualified, not imported: it is used only by this
-- `#[cfg(test)]` helper, and an unqualified import would be
-- `unused_imports` in a build without the tests.
ident text = Expr::Ident (redox_ast::Ident { text = .to_string text, span = (0, 0) })

-- | `#[cfg(test)]`: one layer of parentheses around `inner`.
#[cfg(test)]
paren : Expr -> Expr
paren inner = Expr::Paren { inner = Box::new inner, span = (0, 0) }

test "strips one layer"
  assert_eq! \\-> unparen (&paren (ident "x")) \\-> &ident "x"

-- | `((x))` is as redundant as `(x)`.
test "strips every layer"
  assert_eq! \\-> unparen (&paren (paren (paren (ident "x")))) \\-> &ident "x"

test "leaves an unparenthesised expression alone"
  assert_eq! \\-> unparen (&ident "x") \\-> &ident "x"
''')

write('emit/items', '''
export(crate) emit_items : &mut Sink<'_> -> &[Item] -> usize -> ()
emit_items out items indent =
  tests : Vec<&TestDecl> = items
    |> .iter
    |> .filter_map (\\item ->
    case item of
      Item::Test t -> Some t
      _ -> None)
    |> .collect
  mut written : usize = 0
  for item in items
    -- The `matches!` as a `case`, because a macro's second argument cannot be
    -- a pattern.
    is_test = case item of
      Item::Test _ -> true
      _ -> false
    if is_test
      continue
    if written > 0
      -- The separator belongs to no construct, so it is a synthetic line.
      .unmark out
      .push '\\n' out
    written += 1
    emit_item out item indent
  if !.is_empty tests
    if written > 0
      .unmark out
      .push '\\n' out
    emit_tests_module out (&tests) indent
  ()
''')

print('plan 81 batch 8: five files')
