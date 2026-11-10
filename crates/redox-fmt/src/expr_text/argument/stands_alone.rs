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
    // Opens with a literal or a name.
    Expr::Ident(_)
    | Expr::Path { .. }
    | Expr::Num { .. }
    | Expr::Bool { .. }
    | Expr::Char { .. }
    | Expr::StrLit { .. }
    | Expr::StrInterp { .. }
    | Expr::RawLit { .. } => true,
    // Opens with `(`. A `Section` carries its own parentheses and `Unit` and
    // `Tuple` are parentheses.
    Expr::Unit { .. } | Expr::Tuple { .. } | Expr::Section { .. } => true,
    // Also opens with `(` — though `argument` intercepts both `Paren` cases
    // before it asks, so this arm answers the question for nobody but a
    // future caller.
    Expr::Paren { .. } => true,
    // A postfix form always prints safely, because its receiver is itself
    // printed through `argument` — so the receiver either stands alone or
    // comes out wrapped in parentheses, and either way the postfix form opens
    // with a token the parser accepts.
    //
    // Asking about the **receiver** instead, which is what this did first, is
    // not idempotent: `[1, 2, 3].to_vec ()` has a receiver that does not
    // stand alone, so the whole thing was wrapped — giving
    // `(([1, 2, 3]).to_vec ())`. On the next pass the receiver carried its own
    // parentheses, so the same test said yes and the outer pair came off. The
    // corpus caught it in `examples/draw_condition.sdx`.
    //
    // The lesson generalises: this predicate is a question about the text the
    // printer **emits**, not about the tree it was handed.
    Expr::Field { .. } | Expr::Index { .. } | Expr::Try { .. } => true,
    // Plan 57's `.name`, and the one entry here that is **not** a mirror of
    // `starts_argument`: a `Dot` cannot begin a parenless argument, because
    // `postfix` would have consumed it as a field access on the callee.
    //
    // It is `true` anyway, because the only caller that reaches a `MethodRef`
    // is [`argument`] printing a **callee** — the parser produces no
    // `MethodRef` in argument position — and a callee must print bare:
    // `(.len) xs` is the field *selector* applied to `xs`, a different
    // program. Parenthesising here would be the one normalisation §3 row 3
    // forbids.
    Expr::MethodRef { .. } => true,
    // Opens with `[`, which is indexing after a name.
    Expr::List { .. } | Expr::ArrayRepeat { .. } => false,
    _ => false,
  }
}
