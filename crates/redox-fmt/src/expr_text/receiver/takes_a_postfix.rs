use redox_ast::Expr;

/// Whether a `.` can attach to this expression's printed form directly.
pub(crate) fn takes_a_postfix(expr: &Expr) -> bool {
  match expr {
    // Atoms and bracketed forms: a `.` binds to the whole of each.
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
    // A postfix chain is itself a receiver — that is what a chain is.
    Expr::Field { .. }
    | Expr::MethodCall { .. }
    | Expr::Index { .. }
    | Expr::Try { .. }
    | Expr::OptionalChain { .. } => true,
    // The author's parenthesis is kept or dropped by what it holds.
    Expr::Paren { inner, .. } => takes_a_postfix(inner),
    // An application needs its own parentheses: `(f x).a`, because postfix is
    // consumed inside `application::atom` and would otherwise bind to `x`.
    Expr::Call { .. } => false,
    // Everything else — a binary or unary expression, a lambda, an
    // ascription, a block-bearing form — ends somewhere a `.` cannot follow.
    _ => false,
  }
}
