use redox_ast::Expr;

/// Whether an expression binds looser than `.` and so needs parentheses when
/// it is a method call's receiver.
///
/// Listed rather than inferred from the precedence table, because the
/// question is not "how tightly does this bind" but "is this a *postfix-able*
/// form" — and the forms that are not make a short, closed list.
pub(crate) fn wraps_as_receiver(e: &Expr) -> bool {
  // A pipeline is the exception among binary expressions: its *lowering* is
  // already a method call, so it is postfix-able and parenthesising it turns
  // `numbers.iter().map(f).sum()` into `((numbers.iter()).map(f)).sum()`.
  // Caught by `a_chain_nests_left_to_right`, which is what that test is for.
  if matches!(
    e,
    Expr::Binary {
      op: redox_ast::BinaryOp::Pipeline,
      ..
    }
  ) {
    return false;
  }

  // `?` is itself postfix, so a receiver that is one needs no parentheses:
  // `f()?.len()` chains. `.await` was here too until doc 30 deleted it — an
  // await is a statement now, so it never appears as a receiver.
  if matches!(e, Expr::Try { .. }) {
    return false;
  }

  matches!(
    e,
    Expr::Binary { .. }
      | Expr::Unary { .. }
      | Expr::Ascribe { .. }
      | Expr::Lambda { .. }
      | Expr::Case { .. }
      | Expr::Record { .. }
      | Expr::Section { .. }
      | Expr::Comprehension { .. }
  )
}
