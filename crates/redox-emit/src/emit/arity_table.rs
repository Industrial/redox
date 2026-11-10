use redox_ast::Item;
use std::collections::BTreeMap;

/// The declared arity of every function a file names, by name.
///
/// Plan 56 (`docs/50 - 56 - Currying.md`) leaf 1. Doc 18 §2.2's signature
/// **states** the arity — `add : i32 -> i32 -> i32` has two top-level arrows
/// and therefore two parameters — and `parse_spine` already split that spine,
/// already ignoring the arrows nested inside a verbatim type (doc 03 §14.1).
/// So §1.2's precondition costs nothing: the number is
/// `FnDecl.sig.params.len()`, and this is only the index over it. No symbol
/// table, no resolution, no analysis Redox does not already perform.
///
/// A `BTreeMap` rather than a `HashMap`, for this crate's module-doc reason:
/// the emitter's output is byte-stable, and `examples.rs`'s
/// `the_emitter_contains_no_unordered_collection` reads the source rather than
/// the behaviour — which is that check working as intended, since a map that
/// is only ever looked up would pass a behavioural test and still be the
/// wrong habit.
///
/// # What is deliberately absent, and why the absence is the safety argument
///
/// Only `Item::Fn`, and only under the name a **bare identifier** would spell.
/// §11.1 is the reason: arity is knowable for a signature in this file and for
/// nothing else, so every `Expr::Path`, every `where` local, every equation
/// binder, every lambda parameter and every name reached through `import` is
/// absent — and everything absent is emitted as a call exactly as it always
/// was, with rustc judging it. That is §5 criterion 5's second half, and it is
/// why this row needs no diagnostic: there is no decidable trigger for one.
///
/// Class and instance members are absent for a sharper version of the same
/// reason. They are reached through a receiver or a path, so a bare-name arity
/// for them would describe a call that cannot be written.
///
/// An **inline** `module` body *is* walked, because a bare call inside one
/// reaches that module's own functions. A bodiless `module` carries nothing to
/// walk — `ModuleDecl.items` is `None` and `emit_module` writes `;` — which is
/// §11.8's correction of leaf 1's first draft, and §1.4's boundary: currying is
/// available where the signature is in the file being compiled.
///
/// A name declared twice with **different** arities is dropped rather than
/// resolved. Redox has no name resolution and must not grow one, so a nested
/// `hello : &'static str` beside a top-level `hello : a -> b` leaves nothing
/// able to say which one a bare `hello` means — and the only answer that
/// cannot be wrong is the one this crate already gives every unknown name.
#[derive(Debug, Clone, Default)]
pub(crate) struct ArityTable {
  /// Each indexed function's declared parameter count, by name.
  arities: BTreeMap<String, usize>,
}

impl ArityTable {
  /// Index the signatures `items` declares, inline `module` bodies included.
  pub(crate) fn from_items(items: &[Item]) -> Self {
    let mut table = Self::default();
    let mut ambiguous = Vec::new();
    table.gather(items, &mut ambiguous);
    for name in ambiguous {
      table.arities.remove(&name);
    }
    table
  }

  /// Index one item list, recording any name whose arity clashes.
  fn gather(&mut self, items: &[Item], ambiguous: &mut Vec<String>) {
    for item in items {
      match item {
        Item::Fn(decl) => {
          let arity = decl.sig.params.len();
          let clash = self
            .arities
            .insert(decl.name.text.clone(), arity)
            .is_some_and(|previous| previous != arity);
          if clash {
            ambiguous.push(decl.name.text.clone());
          }
        }
        Item::Module(decl) => {
          if let Some(inner) = &decl.items {
            self.gather(inner, ambiguous);
          }
        }
        _ => {}
      }
    }
  }

  /// The declared arity of `name`, when this file declares exactly one
  /// function by that name.
  pub(crate) fn arity_of(&self, name: &str) -> Option<usize> {
    self.arities.get(name).copied()
  }
}
