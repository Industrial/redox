use crate::TokenKind;
use crate::lexer::cursor::Cursor;
use crate::lexer::operator_table::OPERATOR_TABLE;

/// Scan an operator at the cursor, if one is there.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 7. The matching order
/// lives in [`OPERATOR_TABLE`], longest first.
pub(crate) fn scan_operator(cur: &mut Cursor) -> Option<TokenKind> {
  for (text, kind) in OPERATOR_TABLE {
    if cur.eat(text) {
      return Some(kind.clone());
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::scan_operator;
  use crate::TokenKind;
  use crate::lexer::cursor::Cursor;
  use crate::lexer::operator_table::OPERATOR_TABLE;

  fn scan(src: &str) -> (Option<TokenKind>, usize) {
    let mut cur = Cursor::new(src);
    let kind = scan_operator(&mut cur);
    (kind, cur.pos())
  }

  /// The property the table's order exists for. Every prefix pair is checked
  /// mechanically rather than by eye, so adding a token in the wrong place
  /// fails here instead of silently truncating a longer one.
  #[test]
  fn longer_operators_are_tried_before_their_prefixes() {
    for (i, (text, _)) in OPERATOR_TABLE.iter().enumerate() {
      for (j, (other, _)) in OPERATOR_TABLE.iter().enumerate() {
        if i != j && text.starts_with(*other) && text.len() > other.len() {
          assert!(i < j, "`{text}` must be listed before its prefix `{other}`");
        }
      }
    }
  }

  #[test]
  fn scans_the_three_character_operators_whole() {
    assert_eq!(scan("..."), (Some(TokenKind::DotDotDot), 3));
    assert_eq!(scan("..="), (Some(TokenKind::DotDotEq), 3));
    assert_eq!(scan("??="), (Some(TokenKind::CoalesceEq), 3));
    assert_eq!(scan(">>>"), (Some(TokenKind::Compose), 3));
    assert_eq!(scan("<<<"), (Some(TokenKind::ComposeBack), 3));
  }

  /// Plan 66 criterion 1, and the half the ordering test cannot state.
  ///
  /// `longer_operators_are_tried_before_their_prefixes` proves the table's
  /// order; this proves what the order *buys*, in the one direction a reader
  /// of plan 58 §1.3 will want checked — the shifts are untouched. A `>>>`
  /// listed after `>>` scans two characters here and silently truncates,
  /// which is the failure both tests exist to rule out.
  #[test]
  fn the_shifts_keep_their_own_spellings() {
    assert_eq!(scan(">> 1"), (Some(TokenKind::Shr), 2));
    assert_eq!(scan("<< 1"), (Some(TokenKind::Shl), 2));
    assert_eq!(scan(">"), (Some(TokenKind::Gt), 1));
    assert_eq!(scan("<"), (Some(TokenKind::Lt), 1));
  }

  /// Doc 03 §4.2 corrects doc 01 §21: `..` is exclusive and `..=` inclusive,
  /// matching Rust. Both must lex as themselves, never as two `Dot`s.
  #[test]
  fn scans_both_range_operators() {
    assert_eq!(scan(".."), (Some(TokenKind::DotDot), 2));
    assert_eq!(scan("..=10"), (Some(TokenKind::DotDotEq), 3));
  }

  #[test]
  fn distinguishes_the_dot_family() {
    assert_eq!(scan(".+"), (Some(TokenKind::DotPlus), 2));
    assert_eq!(scan(".("), (Some(TokenKind::DotParen), 2));
    assert_eq!(scan(".x"), (Some(TokenKind::Dot), 1));
  }

  #[test]
  fn distinguishes_the_question_family() {
    assert_eq!(scan("??"), (Some(TokenKind::Coalesce), 2));
    // `?.` is **two** tokens since plan 57 deleted optional chaining.
    assert_eq!(scan("?."), (Some(TokenKind::Question), 1));
    assert_eq!(scan("?"), (Some(TokenKind::Question), 1));
  }

  #[test]
  fn distinguishes_the_at_sigils() {
    assert_eq!(scan("@@"), (Some(TokenKind::AtAt), 2));
    assert_eq!(scan("@x"), (Some(TokenKind::At), 1));
  }

  #[test]
  fn returns_none_for_a_non_operator() {
    assert_eq!(scan("abc"), (None, 0));
  }
}
