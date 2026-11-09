use redox_ast::Expr;

/// The lexer's own text for a literal, for the no-source case.
pub(crate) fn stored(expr: &Expr) -> String {
  match expr {
    Expr::StrLit { value, .. } => value.clone(),
    Expr::RawLit { text, .. } | Expr::Num { text, .. } => text.clone(),
    Expr::Char { value, .. } => format!("'{}'", value.escape_default()),
    _ => String::new(),
  }
}
