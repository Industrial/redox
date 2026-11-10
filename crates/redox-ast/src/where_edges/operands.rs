use crate::{Expr, StrInterpPart};

/// Every sub-expression of `e` that is an **operand**.
///
/// [`Expr::blocks`] answers a different question — the *block-bearing*
/// children, for the tail walk — so this cannot reuse it: a binding's value is
/// an ordinary expression and its parts are operands, arguments and receivers.
///
/// A form carrying a **block** is a leaf here. A block introduces its own
/// scope, so a name inside one is not necessarily this group's, and declining
/// to look is the conservative answer: a missed edge leaves the entry in
/// source order, where it already worked.
pub(crate) fn operands(e: &Expr) -> Vec<&Expr> {
  match e {
    Expr::Call { callee, args, .. } => {
      let mut out = vec![callee.as_ref()];
      out.extend(args.iter());
      out
    }
    Expr::Binary { lhs, rhs, .. } => vec![lhs.as_ref(), rhs.as_ref()],
    Expr::Unary { operand, .. } => vec![operand.as_ref()],
    Expr::Paren { inner, .. } | Expr::Ascribe { inner, .. } | Expr::Try { inner, .. } => {
      vec![inner.as_ref()]
    }
    Expr::Field { recv, .. } => vec![recv.as_ref()],
    Expr::Index { recv, index, .. } => vec![recv.as_ref(), index.as_ref()],
    Expr::Lambda { body, .. } => vec![body.as_ref()],
    Expr::Tuple { items, .. } | Expr::List { items, .. } => items.iter().collect(),
    Expr::ArrayRepeat { value, count, .. } => vec![value.as_ref(), count.as_ref()],
    Expr::MacroCall { args, .. } => args.iter().collect(),
    Expr::StrInterp { parts, .. } => parts
      .iter()
      .filter_map(|p| match p {
        StrInterpPart::Interp { expr, .. } => Some(expr),
        StrInterpPart::Text(_) => None,
      })
      .collect(),
    // A **punned** field — `Point { x, y }` — carries `None` and is a
    // reference to `x`, which this does not see. A group binding mentioned
    // solely as a pun is therefore not an edge; disclosed rather than fixed,
    // because the entry then stays in source order.
    Expr::Record { fields, .. } => fields.iter().filter_map(|f| f.value.as_ref()).collect(),
    _ => Vec::new(),
  }
}
