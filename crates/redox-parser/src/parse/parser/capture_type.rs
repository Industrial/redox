use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::token_text::token_text;
use redox_ast::TypeExpr;
use redox_diag::Code;
use redox_lexer::TokenKind;

/// Capture a type as its **verbatim source text**.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) §1.1. Doc 03 §14.1 fixes type
/// expressions as copied through unchanged, so this does not parse a type
/// grammar — it finds where the type ends and keeps the span.
///
/// The scan stops at the first `,`, `)`, `}`, `|` , `=` or `->` at angle,
/// paren, bracket and brace depth zero, with one refinement that earns its
/// place: **an `->` preceded by `)` does not end a type**, because that is
/// `Fn(i32) -> i32` rather than a signature's arrow. Without it, `&dyn
/// Fn(i32) -> i32` could not be a parameter type and doc 03 §14.1's "every
/// Rust type form is reachable" would be false on day one.
///
/// `}` and `|` joined the set with doc 18 §2.4's declarations, which are the
/// first place a type is followed by either: `{ x :: f64, y :: f64 }` ended
/// its last field's type with the brace *inside* it, and `Key char | Close`
/// swallowed the bar. Neither token can occur in a Rust type at depth zero,
/// so neither costs a type form.
pub(crate) fn capture_type(p: &mut Parser, message: &str) -> Result<TypeExpr, ParseError> {
  let start = p.peek_span().0;
  let mut depth = 0i32;
  let mut text = String::new();
  let mut prev_was_close_paren = false;

  loop {
    let kind = &p.peek().kind;

    if depth == 0 {
      let ends = match kind {
        TokenKind::Comma | TokenKind::RParen | TokenKind::Eq => true,
        // Doc 18 §2.4: a record's closing brace, and the bar between two
        // constructors.
        TokenKind::RBrace | TokenKind::Or | TokenKind::Pipe => true,
        // The refinement: `Fn(i32) -> i32` keeps going; `: i32 ->` stops.
        TokenKind::Arrow => !prev_was_close_paren,
        TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => true,
        _ => false,
      };
      if ends {
        break;
      }
    }

    match kind {
      TokenKind::Lt | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
      TokenKind::Gt | TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
      // `>>` closes two generic argument lists, as in `Vec<Vec<u8>>`.
      TokenKind::Shr => depth -= 2,
      _ => {}
    }
    prev_was_close_paren = matches!(kind, TokenKind::RParen);

    if !text.is_empty() && p.peek().ws_before {
      text.push(' ');
    }
    text.push_str(&token_text(kind));
    p.advance();
  }

  if text.is_empty() {
    return Err(ParseError::with(Code::ExpectedType, message, p.peek_span()));
  }
  Ok(TypeExpr {
    text,
    span: (start, p.prev_span().1),
  })
}
