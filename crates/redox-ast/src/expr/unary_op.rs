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

impl UnaryOp {
  pub fn rust_text(self) -> &'static str {
    match self {
      Self::Neg => "-",
      Self::Not => "!",
      Self::Ref => "&",
      Self::RefMut => "&mut ",
      Self::Deref => "*",
    }
  }
}
