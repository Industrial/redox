pub(crate) mod split_parts;

use crate::lexer::cursor::Cursor;
use crate::lexer::dedent_block::dedent_block;
use crate::lexer::scan_string::split_parts::split_parts;
use crate::{LexError, TokenKind};

/// Scan a `"…"` or `"""…"""` literal, the cursor on the opening quote.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaves 4 and 5.
///
/// A `"""` block is dedented and then handed to the same part-splitting logic
/// a single-quoted string uses, which is why interpolation works inside it for
/// free — the property Emerald's 447-line heredoc pass had to be built to
/// obtain, and which a hand-written cursor gives without effort.
pub(crate) fn scan_string(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  let line = cur.line();
  let col = cur.col();
  let err = |message: String, cur: &Cursor| LexError {
    message,
    span: (start, cur.pos()),
    line,
    col,
  };

  let triple = cur.starts_with("\"\"\"");
  let delim = if triple { "\"\"\"" } else { "\"" };
  cur.eat(delim);

  // Collect the raw body, stopping at the closing delimiter. Escapes are not
  // decoded here — the body is split into parts first, so an escaped `\#`
  // cannot open an interpolation.
  let body_start = cur.pos();
  loop {
    if cur.is_eof() {
      return Err(err(
        format!("unterminated string literal; expected closing {delim}"),
        cur,
      ));
    }
    if cur.starts_with(delim) {
      break;
    }
    if cur.peek() == Some('\\') {
      cur.bump();
      cur.bump();
      continue;
    }
    if !triple && cur.peek() == Some('\n') {
      // Names both the likely cause (a missing closing quote) and the
      // alternative, because a reader who meant to span lines needs the
      // second half and a reader who forgot a quote needs the first.
      return Err(err(
        "unterminated string literal: a single-quoted string may not span \
         lines — use \"\"\" for a multi-line string"
          .to_string(),
        cur,
      ));
    }
    cur.bump();
  }
  let body_end = cur.pos();
  cur.eat(delim);

  let raw = cur.slice((start, cur.pos())).to_string();
  let body = cur.slice((body_start, body_end));

  // A `"""` block drops its delimiter lines and loses its common indentation
  // (doc 01 §7.2), before any part splitting.
  let owned = if triple {
    let inner = body.strip_prefix('\n').unwrap_or(body);
    let inner = inner.trim_end_matches(' ');
    dedent_block(inner.strip_suffix('\n').unwrap_or(inner))
  } else {
    body.to_string()
  };

  Ok(TokenKind::Str {
    parts: split_parts(&owned, body_start, line, col)?,
    raw,
  })
}
