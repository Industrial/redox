# -*- coding: utf-8 -*-
"""Plan 81 batch 10: four more of `voltaic-emit`'s bodies.

`macro_call`'s Rust chains an **irrefutable** `let` — `&& let rewritten =
assert_call(…) &&` — which has no place in an `is` chain, because `is` asks a
refutable question. A length test and a binding say the same thing.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/macro_call', '''
export(crate) macro_call : &str -> &[Expr] -> usize -> String
macro_call name args indent =
  -- A length test and a binding, where the Rust chained a slice pattern and
  -- an **irrefutable** `let`: `is` asks a refutable question, so an
  -- always-succeeding binding cannot be one of its links.
  if name == "assert" && .len args == 1
    rewritten = assert_call (&args[0]) indent
    if !.starts_with "assert!" rewritten
      return rewritten
  supplied : Vec<String> = args |> .iter |> .flat_map (\\a -> macro_argument a indent) |> .collect
  -- Measured on the **arguments**, not on the rendered call: that is what
  -- `FN_CALL_WIDTH` means, and counting `println!(` and `)` with them put
  -- every argument list nine characters nearer the threshold than rustfmt
  -- puts it. `examples/errors.sdx` spread a line rustfmt keeps whole.
  arguments = .join ", " supplied
  one_line = format! \\-> "#{name}!(#{arguments})"
  if !.contains '\\n' arguments && .len arguments <= FN_CALL_WIDTH
    return one_line
  -- The same shape `print_call` spreads to, for the same reason: rustfmt
  -- overflows a block-like last argument and spreads anything else. A
  -- multi-line callback body is block-like, so it hangs off the call — which
  -- is why the test is on the *rendered* argument rather than on its arity.
  if .is_some_and (\\a -> .starts_with '{' a) (.last supplied)
    return one_line
  inner_pad = .repeat (indent + RUST_INDENT) " "
  close_pad = .repeat indent " "
  separator = format! \\-> ",\\n#{inner_pad}"
  parts : Vec<String> = args |> .iter |> .flat_map (\\a -> macro_argument a (indent + RUST_INDENT)) |> .collect
  spread = .join (&separator) parts
  format! \\-> "#{name}!(\\n#{inner_pad}#{spread}\\n#{close_pad})"
''')

write('emit/fields', '''
export(crate) emit_fields : &mut Sink<'_> -> &ConstructorFields -> usize -> bool -> ()
emit_fields out fields indent top =
  case fields of
    ConstructorFields::Unit ->
      closer = if top
        ";\\n"
      else
        ",\\n"
      .push_str closer out
    ConstructorFields::Tuple types ->
      texts : Vec<&str> = types |> .iter |> .map (\\t -> .as_str t.text) |> .collect
      joined = .join ", " texts
      .push_str (&format! \\-> "(#{joined})") out
      closer = if top
        ";\\n"
      else
        ",\\n"
      .push_str closer out
    ConstructorFields::Record decls ->
      -- The comma-joined field text, which is what rustfmt's width thresholds
      -- are measured against — see `STRUCT_VARIANT_WIDTH`.
      texts : Vec<String> = decls
        |> .iter
        |> .map (\\f ->
        vis = .rust_text f.visibility
        format! \\-> "#{vis}#{f.name.text}: #{f.ty.text}")
        |> .collect
      joined = .join ", " texts
      -- A field carrying its own doc comment cannot take the one-line form:
      -- the comment has to go on a line of its own, and collapsing the record
      -- would drop it. Plan 48 leaf 2 found field trivia discarded entirely,
      -- which AGENTS.md calls a load-bearing rule — "comments are trivia,
      -- never discarded" (doc 02 §10.1) — so the width test now yields to it.
      documented = .any (\\f -> !.is_empty f.leading) (.iter decls)
      if !top && !documented && .len joined <= STRUCT_VARIANT_WIDTH
        .push_str (&format! \\-> " { #{joined} },\\n") out
        return ()
      .push_str " {\\n" out
      for f in decls
        .mark f.span out
        emit_trivia out (&f.leading) (indent + RUST_INDENT)
        pad out (indent + RUST_INDENT)
        vis = .rust_text f.visibility
        .push_str (&format! \\-> "#{vis}#{f.name.text}: #{f.ty.text},\\n") out
      .unmark out
      pad out indent
      closer = if top
        "}\\n"
      else
        "},\\n"
      .push_str closer out
  ()
''')

write('emit/tests_module', '''
export(crate) emit_tests_module : &mut Sink<'_> -> &[&TestDecl] -> usize -> ()
emit_tests_module out tests indent =
  if .is_empty tests
    return ()
  .unmark out
  pad out indent
  .push_str "#[cfg(test)]\\n" out
  pad out indent
  .push_str "mod __rdx_tests {\\n" out
  pad out (indent + RUST_INDENT)
  .push_str "use super::*;\\n" out
  mut used : Vec<String> = Vec::new ()
  for t in tests
    base = test_slug (&t.description)
    mut name = .clone base
    mut index = 2
    while .contains (&name) used
      name := format! \\-> "#{base}_#{index}"
      index += 1
    .push (.clone name) used
    .unmark out
    .push '\\n' out
    emit_trivia out (&t.leading) (indent + RUST_INDENT)
    .mark t.span out
    pad out (indent + RUST_INDENT)
    .push_str "#[test]\\n" out
    pad out (indent + RUST_INDENT)
    .push_str (&format! \\-> "fn #{name}() {\\n") out
    -- A test returns nothing, so its last statement is a statement and not a
    -- tail expression.
    emit_stmts out (&t.body.stmts) (indent + RUST_INDENT * 2) false
    .unmark out
    pad out (indent + RUST_INDENT)
    .push_str "}\\n" out
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/comprehension', '''
export(crate) emit_comprehension : &Expr -> &[Qualifier] -> usize -> String
emit_comprehension element qualifiers indent =
  draws : Vec<(&str, &Expr)> = qualifiers
    |> .iter
    |> .filter_map (\\q ->
    case q of
      Qualifier::Draw { name, source } -> Some (.as_str name.text, source)
      Qualifier::Guard _ -> None)
    |> .collect
  guards : Vec<String> = qualifiers
    |> .iter
    |> .filter_map (\\q ->
    case q of
      Qualifier::Guard e -> Some (emit_expr (unparen e) indent)
      Qualifier::Draw { .. } -> None)
    |> .collect
  -- With no generator there is nothing to iterate, so the brackets are a
  -- one-element list. rustc would reject anything else, and this is what the
  -- author wrote.
  if .is_empty draws
    only = emit_expr (unparen element) indent
    return format! \\-> "[#{only}]"
  item = emit_expr (unparen element) indent
  nested = .len draws > 1
  -- `built.0` is the head and `built.1` the segments, where the Rust
  -- destructured the pair.
  built = nest (&draws) (&guards) (&item) indent nested
  mut segments = built.1
  .push (.to_string ".collect()") segments
  chain (&built.0) (&segments) indent
''')

print('plan 81 batch 10: four files')
