# -*- coding: utf-8 -*-
"""Plan 81 batch 16: `emit_stmt` and `emit_data`."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/stmt', '''
export(crate) emit_stmt : &mut Sink<'_> -> &Stmt -> usize -> bool -> ()
emit_stmt out stmt indent tail =
  .mark stmt.span out
  emit_trivia out (&stmt.leading) indent
  -- Plan 20's control flow writes its own padding, its own braces and its own
  -- trailing newline, because each form spans lines and an `else if` has to
  -- reuse the line its `}` is on.
  if .is_control_flow stmt.expr
    emit_control out (&stmt.expr) indent tail
    return ()
  -- Plan 55 leaf 5. A declaration writes its own padding — `emit_item`'s
  -- callees all take an `indent` and pad from it, because a module body has
  -- always needed that — so it is routed before the pad for the same reason
  -- control flow is.
  if &stmt.expr is Expr::Item { item, .. }
    emit_item out item indent
    return ()
  pad out indent
  case &stmt.expr of
    -- Immutable by default (doc 01 §4.3): `mut` is the opt-in, and the type
    -- is emitted only when the author wrote one — rustc infers the rest
    -- (doc 02 §6.2).
    Expr::Binding b ->
      .push_str "let " out
      if b.is_mut
        .push_str "mut " out
      .push_str (&.rust_text b.pattern) out
      -- Plan 62 leaf 1: an ascription that is the whole right-hand side is
      -- Rust's annotated `let`, so it moves to the annotation rather than
      -- emitting a block whose only job is to carry the type.
      --
      -- `annotated.0` is the type and `annotated.1` the value, where the Rust
      -- destructured the pair.
      annotated = annotated_binding b
      if annotated.0 is Some ty
        .push_str ": " out
        .push_str (&ty) out
      .push_str " = " out
      .push_str (&emit_expr annotated.1 indent) out
      .push ';' out
    -- A draw arrow is flattened by `emit_stmts`, which owns the run of
    -- statements its callback body is made of. Reaching it here would mean a
    -- caller emitted statements one at a time, which cannot lower it
    -- correctly — so this is a bug in Redox, not in the program.
    Expr::Draw { .. } ->
      unreachable! \\-> "a draw arrow must be emitted by `emit_stmts`, which has the following statements its callback body needs"
    -- Doc 30's await-bind. A statement, which is the whole of the decision:
    -- `<-` means `await` and nothing else, and every await binds.
    Expr::Await { pattern, value, .. } -> .push_str (&emit_bind pattern value indent) out
    -- Assignment — doc 01 §4.3's compound form from plan 20, and plan 54's
    -- plain `:=`. Rust spells both with `=`, differing only by the operator
    -- in front of it, so one arm prints all six: `None` is `=` and `Some(op)`
    -- is the operator's own Rust text plus `=`.
    --
    -- Redox's two spellings collapse to Rust's one here, which is the point
    -- of the row: `:=` exists to stop *Redox* having to decide whether `=`
    -- means a binding, and Rust never had that problem because `let` says so.
    Expr::Assign { target, op, value, .. } ->
      operator = .map_or "" (\\o -> .unwrap_or "/* unsupported */" (.rust_text o)) (*op)
      left = emit_expr (unparen target) indent
      right = emit_expr (unparen value) indent
      .push_str (&format! \\-> "#{left} #{operator}= #{right};") out
    Expr::Return { value, .. } ->
      .push_str "return" out
      if value is Some e
        .push ' ' out
        .push_str (&emit_expr e indent) out
      .push ';' out
    -- Every other form is a value expression, and a block element that is one
    -- is Rust's expression statement: the expression, then a `;` unless it is
    -- the block's value. Control flow is written above, before the padding,
    -- so none of those forms reaches here.
    value ->
      debug_assert! \\-> !.is_statement_form value \\-> "every statement form has an arm of its own above"
      .push_str (&emit_expr (unparen value) indent) out
      if !tail
        .push ';' out
  emit_inline_comment out (.as_ref stmt.trailing)
  .push '\\n' out
  ()
''')

write('emit/data', '''
export(crate) emit_data : &mut Sink<'_> -> &DataDecl -> usize -> ()
emit_data out d indent =
  .mark d.span out
  emit_trivia out (&d.leading) indent
  if !.is_empty d.derives
    names : Vec<&str> = d.derives |> .iter |> .map (\\i -> .as_str i.text) |> .collect
    joined = .join ", " names
    pad out indent
    .push_str (&format! \\-> "\\#[derive(#{joined})]\\n") out
  as_struct = case .as_slice d.constructors of
    [only] -> only.name.text == d.name.text
    -- No constructors at all is `data Marker` with no `=`, which is the unit
    -- struct. Doc 18 §2.4 writes it `data Marker = Marker`; both reach here.
    [] -> true
    _ -> false
  -- Doc 18 §5.3's synthesised list, in the one slot all three shapes share.
  -- Empty for a declaration with no type variables, which is almost all of
  -- them — so `data Point = Point { x : f64 }` emits exactly the Rust it
  -- emitted before plan 74 existed.
  params = if .is_empty d.generics
    String::new ()
  else
    joined = .join ", " d.generics
    format! \\-> "<#{joined}>"
  if as_struct
    fields = .map_or (&ConstructorFields::Unit) (\\c -> &c.fields) (.first d.constructors)
    -- Plan 53 leaf 3: `newtype` is the single-field case, so it is
    -- `#[repr(transparent)]` and `data` is not. That is the whole difference
    -- between the two keywords in the generated Rust, and before this row
    -- there was none — which made `newtype` a synonym for `data` that three
    -- doc comments and one test nonetheless described as meaningful.
    --
    -- `repr(transparent)` is not decoration. It guarantees the wrapper has
    -- the layout of its field, which is what makes a `newtype` sound to
    -- transmute across an FFI boundary and what tells a reader the wrapper
    -- costs nothing. A `data` declaration promises neither.
    --
    -- **After the derive, immediately above the struct.** Both orders compile
    -- and rustfmt reorders neither, so the choice is for the reader: the
    -- attribute that describes this struct's layout sits closest to the
    -- struct, and the derives — which describe its behaviour — stay grouped
    -- above. `attributes.rs`'s golden pins the order so it cannot drift.
    --
    -- rustc only backstops half of the shape rule: a two-field
    -- `#[repr(transparent)]` is `E0690`, but a *unit* struct with it compiles
    -- clean. Leaf 2's parser check is the only thing that refuses
    -- `newtype Foo = Foo`, which is why this emitter does not try to.
    if d.is_newtype
      pad out indent
      .push_str "#[repr(transparent)]\\n" out
    pad out indent
    -- Plan 45 leaf 4: `union` takes the `struct` slot. Everything after the
    -- keyword — the fields, the generics, the derives — is a struct's, which
    -- is the whole argument for it riding on `data` rather than being a
    -- declaration of its own.
    vis = .rust_text d.visibility
    keyword = if d.is_union
      "union"
    else
      "struct"
    .push_str (&format! \\-> "#{vis}#{keyword} #{d.name.text}#{params}") out
    emit_fields out fields indent true
    return ()
  pad out indent
  vis = .rust_text d.visibility
  .push_str (&format! \\-> "#{vis}enum #{d.name.text}#{params} {\\n") out
  for c in &d.constructors
    .mark c.span out
    -- Plan 68: the constructor's own doc comments, which had nowhere to be
    -- written until this row gave `Constructor` a `leading` field. Emitted
    -- **before** the padding for the reason `emit_trivia` takes an indent —
    -- a comment writes its own, and a `pad` here would double it.
    emit_trivia out (&c.leading) (indent + RUST_INDENT)
    pad out (indent + RUST_INDENT)
    .push_str (&c.name.text) out
    emit_fields out (&c.fields) (indent + RUST_INDENT) false
  .unmark out
  pad out indent
  .push_str "}\\n" out
  -- **A constructor is in scope under its own name**, as it is in Haskell —
  -- so `Close` and `Key 'k'` are written without the type, and a `case` arm
  -- spelled `Close ->` *matches the variant*.
  --
  -- This is a fourth new lowering, beyond the three doc 18 §6 discloses, and
  -- it is not optional sugar. Without it `Close` is an ordinary name: as an
  -- expression it fails to resolve, and as a `case` pattern it silently
  -- becomes an irrefutable binding, so `describe` would return its first arm
  -- for every input. That is doc 02 §5.2's worst failure mode exactly —
  -- output that reads as Rust and means something else — reached by writing
  -- doc 18 §2.4's own example.
  --
  -- Two names colliding is rustc's error to give, which is the same division
  -- of labour Haskell has: both languages reject the collision, neither
  -- resolves it.
  --
  -- **`#[allow(unused_imports)]`, because a file may only declare the type.**
  -- Plan 68 found it porting `redox-ast`: four of its modules declare an
  -- `enum` and never name a constructor, so the glob is an `unused_imports`
  -- warning — and `moon run :lint` makes that an error. No example caught it,
  -- because an example that declares a type also uses it.
  --
  -- The alternative was to emit the glob only when the program names a
  -- constructor, and it is refused rather than deferred: that needs a
  -- program-wide walk over every identifier-shaped reference, and
  -- **under-inclusion is a silent miscompile** — the irrefutable-binding
  -- failure this glob exists to prevent, reintroduced by the attempt to tidy
  -- it away. An `allow` cannot be wrong in that direction.
  pad out indent
  .push_str "#[allow(unused_imports)]\\n" out
  pad out indent
  .push_str (&format! \\-> "use #{d.name.text}::*;\\n") out
  ()
''')

print('plan 81 batch 16: two files')
