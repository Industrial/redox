use crate::expr_text::expr_text;
use redox_ast::{Expr, Qualifier};

/// A comprehension — `[ n * 2 | n in xs, n > 0 ]`.
///
/// The brackets take one space inside, unlike a list literal's `[1, 2, 3]`.
/// That looks like an inconsistency and is the corpus's own: every
/// comprehension in `examples/` is spaced and every list is tight, which
/// §1.5 makes the rule rather than something to reconcile. The forms read
/// differently enough — a comprehension has a `|` to find — that the extra
/// space is doing work.
///
/// The generator keyword is `in`, not `<-`. Doc 30 took `<-` for `await` and
/// nothing else, and §3 notes that a construct with one spelling needs no
/// canonical form: there is no second spelling here to normalise *from*.
pub(crate) fn comprehension_text(
  element: &Expr,
  qualifiers: &[Qualifier],
  indent: usize,
  source: &str,
) -> String {
  let printed: Vec<String> = qualifiers
    .iter()
    .map(|qualifier| match qualifier {
      Qualifier::Draw { name, source: from } => {
        format!("{} in {}", name.text, expr_text(from, 0, indent, source))
      }
      Qualifier::Guard(guard) => expr_text(guard, 0, indent, source),
    })
    .collect();
  format!(
    "[ {} | {} ]",
    expr_text(element, 0, indent, source),
    printed.join(", ")
  )
}
