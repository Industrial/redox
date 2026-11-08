use crate::lexer::cursor::Cursor;
use crate::lexer::scan_escape::scan_escape;
use crate::{LexError, TokenKind};
use redox_diag::Code;

/// Scan a `'`: a char literal, or a lifetime/label.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 7. Doc 03 §4.3 fixes
/// this as **Rust's own rule — lookahead, never context**: `'x'` is a char
/// when a closing quote follows one character or one escape; otherwise
/// `'name` is a lifetime or a block label.
///
/// Context-sensitivity here would make `'a'` mean different things in
/// different positions, which doc 02 §5.2 names as the worst available failure
/// mode.
pub(crate) fn scan_quote(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  let mark = cur.mark();

  cur.bump(); // the `'`

  // Try a char literal first: one character (or one escape), then `'`.
  let decoded = if cur.peek() == Some('\\') {
    cur.bump();
    scan_escape(cur).ok()
  } else {
    cur.bump()
  };

  if let Some(ch) = decoded
    && cur.peek() == Some('\'')
  {
    cur.bump();
    return Ok(TokenKind::Char(ch));
  }

  // Not a char literal: rewind and read a lifetime or label name.
  cur.reset_to(mark);
  cur.bump(); // the `'`
  let name_span = cur.eat_while(|c| c.is_ascii_alphanumeric() || c == '_');
  let name = cur.slice(name_span).to_string();
  if name.is_empty() {
    return Err(LexError {
      code: Code::BadQuote,
      help: None,
      message: "a `'` must open a character literal or name a lifetime or label".to_string(),
      span: (start, cur.pos()),
    });
  }
  Ok(TokenKind::Label(name))
}
