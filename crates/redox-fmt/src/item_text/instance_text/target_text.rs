use redox_ast::TypeApp;

/// The implemented type, in Redox's spelling.
///
/// `TypeApp::rust_text` writes `Head<A, B>`; Redox juxtaposes, so this writes
/// `Head a b` and parenthesises nothing — a type argument is a single `Ident`
/// by construction, so there is no nested application to group.
///
/// The `args` keep the **source's** spelling of a type variable, because
/// `rust_text` applies the uppercasing when it writes rather than storing it.
/// That is exactly the design a signature does not have, and the reason an
/// `instance` can be printed from the tree while a signature cannot.
pub(crate) fn target_text(target: &TypeApp) -> String {
  let mut out = target.head.text.clone();
  for arg in &target.args {
    out.push(' ');
    out.push_str(&arg.text);
  }
  out
}
