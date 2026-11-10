use redox_ast::Expr;

/// Whether this expression is safe as a juxtaposed argument with no
/// parentheses of its own.
///
/// # It is a question about the first token, and the parser already answers it
///
/// `parser::starts_argument` is the authority — *"whether `kind` can begin a
/// parenless argument without being mistaken for an infix operator continuing
/// the callee"* — and its allow-list is short: a literal, an identifier, `(`,
/// `@`, `@@`, `true`, `false`. Nothing else. `-`, `!`, `&` and `*` are
/// excluded on purpose, because `a - b` as a statement would otherwise read as
/// `a (-b)`.
///
/// So this mirrors that list by the token each expression form **opens with**,
/// and the first version of it did not. It allowed `List` and `ArrayRepeat`,
/// which open with `[` — and `[` is not on the list, because after a name it
/// is indexing. `examples/destructuring.sdx` writes
/// `print (ends ([1, 2, 3, 4, 5]))`, with parentheses the author needed, and
/// dropping them produced `ends [1, 2, 3, 4, 5]`, which does not parse. The
/// corpus round-trip caught it; the fixture would not have, because nobody
/// writing the fixture thought to try it.
///
/// A postfix form opens with its **receiver's** first token, so the question
/// passes through it rather than being answered about the postfix itself.
///
/// `self` and `Self` are excluded: neither is on the parser's list, so
/// `f (self.x)` keeps its parentheses. Conservative, and the conservative
/// direction here adds a parenthesis rather than changing a program.
pub(crate) fn stands_alone(expr: &Expr) -> bool {
  match expr {
    Expr::Ident(_)
    | Expr::Path { .. }
    | Expr::Num { .. }
    | Expr::Bool { .. }
    | Expr::Char { .. }
    | Expr::StrLit { .. }
    | Expr::StrInterp { .. }
    | Expr::RawLit { .. } => true,
    Expr::Unit { .. } | Expr::Tuple { .. } | Expr::Section { .. } => true,
    Expr::Paren { .. } => true,
    Expr::Field { .. } | Expr::Index { .. } | Expr::Try { .. } => true,
    Expr::MethodRef { .. } => true,
    Expr::List { .. } | Expr::ArrayRepeat { .. } => false,
    _ => false,
  }
}
