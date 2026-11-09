use crate::lexer::cursor::Cursor;
use crate::lexer::is_ident_continue::is_ident_continue;
use crate::{LexError, TokenKind};

/// Scan a raw identifier `r#type`, the cursor on the `r` — plan 45 leaf 4.
///
/// Doc 03 §18 lists raw identifiers among the constructs a Redox program
/// cannot write, and plan 45 §1.4 measured *why* that is more than an
/// omission: `r#type` did not fail as an unknown token, it failed with
/// `RDX0004: a raw string must open with `"` after `r` and any `#``. Plan 23's
/// raw strings had already claimed the spelling, and the lexer committed to
/// reading one before it reached the `t`.
///
/// So the fix is the lookahead **Rust's own lexer** uses, one branch earlier:
/// after `r` and a single `#`, an identifier start means a raw identifier and
/// a `"` means a raw string. Adding `r#ident` to the identifier rule instead
/// would have read `r#"x"` as the identifier `r#x` followed by a stray quote,
/// and surfaced as a mangled string rather than a syntax error.
///
/// The `r#` **stays in the token's text**. A raw identifier exists to use a
/// Rust keyword as a name, so `r#type` must reach the `.xrs` as `r#type`;
/// stripping the prefix would emit the bare keyword and produce Rust that
/// does not parse. Keeping it also means [`crate::Keyword::from_text`] never
/// matches, which is the behaviour wanted — `r#type` is an identifier in
/// Redox too.
pub(crate) fn scan_raw_ident(cur: &mut Cursor) -> Result<TokenKind, LexError> {
  let start = cur.pos();
  cur.bump(); // the `r`
  cur.bump(); // the `#`
  cur.eat_while(|c| is_ident_continue(c as u8));
  Ok(TokenKind::Ident(cur.slice((start, cur.pos())).to_string()))
}
