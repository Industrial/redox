use crate::Expr;
use crate::where_edges::operands::operands;

/// Every index in `named` whose name appears as a bare identifier in `e`.
///
/// Walks the **tree** and not the source text, which is what makes a name
/// inside an interpolation count: `#{total}` holds a real expression, so it is
/// a real reference.
pub(crate) fn mentions(e: &Expr, named: &[(usize, String)], into: &mut Vec<usize>) {
  if let Expr::Ident(name) = e
    && let Some((i, _)) = named.iter().find(|(_, n)| *n == name.text)
  {
    into.push(*i);
  }
  for child in operands(e) {
    mentions(child, named, into);
  }
}
