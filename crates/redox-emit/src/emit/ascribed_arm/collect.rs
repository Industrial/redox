use redox_ast::Pattern;

/// Every name `pattern` binds, in source order.
///
/// Plan 62 (`docs/56 - 62 - Ascription Completion.md`) leaf 3, serving
/// [`super::sole_binder`]. A testing pattern — a literal, a range, a wildcard,
/// a rest — binds nothing and contributes nothing, which is why the catch-all
/// arm is correct rather than lazy: those are the variants that have no name
/// to give.
pub(crate) fn collect(pattern: &Pattern, into: &mut Vec<String>) {
  match pattern {
    Pattern::Binding(name) | Pattern::NamedRest { name, .. } => into.push(name.text.clone()),
    Pattern::As { name, inner, .. } => {
      into.push(name.text.clone());
      collect(inner, into);
    }
    Pattern::Apply { args, .. } => {
      for arg in args {
        collect(arg, into);
      }
    }
    Pattern::Tuple { items, .. } | Pattern::Slice { items, .. } => {
      for item in items {
        collect(item, into);
      }
    }
    Pattern::Or(items) => {
      for item in items {
        collect(item, into);
      }
    }
    _ => {}
  }
}
