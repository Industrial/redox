use crate::parse::parser::receiver_text::receiver_text;
use redox_ast::{Ident, Param, Signature};

/// Turn a method's first parameter into Rust's receiver, where it is one.
///
/// Doc 18 §2.5: **"the first parameter typed by the class variable lowers to
/// `self`"** — the one place Haskell's dictionary-passing and Rust's receiver
/// differ in spelling but not in content.
///
/// ```text
/// area :: &t -> f64            fn area(&self) -> f64
/// increment :: &mut Self -> () fn increment(&mut self)
/// consume :: t -> u32          fn consume(self) -> u32
/// ```
///
/// The borrow form comes from the *type*, not from a sigil the author writes,
/// which is what keeps `&`, `&mut` and by-value all expressible with one
/// notation. `var` is the class variable; `Self` is accepted alongside it
/// because §2.5's own `instance Counter` example writes `Self` — an inherent
/// `instance` has no class variable to name.
///
/// Only the **first** parameter is converted, and the decision is made from
/// its *type*, not from its name. The type is what doc 18 §2.5 names, and it
/// is also the only thing available: a required method —
/// `area :: &t -> f64` with no equation — has no binder to inspect, and it
/// needs the same receiver as the default method below it.
///
/// A method whose first parameter is not typed by the class variable keeps
/// it, which is how an associated function is written: Rust spells that one
/// with no receiver at all, and so does this.
pub(crate) fn receiver(sig: &mut Signature, names: &[String]) {
  let Some(first) = sig.params.first() else {
    return;
  };
  let Some(ty) = &first.ty else { return };

  let Some(text) = receiver_text(&ty.text, names) else {
    return;
  };
  let span = first.span;
  sig.params[0] = Param {
    // The whole receiver lives in the name, with no type, which is the shape
    // `Param` already had for `self` — the emitter writes a parameter's name
    // and then its type only if there is one.
    name: Ident { text, span },
    ty: None,
    is_mut: false,
    span,
  };
}
