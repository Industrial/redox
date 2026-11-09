use redox_ast::Expr;

/// How tightly this expression binds, or `None` if the question does not apply
/// to it.
///
/// `None` is not "loosest" — it means *this leaf declines to reason about the
/// form*, and every caller treats a decline as "keep the parentheses". That
/// asymmetry is the whole safety argument for §3 row 3: the only normalisation
/// in plan 34 that can change what a program means is dropping a parenthesis,
/// so the default has to be not to.
///
/// # Which forms get a number
///
/// - A binary expression binds at its operator's power, which is
///   `BinaryOp::power` — **the** precedence table, now on the operator rather
///   than in the parser so that the parser and this leaf cannot drift apart.
/// - Application, postfix and the bracketed literals bind tighter than any
///   binary operator, so they get `u8::MAX`: a parenthesis around one of them
///   is never needed to hold a binary expression together.
///
/// # Which forms decline
///
/// Lambdas, `if`/`case`/`do` as expressions, comprehensions, ascriptions and
/// sections. Each of those *ends* at a place that depends on what follows it —
/// a lambda body extends as far right as it can, an ascription's type swallows
/// what comes next — so a surrounding parenthesis is doing real work even when
/// a precedence table would say otherwise.
pub(crate) fn power_of(expr: &Expr) -> Option<u8> {
  match expr {
    Expr::Binary { op, .. } => Some(op.power()),
    // Tighter than every binary operator. Doc 18 §2.1 makes application the
    // primary and the parser consumes postfix inside it, so these are a single
    // group rather than a ladder.
    Expr::Call { .. }
    | Expr::MethodCall { .. }
    | Expr::Field { .. }
    | Expr::Index { .. }
    | Expr::Unary { .. }
    | Expr::Try { .. }
    | Expr::OptionalChain { .. }
    | Expr::Path { .. }
    | Expr::StrLit { .. }
    | Expr::StrInterp { .. }
    | Expr::RawLit { .. }
    | Expr::Num { .. }
    | Expr::Bool { .. }
    | Expr::Char { .. }
    | Expr::Tuple { .. }
    | Expr::List { .. }
    | Expr::ArrayRepeat { .. }
    | Expr::Unit { .. }
    | Expr::SelfField { .. }
    | Expr::SelfType { .. }
    | Expr::Ident(_) => Some(u8::MAX),
    // A parenthesis inside a parenthesis is decided by what *it* holds, so the
    // question passes straight through. `((x)) + 1` reduces to `x + 1` by this
    // line, which is also what makes the normalisation idempotent.
    Expr::Paren { inner, .. } => power_of(inner),
    // Bracket-terminated, so nothing after them can be absorbed: a record
    // ends at `}`, a comprehension at `]`, a section at `)`. They were
    // declining with the open-ended forms below, which parenthesised
    // `[ n * 2 | n in xs ] :: Vec<i32>` — a spelling `examples/comprehension.sdx`
    // writes without parentheses, correctly.
    Expr::Record { .. } | Expr::Comprehension { .. } | Expr::Section { .. } => Some(u8::MAX),
    // Open-ended: each extends as far right as it can, so a surrounding
    // parenthesis is doing real work whatever a precedence table says.
    Expr::Lambda { .. }
    | Expr::Ascribe { .. }
    | Expr::If { .. }
    | Expr::Do { .. }
    | Expr::Case { .. }
    | Expr::AsyncBlock { .. }
    | Expr::TryBlock { .. } => None,
  }
}
