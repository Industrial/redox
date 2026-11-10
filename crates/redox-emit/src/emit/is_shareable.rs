use redox_ast::Expr;

/// Whether `body` is a literal, and so may be shared as a `const`.
///
/// Plan 59 (`docs/53 - 59 - Value Bindings.md`) §1.3's syntactic test. The
/// alternative on the table was *"memoise anything const-evaluable"*, and doc
/// 03 §13.3 declines const evaluation permanently and in these words: it would
/// require shipping a literal-only interpreter, *"a scope expansion of the
/// same order as the type checker doc 02 §12.4 declines permanently"*. So this
/// asks about the **shape of the source** and never about a value.
///
/// # What counts, and the one exclusion that is not obvious
///
/// Literals, aggregates of literals, a parenthesised or negated literal, and a
/// `Path` to another constant. Not [`Expr::Ident`]: a bare lower-case name is
/// as likely to be a function item as a constant, and §1.3 says *"paths to
/// other constants"* — the qualified form, which is the one that cannot be a
/// local binder.
///
/// **[`Expr::StrInterp`] is excluded although it looks like a literal.** A
/// `#{…}` string lowers to `format!`, which is an application and allocates;
/// `"a#{b}c"` is no more a constant than `f x` is. That is the case this test
/// would get wrong if it keyed off "is it spelled with quotes".
pub(crate) fn is_shareable(body: &Expr) -> bool {
  match body {
    Expr::Num { .. }
    | Expr::Bool { .. }
    | Expr::Char { .. }
    | Expr::StrLit { .. }
    | Expr::RawLit { .. }
    | Expr::Path { .. }
    | Expr::Unit { .. } => true,
    Expr::Paren { inner, .. } => is_shareable(inner),
    Expr::Unary { operand, .. } => is_shareable(operand),
    Expr::Tuple { items, .. } | Expr::List { items, .. } => items.iter().all(is_shareable),
    _ => false,
  }
}
