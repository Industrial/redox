use crate::lexer::cursor::Cursor;
use crate::lexer::scan_escape::scan_escape;
use crate::{LexError, TokenKind};

/// Scan a `c"…"` nul-terminated string literal.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 5. Required for FFI,
/// which doc 01 §26 lists as a requirement and doc 03 §18 still records as
/// otherwise unreachable.
pub(crate) fn scan_cstring(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  let line = cur.line();
  let col = cur.col();
  cur.bump(); // the `c`
  cur.eat("\"");

  let mut text = String::new();
  while !cur.starts_with("\"") {
    if cur.is_eof() {
      return Err(LexError {
        message: "unterminated C string".to_string(),
        span: (start, cur.pos()),
        line,
        col,
      });
    }
    if cur.peek() == Some('\\') {
      cur.bump();
      text.push(scan_escape(cur)?);
    } else {
      text.push(cur.bump().expect("not at eof"));
    }
  }
  cur.eat("\"");
  Ok(TokenKind::CStr(text))
}
