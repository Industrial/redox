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
      let mut out = {
        let __rdx_t: Vec<&Expr> = Vec::from([callee.as_ref()]);
        __rdx_t
      };
      out.extend(args.iter());
      out
    }
    Expr::Binary { lhs, rhs, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([lhs.as_ref(), rhs.as_ref()]);
      __rdx_t
    }
    Expr::Unary { operand, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([operand.as_ref()]);
      __rdx_t
    }
    Expr::Paren { inner, .. } | Expr::Ascribe { inner, .. } | Expr::Try { inner, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([inner.as_ref()]);
      __rdx_t
    }
    Expr::Field { recv, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([recv.as_ref()]);
      __rdx_t
    }
    Expr::Index { recv, index, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([recv.as_ref(), index.as_ref()]);
      __rdx_t
    }
    Expr::Lambda { body, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([body.as_ref()]);
      __rdx_t
    }
    Expr::Tuple { items, .. } | Expr::List { items, .. } => items.iter().collect(),
    Expr::ArrayRepeat { value, count, .. } => {
      let __rdx_t: Vec<&Expr> = Vec::from([value.as_ref(), count.as_ref()]);
      __rdx_t
    }
    Expr::MacroCall { args, .. } => args.iter().collect(),
    Expr::StrInterp { parts, .. } => parts
      .iter()
      .filter_map(|p| match p {
        StrInterpPart::Interp { expr, .. } => Some(expr),
        StrInterpPart::Text(_) => None,
      })
      .collect(),
    Expr::Record { fields, .. } => fields.iter().filter_map(|f| f.value.as_ref()).collect(),
    _ => Vec::new(),
  }
}
