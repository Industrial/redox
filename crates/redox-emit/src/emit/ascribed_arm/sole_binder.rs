use crate::emit::ascribed_arm::collect::collect;
use redox_ast::Pattern;

/// The one name `pattern` binds, when it binds exactly one.
///
/// Plan 62 (`docs/56 - 62 - Ascription Completion.md`) leaf 3. An arm's
/// ascription needs a binder to attach to, because rustc will not take a type
/// in a `match` pattern and the lowering is a `let` in the arm body.
///
/// Zero binders, or more than one, yields `None`. That is the leaf's disclosed
/// limit: a wrong binder would compile and annotate the wrong thing, which is
/// worse than declining, and there is no syntax saying which of several the
/// author meant.
pub(crate) fn sole_binder(pattern: &Pattern) -> Option<String> {
  let mut found = Vec::new();
  collect(pattern, &mut found);
  match found.as_slice() {
    [one] => Some(one.clone()),
    _ => None,
  }
}
