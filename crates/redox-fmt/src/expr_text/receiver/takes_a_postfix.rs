use redox_ast::Expr;

/// Whether a `.` can attach to this expression's printed form directly.
pub(crate) fn takes_a_postfix(expr: &Expr) -> bool {
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
    | Expr::Record { .. }
    | Expr::Section { .. }
    | Expr::Comprehension { .. }
    | Expr::SelfField { .. }
    | Expr::SelfType { .. } => true,
    Expr::Field { .. } | Expr::Index { .. } | Expr::Try { .. } => true,
    Expr::MethodRef { .. } => true,
    Expr::Paren { inner, .. } => takes_a_postfix(inner),
    Expr::Call { .. } => false,
    _ => false,
  }
}
