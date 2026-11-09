use redox_ast::BinaryOp;
use redox_lexer::{Keyword, TokenKind};

/// Which operator a token spells, and its `(left, right)` binding powers.
///
/// Left-associative operators have `right = left + 1`, so an equal-precedence
/// operator on the right does not pull.
///
/// # The powers are no longer here
///
/// They moved to `BinaryOp::power` in `redox-ast` when plan 34 leaf 3 made the
/// formatter a second reader of them: dropping a redundant parenthesis is a
/// precedence decision, and a formatter working from a *copy* of this table
/// would drop a needed parenthesis as soon as the two drifted — silently,
/// because both spellings still compile.
///
/// What stays here is the half that is genuinely the parser's: the mapping
/// from **token** to operator. The AST does not know about tokens and should
/// not.
pub(crate) fn binary_power(kind: &TokenKind) -> Option<(BinaryOp, u8, u8)> {
  // Loosest first, so the table still reads in precedence order even though
  // the numbers now live on the operator.
  let op = match kind {
    TokenKind::Pipe => BinaryOp::Pipeline,
    TokenKind::Coalesce => BinaryOp::Coalesce,
    // Doc 03 §4.5 places `catch` *"at `??`'s level"* — plan 28. Being a
    // keyword and not in `starts_argument`'s allow-list is what makes
    // `parse s catch 0` need no parentheses, which §12.2 claims and plan 28
    // §1.4 verified rather than assumed.
    TokenKind::Kw(Keyword::Catch) => BinaryOp::Catch,
    TokenKind::DotDot => BinaryOp::Range,
    TokenKind::DotDotEq => BinaryOp::RangeInclusive,
    TokenKind::OrOr => BinaryOp::Or,
    TokenKind::AndAnd => BinaryOp::And,
    TokenKind::EqEq => BinaryOp::Eq,
    TokenKind::Ne => BinaryOp::Ne,
    TokenKind::Lt => BinaryOp::Lt,
    TokenKind::Gt => BinaryOp::Gt,
    TokenKind::Le => BinaryOp::Le,
    TokenKind::Ge => BinaryOp::Ge,
    TokenKind::Or => BinaryOp::BitOr,
    TokenKind::Caret => BinaryOp::BitXor,
    TokenKind::Amp => BinaryOp::BitAnd,
    TokenKind::Shl => BinaryOp::Shl,
    TokenKind::Shr => BinaryOp::Shr,
    TokenKind::Plus => BinaryOp::Add,
    TokenKind::Minus => BinaryOp::Sub,
    TokenKind::Star => BinaryOp::Mul,
    TokenKind::Slash => BinaryOp::Div,
    TokenKind::Percent => BinaryOp::Rem,
    _ => return None,
  };
  let left = op.power();
  Some((op, left, left + 1))
}

#[cfg(test)]
mod tests {
  use super::binary_power;
  use crate::parse::precedence::prefix_power::PREFIX_POWER;
  use redox_ast::BinaryOp;
  use redox_lexer::TokenKind;

  fn power(kind: TokenKind) -> u8 {
    binary_power(&kind).expect("should have a precedence").1
  }

  /// Rust's ordering, asserted rather than assumed.
  #[test]
  fn precedence_follows_rust() {
    assert!(power(TokenKind::Star) > power(TokenKind::Plus));
    assert!(power(TokenKind::Plus) > power(TokenKind::Shl));
    assert!(power(TokenKind::Shl) > power(TokenKind::Amp));
    assert!(power(TokenKind::Amp) > power(TokenKind::Caret));
    assert!(power(TokenKind::Caret) > power(TokenKind::Or));
    assert!(power(TokenKind::Or) > power(TokenKind::EqEq));
    assert!(power(TokenKind::EqEq) > power(TokenKind::AndAnd));
    assert!(power(TokenKind::AndAnd) > power(TokenKind::OrOr));
  }

  /// Doc 03 §4.5's placement for the operators Rust does not have.
  #[test]
  fn the_new_operators_sit_where_doc_03_places_them() {
    assert!(power(TokenKind::Coalesce) < power(TokenKind::OrOr));
    assert!(power(TokenKind::Pipe) < power(TokenKind::Coalesce));
  }

  #[test]
  fn unary_and_postfix_bind_tighter_than_any_binary() {
    assert!(PREFIX_POWER > power(TokenKind::Star));
  }

  // `POSTFIX_POWER` was asserted here against `PREFIX_POWER` until doc 18 §2.1
  // made application the primary: postfix is now consumed inside
  // `application::atom` rather than ranked on the precedence ladder, so the
  // constant has no value to compare and was deleted with the ladder rung.

  #[test]
  fn binary_operators_are_left_associative() {
    for kind in [TokenKind::Plus, TokenKind::Star, TokenKind::Pipe] {
      let (_, left, right) = binary_power(&kind).expect("should have a precedence");
      assert_eq!(right, left + 1, "{kind:?} should be left-associative");
    }
  }

  #[test]
  fn a_non_operator_has_no_precedence() {
    assert!(binary_power(&TokenKind::Ident("x".into())).is_none());
    assert!(binary_power(&TokenKind::Newline).is_none());
  }

  /// The powers this file reports are the operator's own, not a second copy.
  ///
  /// This is the assertion that the move was a move. If anyone re-introduces
  /// a literal here, it fails.
  #[test]
  fn the_powers_come_from_the_operator() {
    for kind in [
      TokenKind::Pipe,
      TokenKind::Coalesce,
      TokenKind::OrOr,
      TokenKind::EqEq,
      TokenKind::Plus,
      TokenKind::Percent,
    ] {
      let (op, left, right) = binary_power(&kind).expect("should have a precedence");
      assert_eq!(left, op.power(), "{kind:?} reports a power of its own");
      assert_eq!(right, op.power() + 1);
    }
  }

  #[test]
  fn maps_tokens_to_their_operators() {
    assert_eq!(
      binary_power(&TokenKind::Plus).map(|p| p.0),
      Some(BinaryOp::Add)
    );
    assert_eq!(
      binary_power(&TokenKind::DotDot).map(|p| p.0),
      Some(BinaryOp::Range),
      "`..` is exclusive (doc 03 §4.2)"
    );
  }
}
