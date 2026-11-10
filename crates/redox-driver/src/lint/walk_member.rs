use crate::lint::program_walk::ProgramWalk;
use crate::lint::walk_item::walk_item;
use redox_ast::{ClassMember, Item};

/// Walk one class or instance member — plan 53 leaf 4, shared by plan 54
/// leaf 3.
///
/// A method **is** a function, carrying its own `leading` trivia and its own
/// span, so it is walked as one. An associated type and an associated const
/// hold no statements and no trivia of their own.
pub(crate) fn walk_member(member: &ClassMember, walk: &mut dyn ProgramWalk) {
  if let ClassMember::Method(decl) = member {
    walk_item(&Item::Fn(decl.clone()), walk);
  }
}
