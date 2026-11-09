use redox_ast::Expr;

/// Whether a record literal's head names a **constructor** rather than a value.
///
/// Doc 18 §5.3 makes case meaningful — constructors and types begin
/// uppercase, values and type variables lowercase — and §2.4 says that is
/// "why the update form is admissible at all": `Point { x = 1 }` builds and
/// `p { x = 1 }` updates, told apart by nothing but the case of the head.
///
/// A path is judged by its **last** segment, so `shapes::Point { … }` builds
/// and `self.origin { … }` updates. Decidable from the source text alone,
/// which is doc 03 §2's admission test 1 and the reason this needs no name
/// resolution.
pub(crate) fn names_a_constructor(head: &Expr) -> bool {
  let text = match head {
    Expr::Ident(i) => i.text.as_str(),
    Expr::Path { text, .. } => text.rsplit("::").next().unwrap_or(text),
    _ => return false,
  };
  text.chars().next().is_some_and(char::is_uppercase)
}
