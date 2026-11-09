use crate::ParseError;
use crate::parse::parser::return_in_tail::return_in_tail;
use crate::parse::parser::walk_member_for_tails::walk_member_for_tails;
use redox_ast::Item;

/// Walk one item for `RDX0044` — plan 30. A sibling of
/// [`check_callback_tails`](super::check_callback_tails) rather than a
/// private helper, because doc 06 §1 permits one construct per file.
pub(crate) fn walk_item_for_tails(item: &Item, errors: &mut Vec<ParseError>) {
  match item {
    Item::Fn(decl) => {
      for clause in &decl.clauses {
        if let Some(block) = clause.body.plain() {
          errors.extend(return_in_tail(block));
        }
        // A guarded clause's bodies are expressions, not blocks, so a draw
        // arrow cannot appear in one — doc 18 §2.3's guard list takes an
        // expression per arm.
        //
        // `where` bindings are statements of the clause's own scope, and a
        // draw arrow in one would flatten the rest of that scope, so they are
        // walked as a run.
        if !clause.wheres.is_empty() {
          errors.extend(return_in_tail(&redox_ast::Block {
            stmts: clause.wheres.clone(),
            span: clause.span,
          }));
        }
      }
    }
    Item::Class(decl) => {
      for member in &decl.members {
        walk_member_for_tails(member, errors);
      }
    }
    Item::Instance(decl) => {
      for member in &decl.members {
        walk_member_for_tails(member, errors);
      }
    }
    Item::Test(decl) => errors.extend(return_in_tail(&decl.body)),
    // The one item that contains items, so the one that recurses.
    Item::Module(decl) => {
      for inner in decl.items.iter().flatten() {
        walk_item_for_tails(inner, errors);
      }
    }
    // Inline Rust is never parsed, so it holds no callback tail to check
    // — plan 45 leaf 1.
    Item::Data(_)
    | Item::Type(_)
    | Item::Errors(_)
    | Item::Import(_)
    | Item::Const(_)
    | Item::InlineRust(_)
    | Item::Extern(_) => {}
  }
}
