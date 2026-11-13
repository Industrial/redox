# -*- coding: utf-8 -*-
"""Plan 81 batch 7: seven more, and the raw literal that answers plan 80 §2.4.

`double_braces`'s own tests are the place the brace-doubling defect would bite
hardest — every assertion is about a string full of braces. The answer is doc
03 §11.2's **raw literal**: `r"…"` is `Expr::RawLit`, which §11.2 calls *"pure
passthrough"* and which `macro_argument` hands to `emit_expr` with no doubling
at all.

So plan 80 §2.4's framing was incomplete: the defect is real, but a brace-
bearing string in a value position **does** have a spelling today, and it is
`r"…"` rather than the `char` that row reached for. The row owed is still the
macro-name table, because a plain `"…"` is silently rewritten and nothing says
so.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/format_template', '''
export(crate) format_template : &[StrInterpPart] -> usize -> (String, Vec<String>)
format_template parts indent =
  mut template = String::new ()
  mut args = Vec::new ()
  for part in parts
    case part of
      -- Already escaped by the parser; only the brace doubling is the
      -- emitter's, because it is a *format-string* concern and not an
      -- escaping one — the distinction `double_braces` calls load-bearing.
      StrInterpPart::Text text -> .push_str (&double_braces text) template
      StrInterpPart::Interp { expr, spec } ->
        -- The spec is pasted uninterpreted — doc 03 §11.1, and AGENTS.md's
        -- rule applied to a grammar Rust already owns. `#{x:nonsense}` emits
        -- `{:nonsense}` and rustc reports it against the interpolation's own
        -- location through plan 06's map.
        case spec of
          Some s -> .push_str (&format! \\-> "{:#{s}}") template
          None -> .push_str "{}" template
        .push (emit_expr (unparen expr) indent) args
  (template, args)
''')

write('emit/annotated_binding', '''
export(crate) annotated_binding : &Binding -> (Option<String>, &Expr)
annotated_binding b = case (&b.ty, &b.value) of
  (None, Expr::Ascribe { inner, ty, .. }) | !selects_a_container inner -> (Some (.clone ty.text), .as_ref inner)
  (Some ty, value) -> (Some (.clone ty.text), value)
  (None, value) -> (None, value)
''')

write('emit/emit_control_inline', '''
export(crate) emit_control_inline : &mut Sink<'_> -> &Expr -> usize -> bool -> ()
emit_control_inline out expr indent tail =
  guard expr is Expr::If { condition, then_block, r#otherwise, .. } else
    unreachable! \\-> "only an `if` is written inline"
  .push_str (&condition_head "if" condition indent) out
  emit_stmts out (&then_block.stmts) (indent + RUST_INDENT) (tail && .is_some r#otherwise)
  guard r#otherwise is Some other else
    .unmark out
    pad out indent
    .push_str "}\\n" out
    return ()
  .unmark out
  pad out indent
  -- The `matches!` folded into the `case`, because a second function beside
  -- this one would be a second construct in the file (doc 06 §1).
  chained = case .as_slice other.stmts of
    [only] -> case only.expr of
      Expr::If { .. } -> Some only
      _ -> None
    _ -> None
  if chained is Some only
    .push_str "} else " out
    emit_control_inline out (&only.expr) indent tail
    return ()
  .push_str "} else {\\n" out
  emit_stmts out (&other.stmts) (indent + RUST_INDENT) tail
  .unmark out
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/with_map', '''
export emit_with_map : &Program -> &str -> &str -> (String, SourceMap)
emit_with_map program source source_path =
  -- Plan 56 leaf 1. The arity table is installed **here**, at the emitter's
  -- entry, and nowhere else: `voltaic_fmt::format` reads the same `Program` and
  -- must keep printing the author's `add 1` as `add 1`, so nothing about
  -- currying may be visible before this point (§11.3). The guard's lifetime is
  -- §1.4's boundary — currying is available where the signature is in the file
  -- being emitted.
  _arity = ArityScope::enter (ArityTable::from_items (&program.items))
  -- Plan 59 leaf 1, and here for the same reason rather than a parallel one:
  -- `voltaic_fmt::format` reads this `Program` too and must keep printing the
  -- author's `block` as `block`, so the classification may not be visible
  -- before the emitter's entry either. Two guards, one entry point.
  _bindings = BindingScope::enter (BindingTable::from_items (&program.items))
  mut out = Sink::new source
  -- The items, with the blank-line rule, source order and the gathered test
  -- module — all of which a `module` body wants too, which is why they live
  -- in `emit_items` rather than here (plan 14).
  emit_items (&mut out) (&program.items) 0
  .unmark out
  emit_trivia (&mut out) (&program.trailing) 0
  -- A source file's final newline arrives here as a trailing `BlankLines`
  -- trivium, which would otherwise append a stray blank line to every
  -- generated file. Normalising to exactly one terminating newline is a
  -- deterministic rule that also guards against other trailing-whitespace
  -- drift, which matters because golden tests compare exact bytes.
  while .ends_with "\\n\\n" out
    .pop out
  if !.is_empty out && !.ends_with "\\n" out
    .push '\\n' out
  .finish source_path out
''')

write('emit/item', '''
export(crate) emit_item : &mut Sink<'_> -> &Item -> usize -> ()
emit_item out item indent =
  case item of
    -- Plan 59 leaf 2. A parameterless binding with a literal body and an
    -- upper-case name is a value, so it is written as the `const` doc 03
    -- §13.4 makes the default — through plan 16's own emission, not a second
    -- path to the same Rust. Everything else is still a function, and leaf 3
    -- restores the call at the use site.
    Item::Fn f | BindingScope::kind_of (&f.name.text) == Some (BindingKind::Shared) ->
      case shared_const f of
        Some c -> emit_const out (&c) indent
        None -> emit_fn out f indent
    Item::Fn f -> emit_fn out f indent
    Item::Data d -> emit_data out d indent
    Item::Type a -> emit_type_alias out a indent
    Item::Class c -> emit_class out c indent
    Item::Instance i -> emit_instance out i indent
    Item::Errors e -> emit_errors out e indent
    Item::Import i -> emit_import out i indent
    Item::Module m -> emit_module out m indent
    Item::Const c -> emit_const out c indent
    Item::InlineRust r -> emit_inline_rust out r indent
    Item::Extern e -> emit_extern_block out e indent
    Item::Test _ ->
      unreachable! \\-> "tests are gathered, not emitted in place"
  ()
''')

write('emit/class', '''
export(crate) emit_class : &mut Sink<'_> -> &ClassDecl -> usize -> ()
emit_class out c indent =
  .unmark out
  emit_trivia out (&c.leading) indent
  .mark c.span out
  pad out indent
  vis = .rust_text c.visibility
  .push_str (&format! \\-> "#{vis}trait #{c.name.text}") out
  if !.is_empty c.params
    params : Vec<String> = c.params |> .iter |> .map (\\p -> TypeApp::rust_name (&p.text)) |> .collect
    joined = .join ", " params
    .push_str (&format! \\-> "<#{joined}>") out
  if !.is_empty c.supers
    names : Vec<&str> = c.supers |> .iter |> .map (\\s -> .as_str s.text) |> .collect
    joined = .join " + " names
    .push_str (&format! \\-> ": #{joined}") out
  .push_str " {\\n" out
  for m in &c.members
    emit_member out m (indent + RUST_INDENT)
  .unmark out
  pad out indent
  .push_str "}\\n" out
  ()
''')

write('emit/double_braces', '''
export(crate) double_braces : &str -> String
double_braces literal = .replace '}' "}}" (.replace '{' "{{" literal)

-- | Every brace-bearing argument below is a **raw** literal — doc 03 §11.2's
-- | `r"…"`, which §11.2 calls *"pure passthrough"*. A plain `"…"` reaching a
-- | macro is brace-doubled by `macro_argument`, which would rewrite the very
-- | strings these assertions are about.
test "doubles a brace so it prints as itself"
  assert_eq! \\-> double_braces r#""see {this}""# \\-> r#""see {{this}}""#

test "leaves a literal with no braces alone"
  assert_eq! \\-> double_braces r#""Hello, world!""# \\-> r#""Hello, world!""# \\-> "plan 01's golden depends on this being unchanged"

-- | A brace that is already doubled in the source text is user data too, and
-- | doubling it again is what makes it print as two braces.
test "doubles each brace independently"
  assert_eq! \\-> double_braces r"{{" \\-> r"{{{{"
  assert_eq! \\-> double_braces r"}" \\-> r"}}"

test "handles an empty literal"
  assert_eq! \\-> double_braces "" \\-> ""
''')

print('plan 81 batch 7: seven files')
