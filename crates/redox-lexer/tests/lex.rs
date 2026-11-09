//! Behaviour of [`redox_lexer::lex`], through the public API only.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §7: these exercise the crate's
//! contract, so they must not be able to reach its internals.

use redox_lexer::{StrPart, TokenKind, Trivia, TriviaKind, lex};

/// Plan 02 amended criterion 2: the string token changed from raw text to a
/// decoded structure, so this helper rebuilds the expected token rather than
/// every assertion naming `parts` and `raw` separately. The behavioural claims
/// plan 01 made are unchanged.
fn str_token(raw: &str) -> TokenKind {
  let text = raw.trim_matches('"');
  TokenKind::Str {
    parts: if text.is_empty() {
      Vec::new()
    } else {
      vec![StrPart::Text(text.to_string())]
    },
    raw: raw.to_string(),
  }
}

fn kinds(src: &str) -> Vec<TokenKind> {
  lex(src)
    .expect("should lex")
    .into_iter()
    .map(|t| t.kind)
    .collect()
}

fn comment_of(trivia: &[Trivia]) -> Option<&str> {
  trivia
    .iter()
    .find(|t| t.kind == TriviaKind::LineComment)
    .map(|t| t.text.as_str())
}

#[test]
fn lexes_the_hello_slice() {
  assert_eq!(
    kinds("main :: ()\nmain =\n  print \"hello\"\n"),
    vec![
      // Doc 18 §2.2's signature line, then §2.3's equation. The `::` is one
      // token and `()` is the unit return type, which `signature_from` omits
      // from the generated `fn`.
      TokenKind::Ident("main".into()),
      TokenKind::ColonColon,
      TokenKind::LParen,
      TokenKind::RParen,
      TokenKind::Newline,
      TokenKind::Ident("main".into()),
      TokenKind::Eq,
      TokenKind::Newline,
      TokenKind::Indent,
      TokenKind::Ident("print".into()),
      str_token("\"hello\""),
      TokenKind::Newline,
      TokenKind::Dedent,
      TokenKind::Eof,
    ]
  );
}

/// Plan 01 acceptance criterion 6: spans index the real source text.
#[test]
fn tokens_carry_byte_spans() {
  let src = "main :: ()\nmain =\n  print \"hello\"\n";
  let toks = lex(src).expect("should lex");
  assert_eq!(toks[0].span, (0, 4));
  assert_eq!(&src[toks[0].span.0..toks[0].span.1], "main");
  let lit = toks
    .iter()
    .find(|t| matches!(t.kind, TokenKind::Str { .. }))
    .expect("a literal");
  assert_eq!(&src[lit.span.0..lit.span.1], "\"hello\"");
}

#[test]
fn attaches_a_leading_comment_to_the_following_token() {
  let toks = lex("-- a note\nmain :: ()\nmain =\n  print \"x\"\n").expect("should lex");
  assert_eq!(comment_of(&toks[0].leading), Some("a note"));
}

#[test]
fn attaches_a_trailing_comment_to_the_closing_newline() {
  let toks = lex("main = = -- why\n  print \"x\"\n").expect("should lex");
  let nl = toks
    .iter()
    .find(|t| t.kind == TokenKind::Newline)
    .expect("a newline");
  assert_eq!(nl.trailing.as_ref().map(|t| t.text.as_str()), Some("why"));
}

#[test]
fn collapses_a_blank_run_into_one_trivium() {
  let toks = lex("-- a\n\n\n\nmain :: ()\nmain =\n  print \"x\"\n").expect("should lex");
  let kinds: Vec<_> = toks[0].leading.iter().map(|t| t.kind.clone()).collect();
  assert_eq!(kinds, vec![TriviaKind::LineComment, TriviaKind::BlankLines]);
}

#[test]
fn a_blank_line_inside_a_block_does_not_close_it() {
  let toks = kinds("main :: ()\nmain =\n  print \"a\"\n\n  print \"b\"\n");
  assert_eq!(toks.iter().filter(|k| **k == TokenKind::Dedent).count(), 1);
}

#[test]
fn records_whitespace_adjacency() {
  let toks = lex("main :: ()\nmain =\n  print \"x\"\n").expect("should lex");
  assert!(!toks[0].ws_before, "the first token has no preceding space");
  assert!(toks[1].ws_before, "`=` is preceded by a space");
}

