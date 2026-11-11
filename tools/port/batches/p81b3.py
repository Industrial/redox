# -*- coding: utf-8 -*-
"""Plan 81 batch 3: fourteen more of `redox-emit`'s bodies.

`test_slug` meets doc 69 §10.4 the hard way: the Rust writes
`trimmed.starts_with(|c: char| c.is_ascii_digit())`, and the annotation is
**load-bearing** — `str::starts_with` is generic over `Pattern`, so a bare
`\\c -> …` has nothing to infer `char` from. Asking the first character
directly says the same thing with no closure at all.

Every receiver here is written without `&`: plan 80 §2.6's rule, applied
first rather than after ninety lints.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/expr_list', '''
export(crate) expr_list : &[Expr] -> usize -> String
expr_list args indent =
  -- `f ()` is doc 18 §7's replacement for doc 03 §5.2's `!` zero-argument
  -- suffix, so a lone unit argument **is** the empty argument list — `f ()`
  -- must become `f()` and not `f(())`. Juxtaposition needs something to
  -- juxtapose, and `()` is that something.
  --
  -- The trade is that a function genuinely taking a `()` cannot be called by
  -- juxtaposition. Doc 18 §7 accepts it by replacing the suffix with this
  -- spelling, and a unit parameter is vanishingly rare against a
  -- zero-argument call.
  lone_unit = case args of
    [Expr::Unit { .. }] -> true
    _ -> false
  if lone_unit
    return String::new ()
  parts : Vec<String> = args |> .iter |> .map (\\e -> emit_expr (unparen e) indent) |> .collect
  .join ", " parts
''')

write('emit/emit_where_locals', '''
export(crate) emit_where_locals : &mut Sink<'_> -> &[Stmt] -> usize -> ()
emit_where_locals out wheres indent =
  if .len wheres < 2
    emit_stmts out wheres indent false
    return ()
  ordered : Vec<Stmt> = where_order wheres |> .into_iter |> .map (\\i -> .clone wheres[i]) |> .collect
  emit_stmts out (&ordered) indent false
  ()
''')

write('emit/receiver', '''
export(crate) receiver : &Expr -> usize -> String
receiver e indent =
  bare = unparen e
  text = emit_expr bare indent
  if wraps_as_receiver bare
    return format! \\-> "(#{text})"
  text
''')

write('emit/assert_call', '''
export(crate) assert_call : &Expr -> usize -> String
assert_call e indent =
  if unparen e is Expr::Binary { op, lhs, rhs, .. }
    chosen = case op of
      BinaryOp::Eq -> Some "assert_eq"
      BinaryOp::Ne -> Some "assert_ne"
      _ -> None
    if chosen is Some named
      left = emit_expr (unparen lhs) indent
      right = emit_expr (unparen rhs) indent
      return format! \\-> "#{named}!(#{left}, #{right})"
  inner = emit_expr (unparen e) indent
  format! \\-> "assert!(#{inner})"
''')

write('emit/inline_rust', '''
export(crate) emit_inline_rust : &mut Sink<'_> -> &InlineRust -> usize -> ()
emit_inline_rust out r indent =
  .mark r.span out
  emit_trivia out (&r.leading) indent
  for text in .lines r.code
    if .is_empty (.trim text)
      .push '\\n' out
      continue
    pad out indent
    .push_str text out
    .push '\\n' out
  ()
''')

write('emit/test_slug', '''
export(crate) test_slug : &str -> String
test_slug description =
  text = .trim_matches '"' description
  mut out = String::with_capacity (.len text)
  for ch in .chars text
    if .is_alphanumeric ch
      .extend (.to_lowercase ch) out
    else if !.ends_with '_' out
      .push '_' out
  trimmed = .to_string (.trim_matches '_' out)
  if .is_empty trimmed
    return .to_string "_"
  -- The first character directly, where the Rust wrote
  -- `starts_with(|c: char| …)`. A lambda parameter is a bare name (doc 69
  -- §10.4) and that annotation is **load-bearing**: `str::starts_with` is
  -- generic over `Pattern`, so a bare `\\c -> …` has nothing to infer `char`
  -- from. This asks the same question with no closure at all.
  if .is_some_and (\\c -> .is_ascii_digit c) (.next (.chars trimmed))
    return format! \\-> "_#{trimmed}"
  trimmed
''')

write('emit/binding_text', '''
export(crate) binding_text : &str -> String
-- `_` covers `Some (Shared)` and `None` together, which is what the Rust's
-- alternation said. The parentheses are the formatter's to keep: writing them
-- is what found `stands_alone` stripping them from a path argument, which is
-- plan 81 §2's fifth formatter round-trip defect.
binding_text name = case BindingScope::kind_of name of
  Some (BindingKind::Called) -> format! \\-> "#{name}()"
  _ -> .to_string name
''')

write('emit/is_shareable', '''
export(crate) is_shareable : &Expr -> bool
is_shareable body = case body of
  Expr::Num { .. } or Expr::Bool { .. } or Expr::Char { .. } or Expr::StrLit { .. } or Expr::RawLit { .. } or Expr::Path { .. } or Expr::Unit { .. } -> true
  Expr::Paren { inner, .. } -> is_shareable inner
  Expr::Unary { operand, .. } -> is_shareable operand
  Expr::Tuple { items, .. } or Expr::List { items, .. } -> .all is_shareable (.iter items)
  _ -> false
''')

write('emit/rust_hatch_text', '''
export(crate) rust_hatch_text : &str -> usize -> String
rust_hatch_text code indent =
  -- `padding`, where the Rust named the local `pad`: that is the name of the
  -- function one import over, and a binding should not take it.
  padding = .repeat (indent + RUST_INDENT) " "
  mut out = String::new ()
  -- `i` and `text` bound from the pair: a `for` binder is a bare name.
  for pair in .enumerate (.lines code)
    i = pair.0
    text = pair.1
    if i > 0
      .push '\\n' out
      if !.is_empty (.trim text)
        .push_str (&padding) out
    .push_str text out
  out
''')

write('emit/ascribed_arm', '''
export(crate) ascribed_arm : &Pattern -> &str -> Option<String>
ascribed_arm pattern ty =
  binder = (sole_binder pattern)?
  Some (format! \\-> "let #{binder}: #{ty} = #{binder};")
''')

write('emit/coalesce', '''
export(crate) coalesce : &Expr -> &Expr -> usize -> String
coalesce lhs rhs indent =
  arm_pad = .repeat (indent + RUST_INDENT) " "
  close_pad = .repeat indent " "
  scrutinee = emit_expr (unparen lhs) indent
  fallback = emit_expr (unparen rhs) (indent + RUST_INDENT)
  -- `{` and `}` are literal braces in a Redox string, so the emitter doubles
  -- them for Rust — which is what the Rust's `{{` and `}}` spelled by hand.
  format! \\-> "match #{scrutinee} {\\n#{arm_pad}Some(__rdx_v) => __rdx_v,\\n#{arm_pad}None => #{fallback},\\n#{close_pad}}"
''')

write('emit/missing_arity', '''
export(crate) missing_arity : &Expr -> &[Expr] -> Option<usize>
missing_arity callee args =
  guard unparen callee is Expr::Ident name else
    return None
  arity = (ArityScope::arity_of (&name.text))?
  lone_unit = case args of
    [Expr::Unit { .. }] -> true
    _ -> false
  if .is_empty args || lone_unit
    return None
  .then (\\-> arity - .len args) (.len args < arity)
''')

write('emit/chain', '''
export(crate) chain : &str -> &[String] -> usize -> String
chain receiver segments base =
  joined : String = .concat segments
  if .len joined <= CHAIN_WIDTH && !.contains '\\n' joined
    return format! \\-> "#{receiver}#{joined}"
  mut out = .to_string receiver
  for segment in segments
    .push '\\n' out
    pad_to (&mut out) (base + RUST_INDENT)
    .push_str segment out
  out
''')

write('emit/emit_condition', '''
export(crate) emit_condition : &Condition -> usize -> String
emit_condition condition indent =
  parts : Vec<String> = condition.operands
    |> .iter
    |> .map (\\operand ->
    case operand of
      ConditionOperand::Expr e -> emit_expr (unparen e) indent
      -- `let P = e`, with the pattern on the left in both languages — doc 18
      -- §3.1's "operands in Haskell's order" happens to be Rust's order too,
      -- so nothing is reordered.
      ConditionOperand::Pattern { pattern, source } ->
        lowered = .rust_text pattern
        value = emit_expr (unparen source) indent
        format! \\-> "let #{lowered} = #{value}")
    |> .collect
  .join " && " parts
''')

print('plan 81 batch 3: fourteen files')
