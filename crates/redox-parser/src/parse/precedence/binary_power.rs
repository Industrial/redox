use redox_ast::BinaryOp;
use redox_lexer::{Keyword, TokenKind};

/// Rust's precedence table, **unchanged**: a map from token to operator and
/// `(left, right)` binding powers.
///
/// Left-associative operators have `right = left + 1`, so an equal-precedence
/// operator on the right does not pull. The operators Rust lacks sit where
/// doc 03 §4.5 places them: `??` below `||`, `|>` below `??`.
pub(crate) fn binary_power(kind: &TokenKind) -> Option<(BinaryOp, u8, u8)> {
  // Loosest first, so the table reads in precedence order.
  let (op, left) = match kind {
    TokenKind::Pipe => (BinaryOp::Pipeline, 1),
    TokenKind::Coalesce => (BinaryOp::Coalesce, 3),
    // Doc 03 §4.5 places `catch` *"at `??`'s level"* — plan 28. Being a
    // keyword and not in `starts_argument`'s allow-list is what makes
    // `parse s catch 0` need no parentheses, which §12.2 claims and plan 28
    // §1.4 verified rather than assumed.
    TokenKind::Kw(Keyword::Catch) => (BinaryOp::Catch, 3),
    TokenKind::DotDot => (BinaryOp::Range, 5),
    TokenKind::DotDotEq => (BinaryOp::RangeInclusive, 5),
    TokenKind::OrOr => (BinaryOp::Or, 7),
    TokenKind::AndAnd => (BinaryOp::And, 9),
    // Comparison is non-associative in Rust. Modelling it as left-associative
    // with equal powers lets `a < b < c` parse so that rustc rejects it, which
    // is the division of labour doc 02 §6.2 prescribes.
    TokenKind::EqEq => (BinaryOp::Eq, 11),
    TokenKind::Ne => (BinaryOp::Ne, 11),
    TokenKind::Lt => (BinaryOp::Lt, 11),
    TokenKind::Gt => (BinaryOp::Gt, 11),
    TokenKind::Le => (BinaryOp::Le, 11),
    TokenKind::Ge => (BinaryOp::Ge, 11),
    TokenKind::Or => (BinaryOp::BitOr, 13),
    TokenKind::Caret => (BinaryOp::BitXor, 15),
    TokenKind::Amp => (BinaryOp::BitAnd, 17),
    TokenKind::Shl => (BinaryOp::Shl, 19),
    TokenKind::Shr => (BinaryOp::Shr, 19),
    TokenKind::Plus => (BinaryOp::Add, 21),
    TokenKind::Minus => (BinaryOp::Sub, 21),
    TokenKind::Star => (BinaryOp::Mul, 23),
    TokenKind::Slash => (BinaryOp::Div, 23),
    TokenKind::Percent => (BinaryOp::Rem, 23),
    _ => return None,
  };
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
