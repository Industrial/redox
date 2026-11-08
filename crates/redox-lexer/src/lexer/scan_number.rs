use crate::lexer::cursor::Cursor;
use crate::{LexError, TokenKind};

/// Scan a numeric literal, the cursor on its first digit.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 6. The text is carried
/// **verbatim**, including `_` separators and any Rust type suffix, so the
/// emitter passes it straight through and Redox never has to agree with rustc
/// about numeric limits.
pub(crate) fn scan_number(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  let line = cur.line();
  let col = cur.col();
  let err = |message: String, cur: &Cursor| LexError {
    message,
    span: (start, cur.pos()),
    line,
    col,
  };

  // A radix prefix forbids a fractional part, so handle it first and return.
  if cur.peek() == Some('0')
    && let Some(radix) = cur.peek_at(1)
  {
    let (name, valid): (&str, fn(char) -> bool) = match radix {
      'x' | 'X' => ("hexadecimal", |c| c.is_ascii_hexdigit()),
      'o' | 'O' => ("octal", |c| matches!(c, '0'..='7')),
      'b' | 'B' => ("binary", |c| matches!(c, '0' | '1')),
      _ => ("", |_| false),
    };
    if !name.is_empty() {
      cur.bump();
      cur.bump();
      let digits = cur.eat_while(|c| valid(c) || c == '_');
      if cur.slice(digits).chars().all(|c| c == '_') {
        return Err(err(format!("{name} literal has no digits"), cur));
      }
      // A suffix such as `u8` follows the digits directly.
      cur.eat_while(|c| c.is_ascii_alphanumeric() || c == '_');
      return Ok(TokenKind::Int(cur.slice((start, cur.pos())).to_string()));
    }
  }

  cur.eat_while(|c| c.is_ascii_digit() || c == '_');

  let mut is_float = false;

  // A `.` is a fractional point only when a digit follows it; `1..10` is a
  // range and `1.method()` is a method call, and both must stay intact.
  if cur.peek() == Some('.') && cur.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
    is_float = true;
    cur.bump();
    cur.eat_while(|c| c.is_ascii_digit() || c == '_');
  }

  if matches!(cur.peek(), Some('e' | 'E'))
    && cur
      .peek_at(1)
      .is_some_and(|c| c.is_ascii_digit() || c == '+' || c == '-')
  {
    is_float = true;
    cur.bump();
    if matches!(cur.peek(), Some('+' | '-')) {
      cur.bump();
    }
    let digits = cur.eat_while(|c| c.is_ascii_digit() || c == '_');
    if cur.slice(digits).is_empty() {
      return Err(err("exponent has no digits".to_string(), cur));
    }
  }

  cur.eat_while(|c| c.is_ascii_alphanumeric() || c == '_');

  let text = cur.slice((start, cur.pos())).to_string();
  Ok(if is_float {
    TokenKind::Float(text)
  } else {
    TokenKind::Int(text)
  })
}

#[cfg(test)]
mod tests {
  use super::scan_number;
  use crate::TokenKind;
  use crate::lexer::cursor::Cursor;

  fn scan(src: &str) -> (TokenKind, usize) {
    let mut cur = Cursor::new(src);
    let kind = scan_number(&mut cur).expect("should scan");
    (kind, cur.pos())
  }

  #[test]
  fn scans_decimal_integers_with_separators() {
    assert_eq!(scan("1_000_000").0, TokenKind::Int("1_000_000".into()));
    assert_eq!(scan("42").0, TokenKind::Int("42".into()));
  }

  #[test]
  fn scans_radix_prefixes() {
    assert_eq!(scan("0xDEAD_beef").0, TokenKind::Int("0xDEAD_beef".into()));
    assert_eq!(scan("0o755").0, TokenKind::Int("0o755".into()));
    assert_eq!(scan("0b1010").0, TokenKind::Int("0b1010".into()));
  }

  #[test]
  fn scans_floats_with_exponents() {
    assert_eq!(scan("2.5").0, TokenKind::Float("2.5".into()));
    assert_eq!(scan("2.5e-3").0, TokenKind::Float("2.5e-3".into()));
    assert_eq!(scan("1E10").0, TokenKind::Float("1E10".into()));
  }

  #[test]
  fn carries_a_type_suffix_verbatim() {
    assert_eq!(scan("42u8").0, TokenKind::Int("42u8".into()));
    assert_eq!(scan("1.5f64").0, TokenKind::Float("1.5f64".into()));
  }

  /// `1..10` is a range, not a malformed float: the `.` is only a fractional
  /// point when a digit follows it.
  #[test]
  fn leaves_a_range_operator_alone() {
    let (kind, end) = scan("1..10");
    assert_eq!(kind, TokenKind::Int("1".into()));
    assert_eq!(end, 1, "the cursor stops before `..`");
  }

  /// `1.method()` likewise.
  #[test]
  fn leaves_a_method_call_alone() {
    let (kind, end) = scan("1.abs()");
    assert_eq!(kind, TokenKind::Int("1".into()));
    assert_eq!(end, 1);
  }

  #[test]
  fn rejects_a_radix_prefix_with_no_digits() {
    let mut cur = Cursor::new("0x");
    assert!(scan_number(&mut cur).is_err());
  }
}
