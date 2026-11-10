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
/// The class may carry arguments of its own — `instance Convert f64 Meters` —
/// so it prints through the same `target_text` the target does, and the
/// implementing type stays last, which is where plan 67 §9 decision 3 put it.
///
/// **A target with arguments is parenthesised**, because the parentheses are
/// what say where it begins: `instance Describe (Wrapper t)` printed flat is
/// `instance Describe Wrapper t`, and plan 67 §9 decision 3's reading takes
/// the last name as the implementing type, so that re-parses as `Describe<Wrapper>`
/// for `T`. The flat form happened to round-trip under the previous split,
/// which took the first argument instead — so this is a fidelity bug the
/// split exposed rather than one it introduced, and the inherent generic form
/// `instance (Wrapper t)` was already losing its parentheses either way.
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
    .map_or(String::new(), |class| format!("{} ", target_text(class)));
  let target = if decl.target.args.is_empty() {
    target_text(&decl.target)
  } else {
    format!("({})", target_text(&decl.target))
  };
  let header = format!("instance {constraints}{class}{target}");
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
