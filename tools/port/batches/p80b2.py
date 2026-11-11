# -*- coding: utf-8 -*-
"""Plan 80 batch 2: the two `class` declarations, and three bodies around them.

`redox-driver` is the first ported crate with a **trait of its own** —
`ProgramWalk` and `CacheReporter` — so these are the first `class`
declarations a port has written. `examples/class.sdx` has the shape: a member
is `name : type` with no equation, and a default is an ordinary one.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lint/program_walk', '''
export(crate) class ProgramWalk
  -- | One item, before anything it contains.
  -- |
  -- | Reached for a nested `module`'s items and for a class or instance
  -- | method, which *is* a function and is walked as one.
  item : &mut self -> &Item -> ()

  -- | One statement run, in source order.
  -- |
  -- | A clause body, a guard row's value, a clause's `where` locals, a `test`
  -- | body, or a block some statement owns. Each call is **one list** — which
  -- | is the unit `RDX1003` is defined over, and the reason it cannot see
  -- | across a block boundary.
  stmts : &mut self -> &[Stmt] -> ()
''')

write('cache/reporter', '''
export class CacheReporter
  -- | `label` names the file; `hit` is whether its result was reused.
  report : &self -> &str -> bool -> ()

-- | The shape a test uses: collect, do not print.
-- |
-- | `#[cfg(test)]`, and at file scope: Redox puts nothing but `#[test] fn`s
-- | inside its generated `mod __rdx_tests` (plan 73 §2.9).
#[cfg(test)]
data Collect = Collect std::cell::RefCell<Vec<String>>

#[cfg(test)]
instance CacheReporter Collect
  report self label hit =
    outcome = if hit
      "HIT"
    else
      "MISS"
    .push (format! \\-> "#{label} #{outcome}") (.borrow_mut self.0)
    ()

test "a reporter sees both outcomes"
  collect = Collect (std::cell::RefCell::new (Vec::new ()))
  .report "a.sdx" false collect
  .report "a.sdx" true collect
  assert_eq! \\-> .as_slice (.borrow collect.0) \\-> ["a.sdx MISS", "a.sdx HIT"]
''')

write('lint/pragma_walk', '''
instance ProgramWalk PragmaWalk
  -- | A pragma leading a declaration suppresses the declaration's **whole
  -- | span** — `FnDecl::span` runs from the signature line to the end of the
  -- | last clause, so one attribute above it covers every site inside.
  -- |
  -- | The signature is restated because `ProgramWalk` is declared in another
  -- | file: `RDX0101` asks for it, and `CacheReporter`'s instance needs none
  -- | because its class is declared directly above it.
  item : &mut self -> &Item -> ()
  item self item =
    if .any (\\t -> allow_pragma t self.lint) (.iter (redox_ast::leading item))
      .push (.span item) self.found
    ()

  -- | A pragma leading a statement suppresses that statement's **own span**,
  -- | which runs to the end of everything the statement owns — so an attribute
  -- | above an `if` covers its branches, and one above a `for` covers its
  -- | body.
  stmts : &mut self -> &[Stmt] -> ()
  stmts self stmts =
    for stmt in stmts
      if .any (\\t -> allow_pragma t self.lint) (.iter stmt.leading)
        .push stmt.span self.found
    ()
''')

write('lint/shadow_sites', '''
export(crate) shadow_sites : &Program -> Vec<(Span, String)>
shadow_sites program =
  mut walk = ShadowWalk { found = Vec::new () }
  walk_program program (&mut walk)

  ranges = suppressed_spans program Lint::Shadow
  walk.found
    |> .into_iter
    -- `span` bound first, where the Rust destructured `|(span, _)|`: a
    -- lambda parameter is a bare name, and `pair.0.0` would lex `0.0` as a
    -- float.
    |> .filter (\\pair ->
    span = pair.0
    !.any (\\r -> r.0 <= span.0 && span.1 <= r.1) (.iter ranges))
    |> .collect
''')

write('remap/generated_offset', '''
export(crate) generated_offset : &str -> usize -> usize -> usize
generated_offset source line col =
  .unwrap_or_else (\\-> .saturating_sub 1 (.len source)) (char_offset source line col)

-- | `#[cfg(test)]`: the source every test below resolves a position in.
#[cfg(test)]
const SRC : &str = "fn main() {\\n}\\n"

test "resolves a real position"
  assert_eq! \\-> generated_offset SRC 2 1 \\-> 12
  -- The character rather than the slice, and **not** a simplification: a
  -- string literal reaching a macro is brace-doubled (`macro_argument`), so
  -- `assert_eq! \\-> &SRC[12 .. 13] \\-> "}"` emits `"}}"` and compares the
  -- slice against two braces. A `char` literal is not a format template and
  -- is passed through, so this asserts exactly what the Rust did. Plan 80
  -- §2.4 has the defect; it is owed a row.
  assert_eq! \\-> .next (.chars SRC[12 .. 13]) \\-> Some '}'

test "clamps a position past the end"
  assert_eq! \\-> generated_offset SRC 99 99 \\-> .len SRC - 1

test "survives an empty source"
  assert_eq! \\-> generated_offset "" 1 1 \\-> 0
''')

print('plan 80 batch 2: five files')
