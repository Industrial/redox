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
///
/// The walk itself is `Pattern::binders`, in `redox-ast`. Plan 62 wrote it
/// here as `collect`; plan 61 needs the same answer in the **parser**, so it
/// moved to the crate both can see rather than being copied.
pub(crate) fn sole_binder(pattern: &Pattern) -> Option<String> {
  match pattern.binders().as_slice() {
    [one] => Some(one.clone()),
    _ => None,
  }
}
