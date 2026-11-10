use crate::parse::parser::rewritten::rewritten;
use crate::parse::parser::rust_generic::rust_generic;
use crate::parse::parser::type_vars::type_vars;
use redox_ast::{Constraint, Signature};

/// Fill in a signature's generic list and bounds, and rewrite its types —
/// doc 18 §§2.2 and 5.3.
///
/// ```text
/// largest : Ord t => &[t] -> &t     fn largest<T>(xs: &[T]) -> &T
///                                    where
///                                        T: Ord,
/// ```
///
/// Three steps, in this order because each needs the last:
///
/// 1. **the variables**, from the types in order of first appearance, with
///    lifetimes before type parameters because rustc requires that order;
/// 2. **the bounds**, by matching each `=>` constraint to the variable it
///    names — a constraint naming a variable the signature does not use is
///    kept anyway, because rustc reports it and Redox has no name resolution
///    to do better (AGENTS.md);
/// 3. **the rewrite**, replacing each variable with its Rust spelling
///    throughout the parameter and return types.
///
/// A signature with no variables is left exactly as it was, which is what
/// keeps `add : i32 -> i32 -> i32` emitting the Rust it emitted before any
/// of this existed.
///
/// `reserved` names variables that are **not** the method's own: inside a
/// `class Shape t`, the `t` in `area : &t -> f64` is the implementing type,
/// so making it a generic parameter of `area` produces
/// `fn area<T>(__rdx_a0: &T)` where the trait wanted `fn area(&self)`. Found
/// exactly that way — the class example stopped compiling the moment
/// functions grew generics.
pub(crate) fn generalise(sig: &mut Signature, constraints: &[Constraint], reserved: &[String]) {
  let mut texts: Vec<&str> = sig
    .params
    .iter()
    .filter_map(|p| p.ty.as_ref().map(|t| t.text.as_str()))
    .collect();
  if let Some(ret) = &sig.ret {
    texts.push(ret.text.as_str());
  }

  let found: Vec<String> = type_vars(&texts)
    .into_iter()
    .filter(|v| !reserved.contains(v))
    .collect();
  if found.is_empty() {
    return;
  }

  sig.generics = found.iter().map(|v| rust_generic(v)).collect();
  sig.bounds = constraints
    .iter()
    .filter(|c| found.contains(&c.var.text))
    .map(|c| (rust_generic(&c.var.text), c.trait_name.text.clone()))
    .collect();

  for var in &found {
    if var.starts_with('\'') {
      continue;
    }
    let subject = rust_generic(var);
    for param in &mut sig.params {
      if let Some(ty) = &mut param.ty {
        ty.text = rewritten(&ty.text, var, &subject);
      }
    }
    if let Some(ret) = &mut sig.ret {
      ret.text = rewritten(&ret.text, var, &subject);
    }
  }
}
