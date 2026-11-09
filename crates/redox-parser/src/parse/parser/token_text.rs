use redox_lexer::TokenKind;

/// The source spelling of a token.
///
/// Used to re-emit a captured type (plan 03 §1.1), where the parser keeps text
/// rather than structure. A sibling of `capture_type` rather than a child
/// because doc 06 §2.4 caps depth at three levels below `src/`, and because
/// re-spelling a token is a general operation rather than one the type scan
/// owns.
pub(crate) fn token_text(kind: &TokenKind) -> String {
  match kind {
    TokenKind::Ident(t) => t.clone(),
    TokenKind::Kw(kw) => kw.text().to_string(),
    TokenKind::Int(t) | TokenKind::Float(t) => t.clone(),
    // `char`'s and `u8`'s `Debug` spellings *are* Rust source syntax, escapes
    // included, which is why re-spelling a literal needs no escape table of
    // its own. Reached from a literal pattern rather than from a type: doc 18
    // §2.3's `case 'a' or 'e' -> …` was emitting `Char('a')`, the debug-arm
    // marker, because the literal arms came through here and only the type
    // tokens had spellings.
    TokenKind::Char(c) => format!("{c:?}"),
    TokenKind::Byte(b) => format!("b{:?}", char::from(*b)),
    TokenKind::Str { raw, .. } => raw.clone(),
    TokenKind::Label(name) => format!("'{name}"),
    TokenKind::ColonColon => "::".to_string(),
    TokenKind::Lt => "<".to_string(),
    TokenKind::Gt => ">".to_string(),
    TokenKind::Shr => ">>".to_string(),
    TokenKind::LParen => "(".to_string(),
    TokenKind::RParen => ")".to_string(),
    TokenKind::LBracket => "[".to_string(),
    TokenKind::RBracket => "]".to_string(),
    // Doc 03 §13.1's brace-nested import trees — `{self, Read, Write}`. The
    // braces reached the debug fallback and emitted `LBraceself`, which is
    // what that fallback is for: a visible marker rather than a silent wrong
    // answer.
    TokenKind::LBrace => "{".to_string(),
    TokenKind::RBrace => "}".to_string(),
    TokenKind::Comma => ",".to_string(),
    TokenKind::Semi => ";".to_string(),
    TokenKind::Amp => "&".to_string(),
    TokenKind::Star => "*".to_string(),
    TokenKind::Plus => "+".to_string(),
    TokenKind::Arrow => "->".to_string(),
    TokenKind::Colon => ":".to_string(),
    TokenKind::Bang => "!".to_string(),
    TokenKind::Dot => ".".to_string(),
    // A token with no business inside a type reaches here only if the scan's
    // terminator set is wrong, so the debug spelling is a visible marker
    // rather than a silent wrong answer.
    other => format!("{other:?}"),
  }
}

#[cfg(test)]
mod tests {
  use super::token_text;
  use redox_lexer::{Keyword, TokenKind};

  #[test]
  fn respells_the_tokens_a_type_can_contain() {
    assert_eq!(token_text(&TokenKind::Ident("Vec".into())), "Vec");
    assert_eq!(token_text(&TokenKind::Lt), "<");
    assert_eq!(token_text(&TokenKind::Shr), ">>");
    assert_eq!(token_text(&TokenKind::Amp), "&");
    assert_eq!(token_text(&TokenKind::Kw(Keyword::Mut)), "mut");
    assert_eq!(token_text(&TokenKind::Label("a".into())), "'a");
  }
}
