# -*- coding: utf-8 -*-
"""Plan 81 batch 1: sixteen of `redox-emit`'s 106 owed bodies — the small ones.

Five are `const`s and the rest are a few lines each. `emit_inline_comment` is
the first ported body with a **let-chain**: `if let Some(t) = trivia && t.kind
== …`. `is` combines with `&&` (plan 68 wrote one in `where_cycle.sdx`), so it
carries over as written.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/rust_indent', '''
export(crate) const RUST_INDENT : usize = 4
''')

write('emit/struct_lit_width', '''
export(crate) const STRUCT_LIT_WIDTH : usize = 18
''')

write('emit/single_line_if_else_max_width', '''
export(crate) const SINGLE_LINE_IF_ELSE_MAX_WIDTH : usize = 50
''')

write('emit/chain_width', '''
export(crate) const CHAIN_WIDTH : usize = 60
''')

write('emit/fn_call_width', '''
export(crate) const FN_CALL_WIDTH : usize = 60
''')

write('emit/pad', '''
export(crate) pad : &mut Sink<'_> -> usize -> ()
pad out n =
  for _ in 0 .. n
    .push ' ' out
  ()
''')

write('emit/pad_to', '''
export(crate) pad_to : &mut String -> usize -> ()
pad_to out n =
  for _ in 0 .. n
    .push ' ' out
  ()
''')

write('emit/label_prefix', '''
export(crate) label_prefix : &Option<String> -> String
label_prefix label = case label of
  Some name -> format! \\-> "'#{name}: "
  None -> String::new ()
''')

write('emit/closure_param', '''
export(crate) closure_param : usize -> String
closure_param index = format! \\-> "__rdx_#{index + 1}"
''')

write('emit/binding_table/is_upper_case', '''
export(crate) is_upper_case : &str -> bool
is_upper_case name =
  .any char::is_alphabetic (.chars name) && !.any (\\c -> .is_ascii_lowercase c) (.chars name)
''')

write('emit/condition_head/breaks', '''
export(super) breaks : &Condition -> bool
breaks condition =
  n = .len condition.operands
  has_let = .has_pattern condition
  -- A `case` where the Rust negated nothing and `matches!` was the shape: the
  -- macro's second argument is a pattern, so a `case` says it.
  first_is_let = case .first condition.operands of
    Some (ConditionOperand::Pattern { .. }) -> true
    _ -> false
  has_let && (n > 2 || (n == 2 && first_is_let))
''')

write('emit/sole_expression', '''
export(crate) sole_expression : &Guard -> usize -> Option<String>
-- `row`, where the Rust named the parameter `guard`: `guard` is doc 03 §7.1's
-- keyword, and a parameter cannot be one.
sole_expression row indent =
  .map (\\e -> emit_expr (unparen e) indent) (.sole_expression row.body)
''')

write('emit/trivia/line', '''
export(crate) line : &mut Sink<'_> -> usize -> &str -> &str -> ()
line out indent marker text =
  pad out indent
  if .is_empty text
    .push_str marker out
    .push '\\n' out
  else
    _dropped = writeln! \\-> out \\-> "#{marker} #{text}"
  ()
''')

write('emit/inline_comment', '''
export(crate) emit_inline_comment : &mut Sink<'_> -> Option<&Trivia> -> ()
emit_inline_comment out trivia =
  if trivia is Some t && t.kind == TriviaKind::LineComment
    if .is_empty t.text
      .push_str " //" out
    else
      _dropped = write! \\-> out \\-> " // #{t.text}"
  ()
''')

write('emit/one_field', '''
export(crate) one_field : &FieldInit -> usize -> String
one_field f indent = case &f.value of
  Some _ -> format! \\-> "#{f.name.text}: #{field_value f indent}"
  None -> .clone f.name.text
''')

write('emit/pipeline/callable', '''
export(crate) callable : &Expr -> usize -> String
callable callee indent = case callee of
  Expr::Ident _ or Expr::Path { .. } -> emit_expr callee indent
  _ -> format! \\-> "(#{emit_expr callee indent})"
''')

print('plan 81 batch 1: sixteen files')
