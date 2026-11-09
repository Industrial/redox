use crate::expr_text::argument::argument;
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
pub(crate) fn section_text(section: &Section, source: &str) -> String {
  match section {
    // `(> 0)` — the operand is on the right, so the operator leads.
    Section::Right { op, operand } => {
      format!("({} {})", op.redox_text(), argument(operand, source))
    }
    // `(0 <)` — doc 18 §4 gives it the same lowering as its flip, and the
    // formatter keeps the side the author wrote. Flipping it would be a
    // refactor, and it would make the two spellings indistinguishable in a
    // diff.
    Section::Left { op, operand } => {
      format!("({} {})", argument(operand, source), op.redox_text())
    }
    Section::Field(name) => format!("(.{})", name.text),
    Section::Method(name) => format!("(.{} ())", name.text),
  }
}
