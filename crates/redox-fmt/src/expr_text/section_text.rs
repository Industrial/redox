use crate::expr_text::expr_text;

use redox_ast::Section;

/// An operator or accessor section — `(> 0)`, `(+ 1)`, `(.age)`.
///
/// The parentheses are part of the form rather than grouping around it: the
/// parser consumes them into `Section`, so there is no `Paren` node to drop
/// and nothing for §3 row 3 to decide. They are written here.
///
/// The corpus's own spellings are the canonical ones — `(> 0)`, `(+ 1)`,
/// `(.age)` — which is §1.5's rule, that the hand-written examples are the
/// style guide rather than the thing measured against one.
pub(crate) fn section_text(section: &Section, indent: usize, source: &str) -> String {
  match section {
    Section::Right { op, operand } => format!(
      "({} {})",
      (*op).redox_text(),
      expr_text(operand, 0, indent, source)
    ),
    Section::Left { op, operand } => format!(
      "({} {})",
      expr_text(operand, 0, indent, source),
      (*op).redox_text()
    ),
    Section::Field(name) => format!("(.{})", name.text),
  }
}
