/// A unary prefix operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
  /// `-x`
  Neg,
  /// `!x`
  Not,
  /// `&x`
  Ref,
  /// `&mut x`
  RefMut,
  /// `*x`
  Deref,
}
#[allow(unused_imports)]
use UnaryOp::*;

impl UnaryOp {
  pub fn rust_text(self) -> &'static str {
    match self {
      Neg => "-",
      Not => "!",
      Ref => "&",
      RefMut => "&mut ",
      Deref => "*",
    }
  }
}
