use crate::item_text::member_text::member_text;
use crate::item_text::visibility_text::visibility_text;
use redox_ast::ClassDecl;

/// A `class` declaration and its members.
///
/// ```text
/// class Shape t
///   const SIDES :: u32
///   area :: &t -> f64
///   describe self = "a shape"
/// ```
///
/// A superclass constraint goes before the head with `=>`, in the same shape a
/// signature's bound takes — `class Eq t => Ord t`. The corpus has no example
/// of one, which is worth stating rather than leaving as a silent guess: the
/// spelling comes from the parser's grammar and doc 18's `=>`, and the
/// round-trip property is what checks it.
pub(crate) fn class_text(decl: &ClassDecl, indent: usize, source: &str) -> String {
  let supers = if decl.supers.is_empty() {
    String::new()
  } else {
    let printed: Vec<String> = decl
      .supers
      .iter()
      .map(|c| format!("{} {}", c.trait_name.text, c.var.text))
      .collect();
    format!("{} => ", printed.join(", "))
  };
  let header = format!(
    "{}class {supers}{} {}",
    visibility_text(&decl.visibility),
    decl.name.text,
    decl.var.text
  );
  if decl.members.is_empty() {
    return header;
  }
  let printed: Vec<String> = decl
    .members
    .iter()
    .map(|member| member_text(member, indent + 1, source))
    .collect();
  format!("{header}\n{}", printed.join("\n"))
}
