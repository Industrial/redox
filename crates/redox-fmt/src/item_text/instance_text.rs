pub(crate) mod target_text;

use crate::item_text::instance_text::target_text::target_text;
use crate::item_text::member_text::member_text;
use redox_ast::InstanceDecl;

/// An `instance` declaration and its members.
///
/// Two shapes, both in `examples/class.sdx`:
///
/// ```text
/// instance Shape Square     -- a trait implementation
/// instance Square           -- an inherent one, with no class
/// ```
///
/// The inherent form is `class: None`, which is why the class is an `Option`
/// rather than a name that might be empty.
///
/// A constraint goes before the head with `=>`. The target's own spelling is
/// `target_text`'s, which records why an `instance` can be printed from the
/// tree where a signature cannot.
pub(crate) fn instance_text(decl: &InstanceDecl, indent: usize, source: &str) -> String {
  let constraints = if decl.constraints.is_empty() {
    String::new()
  } else {
    let printed: Vec<String> = decl
      .constraints
      .iter()
      .map(|c| format!("{} {}", c.trait_name.text, c.var.text))
      .collect();
    format!("{} => ", printed.join(", "))
  };
  let class = decl
    .class
    .as_ref()
    .map_or(String::new(), |name| format!("{} ", name.text));
  let header = format!("instance {constraints}{class}{}", target_text(&decl.target));
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
