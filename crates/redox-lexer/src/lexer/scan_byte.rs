use crate::lexer::cursor::Cursor;
use crate::lexer::scan_escape::scan_escape;
use crate::{LexError, TokenKind};
use redox_diag::Code;

/// Scan a `b`-prefixed literal: `b'x'` or `b"…"`.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 5. Needed for binary
/// protocol and parser work (doc 03 §11.2); passed through verbatim by the
/// emitter.
pub(crate) fn scan_byte(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  let err = |message: String, cur: &Cursor| LexError {
    code: Code::UnterminatedByteLiteral,
    help: None,
    message,
    span: (start, cur.pos()),
  };

  cur.bump(); // the `b`

  if cur.eat("'") {
    let ch = if cur.peek() == Some('\\') {
      cur.bump();
      scan_escape(cur)?
    } else {
      cur
        .bump()
        .ok_or_else(|| err("unterminated byte literal".to_string(), cur))?
    };
    if !cur.eat("'") {
      return Err(err("unterminated byte literal".to_string(), cur));
    }
    let value =
      u8::try_from(u32::from(ch)).map_err(|_| err(format!("`{ch}` is not a single byte"), cur))?;
    return Ok(TokenKind::Byte(value));
  }

  if cur.eat("\"") {
    let mut text = String::new();
    while !cur.starts_with("\"") {
      if cur.is_eof() {
        return Err(err("unterminated byte string".to_string(), cur));
      }
      if cur.peek() == Some('\\') {
        cur.bump();
        text.push(scan_escape(cur)?);
      } else {
        text.push(cur.bump().expect("not at eof"));
      }
    }
    cur.eat("\"");
    return Ok(TokenKind::ByteStr(text));
  }

  Err(err("`b` must be followed by `'` or `\"`".to_string(), cur))
}
