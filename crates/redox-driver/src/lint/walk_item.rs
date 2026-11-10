use crate::lint::program_walk::ProgramWalk;
use crate::lint::walk_member::walk_member;
use crate::lint::walk_stmts::walk_stmts;
use redox_ast::{ClauseBody, Item};

/// Walk one item — plan 53 leaf 4, shared by plan 54 leaf 3.
///
/// The item is offered first, which is criterion 5's declaration half for the
/// suppression consumer: `FnDecl::span` runs from the signature line to the
/// end of the last clause, so one attribute above a declaration covers
/// everything inside it.
///
/// Then the statements. Shaped after the parser's `walk_item_for_tails`,
/// which walks the same containment for `RDX0044` — a function's clauses and
/// their `where` locals, a class's and an instance's members, a `test` body,
/// and a `module`'s items recursively.
///
/// Two deliberate differences from that walk. A **guarded** clause's rows are
/// walked here: each row's value is a `Block` of one tail statement, so a
/// pragma can lead it and a binding can live in it, where `RDX0044` had no
/// reason to look. And the items that hold no statements are listed by name
/// rather than matched by a wildcard, so a new declaration kind has to be
/// considered rather than silently excluded.
pub(crate) fn walk_item(item: &Item, walk: &mut dyn ProgramWalk) {
  walk.item(item);

  match item {
    Item::Fn(decl) => {
      for clause in &decl.clauses {
        match &clause.body {
          ClauseBody::Plain(block) => walk_stmts(&block.stmts, walk),
          ClauseBody::Guarded(rows) => {
            for row in rows {
              walk_stmts(&row.body.stmts, walk);
            }
          }
        }
        // `where` locals are statements of the clause's own scope, so a
        // pragma leads one exactly as it leads a body statement — and a
        // binding shadows within them on the same terms.
        walk_stmts(&clause.wheres, walk);
      }
    }
    Item::Class(decl) => {
      for member in &decl.members {
        walk_member(member, walk);
      }
    }
    Item::Instance(decl) => {
      for member in &decl.members {
        walk_member(member, walk);
      }
    }
    Item::Test(decl) => walk_stmts(&decl.body.stmts, walk),
    // The one item that contains items, so the one that recurses.
    Item::Module(decl) => {
      for inner in decl.items.iter().flatten() {
        walk_item(inner, walk);
      }
    }
    // No statement blocks. Inline Rust is never parsed, so it holds no call
    // site Redox could lint, no binding Redox could see and no pragma Redox
    // could read.
    Item::Data(_)
    | Item::Type(_)
    | Item::Errors(_)
    | Item::Import(_)
    | Item::Const(_)
    | Item::InlineRust(_)
    | Item::Extern(_) => {}
  }
}
