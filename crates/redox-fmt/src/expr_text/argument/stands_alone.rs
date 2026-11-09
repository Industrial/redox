use redox_ast::Expr;

/// Whether this expression is safe beside another without parentheses.
pub(crate) fn stands_alone(expr: &Expr) -> bool {
  match expr {
    Expr::Ident(_)
    | Expr::Path { .. }
    | Expr::Num { .. }
    | Expr::Bool { .. }
    | Expr::Char { .. }
    | Expr::StrLit { .. }
    | Expr::StrInterp { .. }
    | Expr::RawLit { .. }
    | Expr::Unit { .. }
    | Expr::Tuple { .. }
    | Expr::List { .. }
    | Expr::ArrayRepeat { .. }
    | Expr::Section { .. }
    | Expr::SelfField { .. }
    | Expr::SelfType { .. }
    | Expr::Field { .. }
    | Expr::MethodCall { .. }
    | Expr::Index { .. }
    | Expr::Try { .. }
    | Expr::OptionalChain { .. } => true,
    // §3 row 1 itself: the parenthesis goes only if what it holds would have
    // stood alone anyway. `f (x)` becomes `f x`; `f (g x)` keeps its own.
    Expr::Paren { inner, .. } => stands_alone(inner),
    _ => false,
  }
}
