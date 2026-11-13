# -*- coding: utf-8 -*-
"""Plan 81 batch 15: `emit_application` and `emit_errors`."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/application', '''
export(crate) emit_application : &Expr -> &[Expr] -> usize -> String
emit_application callee args indent =
  -- The `matches!` as a `case`, bound before the chain, because a macro's
  -- second argument cannot be a pattern.
  if unparen callee is Expr::Call { callee = inner, args = inner_args, .. }
    inner_is_method_ref = case unparen inner of
      Expr::MethodRef { .. } -> true
      _ -> false
    if .is_some (missing_arity inner inner_args) || inner_is_method_ref
      mut flattened = .clone inner_args
      .extend (.cloned (.iter args)) flattened
      return emit_application inner (&flattened) indent
  -- Plan 57's method-as-function. Not a call of a callee at all: the **last**
  -- argument is the receiver, so `.cmp y x` is `x.cmp(y)`. It is reached from
  -- here rather than from `emit_expr`'s `MethodRef` arm because that arm sees
  -- the method applied to *nothing* — the value awaiting its receiver — and
  -- this one sees the application.
  --
  -- The flatten above covers it too: `(.insert k) v m` is `.insert k v m` by
  -- associativity, which §11.6 is the note for, and `$` reaches the same
  -- shape.
  if unparen callee is Expr::MethodRef { name, .. }
    return method_ref name args indent
  head = receiver callee indent
  -- **A constructor applied to `()` passes unit.** `expr_list` collapses a
  -- lone unit argument to the empty list, because `f ()` is doc 18 §7's
  -- zero-argument call — and its doc comment accepts the trade that a
  -- function genuinely taking a `()` then cannot be called by juxtaposition,
  -- on the grounds that a unit parameter is rare. It is not: it is how every
  -- `Result<(), E>` function returns, and plan 74 found `return Ok (())`
  -- emitting `return Ok();` eleven times in `voltaic-lexer`.
  --
  -- The two readings are told apart by the convention the language already
  -- runs on — the case of the first letter, which decides a type variable
  -- (doc 18 §5.3) and a constructor pattern's path (plan 68). An uppercase
  -- last segment is a constructor, so `Ok ()` is `Ok(())`; a lowercase one is
  -- a function, so `String::new ()` stays `String::new()`.
  --
  -- Nothing takes this away: a unit struct is written `Marker` with no call,
  -- and `data` cannot declare a zero-field tuple struct.
  lone_unit = case args of
    [Expr::Unit { .. }] -> true
    _ -> false
  unit_argument = lone_unit && .is_some_and (\\seg -> .starts_with char::is_uppercase seg) (.next (.rsplit "::" head))
  guard missing_arity callee args is Some missing else
    if unit_argument
      return format! \\-> "#{head}(())"
    listed = expr_list args indent
    return format! \\-> "#{head}(#{listed})"
  params : Vec<String> = .collect (.map closure_param (0 .. missing))
  listed = expr_list args indent
  joined = .join ", " params
  body = format! \\-> "#{head}(#{listed}, #{joined})"
  closure (&params) (&body)
''')

write('emit/errors', '''
export(crate) emit_errors : &mut Sink<'_> -> &ErrorsDecl -> usize -> ()
emit_errors out d indent =
  .unmark out
  emit_trivia out (&d.leading) indent
  .mark d.span out
  name = &d.name.text
  pad out indent
  .push_str "#[derive(Debug)]\\n" out
  -- Doc 03 §12.5's example shows `pub enum`, which reads as having been
  -- written `export errors ConfigError`: the marker governs this declaration
  -- like any other, and §9.6's rule that absent means private is not
  -- suspended for one construct.
  pad out indent
  vis = .rust_text d.visibility
  .push_str (&format! \\-> "#{vis}enum #{name} {\\n") out
  for v in &d.variants
    .mark v.span out
    pad out (indent + RUST_INDENT)
    .push_str (&v.name.text) out
    if !.is_empty v.fields
      texts : Vec<&str> = v.fields |> .iter |> .map (\\t -> .as_str t.text) |> .collect
      joined = .join ", " texts
      .push_str (&format! \\-> "(#{joined})") out
    .push_str ",\\n" out
  .unmark out
  pad out indent
  .push_str "}\\n\\n" out
  -- The variants come into scope under their own names, exactly as `data`'s
  -- do — `emit_data`'s enum branch ends with the same two lines, for the same
  -- reason and with the same wording.
  --
  -- Plan 53 leaf 1. Before it, `errors` was the one declaration that
  -- introduced constructors and then hid them: `data Event = Click | Close`
  -- gave you `Click`, and `errors ConfigError` with a `Missing` row gave you
  -- `ConfigError::Missing` and nothing shorter. Haskell has no such
  -- distinction, and neither document that specifies `errors` — doc 03 §12.5
  -- and doc 18 §8 — asks for one. It was an omission, and the symptom was
  -- `E0425: cannot find function, tuple struct or tuple variant` reported
  -- against a line that reads like ordinary Haskell.
  --
  -- Private, and a plain `use` rather than `pub use`: `data`'s is plain, and
  -- re-exporting the variants from the enclosing module would be a visibility
  -- decision nobody made. A collision between two names is rustc's error to
  -- give — the same division of labour `emit_data` records.
  pad out indent
  .push_str (&format! \\-> "use #{name}::*;\\n\\n") out
  pad out indent
  .push_str (&format! \\-> "impl std::fmt::Display for #{name} {\\n") out
  pad out (indent + RUST_INDENT)
  .push_str "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\\n" out
  pad out (indent + RUST_INDENT * 2)
  .push_str "match self {\\n" out
  for v in &d.variants
    .mark v.span out
    -- `built.0` is the template and `built.1` its arguments, where the Rust
    -- destructured the pair — and `format` would have been a poor name for a
    -- `String` anyway.
    built = error_message (&v.message)
    count = .len v.fields
    binders : Vec<String> = (0 .. count) |> .map (\\i -> format! \\-> "_#{i}") |> .collect
    pad out (indent + RUST_INDENT * 3)
    .push_str (&format! \\-> "#{name}::#{v.name.text}") out
    if !.is_empty binders
      joined = .join ", " binders
      .push_str (&format! \\-> "(#{joined})") out
    .push_str (&format! \\-> " => write!(f, \\"#{built.0}\\"") out
    for arg in &built.1
      .push_str (&format! \\-> ", #{arg}") out
    .push_str "),\\n" out
  .unmark out
  pad out (indent + RUST_INDENT * 2)
  .push_str "}\\n" out
  pad out (indent + RUST_INDENT)
  .push_str "}\\n" out
  pad out indent
  .push_str "}\\n\\n" out
  pad out indent
  .push_str (&format! \\-> "impl std::error::Error for #{name} {}\\n") out
  for v in &d.variants
    if !v.from
      continue
    ty = &v.fields[0].text
    .mark v.span out
    .push '\\n' out
    pad out indent
    .push_str (&format! \\-> "impl From<#{ty}> for #{name} {\\n") out
    pad out (indent + RUST_INDENT)
    .push_str (&format! \\-> "fn from(e: #{ty}) -> Self {\\n") out
    pad out (indent + RUST_INDENT * 2)
    .push_str (&format! \\-> "#{name}::#{v.name.text}(e)\\n") out
    pad out (indent + RUST_INDENT)
    .push_str "}\\n" out
    pad out indent
    .push_str "}\\n" out
  .unmark out
  ()
''')

print('plan 81 batch 15: two files')
