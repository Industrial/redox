# -*- coding: utf-8 -*-
"""Plan 80 batch 4: four more of `voltaic-driver`'s bodies.

`char_offset` is the first ported body with a **tuple binder in a `for`** —
`for (n, text) in source.lines().enumerate()`. A `for` binder is a bare name,
so the pair is bound and projected, which is the same treatment plan 76 gave
`data_text`'s `.enumerate`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('compile_dir', '''
export compile_dir : &Path -> &Path -> Result<Compiled, Vec<Diagnostic>>
compile_dir src out = compile_dir_with src out (&Silent)
''')

write('compile_dir/compiled', '''
instance Compiled
  -- | The `cargo::rerun-if-changed` lines a build script should print.
  -- |
  -- | Returned as lines rather than printed here: a library that writes to
  -- | stdout is a library that cannot be tested, and Cargo's directives are
  -- | stdout-sensitive.
  export rerun_directives : &self -> Vec<String>
  rerun_directives self =
    self.inputs
      |> .iter
      |> .map (\\p ->
      shown = .display p
      format! \\-> "cargo::rerun-if-changed=#{shown}")
      |> .collect

  export is_empty : &self -> bool
  is_empty self = .is_empty self.inputs

test "names every input for cargo"
  compiled = Compiled { inputs = vec! \\-> (PathBuf::from "src/a.sdx") \\-> (PathBuf::from "src/b.sdx"), outputs = vec! \\-> (PathBuf::from "out/a.rs") \\-> (PathBuf::from "out/b.rs") }
  assert_eq! \\-> .rerun_directives compiled \\-> vec! \\-> "cargo::rerun-if-changed=src/a.sdx" \\-> "cargo::rerun-if-changed=src/b.sdx"

test "an empty run asks for no reruns"
  empty = Compiled::default ()
  directives = .rerun_directives empty
  assert! \\-> .is_empty directives
  assert! \\-> .is_empty empty
''')

write('lint/shadow_walk', '''
instance ProgramWalk ShadowWalk
  -- | Nothing. A shadow is always a statement rebinding another statement's
  -- | name, so a declaration carries no site of its own — and two *items*
  -- | sharing a name is rustc's `E0428`, which is a check rustc makes.
  item : &mut self -> &Item -> ()
  item self _item = ()

  stmts : &mut self -> &[Stmt] -> ()
  stmts self stmts =
    mut bound : Vec<&str> = Vec::new ()
    for stmt in stmts
      -- Only a binding statement, because only a binding statement has an
      -- alternative spelling to suggest. `for x <- xs` scopes its binder to
      -- the loop body and is not a shadow at all; `n <- f`, `guard P = e`
      -- and a draw arrow do bind into the rest of the list, but none of them
      -- has a `:=` form — so a warning on one would name a problem and no
      -- fix. Plan 54 §6's residue, stated rather than widened.
      guard &stmt.expr is Expr::Binding binding else
        continue
      guard binding_name binding is Some ident else
        continue

      -- A **trailing** pragma is not any construct's leading trivia, so
      -- `suppressed_spans` cannot see it — the same split plan 38's line rule
      -- covers for `RDX1001`, and keyed on the statement here because a
      -- shadow site always has a statement of its own to hang it on.
      suppressed = .is_some_and (\\t -> allow_pragma t Lint::Shadow) (.as_ref stmt.trailing)

      if .contains (&.as_str ident.text) bound && !suppressed
        .push (ident.span, .clone ident.text) self.found
      -- Recorded whether or not it warned: a suppressed shadow still binds,
      -- and a third `=` on the same name is a fresh site.
      .push (&ident.text) bound
    ()
''')

write('remap/char_offset', '''
export(crate) char_offset : &str -> usize -> usize -> Option<usize>
char_offset source line col =
  if line == 0 || col == 0
    return None
  mut offset = 0
  -- `n` and `text` bound from the pair, where the Rust destructured
  -- `for (n, text) in …`: a `for` binder is a bare name.
  for pair in .enumerate (.lines source)
    n = pair.0
    text = pair.1
    if n + 1 == line
      if .count (.chars text) < col - 1
        return None
      within : usize = text |> .chars |> .take (col - 1) |> .map char::len_utf8 |> .sum
      return Some (offset + within)
    offset += .len text + 1
  None

-- | `#[cfg(test)]`: the source every test below resolves a position in.
#[cfg(test)]
const SRC : &str = "ab\\n  cd\\n"

test "resolves a position"
  assert_eq! \\-> char_offset SRC 1 1 \\-> Some 0
  assert_eq! \\-> char_offset SRC 2 3 \\-> Some 5
  assert_eq! \\-> &SRC[5 .. 7] \\-> "cd"

-- | One past the last character is a valid insertion point.
test "allows the position just past a line"
  assert_eq! \\-> char_offset SRC 1 3 \\-> Some 2

test "declines a column past the line"
  assert_eq! \\-> char_offset SRC 1 9 \\-> None

test "declines a line past the text"
  assert_eq! \\-> char_offset SRC 9 1 \\-> None

test "declines a zero line or column"
  assert_eq! \\-> char_offset SRC 0 1 \\-> None
  assert_eq! \\-> char_offset SRC 1 0 \\-> None

-- | Columns are counted in characters and offsets in bytes, so a multi-byte
-- | character must not make the result slice through one.
test "counts characters and returns bytes"
  assert_eq! \\-> char_offset "é x\\n" 1 2 \\-> Some 2
  assert_eq! \\-> &"é x\\n"[2 .. 3] \\-> " "
''')

print('plan 80 batch 4: four files')
