use crate::emit::binding_kind::BindingKind;
use crate::emit::binding_table::classify::classify;
use redox_ast::Item;
use std::collections::BTreeMap;

/// A declaration's binding kind, or none when it is not a parameterless binding.
pub(crate) mod classify;
/// Whether a name is spelled the way Rust spells a constant.
pub(crate) mod is_upper_case;

/// Every **parameterless** binding a file declares, and which Rust item it
/// becomes.
///
/// Plan 59 (`docs/53 - 59 - Value Bindings.md`) leaf 1. This is `ArityTable`'s
/// sibling and deliberately a separate walk: that table answers *how many
/// parameters*, this one answers *what a name with none of them means*, and
/// plan 56 §11.1's safety argument — everything absent is emitted exactly as
/// it always was — only holds if neither table has to explain the other's
/// absences. A third table over the same items is where merging the walk stops
/// being optional, which is plan 50's rule about the third copy.
///
/// A `BTreeMap` for this crate's module-doc reason: the emitter's output is
/// byte-stable, and `examples.rs`'s
/// `the_emitter_contains_no_unordered_collection` reads the source rather than
/// the behaviour.
///
/// # Keyed twice, because leaf 3's set has two spellings
///
/// A binding is indexed under its bare name *and*, inside an **inline**
/// `module`, under its qualified path — `hello` and `inner::hello`, `deep` and
/// `inner::deeper::deep`. Both are use sites criterion 1 names, and an
/// `Expr::Path` carries verbatim text, so the qualified key is the only thing
/// that can match one.
///
/// A **bodiless** `module` contributes nothing. `ModuleDecl.items` is `None`,
/// `emit_module` writes `;`, and the referent is in another file — §11.1's
/// correction, and the same boundary plan 56 §11.8 withdrew its first draft
/// over.
///
/// A name declared twice with **different** kinds is dropped rather than
/// resolved, exactly as `ArityTable` drops a clashing arity: Redox has no name
/// resolution and the only answer that cannot be wrong is the one every
/// unknown name already gets.
#[derive(Debug, Clone, Default)]
pub(crate) struct BindingTable {
  /// Each indexed binding's kind, by bare name and by path.
  kinds: BTreeMap<String, BindingKind>,
}

impl BindingTable {
  /// Index the parameterless bindings `items` declares, inline `module`
  /// bodies included.
  pub(crate) fn from_items(items: &[Item]) -> Self {
    let mut table = Self::default();
    let mut ambiguous = Vec::new();
    table.gather(items, &[], &mut ambiguous);
    for name in ambiguous {
      table.kinds.remove(&name);
    }
    table
  }

  /// Index one item list under a module prefix, recording any name whose kind clashes.
  fn gather(&mut self, items: &[Item], prefix: &[String], ambiguous: &mut Vec<String>) {
    for item in items {
      match item {
        Item::Fn(decl) => {
          let Some(kind) = classify(decl) else {
            continue;
          };
          let bare = decl.name.text.clone();
          self.insert(bare.clone(), kind, ambiguous);
          if !prefix.is_empty() {
            let mut path = prefix.join("::");
            path.push_str("::");
            path.push_str(&bare);
            self.insert(path, kind, ambiguous);
          }
        }
        Item::Module(decl) => {
          if let Some(inner) = &decl.items {
            let mut nested = prefix.to_vec();
            nested.push(decl.name.text.clone());
            self.gather(inner, &nested, ambiguous);
          }
        }
        _ => {}
      }
    }
  }

  /// Record one name's kind, noting it as ambiguous if it already had another.
  fn insert(&mut self, key: String, kind: BindingKind, ambiguous: &mut Vec<String>) {
    if self
      .kinds
      .insert(key.clone(), kind)
      .is_some_and(|previous| previous != kind)
    {
      ambiguous.push(key);
    }
  }

  /// What `name` means in the file being emitted, when it declares exactly one
  /// parameterless binding by that name.
  pub(crate) fn kind_of(&self, name: &str) -> Option<BindingKind> {
    self.kinds.get(name).copied()
  }
}
