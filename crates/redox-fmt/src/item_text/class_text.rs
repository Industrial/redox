use crate::item_text::member_text::member_text;

use crate::item_text::visibility_text::visibility_text;

use redox_ast::ClassDecl;

/// A `class` declaration and its members.
///
/// ```text
/// class Shape
///   const SIDES : u32
///   area : &self -> f64
///   describe self = "a shape"
///
/// class Convert t
///   wrap : t -> Self
/// ```
///
/// A superclass constraint goes before the head with `=>` — `class Eq => Ord`.
/// It is a trait name alone since plan 67, because a superclass constrains
/// `Self` and there is no variable left to name.
///
/// The trait's own parameters follow the name, space-separated and in source
/// order. Most classes have none, which is the common shape and the one the
/// corpus shows; the parameterised head is what makes `trait From<T>`
/// reachable — plan 67 §1.4.
pub(crate) fn class_text(decl: &ClassDecl, indent: usize, source: &str) -> String {
  let supers = if decl.supers.is_empty() {
    String::new()
  } else {
    let names: Vec<&str> = decl.supers.iter().map(|s| s.text.as_str()).collect();
    format!("{} => ", names.join(", "))
  };
  let params: String = decl.params.iter().map(|p| format!(" {}", p.text)).collect();
  let vis = visibility_text(&decl.visibility);
  let header = format!("{}class {}{}{}", vis, supers, decl.name.text, params);
  if decl.members.is_empty() {
    return header;
  }
  let printed: Vec<String> = decl
    .members
    .iter()
    .map(|member| member_text(member, indent + 1, source))
    .collect();
  format!("{}\n{}", header, printed.join("\n"))
}
