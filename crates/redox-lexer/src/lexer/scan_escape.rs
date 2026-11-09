use crate::LexError;
use crate::lexer::cursor::Cursor;
use redox_diag::Code;

/// Decode one escape sequence, the backslash already consumed.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 4. An unknown escape is
/// a named error rather than a silent passthrough, because a silently-kept
/// `\q` would reach the generated Rust and fail there instead, pointing at a
/// line the user never wrote.
pub(crate) fn scan_escape(cur: &mut Cursor) -> Result<char, LexError> {
  let start = cur.pos().saturating_sub(1);

  let err = |message: String, cur: &Cursor| LexError {
    code: Code::UnknownEscape,
    help: None,
    message,
    span: (start, cur.pos()),
  };

  let Some(ch) = cur.bump() else {
    return Err(err("unterminated escape sequence".to_string(), cur));
  };

  Ok(match ch {
    'n' => '\n',
    'r' => '\r',
    't' => '\t',
    '0' => '\0',
    '\\' => '\\',
    '"' => '"',
    '\'' => '\'',
    '#' => '#',
    'x' => {
      // **Exactly two characters, not as many hex digits as follow.** Rust
      // reads two and stops, so `b"\\x7fELF"` is `0x7f` then `E`, `L`, `F`.
      // Scanning greedily consumed the `E` as a third digit and rejected a
      // literal rustc accepts — found by plan 23's example, which is the only
      // place a hex escape has ever been followed by a letter.
      let start = cur.pos();
      cur.bump();
      cur.bump();
      let text = cur.slice((start, cur.pos()));
      if text.len() != 2 || !text.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(err(
          "`\\x` takes exactly two hexadecimal digits".to_string(),
          cur,
        ));
      }
      let value = u8::from_str_radix(text, 16)
        .map_err(|_| err(format!("`\\x{text}` is not a valid byte escape"), cur))?;
      value as char
    }
    'u' => {
      if !cur.eat("{") {
        return Err(err("`\\u` must be followed by `{`".to_string(), cur));
      }
      let digits = cur.eat_while(|c| c.is_ascii_hexdigit());
      let text = cur.slice(digits).to_string();
      if !cur.eat("}") {
        return Err(err("unterminated `\\u{…}` escape".to_string(), cur));
      }
      let value = u32::from_str_radix(&text, 16)
        .map_err(|_| err(format!("`\\u{{{text}}}` is not valid hexadecimal"), cur))?;
      char::from_u32(value).ok_or_else(|| {
        err(
          format!("`\\u{{{text}}}` is not a Unicode scalar value"),
          cur,
        )
      })?
    }
    other => {
      return Err(err(format!("unknown escape sequence `\\{other}`"), cur));
    }
  })
}

#[cfg(test)]
mod tests {
  use super::scan_escape;
  use crate::lexer::cursor::Cursor;

  fn decode(after_backslash: &str) -> Result<char, String> {
    let mut cur = Cursor::new(after_backslash);
    scan_escape(&mut cur).map_err(|e| e.message)
  }

  #[test]
  fn decodes_the_simple_escapes() {
    assert_eq!(decode("n"), Ok('\n'));
    assert_eq!(decode("t"), Ok('\t'));
    assert_eq!(decode("0"), Ok('\0'));
    assert_eq!(decode("\\"), Ok('\\'));
    assert_eq!(decode("\""), Ok('"'));
    // `\#` exists because `#{` is interpolation inside an ordinary string.
    assert_eq!(decode("#"), Ok('#'));
  }

  #[test]
  fn decodes_byte_and_unicode_escapes() {
    assert_eq!(decode("x41"), Ok('A'));
    assert_eq!(decode("u{1F600}"), Ok('\u{1F600}'));
  }

  #[test]
  fn rejects_a_malformed_escape() {
    assert!(decode("q").is_err(), "unknown escape should fail");
    assert!(decode("x4").is_err(), "short byte escape should fail");
    assert!(decode("u41").is_err(), "missing brace should fail");
    assert!(
      decode("u{D800}").is_err(),
      "a surrogate is not a scalar value"
    );
  }
}
