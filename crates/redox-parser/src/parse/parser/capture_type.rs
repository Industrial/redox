use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::can_end_a_type::can_end_a_type;
use crate::parse::parser::token_text::token_text;
use crate::parse::parser::type_stop::TypeStop;
use crate::parse::parser::ungrouped::ungrouped;
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
/// place: **an `->` after a parenthesised group that followed a name does not
/// end a type**, because that is `Fn(i32) -> i32` rather than a signature's
/// arrow. Without it, `&dyn Fn(i32) -> i32` could not be a parameter type and
/// doc 03 §14.1's "every Rust type form is reachable" would be false on day
/// one.
///
/// "That followed a name" is the half the first version left out, and it cost
/// a wrong answer rather than a missing one: `() -> Option<T>` is a spine
/// whose first type is **unit**, and testing only for a preceding `)`
/// swallowed the arrow and the return type into it. The function then
/// reported zero parameters where the equation bound one, so the diagnostic
/// blamed the equation.
///
/// `}` and `|` joined the set with doc 18 §2.4's declarations, which are the
/// first place a type is followed by either: `{ x : f64, y : f64 }` ended
/// its last field's type with the brace *inside* it, and `Key char | Close`
/// swallowed the bar. A string literal joined it with doc 03 §12.5's
/// `errors` rows, where the `Display` message follows the payload types with
/// no separator. None of them can occur in a Rust type at depth zero, so
/// none costs a type form.
pub(crate) fn capture_type(
  p: &mut Parser,
  message: &str,
  stop: TypeStop,
) -> Result<TypeExpr, ParseError> {
  let start = p.peek_span().0;
  let mut depth = 0i32;
  let mut text = String::new();
  // Whether the `(` of the group just closed followed an identifier, which
  // is what makes it `Fn(…)` rather than a unit or a tuple.
  let mut group_followed_a_name = false;
  let mut prev_was_a_name = false;
  let mut closed_a_named_group = false;
  let mut prev_kind = TokenKind::Eof;

  loop {
    let kind = &p.peek().kind;

    // A juxtaposed payload list reads one type per item, so a whitespace
    // boundary between something that can *end* a type and something that
    // can *begin* one ends this atom. Without it `Invalid String u32` is a
    // single field spelled `String u32`.
    if depth == 0
      && stop == TypeStop::Atom
      && !text.is_empty()
      && p.peek().ws_before
      && can_end_a_type(&prev_kind)
    {
      break;
    }

    // **`Eof` ends the loop at any depth**, and this is a termination proof
    // rather than a convenience. Every other condition below is guarded by
    // `depth == 0`, and `p.advance()` at `Eof` does not advance — so a depth
    // that is wrong by even one turns this loop into a hang that grows `text`
    // until the process is killed. That happened: see the stray-closer note
    // on the match below.
    if matches!(kind, TokenKind::Eof) {
      break;
    }

    if depth == 0 {
      let ends = match kind {
        TokenKind::Comma | TokenKind::RParen | TokenKind::Eq => true,
        // Doc 18 §2.4: a record's closing brace, and the bar between two
        // constructors.
        TokenKind::RBrace | TokenKind::Or | TokenKind::Pipe => true,
        // Doc 03 §12.5: an `errors` row's `Display` message follows its
        // payload types with no separator, which is what lets the rows read
        // as a table. A string literal cannot occur in a Rust type, so
        // ending here costs no type form.
        TokenKind::Str { .. } | TokenKind::RawStr { .. } => true,
        // `as` never occurs inside a Rust type — it is a cast operator — and
        // doc 03 §12.5's `from io::Error as Io` needs the type to end there.
        TokenKind::Kw(redox_lexer::Keyword::As) => true,
        // The refinement: `Fn(i32) -> i32` keeps going; `: i32 ->` stops.
        // The `(` has to have followed a *name* for that to apply — `()` is
        // the unit type, and `() -> Option<T>` is a signature spine whose
        // first type is unit, not a function type. Testing only for a
        // preceding `)` swallowed the arrow and the return type into the
        // unit, and reported the arity of a zero-parameter function as zero
        // when the equation bound one.
        TokenKind::Arrow => !closed_a_named_group,
        TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => true,
        // **A closer with no opener ends the type**, for the same reason
        // `RParen` and `RBrace` two arms up do: at depth zero there is
        // nothing for it to close, so it belongs to whatever encloses this
        // type and the caller's own `expect` should be the one to complain
        // about it.
        //
        // Without this, `>` fell through to the match below and took `depth`
        // to **-1**, which disabled every condition in this block
        // permanently — a 28-byte input (`n : i32 !> i32 = 1`) hung the
        // compiler until the OOM killer stopped it. No valid Rust type has a
        // depth-zero closer: in `Vec<u8>` the `<` has already raised the
        // depth before the `>` arrives, and `[u8; 4]` works the same way.
        //
        // `>>>` is here for the same reason, and plan 66 is what adds it:
        // `Vec<Vec<Vec<i32>>>` closes as one token, so a type ending in a
        // triple closer reaches depth zero on it.
        TokenKind::Gt | TokenKind::Shr | TokenKind::Compose | TokenKind::RBracket => true,
        _ => false,
      };
      if ends {
        break;
      }
    }

    match kind {
      TokenKind::LParen => {
        if depth == 0 {
          group_followed_a_name = prev_was_a_name;
        }
        depth += 1;
      }
      TokenKind::Lt | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
      // Reached only at `depth > 0` — every one of these is a depth-zero
      // terminator above, which is what keeps `depth` from going negative and
      // is the invariant the `Eof` guard no longer has to rely on.
      TokenKind::Gt | TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
      // `>>` closes two generic argument lists, as in `Vec<Vec<u8>>`.
      TokenKind::Shr => depth -= 2,
      // And `>>>` closes three, as in `Vec<Vec<Vec<i32>>>` — plan 66. The
      // **opening** side needs nothing: an identifier separates the angles of
      // a nested generic, so that type opens as three `Lt` tokens.
      TokenKind::Compose => depth -= 3,
      _ => {}
    }
    closed_a_named_group = matches!(kind, TokenKind::RParen) && group_followed_a_name;
    prev_was_a_name = matches!(
      kind,
      TokenKind::Ident(_) | TokenKind::Kw(redox_lexer::Keyword::Fn)
    );

    if !text.is_empty() && p.peek().ws_before {
      text.push(' ');
    }
    text.push_str(&token_text(kind));
    prev_kind = kind.clone();
    p.advance();
  }

  if text.is_empty() {
    return Err(ParseError::with(Code::ExpectedType, message, p.peek_span()));
  }
  Ok(TypeExpr {
    text: ungrouped(&text),
    span: (start, p.prev_span().1),
  })
}
