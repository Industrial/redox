use crate::lint::allow_pragma::allow_pragma;
use crate::lint::walk_member_for_pragmas::walk_member_for_pragmas;
use crate::lint::walk_stmts_for_pragmas::walk_stmts_for_pragmas;
use redox_ast::{ClauseBody, Item};
use redox_diag::Span;

/// Walk one item for suppression pragmas — plan 53 leaf 4.
///
/// A pragma leading the item suppresses the item's **whole span**, which is
/// criterion 5's declaration half: `FnDecl::span` runs from the signature
/// line to the end of the last clause, so one attribute above a declaration
/// covers every call site inside it.
///
/// Then the statements. Shaped after the parser's `walk_item_for_tails`,
/// which walks the same containment for `RDX0044` — a function's clauses and
/// their `where` locals, a class's and an instance's members, a `test` body,
/// and a `module`'s items recursively.
///
/// Two deliberate differences from that walk. A **guarded** clause's rows are
/// walked here: each row's value is a `Block` of one tail statement, so a
/// pragma can lead it, where `RDX0044` had no reason to look. And the items
/// that hold no statements are listed by name rather than matched by a
/// wildcard, so a new declaration kind has to be considered rather than
/// silently excluded.
pub(crate) fn walk_item_for_pragmas(item: &Item, found: &mut Vec<Span>) {
  if redox_ast::leading(item).iter().any(allow_pragma) {
    found.push(item.span());
  }

  match item {
    Item::Fn(decl) => {
      for clause in &decl.clauses {
        match &clause.body {
          ClauseBody::Plain(block) => walk_stmts_for_pragmas(&block.stmts, found),
          ClauseBody::Guarded(rows) => {
            for row in rows {
              walk_stmts_for_pragmas(&row.body.stmts, found);
            }
          }
        }
        // `where` locals are statements of the clause's own scope, so a
        // pragma leads one exactly as it leads a body statement.
        walk_stmts_for_pragmas(&clause.wheres, found);
      }
    }
    Item::Class(decl) => {
      for member in &decl.members {
        walk_member_for_pragmas(member, found);
      }
    }
    Item::Instance(decl) => {
      for member in &decl.members {
        walk_member_for_pragmas(member, found);
      }
    }
    Item::Test(decl) => walk_stmts_for_pragmas(&decl.body.stmts, found),
    // The one item that contains items, so the one that recurses.
    Item::Module(decl) => {
      for inner in decl.items.iter().flatten() {
        walk_item_for_pragmas(inner, found);
      }
    }
    // No statement blocks. Inline Rust is never parsed, so it holds no call
    // site Redox could lint and no pragma Redox could read.
    Item::Data(_)
    | Item::Type(_)
    | Item::Errors(_)
    | Item::Import(_)
    | Item::Const(_)
    | Item::InlineRust(_)
    | Item::Extern(_) => {}
  }
}
