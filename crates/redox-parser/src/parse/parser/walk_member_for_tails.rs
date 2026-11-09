use crate::ParseError;
use crate::parse::parser::walk_item_for_tails::walk_item_for_tails;

/// Walk one class or instance member for `RDX0044` — plan 30.
pub(crate) fn walk_member_for_tails(member: &redox_ast::ClassMember, errors: &mut Vec<ParseError>) {
  if let redox_ast::ClassMember::Method(decl) = member {
    walk_item_for_tails(&redox_ast::Item::Fn(decl.clone()), errors);
  }
}
