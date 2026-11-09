use crate::LexError;
use crate::lexer::cursor::Cursor;
use redox_diag::Code;

/// Advance past an interpolation's body, the cursor just after `#{`, leaving
/// it just after the matching `}` — plan 23.
///
/// ```text
/// "outer #{String::from("inner")} done"
///          ^ cursor in                ^ ... out is here, after the `}`
/// ```
///
/// # Why a string inside an interpolation needs its own skipping
///
/// `lex_fragment`'s doc comment has claimed since plan 02 that
/// `"#{f(x, "y")}"` works, *"and nothing needs a second tokenizer"*. It did
/// not: the **outer** string's scanner stopped at the first unescaped `"`,
/// which is the one opening `"y"`, so the string ended mid-interpolation and
/// the error was `unterminated interpolation; expected }`. Plan 23 found it by
/// writing the case into an example — it is the kind of claim only trying
/// catches, because every simpler interpolation works.
///
/// So the two scanners that walk an interpolation's body — the one finding the
/// outer string's closing quote and the one finding the interpolation's own
/// `}` — must agree, and they agree by both calling this.
///
/// Braces are counted outside string literals; inside one they are text. A
/// nested interpolation inside that nested string is skipped with it and
/// handled when the fragment is lexed recursively, which is the same
/// mechanism one level down.
///
/// Returns the **body's** byte range — what lies between `#{` and its `}` —
/// so the caller that needs to lex it does not have to recompute where it
/// began.
///
/// The cursor enters **on** the `#{` and leaves after the matching `}`, so the
/// function owns the whole region and the diagnostic span covers the `#{` that
/// opened it. Entering after the `#{` instead moved the span four columns
/// right and dropped the two characters the message is about, which the UI
/// corpus caught.
pub(crate) fn skip_interpolation(
  cur: &mut Cursor,
  base: usize,
) -> Result<(usize, usize), LexError> {
  let start = cur.pos();
  cur.eat("#{");
  let body_start = cur.pos();
  let unterminated = |cur: &Cursor| LexError {
    code: Code::UnterminatedInterpolation,
    help: None,
    message: "unterminated interpolation; expected `}`".to_string(),
    span: (base + start, base + cur.pos()),
  };

  let mut depth = 1i32;
  while depth > 0 {
    let Some(ch) = cur.bump() else {
      return Err(unterminated(cur));
    };
    match ch {
      '{' => depth += 1,
      '}' => depth -= 1,
      // A string literal's contents are text, braces included. Escapes are
      // honoured so that `"\""` does not end the string early.
      '"' => loop {
        match cur.bump() {
          None => return Err(unterminated(cur)),
          Some('\\') => {
            cur.bump();
          }
          Some('"') => break,
          Some(_) => {}
        }
      },
      _ => {}
    }
  }
  // Less the closing `}`, which is not part of the body.
  Ok((body_start, cur.pos() - 1))
}
