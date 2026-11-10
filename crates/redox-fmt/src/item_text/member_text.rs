use crate::expr_text::expr_text;

use crate::item_text::fn_text::fn_text;

use crate::stmt_text::indent_text::indent_text;

use crate::stmt_text::trivia_text::trivia_text;

use redox_ast::ClassMember;

/// One member of a `class` or `instance`.
///
/// Three shapes, all at `indent`:
///
/// ```text
/// type Item
/// const SIDES : u32
/// const SIDES : u32 = 4
/// area : &t -> f64
/// area self = self.side * self.side
/// ```
///
/// An associated constant takes a value in an `instance` and not in a `class`,
/// which is the `Option` — rustc decides which is allowed where, as AGENTS.md
/// prescribes, so both print the same way.
///
/// A method is a whole `FnDecl`, signature line and equations, so it goes
/// through `fn_text` and inherits the verbatim signature slice. That is what
/// keeps `area : &t -> f64` inside a `class Shape t` from being rewritten to
/// `&T` — the member's `t` is the implementing type and `generalise` reserves
/// it, but the type *text* is rewritten all the same wherever a variable is
/// not reserved.
pub(crate) fn member_text(member: &ClassMember, indent: usize, source: &str) -> String {
  let pad = indent_text(indent);
  match member {
    ClassMember::AssocType(name) => format!("{}type {}", pad, name.text),
    ClassMember::AssocConst { name, ty, value } => {
      let assigned = match value {
        Some(v) => format!(" = {}", expr_text(v, 0, indent, source)),
        None => String::new(),
      };
      format!("{}const {} : {}{}", pad, name.text, ty.text, assigned)
    }
    ClassMember::Method(decl) => {
      let mut out = String::new();
      for trivium in &decl.leading {
        out.push_str(&trivia_text(trivium, indent));
        out.push('\n');
      }
      format!("{}{}{}", out, pad, fn_text(decl, indent, source))
    }
  }
}
