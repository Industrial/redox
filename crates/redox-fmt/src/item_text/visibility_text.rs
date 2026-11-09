use redox_ast::Visibility;

/// A visibility marker, with its trailing space — or nothing.
///
/// Redox says `export` where Rust says `pub`, which is doc 18's own choice and
/// `examples/visibility.sdx`'s spelling: `export data Point`,
/// `export(crate) type Depth = f64`, and a per-field `export x :: f64`.
///
/// `Visibility::rust_text` gives the Rust side. This is the third printer in
/// the plan that needed a Redox spelling where the AST carried only the Rust
/// one — after `BinaryOp::redox_text` and `pattern_text` — and the pattern is
/// consistent enough to name: **the AST stores what the emitter needs, so a
/// formatter has to supply the surface itself.** That is the right division;
/// it just means a reuse is usually wrong.
pub(crate) fn visibility_text(visibility: &Visibility) -> String {
  match visibility {
    Visibility::Private => String::new(),
    Visibility::Public => "export ".to_string(),
    Visibility::Crate => "export(crate) ".to_string(),
    Visibility::Super => "export(super) ".to_string(),
    Visibility::In(path) => format!("export(in {path}) "),
  }
}
