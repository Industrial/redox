use crate::lint::program_walk::ProgramWalk;
use crate::lint::walk_item::walk_item;
use redox_ast::Program;

/// Walk a whole program — plan 53 leaf 4, shared by plan 54 leaf 3.
///
/// The entry point every lint that needs the tree goes through, so there is
/// one answer to *what does a lint see*. What it does with what it sees is
/// [`ProgramWalk`]'s.
pub(crate) fn walk_program(program: &Program, walk: &mut dyn ProgramWalk) {
  for item in &program.items {
    walk_item(item, walk);
  }
}
