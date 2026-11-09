use crate::{Ident, TypeExpr};
use redox_lexer::{Span, Trivia};

/// `type Handler = Box<dyn Fn(Req) -> Res + Send>` — doc 18 §2.4.
///
/// The one declaration form that is already Rust: the keyword, the `=` and
/// the type are unchanged, and the type is carried verbatim as every type in
/// Redox is (doc 03 §14.1). It sits beside `data` rather than inside it
/// because an alias has no constructors and no `deriving`, and giving it a
/// `DataDecl` with both empty would make every reader check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias {
  pub name: Ident,
  pub ty: TypeExpr,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
