use redox_ast::Expr;

/// An expression's own source text, unchanged.
///
/// Plan 34's criterion 7 — *"a construct the formatter does not yet print is
/// passed through verbatim rather than mangled"* — applied one level below
/// leaf 1, which did the same for items. It is what lets leaf 3 ship before
/// leaf 4 exists: a block-bearing expression keeps the author's text, comments
/// and indentation included, instead of being half-printed.
///
/// A span that does not land on a character boundary or runs past the end
/// would be a defect in whichever node produced it, not something to paper
/// over, so this slices and lets a bad span panic rather than silently
/// returning the wrong text.
pub(crate) fn verbatim(expr: &Expr, source: &str) -> String {
  let (start, end) = expr.span();
  source[start..end].to_string()
}
