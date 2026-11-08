//! Behaviour of [`redox_lexer::lex`], through the public API only.
//!
//! An integration test rather than a `#[cfg(test)]` module, per
//! `docs/06 - Redox Code Architecture.md` §6: these exercise the crate's
//! contract, so they must not be able to reach its internals.

use redox_lexer::{TokenKind, Trivia, TriviaKind, lex};

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
    kinds("main = ->\n  print \"hello\"\n"),
    vec![
      TokenKind::Ident("main".into()),
      TokenKind::Eq,
      TokenKind::Arrow,
      TokenKind::Newline,
      TokenKind::Indent,
      TokenKind::Ident("print".into()),
      TokenKind::StrLit("\"hello\"".into()),
      TokenKind::Newline,
      TokenKind::Dedent,
      TokenKind::Eof,
    ]
  );
}

/// Plan 01 acceptance criterion 6: spans index the real source text.
#[test]
fn tokens_carry_byte_spans() {
  let src = "main = ->\n  print \"hello\"\n";
  let toks = lex(src).expect("should lex");
  assert_eq!(toks[0].span, (0, 4));
  assert_eq!(&src[toks[0].span.0..toks[0].span.1], "main");
  let lit = toks
    .iter()
    .find(|t| matches!(t.kind, TokenKind::StrLit(_)))
    .expect("a literal");
  assert_eq!(&src[lit.span.0..lit.span.1], "\"hello\"");
}

#[test]
fn attaches_a_leading_comment_to_the_following_token() {
  let toks = lex("# a note\nmain = ->\n  print \"x\"\n").expect("should lex");
  assert_eq!(comment_of(&toks[0].leading), Some("a note"));
}

#[test]
fn attaches_a_trailing_comment_to_the_closing_newline() {
  let toks = lex("main = -> # why\n  print \"x\"\n").expect("should lex");
  let nl = toks
    .iter()
    .find(|t| t.kind == TokenKind::Newline)
    .expect("a newline");
  assert_eq!(nl.trailing.as_ref().map(|t| t.text.as_str()), Some("why"));
}

#[test]
fn collapses_a_blank_run_into_one_trivium() {
  let toks = lex("# a\n\n\n\nmain = ->\n  print \"x\"\n").expect("should lex");
  let kinds: Vec<_> = toks[0].leading.iter().map(|t| t.kind.clone()).collect();
  assert_eq!(kinds, vec![TriviaKind::LineComment, TriviaKind::BlankLines]);
}

#[test]
fn a_blank_line_inside_a_block_does_not_close_it() {
  let toks = kinds("main = ->\n  print \"a\"\n\n  print \"b\"\n");
  assert_eq!(toks.iter().filter(|k| **k == TokenKind::Dedent).count(), 1);
}

#[test]
fn records_whitespace_adjacency() {
  let toks = lex("main = ->\n  print \"x\"\n").expect("should lex");
  assert!(!toks[0].ws_before, "the first token has no preceding space");
  assert!(toks[1].ws_before, "`=` is preceded by a space");
}

/// Plan 01 acceptance criterion 8.
#[test]
fn rejects_a_tab_in_indentation() {
  let err = lex("main = ->\n\tprint \"x\"\n").expect_err("a tab is an error");
  assert!(err.message.contains("tab"), "{}", err.message);
  assert_eq!(err.line, 2);
  assert_eq!(err.col, 1);
}

/// Plan 01 acceptance criterion 9.
#[test]
fn rejects_an_inconsistent_dedent() {
  let err = lex("main = ->\n    print \"a\"\n  print \"b\"\n").expect_err("should fail");
  assert!(
    err.message.contains("inconsistent dedent"),
    "{}",
    err.message
  );
  assert_eq!(err.line, 3);
}

#[test]
fn rejects_an_unterminated_string() {
  let err = lex("main = ->\n  print \"oops\n").expect_err("should fail");
  assert!(err.message.contains("unterminated"), "{}", err.message);
}

#[test]
fn closes_open_blocks_at_eof_without_a_trailing_newline() {
  let toks = kinds("main = ->\n  print \"x\"");
  assert_eq!(toks.last(), Some(&TokenKind::Eof));
  assert_eq!(toks.iter().filter(|k| **k == TokenKind::Dedent).count(), 1);
}

#[test]
fn keeps_trailing_file_trivia_on_eof() {
  let toks = lex("main = ->\n  print \"x\"\n# the end\n").expect("should lex");
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
  let src = "a = ->\n  print \"1\"\n# between\nb = ->\n  print \"2\"\n";
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
  let toks = lex("main = ->\n  print \"x\"\n# tail\n").expect("should lex");
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
  let toks = kinds("main = ->\r\n  print \"x\"\r\n");
  assert!(toks.contains(&TokenKind::Indent));
  assert_eq!(toks.last(), Some(&TokenKind::Eof));
}

/// Doc 01 §3.1 prescribes four spaces while every example uses two. The lexer
/// fixes no width and rejects only inconsistency — `docs/COMPILER.md` §4.
#[test]
fn accepts_any_consistent_indentation_width() {
  for src in [
    "main = ->\n  print \"x\"\n",
    "main = ->\n    print \"x\"\n",
    "main = ->\n\u{20}\u{20}\u{20}print \"x\"\n",
  ] {
    let toks = kinds(src);
    assert!(toks.contains(&TokenKind::Indent), "{src:?}");
    assert!(toks.contains(&TokenKind::Dedent), "{src:?}");
  }
}
