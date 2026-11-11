# -*- coding: utf-8 -*-
"""Plan 81 batch 19: `emit_expr`, the crate's largest body.

Forty-odd arms, and only three need anything said about them. A record
pattern's field may be matched against a **single-segment** path, so
`op: BinaryOp::Pipeline` is written as a guard comparison — the same thing
`wraps_as_receiver` needed in batch 8. `pattern: None` and `pattern: Some(_)`
are single-segment and carry over as written.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/expr', '''
export(crate) emit_expr : &Expr -> usize -> String
emit_expr e indent = case e of
  -- Doc 18 §4. A Rust closure, with `move` and types carried through.
  Expr::Lambda { params, body, is_move, .. } ->
    binders : Vec<String> = params
      |> .iter
      |> .map (\\param ->
      case &param.ty of
        Some ty -> format! \\-> "#{param.name.text}: #{ty.text}"
        None -> .clone param.name.text)
      |> .collect
    marker = if *is_move
      "move "
    else
      ""
    joined = .join ", " binders
    inner = emit_expr body indent
    format! \\-> "#{marker}|#{joined}| #{inner}"
  -- Doc 18 §6. A block, because Rust has no expression-level ascription:
  -- `{ let __rdx_t: T = e; __rdx_t }` states the type and yields the value.
  -- The `__rdx_` prefix is reserved for generated identifiers (doc 02), so
  -- this cannot collide with anything the author wrote.
  -- The block spans three lines, so it is the first lowering that has to
  -- know its own column.
  Expr::Ascribe { inner, ty, .. } ->
    inner_pad = .repeat (indent + RUST_INDENT) " "
    close_pad = .repeat indent " "
    -- **Through a parenthesis.** `let __rdx_t: T = (e);` is clippy's
    -- `unnecessary_parens`, which `-D warnings` makes an error — and the
    -- parenthesis is not the author's to drop: `(Box::new h) : Box<dyn
    -- Display>` needs it to ascribe the *call* rather than `h`. The value
    -- lands in `let` position, where no expression needs bracketing, so the
    -- lowering unwraps it. Plan 69's `Unary` arm is the same fix for the
    -- same reason; plan 73 found this one in `redox-diag`'s `reportable`.
    bare = unparen inner
    -- Doc 18 §6's third lowering: an ascribed list literal names its
    -- container, so its value is `C::from([…])`. Inside the block, not
    -- instead of it — see `ascribed_list`.
    value = .unwrap_or_else (\\-> emit_expr bare (indent + RUST_INDENT)) (ascribed_list bare ty (indent + RUST_INDENT))
    format! \\-> "{\\n#{inner_pad}let __rdx_t: #{ty.text} = #{value};\\n#{inner_pad}__rdx_t\\n#{close_pad}}"
  -- Doc 18 §4. An operator with one side supplied, as a closure.
  Expr::Section { section, .. } -> emit_section section indent
  -- Doc 18 §2.4. A record literal: construction, punning, or the update
  -- lowering, decided by the case of the head.
  Expr::Record { head, fields, .. } -> emit_record head fields indent
  -- Doc 18 §2.3. Rust's `match`, with `->` arms read as `=>`.
  Expr::Case { scrutinee, arms, .. } -> emit_case scrutinee arms indent
  -- An indented block as a value — plan 20's `do`, and marker-free since
  -- plan 55 leaf 7. Labelled when the author labelled it.
  Expr::Block { label, body, .. } -> emit_block label body indent
  Expr::Unit { .. } -> .to_string "()"
  -- Rust's try operator, unchanged. The receiver needs parentheses for the
  -- same reason a method call's does: `?` binds tighter than any operator.
  Expr::Try { inner, .. } -> format! \\-> "#{receiver inner indent}?"
  -- Rust's array literal, unchanged — a spelling, not a lowering.
  Expr::List { items, .. } -> format! \\-> "[#{expr_list items indent}]"
  -- Doc 18 §3.2. An iterator chain, built inside out.
  Expr::Comprehension { element, qualifiers, .. } -> emit_comprehension element qualifiers indent
  Expr::StrLit { value, .. } -> rust_escapes value
  -- Doc 03 §11.1's interpolation — plan 23. `format!` here; `print_call`
  -- takes the same template to `println!` instead.
  Expr::StrInterp { parts, .. } ->
    -- `built.0` is the template and `built.1` its arguments.
    built = format_template parts indent
    if .is_empty built.1
      return format! \\-> "format!(\\"#{built.0}\\")"
    joined = .join ", " built.1
    format! \\-> "format!(\\"#{built.0}\\", #{joined})"
  -- Plan 57's method-as-function, applied to nothing: a value awaiting its
  -- receiver. `emit_application` is where it is applied.
  Expr::MethodRef { name, .. } -> method_ref name (&[]) indent
  -- Doc 03 §12.4's `try` block — plan 28, and a block *modifier* since plan
  -- 55 leaf 7.
  -- Plan 60 leaf 3: inside a `try`, `<-` is the `?`. The guard lives for
  -- the body's emission and restores the outer meaning after it, because
  -- the modifiers nest in both directions.
  Expr::TryBlock { body, .. } ->
    _bind = BindScope::enter BindMode::Question
    try_block body indent
  -- Doc 03 §14.3's `async` block — plan 39. `emit_block` already writes a
  -- block expression, so this is that block with one word in front, which
  -- is what plan 55 §1.7 means by a **modifier**: the lowering was always
  -- a block plus an adjective.
  Expr::AsyncBlock { body, .. } ->
    -- And inside an `async` block it is `await` again, even when that block
    -- sits inside a `try`.
    _bind = BindScope::enter BindMode::Await
    format! \\-> "async #{emit_block (&None) body indent}"
  -- Plan 45 leaf 2. Delegated because a one-expression body collapses to a
  -- single line, which is what rustfmt does and what the gate compares
  -- against.
  Expr::UnsafeBlock { body, .. } -> emit_unsafe_block body indent
  -- A tuple expression — plan 33. Verbatim, and `expr_list` is what every
  -- other comma-separated run uses, so the spacing matches.
  Expr::Tuple { items, .. } ->
    -- A one-element tuple keeps Rust's trailing comma, for the reason
    -- `Pattern::Tuple` records: `(a)` is a group and `(a,)` is a tuple.
    inner = expr_list items indent
    if .len items == 1
      return format! \\-> "(#{inner},)"
    format! \\-> "(#{inner})"
  -- Doc 03 §10.4's array repeat — plan 29. **Verbatim, never `vec!`**: a
  -- repeat expression is a fixed-size array and `vec!` is a heap
  -- allocation. Both compile, so only this line tells them apart.
  Expr::ArrayRepeat { value, count, .. } ->
    v = emit_expr (unparen value) indent
    n = emit_expr (unparen count) indent
    format! \\-> "[#{v}; #{n}]"
  -- Doc 03 §11.2's four literal kinds — verbatim, never a format string.
  Expr::RawLit { text, .. } -> .clone text
  Expr::Num { text, .. } -> .clone text
  Expr::Bool { value, .. } -> .to_string value
  Expr::Char { value, .. } -> format! \\-> "#{value:?}"
  -- Doc 03 §12.6's bare panic-family spellings. A lookup rather than a
  -- keyword, because nothing else distinguishes `todo` from a function the
  -- author wrote and has not defined yet — which is, after all, what
  -- `todo` is for.
  Expr::Ident i -> binding_text (&i.text)
  Expr::Path { text, .. } -> binding_text text
  -- Doc 01 §16: `@field` is `self.field` and `@@name` is `Self::name`.
  Expr::SelfField { name, .. } -> format! \\-> "self.#{name.text}"
  Expr::SelfType { name, .. } -> format! \\-> "Self::#{name.text}"
  Expr::Paren { inner, .. } -> format! \\-> "(#{emit_expr inner indent})"
  Expr::Unary { op, operand, .. } -> format! \\-> "#{.rust_text op}#{emit_expr operand indent}"
  -- Doc 18 §4's pipeline is a *lowering*, not a spelling: `a |> f` becomes
  -- `a.f()`, which needs the right operand taken apart.
  --
  -- The four operator arms below compare in a guard, where the Rust matched
  -- the field against a path: a record pattern's field may be matched
  -- against a *single-segment* path, and `BinaryOp::Pipeline` is two.
  Expr::Binary { op, lhs, rhs, .. } | *op == BinaryOp::Pipeline -> pipeline lhs rhs indent
  -- Doc 03 §12.1, which doc 18 leaves unchanged: `??` is `Option`-only and
  -- lowers to a `match`. A *lowering*, not a spelling, which is why it
  -- sits beside the pipeline rather than in the operator table.
  Expr::Binary { op, lhs, rhs, .. } | *op == BinaryOp::Coalesce -> coalesce lhs rhs indent
  -- Doc 03 §12.2's `catch` — `??`'s mirror for `Result`, and the same
  -- `match` lowering for §12.1's reasons. Plan 28.
  Expr::Binary { op, lhs, rhs, .. } | *op == BinaryOp::Catch -> catch_fallback lhs rhs indent
  -- Plan 66's composition, a third *lowering* rather than a spelling: Rust
  -- has no composition operator, so `f >>> g` is the closure that applies
  -- one and then the other. The two directions share it, operands
  -- exchanged — see `compose`.
  Expr::Binary { op, lhs, rhs, .. } | *op == BinaryOp::Compose || *op == BinaryOp::ComposeBack -> compose (*op) lhs rhs 0 indent
  Expr::Binary { op, lhs, rhs, .. } -> case .rust_text op of
    -- A range operator is written **tight**, which is Rust's own style and
    -- therefore rustfmt's: it rewrites `1 .. 4` to `1..4`, and plan 04's
    -- guarantee is that the emitter does not produce output rustfmt
    -- rewrites. Every other operator takes its spaces.
    Some text | .is_range op ->
      left = emit_expr lhs indent
      right = emit_expr rhs indent
      format! \\-> "#{left}#{text}#{right}"
    Some text ->
      left = emit_expr lhs indent
      right = emit_expr rhs indent
      format! \\-> "#{left} #{text} #{right}"
    -- Both operators that need a lowering rather than a spelling — `??`
    -- and `|>` — are matched above, so this arm is unreachable through any
    -- source. Kept as a hard error rather than removed: a *new* operator
    -- with no lowering must fail loudly in the generated file, not quietly
    -- produce something that looks plausible.
    None -> format! \\-> "compile_error!(\\"`#{op:?}` is parsed but not yet lowered\\")"
  -- `receiver` rather than `emit_expr` for all three: it strips a grouping
  -- paren the author wrote and adds one where `.` or `[` would otherwise
  -- bind tighter, which is the same question asked in both directions.
  Expr::Field { recv, name, .. } -> format! \\-> "#{receiver recv indent}.#{name.text}"
  Expr::Index { recv, index, .. } ->
    head = receiver recv indent
    at = emit_expr index indent
    format! \\-> "#{head}[#{at}]"
  Expr::Call { callee, args, .. } ->
    -- Plan 59 leaf 3. `block ()` and `block` are the same program, so they
    -- emit the same Rust. A lone unit is the *empty argument list* (doc 18
    -- §7), so applying a parameterless binding reads it once — where
    -- routing the callee through `binding_text` and then applying the
    -- result would write `block()()`.
    --
    -- Both spellings are kept working rather than one being broken here:
    -- criterion 4's diagnostic deprecates the parenthesised form, and a
    -- diagnostic is a thing a reader can act on where a miscompile is not.
    lone_unit = case .as_slice args of
      [Expr::Unit { .. }] -> true
      _ -> false
    if .is_empty args || lone_unit
      named = case .as_ref callee of
        Expr::Ident name -> Some (.as_str name.text)
        Expr::Path { text, .. } -> Some (.as_str text)
        _ -> None
      if named is Some name && .is_some (BindingScope::kind_of name)
        return binding_text name
    -- Doc 18 §2.1's application, and plan 56's currying: a saturated call,
    -- or the closure an under-applied one becomes. Delegated because the
    -- decision needs the arity table and the `$` flatten, and neither
    -- belongs in a match arm.
    emit_application callee args indent
  -- ---- The forms plan 55 leaf 1 promoted out of `StmtKind` ----
  --
  -- None is reachable through today's grammar: `prefix` has no production
  -- for any of them, so each arrives only as a block element and
  -- `emit_stmt` claims those first. They are lowered anyway, for the reason
  -- this function's doc gives about `todo!()` and for a second one —
  -- `redox-fuzz` asserts the emitter never writes Rust `syn` cannot parse,
  -- so even the refusals have to be *parsable* refusals.
  --
  -- Eight of the twelve are genuine Rust expressions and lower to
  -- themselves with no `;`: `return` and `break` have type `!`, `x += 1`
  -- has type `()`, an `assert` is a macro call, a bare `.await` is a
  -- postfix, and the three loops are expressions in Rust exactly as they
  -- are in Redox.
  Expr::Return { value, .. } -> case value of
    Some v -> format! \\-> "return #{emit_expr v indent}"
    None -> .to_string "return"
  Expr::Break { label, value, .. } ->
    mut out = .to_string "break"
    if label is Some name
      .push_str (&format! \\-> " '#{name}") out
    if value is Some v
      .push_str (&format! \\-> " #{emit_expr (unparen v) indent}") out
    out
  Expr::Continue { label, .. } -> case label of
    Some name -> format! \\-> "continue '#{name}"
    None -> .to_string "continue"
  Expr::Assign { target, op, value, .. } ->
    operator = .map_or "" (\\o -> .unwrap_or "/* unsupported */" (.rust_text o)) (*op)
    left = emit_expr (unparen target) indent
    right = emit_expr (unparen value) indent
    format! \\-> "#{left} #{operator}= #{right}"
  -- Plan 64. The arguments are expressions and the `\\->` that spelled
  -- them is gone, so nothing here knows or cares whether the macro defers:
  -- it hands Rust the values and Rust hands them to the macro.
  Expr::MacroCall { name, args, .. } -> macro_call name args indent
  -- Plan 64 leaf 4. Verbatim, re-indented to the position it stands in —
  -- the one "lowering" that changes nothing but whitespace, for the reason
  -- `emit_inline_rust` gives one position over.
  Expr::RustHatch { code, .. } -> rust_hatch_text code indent
  -- The effect-only `<- f ()`, which is Rust's `f().await` and needs no
  -- binder. `receiver` for `emit_await`'s reason: `.await` is a postfix and
  -- binds tighter than every operator.
  Expr::Await { pattern = None, value, .. } -> format! \\-> "#{receiver (unparen value) indent}.await"
  -- An `if` in an expression position — plan 55 leaf 2 made it reachable
  -- and leaf 7 made it the only spelling, so `control_text` writes every
  -- one. It is what decides whether rustfmt would have collapsed the two
  -- branches onto one line (`one_line_if`), which is the lowering the
  -- retired `if c then a else b` used to reach directly.
  Expr::If { .. } or Expr::While { .. } or Expr::For { .. } or Expr::Loop { .. } -> control_text e indent
  -- The four with no expression spelling at all — `unlowerable` records
  -- which they are, why `compile_error!` is the refusal, and why this is
  -- not a Redox diagnostic.
  Expr::Binding _ -> unlowerable "a binding"
  Expr::Item { .. } -> unlowerable "a declaration"
  Expr::Guard { .. } -> unlowerable "a `guard`"
  Expr::Draw { .. } -> unlowerable "a `with` binder"
  Expr::Await { pattern = Some _, .. } -> unlowerable "an `await` that binds"
''')

print('plan 81 batch 19: one file')
