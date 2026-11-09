use redox_ast::{Constraint, TypeApp};

/// The `<T: Display, U>` list an `instance` needs, or an empty string.
///
/// Doc 18 §5.3: **"type variables need no declaration … the generic list is
/// synthesised from first appearance"**. The variables come from the target's
/// arguments in source order, and each one's bounds are the constraints that
/// name it, in the order they were written.
///
/// Order is taken from the source rather than from a map, because the
/// emitter's output has to be byte-stable (doc 01 §3.4, and the goldens
/// compare exact bytes). A `HashMap` here would make the generic list depend
/// on iteration order, which is the defect `redox-emit`'s own module doc
/// forbids in as many words.
///
/// A constraint naming a variable the target does not use is emitted anyway,
/// as a bound on nothing — rustc reports it, and that is the right place for
/// it: Redox has no name resolution and must not grow one (AGENTS.md).
pub(crate) fn generics(target: &TypeApp, constraints: &[Constraint]) -> String {
  let vars = target.vars();
  if vars.is_empty() {
    return String::new();
  }

  let params: Vec<String> = vars
    .iter()
    .map(|v| {
      let bounds: Vec<&str> = constraints
        .iter()
        .filter(|c| c.var.text == *v)
        .map(|c| c.trait_name.text.as_str())
        .collect();
      let name = TypeApp::rust_name(v);
      if bounds.is_empty() {
        return name;
      }
      format!("{name}: {}", bounds.join(" + "))
    })
    .collect();

  format!("<{}>", params.join(", "))
}
