use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::token_text::token_text;
use redox_diag::Code;
use redox_lexer::TokenKind;

/// Capture an import path as **verbatim source text** — doc 03 §13.1, plan
/// 14 §1.3.
///
/// ```text
/// std::collections::HashMap
/// std::io::{self, Read, Write}
/// crate::models::*
/// ```
///
/// Doc 03 §14.1's discipline, applied where it applies equally: the lowering
/// is `use` plus this text, so a nested tree Redox parsed would be a nested
/// tree Redox could lag behind. §13.1's five forms are five spellings of one
/// string.
///
/// The scan ends at a line break or at `as`, and nowhere else. `as` is a
/// reserved word, so it is the one token understood rather than copied —
/// `capture_type` already stops there, a refinement plan 17 added for
/// `errors` rows and this reuses unchanged.
///
/// Brace depth is tracked so a `,` **inside** a tree is part of the path
/// while one outside would not be — which is not currently reachable, since
/// nothing follows an import on its line, and is tracked anyway because the
/// alternative is a scan that happens to be right.
pub(crate) fn import_path(p: &mut Parser) -> Result<String, ParseError> {
  let start = p.peek_span();
  let mut depth = 0i32;
  let mut text = String::new();

  loop {
    let kind = &p.peek().kind;
    if depth == 0
      && matches!(
        kind,
        TokenKind::Newline
          | TokenKind::Indent
          | TokenKind::Dedent
          | TokenKind::Eof
          | TokenKind::Kw(redox_lexer::Keyword::As)
      )
    {
      break;
    }

    match kind {
      TokenKind::LBrace => depth += 1,
      TokenKind::RBrace => depth -= 1,
      _ => {}
    }

    // No spaces are inserted. A path has none at depth zero, and inside a
    // tree Rust's own style is `{self, Read, Write}` — one space after each
    // comma, which is what the comma arm writes.
    text.push_str(&token_text(kind));
    if matches!(kind, TokenKind::Comma) {
      text.push(' ');
    }
    p.advance();
  }

  if text.is_empty() {
    return Err(ParseError::with(
      Code::ExpectedName,
      "expected a module path after `import`",
      start,
    ));
  }
  Ok(text)
}
