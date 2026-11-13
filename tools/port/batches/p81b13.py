# -*- coding: utf-8 -*-
"""Plan 81 batch 13: three more of `voltaic-emit`'s bodies."""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('emit/trivia', '''
export(crate) emit_trivia : &mut Sink<'_> -> &[Trivia] -> usize -> ()
emit_trivia out trivia indent =
  for t in trivia
    -- Consumed before anything is marked or written, so the pragma leaves no
    -- trace in the `.xrs` and none in the span map either.
    --
    -- The `matches!` as a `case`, because a macro's second argument cannot be
    -- a pattern.
    is_attribute = case t.kind of
      TriviaKind::Attribute -> true
      _ -> false
    if is_attribute && .is_some (Lint::from_allow_attribute (.trim t.text))
      continue
    -- A comment the author wrote has a source position of its own, so the
    -- generated comment line maps to it. Plan 06's totality test found this:
    -- leaving trivia unmarked made every comment line in the output
    -- "synthetic", which is false — the author wrote it, and rustc can
    -- complain about a doc comment.
    case t.kind of
      TriviaKind::BlankLines -> .unmark out
      _ -> .mark t.span out
    case t.kind of
      TriviaKind::LineComment -> line out indent "//" (&t.text)
      TriviaKind::DocComment -> line out indent "///" (&t.text)
      TriviaKind::InnerDocComment -> line out indent "//!" (&t.text)
      TriviaKind::FencedComment ->
        pad out indent
        _dropped = writeln! \\-> out \\-> "/* #{t.text} */"
      -- Doc 03 §13.2: verbatim. The brackets are written here rather than
      -- stored, because an inner attribute needs different ones and the text
      -- is the same either way.
      TriviaKind::Attribute ->
        pad out indent
        _dropped = writeln! \\-> out \\-> "\\#[#{t.text}]"
      TriviaKind::InnerAttribute ->
        pad out indent
        _dropped = writeln! \\-> out \\-> "\\#![#{t.text}]"
      TriviaKind::BlankLines ->
        if !.is_empty out && !.ends_with "{\\n" out && !.ends_with "\\n\\n" out
          .push '\\n' out
  ()
''')

write('emit/emit_clauses', '''
export(crate) emit_clauses : &mut Sink<'_> -> &FnDecl -> usize -> ()
emit_clauses out f base =
  -- A `()`-returning function's body is not a tail position: its last element
  -- is §1.6's written `()`, and the elements before it are statements. Spelled
  -- `ret.is_some()` until plan 55 leaf 4 recorded a declared `()` in the tree,
  -- which made that spelling mean something else.
  tail = !.returns_unit f.sig
  simple = .len f.clauses == 1 && .all voltaic_ast::Pattern::is_irrefutable (.iter f.clauses[0].patterns)
  if simple
    -- `where` locals come **first**, which is the only order Rust allows: a
    -- `let` has to precede its use. Haskell writes them after the expression
    -- that uses them, and moving them up is half of doc 03 §8.3's lowering.
    --
    -- The other half is plan 61: they are also emitted in **dependency
    -- order**, so the author's order does not have to be Rust's. The sentence
    -- this comment used to end with — *"that reordering is the whole of the
    -- lowering"* — was true and is one of four copies the row amends.
    emit_where_locals out (&f.clauses[0].wheres) (base + RUST_INDENT)
    -- The body is the function's own, so a narrow two-armed guard ladder may
    -- go on one line — unless a `where` local precedes it, which is a
    -- position rustfmt's behaviour has not been measured at and so is not
    -- claimed.
    joinable = .is_empty f.clauses[0].wheres
    emit_body out (&f.clauses[0].body) (base + RUST_INDENT) tail joinable
    return ()
  names : Vec<&str> = f.sig.params |> .iter |> .map (\\p -> .as_str p.name.text) |> .collect
  scrutinee = case .as_slice names of
    [one] -> .to_string (*one)
    many -> format! \\-> "(#{.join ", " many})"
  pad out (base + RUST_INDENT)
  .push_str (&format! \\-> "match #{scrutinee} {\\n") out
  for clause in &f.clauses
    row : Vec<String> = clause.patterns |> .iter |> .map voltaic_ast::Pattern::rust_text |> .collect
    arm = case .as_slice row of
      [one] -> .clone one
      many -> format! \\-> "(#{.join ", " many})"
    emit_arm out clause (&arm) (base + RUST_INDENT * 2) tail
  pad out (base + RUST_INDENT)
  .push_str "}\\n" out
  ()
''')

write('emit/signature', '''
export(crate) signature : &Signature -> usize -> String
signature sig indent =
  generic_list = if .is_empty sig.generics
    String::new ()
  else
    joined = .join ", " sig.generics
    format! \\-> "<#{joined}>"
  mut ordered : Vec<&Param> = .collect (.iter sig.params)
  if .position (\\p -> .is_none p.ty) (.iter ordered) is Some at
    marked = .remove at ordered
    .insert 0 marked ordered
  params : Vec<String> = ordered
    |> .iter
    |> .map (\\p ->
    mut text = String::new ()
    if p.is_mut
      .push_str "mut " text
    .push_str (&p.name.text) text
    if &p.ty is Some ty
      .push_str ": " text
      .push_str (&ty.text) text
    text)
    |> .collect
  joined = .join ", " params
  mut out = format! \\-> "#{generic_list}(#{joined})"
  -- **A unit return is omitted**, not written `-> ()`: rustfmt removes the
  -- arrow, so writing it would be output rustfmt rewrites. The decision lives
  -- here rather than in the parser since plan 55 leaf 4 — the tree records
  -- what the author declared, and §1.6's unit rule needs that.
  if &sig.ret is Some ret && !.returns_unit sig
    .push_str " -> " out
    .push_str (&ret.text) out
  -- Doc 18 §2.2's lowering puts the bounds in a `where` clause rather than
  -- inline in the generic list. One bound per line with a trailing comma,
  -- which is the form rustfmt leaves alone.
  --
  -- No trailing newline: the caller decides whether the body's `{` follows a
  -- space or a line break, and it has to be a line break after a `where`
  -- clause. Ending the clause here with a newline put a stray space before
  -- the brace, which rustfmt removes — so the emitter was producing output
  -- rustfmt rewrites.
  if !.is_empty sig.bounds
    -- `padding`, where the Rust named the local `pad`: that is the name of
    -- the function one import over.
    padding = .repeat indent " "
    .push_str (&format! \\-> "\\n#{padding}where") out
    -- `pair.0` is the parameter and `pair.1` the bound, where the Rust
    -- destructured `for (param, bound) in …`.
    for pair in &sig.bounds
      .push_str (&format! \\-> "\\n#{padding}    #{pair.0}: #{pair.1},") out
  out
''')

print('plan 81 batch 13: three files')
