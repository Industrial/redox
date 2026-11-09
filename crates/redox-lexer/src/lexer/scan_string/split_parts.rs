use crate::LexError;
use crate::lexer::cursor::Cursor;
use crate::lexer::lex_fragment::lex_fragment;
use crate::lexer::scan_escape::scan_escape;
use crate::lexer::skip_interpolation::skip_interpolation;
use crate::lexer::split_format_spec::split_format_spec;
use crate::token::str_part::StrPart;

/// Split a string body into literal text and interpolations.
///
/// Plan 02 (`docs/07 - 02 - Lexical Surface.md`), leaf 4. `base` is the body's
/// byte offset in the original file, so interpolated tokens carry spans that
/// index the real source.
pub(crate) fn split_parts(body: &str, base: usize) -> Result<Vec<StrPart>, LexError> {
  let mut parts = Vec::new();
  let mut text = String::new();
  let mut cur = Cursor::new(body);

  while !cur.is_eof() {
    if cur.starts_with("#{") {
      if !text.is_empty() {
        parts.push(StrPart::Text(std::mem::take(&mut text)));
      }

      // The body ends at the `}` that returns brace depth to zero, so a
      // struct literal or a closure inside an interpolation stays intact —
      // and a *string* inside it keeps its braces, which is why this is the
      // same function the outer scanner uses. Two scanners walking the same
      // text must agree about where it ends; plan 23 §10 records what it cost
      // when they did not.
      let (open, close) = skip_interpolation(&mut cur, base)?;

      let inner = &body[open..close];
      let (expr, spec) = split_format_spec(inner);
      parts.push(StrPart::Interp {
        tokens: lex_fragment(expr, base + open)?,
        spec: spec.map(str::to_string),
      });
      continue;
    }

    if cur.peek() == Some('\\') {
      cur.bump();
      // `cur` runs over the string's body, so an escape error reports an
      // offset into that body; rebase it onto the file or the diagnostic
      // names the wrong line entirely.
      text.push(scan_escape(&mut cur).map_err(|e| e.offset_by(base))?);
      continue;
    }

    text.push(cur.bump().expect("not at eof"));
  }

  if !text.is_empty() {
    parts.push(StrPart::Text(text));
  }
  Ok(parts)
}
