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
pub(crate) fn section_text(section: &Section, indent: usize, source: &str) -> String {
  match section {
    // `(> 0)` — the operand is on the right, so the operator leads.
    Section::Right { op, operand } => {
      format!(
        "({} {})",
        op.redox_text(),
        argument(operand, indent, source)
      )
    }
    // `(0 <)` — doc 18 §4 gives it the same lowering as its flip, and the
    // formatter keeps the side the author wrote. Flipping it would be a
    // refactor, and it would make the two spellings indistinguishable in a
    // diff.
    Section::Left { op, operand } => {
      format!(
        "({} {})",
        argument(operand, indent, source),
        op.redox_text()
      )
    }
    Section::Field(name) => format!("(.{})", name.text),
    // `(|> len)`, not `(.len ())`. `examples/section.sdx` says it in a comment
    // two lines above the code — *"method: `(.name)` selects and `(|> name)`
    // calls"* — and `(.len ())` does not parse at all: the section grammar
    // wants `)` after the name.
    //
    // Guessing it from `Field`'s spelling cost a corpus round-trip failure
    // reported as *"did not re-parse"* on a line whose section was the only
    // thing wrong with it. The `|>` is the clue: a method section is a
    // pipeline stage with its receiver left out, which is exactly what it
    // lowers to.
    Section::Method(name) => format!("(|> {})", name.text),
  }
}
