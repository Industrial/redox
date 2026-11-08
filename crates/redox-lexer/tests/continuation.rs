//! Continuation suppression — plan 02 leaf 2, criteria 6 and 7.
//!
//! Doc 01 §4.2 says "newlines terminate statements unless an expression is
//! continued" without saying when. These tests fix when.

use redox_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
  lex(src)
    .expect("should lex")
    .into_iter()
    .map(|t| t.kind)
    .collect()
}

fn structural_count(src: &str) -> usize {
  kinds(src)
    .iter()
    .filter(|k| {
      matches!(
        k,
        TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
      )
    })
    .count()
}

/// Criterion 6: inside brackets, newlines and indentation are insignificant.
#[test]
fn a_bracketed_expression_emits_no_structural_tokens() {
  let flat = "xs = [1, 2, 3]\n";
  let spread = "xs = [\n  1,\n  2,\n  3,\n]\n";
  assert_eq!(
    structural_count(flat),
    structural_count(spread),
    "a multi-line bracketed list should lex like the one-line form"
  );
}

#[test]
fn nested_delimiters_track_depth() {
  let src = "xs = f(\n  g(\n    1\n  )\n)\n";
  assert_eq!(
    structural_count(src),
    structural_count("xs = f(g(1))\n"),
    "depth must return to zero only at the outermost close"
  );
}

/// A line ending in a binary operator continues.
#[test]
fn a_trailing_binary_operator_continues_the_line() {
  let src = "total = a +\n  b\n";
  assert!(
    !kinds(src).contains(&TokenKind::Indent),
    "the continuation line must not open a block"
  );
}

/// Doc 03 §5.3 writes pipelines with `|>` leading each line, so continuation
/// is also a property of the line *ahead*.
#[test]
fn a_leading_pipeline_operator_continues_the_line() {
  let src = "result = nums\n  |> iter\n  |> sum\n";
  let toks = kinds(src);
  assert!(
    !toks.contains(&TokenKind::Indent),
    "a leading `|>` must not open a block: {toks:?}"
  );
  assert_eq!(
    toks.iter().filter(|k| **k == TokenKind::Pipe).count(),
    2,
    "both pipeline stages should be present"
  );
}

/// A leading `.` must **not** continue, because doc 03 §7.5's `with` block
/// uses it to introduce statements.
#[test]
fn a_leading_dot_does_not_continue_a_with_block() {
  let src = "with cmd\n  .arg\n  .run\n";
  let toks = kinds(src);
  assert!(
    toks.contains(&TokenKind::Indent),
    "a `with` block's body is indented, not a continuation: {toks:?}"
  );
  assert!(
    toks.contains(&TokenKind::Newline),
    "its dot-shorthand lines are separate statements"
  );
}

/// `->` ends a header whose body is indented, so it must not suppress the
/// newline. This is the case that proved the first version of the rule wrong.
#[test]
fn a_trailing_arrow_does_not_suppress_the_newline() {
  let toks = kinds("main = ->\n  print \"x\"\n");
  assert!(
    toks.contains(&TokenKind::Newline),
    "the header's newline must survive: {toks:?}"
  );
  assert!(
    toks.contains(&TokenKind::Indent),
    "the body must open a block: {toks:?}"
  );
}

/// Criterion 7: an unbalanced delimiter is a named error, not a cascade of
/// dedents at `Eof`.
#[test]
fn an_unbalanced_delimiter_is_a_named_error() {
  for src in ["xs = [1, 2\n", "xs = f(1\n", "xs = #{1\n"] {
    let err = lex(src).expect_err("should fail");
    assert!(
      err.message.contains("unbalanced delimiter"),
      "{src:?} gave {:?}",
      err.message
    );
  }
}

#[test]
fn a_balanced_delimiter_is_accepted() {
  for src in ["xs = [1, 2]\n", "xs = f(1)\n", "xs = #{1}\n"] {
    assert!(lex(src).is_ok(), "{src:?} should lex");
  }
}
