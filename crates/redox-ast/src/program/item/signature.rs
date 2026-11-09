use crate::{Param, TypeExpr};
use redox_lexer::Span;

/// A function's parameters and return type.
///
/// Both are optional: doc 01 §5.1 makes the return type optional, and a
/// zero-parameter function writes no list at all (`main :: ()` / `main =`), which is the
/// form plan 01 shipped.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature {
  pub params: Vec<Param>,
  pub ret: Option<TypeExpr>,
  pub span: Span,
}