/// Plan 01 acceptance criterion 8.
///
/// The position is asserted through the span, not a stored line/col pair:
/// plan 05 removed those fields because a scanner over a sub-slice stamped
/// them wrongly, and the span is now the only claim an error makes about
/// where it is.
#[test]
fn rejects_a_tab_in_indentation() {
  let src = "main :: ()\nmain =\n\tprint \"x\"\n";
  let err = lex(src).expect_err("a tab is an error");
  assert!(err.message.contains("tab"), "{}", err.message);
  assert_eq!(&src[err.span.0..err.span.1], "\t");
}

/// Plan 01 acceptance criterion 9.
#[test]
fn rejects_an_inconsistent_dedent() {
  let src = "main :: ()\nmain =\n    print \"a\"\n  print \"b\"\n";
  let err = lex(src).expect_err("should fail");
  assert!(
    err.message.contains("inconsistent dedent"),
    "{}",
    err.message
  );
  assert_eq!(
    src[..err.span.0].matches('\n').count() + 1,
    4,
    "the span lands on the offending line"
  );
}

#[test]
fn rejects_an_unterminated_string() {
  let err = lex("main :: ()\nmain =\n  print \"oops\n").expect_err("should fail");
  assert!(err.message.contains("unterminated"), "{}", err.message);
}

#[test]
fn closes_open_blocks_at_eof_without_a_trailing_newline() {
  let toks = kinds("main :: ()\nmain =\n  print \"x\"");
  assert_eq!(toks.last(), Some(&TokenKind::Eof));
  assert_eq!(toks.iter().filter(|k| **k == TokenKind::Dedent).count(), 1);
}

#[test]
fn keeps_trailing_file_trivia_on_eof() {
  let toks = lex("main :: ()\nmain =\n  print \"x\"\n-- the end\n").expect("should lex");
  let eof = toks.last().expect("eof");
  assert_eq!(eof.kind, TokenKind::Eof);
  // Found by kind, not by position: the file's own final newline arrives as a
  // `BlankLines` trivium after the comment.
  assert_eq!(comment_of(&eof.leading), Some("the end"));
}

/// Regression: found by running the real binary, not by inspection. A comment
/// between two top-level items used to attach to the `Dedent` closing the
/// first item's body, where the parser never reads it, so it vanished from the
/// generated Rust.
#[test]
fn a_comment_between_items_attaches_to_the_next_item() {
  let src = "a :: ()\na =\n  print \"1\"\n-- between\nb :: ()\nb =\n  print \"2\"\n";
  let toks = lex(src).expect("should lex");
  let b = toks
    .iter()
    .find(|t| t.kind == TokenKind::Ident("b".into()))
    .expect("the second item's name");
  assert_eq!(
    comment_of(&b.leading),
    Some("between"),
    "the comment should reach `b`, not the preceding Dedent"
  );
}

/// Regression, same cause: structural tokens must not absorb trivia, or a
/// trailing comment never reaches `Eof`.
#[test]
fn structural_tokens_never_claim_trivia() {
  let toks = lex("main :: ()\nmain =\n  print \"x\"\n-- tail\n").expect("should lex");
  for t in &toks {
    if matches!(
      t.kind,
      TokenKind::Indent | TokenKind::Dedent | TokenKind::Newline
    ) {
      assert!(t.leading.is_empty(), "{:?} claimed trivia", t.kind);
    }
  }
}

#[test]
fn handles_crlf_line_endings() {
  let toks = kinds("main :: ()\r\nmain =\r\n  print \"x\"\r\n");
  assert!(toks.contains(&TokenKind::Indent));
  assert_eq!(toks.last(), Some(&TokenKind::Eof));
}

/// Doc 01 §3.1 prescribes four spaces while every example uses two. The lexer
/// fixes no width and rejects only inconsistency — `docs/COMPILER.md` §4.
#[test]
fn accepts_any_consistent_indentation_width() {
  for src in [
    "main :: ()\nmain =\n  print \"x\"\n",
    "main :: ()\nmain =\n    print \"x\"\n",
    "main :: ()\nmain =\n\u{20}\u{20}\u{20}print \"x\"\n",
  ] {
    let toks = kinds(src);
    assert!(toks.contains(&TokenKind::Indent), "{src:?}");
    assert!(toks.contains(&TokenKind::Dedent), "{src:?}");
  }
}
