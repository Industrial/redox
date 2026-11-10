use crate::lint::walk_item_for_pragmas::walk_item_for_pragmas;
use redox_ast::{ClassMember, Item};
use redox_diag::Span;

/// Walk one class or instance member for suppression pragmas — plan 53
/// leaf 4.
///
/// A method **is** a function, carrying its own `leading` trivia and its own
/// span, so it is walked as one. An associated type and an associated const
/// hold no statements and no trivia of their own.
pub(crate) fn walk_member_for_pragmas(member: &ClassMember, found: &mut Vec<Span>) {
  if let ClassMember::Method(decl) = member {
    walk_item_for_pragmas(&Item::Fn(decl.clone()), found);
  }
}
