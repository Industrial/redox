# -*- coding: utf-8 -*-
"""Plan 81 batch 4: twelve more of `voltaic-emit`'s bodies.

`compose_chain` is the first ported body with an **as-pattern over an
alternation** — `op: inner @ (BinaryOp::Compose | BinaryOp::ComposeBack)` —
which is doc 03 §6.4's `@` and §6.2's `or` meeting in one pattern.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/shared_const', '''
export(crate) shared_const : &FnDecl -> Option<ConstDecl>
shared_const decl =
  ty = (.clone decl.sig.ret)?
  guard .as_slice decl.clauses is [clause] else
    return None
  guard &clause.body is ClauseBody::Plain block else
    return None
  guard .as_slice block.stmts is [stmt] else
    return None
  Some (ConstDecl { name = .clone decl.name, ty, value = .clone stmt.expr, is_static = false, visibility = .clone decl.visibility, span = decl.span, leading = .clone decl.leading })
''')

write('emit/ascribed_list', '''
export(crate) ascribed_list : &Expr -> &TypeExpr -> usize -> Option<String>
ascribed_list inner ty indent =
  guard unparen inner is Expr::List { items, .. } else
    return None
  head = .trim (.unwrap_or (&ty.text) (.next (.split '<' ty.text)))
  listed = expr_list items indent
  Some (format! \\-> "#{head}::from([#{listed}])")
''')

write('emit/binding_table/classify', '''
export(crate) classify : &FnDecl -> Option<BindingKind>
classify decl =
  if !.is_empty decl.sig.params
    return None
  guard .as_slice decl.clauses is [clause] else
    return None
  if !.is_empty clause.patterns
    return None
  guard &clause.body is ClauseBody::Plain block else
    return Some (BindingKind::Called)
  shared = .is_empty clause.wheres && .len block.stmts == 1 && is_shareable (&block.stmts[0].expr) && is_upper_case (&decl.name.text)
  if shared
    return Some (BindingKind::Shared)
  Some (BindingKind::Called)
''')

write('emit/generics', '''
export(crate) generics : &TypeApp -> &[Constraint] -> String
generics target constraints =
  vars = .vars target
  if .is_empty vars
    return String::new ()
  params : Vec<String> = vars
    |> .iter
    |> .map (\\v ->
    bounds : Vec<&str> = constraints
      |> .iter
      |> .filter (\\c -> c.var.text == *v)
      |> .map (\\c -> .as_str c.trait_name.text)
      |> .collect
    name = TypeApp::rust_name v
    if .is_empty bounds
      name
    else
      joined = .join " + " bounds
      format! \\-> "#{name}: #{joined}")
    |> .collect
  joined = .join ", " params
  format! \\-> "<#{joined}>"
''')

write('emit/compose_chain', '''
export(crate) compose_chain : BinaryOp -> &'a Expr -> &'a Expr -> &mut Vec<&'a Expr> -> ()
compose_chain op lhs rhs out =
  -- `first` and `second` bound from the pair, where the Rust destructured.
  sides = if matches_compose_back op
    (rhs, lhs)
  else
    (lhs, rhs)
  for side in [sides.0, sides.1]
    case unparen side of
      Expr::Binary { op = inner @ (BinaryOp::Compose or BinaryOp::ComposeBack), lhs, rhs, .. } ->
        compose_chain (*inner) lhs rhs out
      other -> .push other out
  ()

-- | `op` is `ComposeBack`, which the Rust asked with `matches!`: the macro's
-- | second argument is a pattern, so a `case` is the spelling.
matches_compose_back : BinaryOp -> bool
matches_compose_back op = case op of
  BinaryOp::ComposeBack -> true
  _ -> false
''')

write('emit/method_ref', '''
export(crate) method_ref : &Ident -> &[Expr] -> usize -> String
method_ref name args indent =
  guard .split_last args is Some split else
    param = closure_param 0
    body = format! \\-> "#{param}.#{name.text}()"
    return closure (&[param]) (&body)
  recv = receiver split.0 indent
  rest = expr_list split.1 indent
  format! \\-> "#{recv}.#{name.text}(#{rest})"
''')

write('emit/two_armed', '''
export(crate) two_armed : &[Guard] -> usize -> Option<String>
two_armed guards indent =
  guard guards is [first, second] else
    return None
  condition = (.as_ref first.condition)?
  if .is_some second.condition
    return None
  then_text = (sole_expression first indent)?
  else_text = (sole_expression second indent)?
  head = emit_expr (unparen condition) indent
  text = format! \\-> "if #{head} { #{then_text} } else { #{else_text} }"
  if .len text > SINGLE_LINE_IF_ELSE_MAX_WIDTH || .contains '\\n' text
    return None
  Some text
''')

write('emit/emit_block', '''
export(crate) emit_block : &Option<String> -> &Block -> usize -> String
emit_block label body indent =
  mut inner = Sink::new ""
  emit_stmts (&mut inner) (&body.stmts) (indent + RUST_INDENT) true
  -- `finished.0` is the text and `finished.1` the map, where the Rust
  -- destructured and discarded the second half.
  finished = .finish "" inner
  prefix = label_prefix label
  mut out = format! \\-> "#{prefix}{\\n"
  .push_str (&finished.0) out
  pad_to (&mut out) indent
  .push '}' out
  out
''')

write('emit/emit_bind', '''
export(crate) emit_bind : &Option<Pattern> -> &Expr -> usize -> String
emit_bind pattern value indent =
  postfix = case BindScope::mode () of
    BindMode::Await -> ".await"
    BindMode::Question -> "?"
  recv = receiver (unparen value) indent
  bound = format! \\-> "#{recv}#{postfix}"
  case pattern of
    Some p ->
      lowered = .rust_text p
      format! \\-> "let #{lowered} = #{bound};"
    None -> format! \\-> "#{bound};"
''')

write('emit/macro_call/macro_argument', '''
export(crate) macro_argument : &Expr -> usize -> Vec<String>
macro_argument arg indent = case unparen arg of
  Expr::StrLit { value, .. } -> vec! \\-> (double_braces (&rust_escapes value))
  Expr::StrInterp { parts, .. } ->
    -- `built.0` is the template and `built.1` its arguments, where the Rust
    -- destructured the pair.
    built = format_template parts indent
    mut supplied = vec! \\-> (format! \\-> "\\"#{built.0}\\"")
    .extend built.1 supplied
    supplied
  other -> vec! \\-> (emit_expr other indent)
''')

write('emit/compose', '''
export(crate) compose : BinaryOp -> &Expr -> &Expr -> usize -> usize -> String
compose op lhs rhs param indent =
  mut links = Vec::new ()
  compose_chain op lhs rhs (&mut links)
  name = closure_param param
  mut applied = Expr::Ident (Ident { text = .clone name, span = .span lhs })
  for function in links
    applied := Expr::Call { callee = Box::new (.clone function), args = vec! \\-> applied, is_dollar = false, span = .span function }
  body = emit_expr (&applied) indent
  closure (&[name]) (&body)
''')

write('emit/instance', '''
export(crate) emit_instance : &mut Sink<'_> -> &InstanceDecl -> usize -> ()
emit_instance out i indent =
  .unmark out
  emit_trivia out (&i.leading) indent
  .mark i.span out
  pad out indent
  .push_str "impl" out
  .push_str (&generics (&i.target) (&i.constraints)) out
  .push ' ' out
  if &i.r#class is Some named
    lowered = .rust_text named
    .push_str (&format! \\-> "#{lowered} for ") out
  .push_str (&.rust_text i.target) out
  .push_str " {\\n" out
  for m in &i.members
    emit_member out m (indent + RUST_INDENT)
  .unmark out
  pad out indent
  .push_str "}\\n" out
  ()
''')

print('plan 81 batch 4: twelve files')
