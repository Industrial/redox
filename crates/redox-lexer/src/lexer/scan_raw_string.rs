use crate::lexer::cursor::Cursor;
use crate::{LexError, TokenKind};

/// Scan a raw string `r"…"` or `r#"…"#`, the cursor on the `r`.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 5. No escape processing
/// and no interpolation — more necessary in Redox than in Rust, because `#{`
/// is live inside an ordinary string literal, so a regex or a Windows path
/// needs a form that does not interpret it.
pub(crate) fn scan_raw_string(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  let line = cur.line();
  let col = cur.col();
  cur.bump(); // the `r`

  let hash_span = cur.eat_while(|c| c == '#');
  let hashes = cur.slice(hash_span).len();

  if !cur.eat("\"") {
    return Err(LexError {
      message: "a raw string must open with `\"` after `r` and any `#`".to_string(),
      span: (start, cur.pos()),
      line,
      col,
    });
  }

  let closing = format!("\"{}", "#".repeat(hashes));
  let text_start = cur.pos();
  while !cur.starts_with(&closing) {
    if cur.bump().is_none() {
      return Err(LexError {
        message: format!("unterminated raw string; expected closing `{closing}`"),
        span: (start, cur.pos()),
        line,
        col,
      });
    }
  }
  let text = cur.slice((text_start, cur.pos())).to_string();
  cur.eat(&closing);
  Ok(TokenKind::RawStr { text, hashes })
}
