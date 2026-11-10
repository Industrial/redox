use crate::parse::parser::receiver_text::receiver_text;
use redox_ast::{Ident, Param, Signature};

/// Turn the parameter **marked** `self` into Rust's receiver, where there is
/// one.
///
/// Doc 18 §2.5, as plan 67 rewrote it: **`self` in a type is the receiver**,
/// at whatever position it is written.
///
/// ```text
/// area : &self -> f64           fn area(&self) -> f64
/// retitle : &mut self -> ()     fn retitle(&mut self)
/// consume : self -> u32         fn consume(self) -> u32
/// scaled : &self -> f64 -> Self fn scaled(&self, k: f64) -> Self
/// scaled : f64 -> &self -> Self the same Rust, from the other order
/// unit : f64 -> Self            fn unit(__rdx_a0: f64) -> Self
/// ```
///
/// The borrow form comes from the *type*, not from a sigil the author writes,
/// which is what keeps `&`, `&mut` and by-value all expressible with one
/// notation. What plan 67 deleted is the other half: which parameter is the
/// receiver used to be *position* one plus whichever of up to three spellings
/// happened to name the implementing type — §1.1.
///
/// The decision is made from the *type*, not from the binder's name, and that
/// is not a choice: a required method — `area : &self -> f64` with no
/// equation — has no binder to inspect, and it needs the same receiver as the
/// default method below it.
///
/// A method with no `self` in its types keeps all its parameters, which is
/// how an associated function is written: Rust spells that one with no
/// receiver at all, and so does this.
///
/// **The marked parameter is rewritten in place and not moved.** Rust writes
/// the receiver first and `emit::signature` is what puts it there — see that
/// function for the measurement. Moving it here would leave the signature's
/// parameters in Rust's order and each clause's pattern row in the source's,
/// which is the pairing `emit_clauses` builds a `match` out of; and moving
/// the pattern row to match would be a tree rewrite `redox-fmt` prints, since
/// `clause_text` prints an equation's patterns from the tree where
/// `signature_line` slices the signature from the source. Plan 61 §11.6's
/// two-consumers rule does bind for the pattern row even though plan 67 §1.7
/// measured it not binding for the signature line.
pub(crate) fn receiver(sig: &mut Signature) {
  let Some(at) = sig.params.iter().position(|p| {
    p.ty
      .as_ref()
      .is_some_and(|ty| receiver_text(&ty.text).is_some())
  }) else {
    return;
  };
  let text = sig.params[at]
    .ty
    .as_ref()
    .and_then(|ty| receiver_text(&ty.text))
    .expect("the position just matched");
  let span = sig.params[at].span;
  sig.params[at] = Param {
    // The whole receiver lives in the name, with no type, which is the shape
    // `Param` already had for `self` — the emitter writes a parameter's name
    // and then its type only if there is one.
    name: Ident { text, span },
    ty: None,
    is_mut: false,
    span,
  };

  // A **generated** name — doc 18 §2.3's `__rdx_aN`, which a required method
  // or a testing pattern gets instead of a binder — is numbered by its
  // position in the *Rust* signature rather than in the spine. That is what
  // makes `scaled : &self -> f64 -> Self` and `scaled : f64 -> &self -> Self`
  // byte-identical with no equation to supply a name: both give
  // `fn scaled(&self, __rdx_a1: f64)`. Nothing existing moves, because a
  // receiver written first already numbered this way.
  if at > 0 {
    let mut order: Vec<usize> = (0..sig.params.len()).collect();
    order.remove(at);
    order.insert(0, at);
    for (rust_at, from) in order.into_iter().enumerate() {
      let name = &mut sig.params[from].name.text;
      if name.starts_with("__rdx_a") {
        *name = format!("__rdx_a{rust_at}");
      }
    }
  }
}
