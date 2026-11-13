# -*- coding: utf-8 -*-
"""Plan 81 batch 2: fourteen more of `voltaic-emit`'s bodies.

`where_order/visit` is the first ported body with a **`&dep` binder in a
`for`** — a `for` binder is a bare name, so it binds the reference and
dereferences at the use. `one_tail_expression` is the first with a **guarded
slice pattern**, which carries over as written.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/struct_variant_width', '''
export(crate) const STRUCT_VARIANT_WIDTH : usize = 34
''')

write('emit/annotated_binding/selects_a_container', '''
export(crate) selects_a_container : &Expr -> bool
selects_a_container inner = case inner of
  Expr::List { .. } or Expr::ArrayRepeat { .. } -> true
  _ -> false
''')

write('emit/closure', '''
export(crate) closure : &[String] -> &str -> String
closure params body =
  joined = .join ", " params
  format! \\-> "|#{joined}| #{body}"
''')

write('emit/field_value', '''
export(crate) field_value : &FieldInit -> usize -> String
field_value f indent = case &f.value of
  Some e -> emit_expr (unparen e) indent
  None -> .clone f.name.text
''')

write('emit/where_order/visit', '''
export(crate) visit : usize -> &[Vec<usize>] -> &mut [bool] -> &mut [bool] -> &mut Vec<usize> -> ()
visit at graph done visiting sorted =
  if done[at] || visiting[at]
    return ()
  visiting[at] := true
  -- `dep` is the reference, where the Rust wrote `for &dep in …`: a `for`
  -- binder is a bare name, so the dereference moves to the use.
  for dep in &graph[at]
    visit (*dep) graph done visiting sorted
  visiting[at] := false
  done[at] := true
  .push at sorted
  ()
''')

write('emit/names_a_constructor', '''
export(crate) names_a_constructor : &Expr -> bool
names_a_constructor head =
  -- `name`, not a second `text`: the `Expr::Path` arm binds one.
  name = case head of
    Expr::Ident i -> .as_str i.text
    Expr::Path { text, .. } -> .unwrap_or text (.next (.rsplit "::" text))
    -- Its own line: `return` is not accepted as an *inline* arm body, the
    -- same limit `break` has.
    _ ->
      return false
  .is_some_and char::is_uppercase (.next (.chars name))
''')

write('emit/one_tail_expression', '''
export(crate) one_tail_expression : &Clause -> bool -> Option<&Stmt>
one_tail_expression clause tail =
  if !tail
    return None
  plain = (.plain clause.body)?
  case .as_slice plain.stmts of
    [stmt] | !.is_statement_form stmt.expr && .is_empty stmt.leading -> Some stmt
    _ -> None
''')

write('emit/type_alias', '''
export(crate) emit_type_alias : &mut Sink<'_> -> &TypeAlias -> usize -> ()
emit_type_alias out a indent =
  .mark a.span out
  emit_trivia out (&a.leading) indent
  pad out indent
  vis = .rust_text a.visibility
  .push_str (&format! \\-> "#{vis}type #{a.name.text} = #{a.ty.text};\\n") out
  ()
''')

write('emit/ascribed_arm/sole_binder', '''
export(crate) sole_binder : &Pattern -> Option<String>
sole_binder pattern = case .as_slice (.binders pattern) of
  [one] -> Some (.clone one)
  _ -> None
''')

write('emit/unsafe_block', '''
export(crate) emit_unsafe_block : &Block -> usize -> String
emit_unsafe_block body indent =
  if .sole_expression body is Some e
    inner = emit_expr e indent
    -- `{` is a literal brace in a Voltaic string, so the emitter doubles it for
    -- Rust — which is exactly what the Rust's `{{ }}` spelled by hand.
    return format! \\-> "unsafe { #{inner} }"
  block = emit_block (&None) body indent
  format! \\-> "unsafe #{block}"
''')

write('emit/emit_const', '''
export(crate) emit_const : &mut Sink<'_> -> &ConstDecl -> usize -> ()
emit_const out c indent =
  .unmark out
  emit_trivia out (&c.leading) indent
  .mark c.span out
  pad out indent
  keyword = if c.is_static
    "static"
  else
    "const"
  vis = .rust_text c.visibility
  value = emit_expr (&c.value) indent
  .push_str (&format! \\-> "#{vis}#{keyword} #{c.name.text}: #{c.ty.text} = #{value};\\n") out
  ()
''')

write('emit/import', '''
export(crate) emit_import : &mut Sink<'_> -> &ImportDecl -> usize -> ()
emit_import out i indent =
  .unmark out
  emit_trivia out (&i.leading) indent
  .mark i.span out
  pad out indent
  vis = .rust_text i.visibility
  .push_str (&format! \\-> "#{vis}use #{i.path}") out
  if &i.alias is Some alias
    .push_str (&format! \\-> " as #{alias.text}") out
  .push_str ";\\n" out
  ()
''')

write('emit/emit_body', '''
export(crate) emit_body : &mut Sink<'_> -> &ClauseBody -> usize -> bool -> bool -> ()
emit_body out body indent tail joinable =
  case body of
    ClauseBody::Plain block -> emit_stmts out (&block.stmts) indent tail
    ClauseBody::Guarded guards -> emit_guards out guards indent tail joinable
  ()
''')

write('emit/unlowerable', '''
export(crate) unlowerable : &str -> String
unlowerable what =
  format! \\-> "compile_error!(\\"#{what} is a statement in Rust and has no expression spelling; this is a Voltaic defect, not a defect in your program\\")"
''')

print('plan 81 batch 2: fourteen files')
