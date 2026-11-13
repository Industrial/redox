//! Continuation suppression — plan 02 leaf 2, criteria 6 and 7.
//!
//! Doc 01 §4.2 says "newlines terminate statements unless an expression is
//! continued" without saying when. These tests fix when.

use voltaic_lexer::{TokenKind, lex};

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

/// Count `Indent`s and `Dedent`s, which must balance over a whole file.
fn indent_balance(src: &str) -> (usize, usize) {
  let toks = kinds(src);
  (
    toks.iter().filter(|k| **k == TokenKind::Indent).count(),
    toks.iter().filter(|k| **k == TokenKind::Dedent).count(),
  )
}

/// Plan 65 leaf 1 **replaces** plan 02 leaf 2's criterion 6.
///
/// Criterion 6 asserted that a bracketed expression emits no structural
/// tokens at all, which is what made `print (if c` with an indented body
/// unwritable — `statement_block` had no `Indent` to find. A bracket now opens
/// a layout *context* instead of suppressing layout, so a multi-line list does
/// carry `Newline`s and an `Indent` the one-line form does not.
///
/// What survives is the property criterion 6 was protecting: the closer pops
/// every level its interior opened, so the indent stack after the `]` is the
/// one before the `[` and the line after it reconciles against the file.
#[test]
fn a_bracket_closes_every_level_its_interior_opened() {
  let spread = "xs = [\n  1,\n  2,\n  3,\n]\ny = 1\n";
  let (indents, dedents) = indent_balance(spread);
  assert_eq!(
    (indents, dedents),
    (1, 1),
    "the interior opens one level and the `]` closes it: {:?}",
    kinds(spread)
  );
  assert!(
    structural_count(spread) > structural_count("xs = [1, 2, 3]\ny = 1\n"),
    "the spread form carries layout the one-line form does not — that is the row"
  );
}

#[test]
fn nested_delimiters_each_close_their_own_context() {
  let src = "xs = f(\n  g(\n    1\n  )\n)\ny = 1\n";
  assert_eq!(
    indent_balance(src),
    (2, 2),
    "each `(` opens a level and each `)` unwinds to its own mark: {:?}",
    kinds(src)
  );
}

/// Plan 65 criterion 2, and §1.2's whole reason the suppression existed.
///
/// A continuation aligned under its opener sits at a column matching no
/// enclosing level — seven, under `f (` — which with one shared stack is
/// `RDX0002`. The context mark is what makes it one consistent deeper level
/// instead.
#[test]
fn a_continuation_aligned_under_its_opener_is_not_an_inconsistent_dedent() {
  for src in [
    "m = f (a,\n       b)\n",
    "m = f (a\n       + b)\n",
    "xs = [ 1\n     , 2\n     ]\n",
    "p = Point { x = 1\n          , y = 2\n          }\n",
  ] {
    assert!(
      lex(src).is_ok(),
      "{src:?} gave {:?}",
      lex(src).err().map(|e| e.message)
    );
  }
}

/// Criterion 2 again, for the half a continuation line never reaches.
///
/// A *continuation* is suppressed before `sync_indent` ever runs, so the
/// alignment above was only ever half the case. Once a bracket can hold a
/// real statement — which is this row — its interior pushes genuine levels,
/// and a closer placed to the left of them matched none of them: `RDX0002` on
/// a `)`, decided by nothing but which column the author put it in.
///
/// A line that *opens* with a closer is therefore not content at that
/// indentation. `track_delims` already owes the `Dedent`s, so every column is
/// equally fine and none of them is semantic.
#[test]
fn a_closers_own_column_is_never_an_inconsistent_dedent() {
  // The interior opens a level at 9; the closer is then placed at the base,
  // short of it, under it, and at column zero.
  for closer in ["  )", "        )", "         )", ")"] {
    // The statement prints a literal rather than interpolating `m`: this is
    // a `format!` literal, and `#{m}` inside one is Rust's own
    // named-argument placeholder, not Voltaic's interpolation. What the
    // statement prints is incidental — the test is about the closer's column.
    let src = format!("main =\n  m = f (1 +\n         2\n{closer}\n  println! \\-> \"done\"\n");
    assert!(
      lex(&src).is_ok(),
      "a `)` at column {} was rejected: {:?}",
      closer.len() - 1,
      lex(&src).err().map(|e| e.message)
    );
  }
}

/// Plan 65 criterion 3 and decision 3: a fresh base is not an amnesty.
#[test]
fn an_inconsistent_dedent_inside_brackets_is_still_rejected() {
  // Four spaces, then three — which matches neither the level above it nor
  // the context's base.
  let err = lex("xs = [ f\n    a\n   b\n     ]\n").expect_err("should fail");
  assert!(
    err.message.contains("inconsistent dedent"),
    "got {:?}",
    err.message
  );
}

/// Plan 65 criterion 1, at the lexical level: the `Indent` that
/// `statement_block` needs now exists inside `(…)`.
#[test]
fn an_indented_block_inside_parentheses_carries_its_layout() {
  let toks = kinds("v = show (if c\n  a\nelse\n  b)\n");
  assert!(
    toks.contains(&TokenKind::Indent),
    "the branch bodies must open blocks: {toks:?}"
  );
  assert_eq!(
    indent_balance("v = show (if c\n  a\nelse\n  b)\n"),
    (2, 2),
    "one level per branch, both closed inside the parentheses: {toks:?}"
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

/// Plan 66 criterion 3, and the half plan 58 did not find.
///
/// `next_line_continues` tested for the literal `"|>"` and nothing else, so a
/// composition chain written the way the corpus writes a pipeline — operator
/// leading each continuation line — did not lex at all. The corpus leads
/// **every** multi-line pipeline with the operator and has no trailing `|>`
/// anywhere, so that layout is the house style rather than one of two.
#[test]
fn a_leading_composition_operator_continues_the_line() {
  for (src, op, stages) in [
    (
      "f = trimmed\n  >>> wrap\n  >>> shout\n",
      TokenKind::Compose,
      2,
    ),
    ("f = shout\n  <<< wrap\n", TokenKind::ComposeBack, 1),
  ] {
    let toks = kinds(src);
    assert!(
      !toks.contains(&TokenKind::Indent),
      "a leading {op:?} must not open a block: {toks:?}"
    );
    assert_eq!(
      toks.iter().filter(|k| **k == op).count(),
      stages,
      "every stage should be present: {toks:?}"
    );
  }
}

/// And the forward operator does **not** continue from the end of a line.
///
/// `continues_line` excludes it for the reason it excludes `>` and `>>`: it
/// closes a generic argument list. `<<<` is in that set because it closes
/// nothing, so a backward chain is writable either way and a forward one is
/// writable only with the operator leading.
#[test]
fn a_trailing_composition_operator_follows_its_angle_brackets() {
  let forward = kinds("a = f >>>\n  g\nb = 1\n");
  assert!(
    forward.contains(&TokenKind::Newline),
    "a trailing `>>>` must end the line, as `>>` does: {forward:?}"
  );
  let backward = kinds("a = f <<<\n  g\n");
  assert!(
    !backward.contains(&TokenKind::Indent),
    "a trailing `<<<` continues, as `<<` does: {backward:?}"
  );
}

/// A triple closer is one token, and it must not continue the line either —
/// `capture_type` ends a type at a newline, so a suppressed one swallows the
/// next declaration whole. Plan 66 §3 case 3's lexical half.
#[test]
fn a_trailing_triple_angle_bracket_does_not_continue_the_line() {
  let toks = kinds("a = x : Vec<Vec<Vec<i32>>>\nb = 1\n");
  assert!(
    toks.contains(&TokenKind::Compose),
    "the triple closer should be one token: {toks:?}"
  );
  assert!(
    toks.contains(&TokenKind::Newline),
    "the line must end: {toks:?}"
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
  let toks = kinds("main : ()\nmain =\n  println! \\-> \"x\"\n");
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
  for src in ["xs = [1, 2\n", "xs = f(1\n", "xs = {1\n"] {
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
  for src in ["xs = [1, 2]\n", "xs = f(1)\n", "xs = {1}\n"] {
    assert!(lex(src).is_ok(), "{src:?} should lex");
  }
}

/// `>` does not continue a line, and that is a deliberate reversal.
///
/// It was in the continuation set because it is a binary operator, which made
/// `x = Some 1 : Option<u8>` swallow the whole next line: `capture_type`
/// ends a type at a newline, and the newline had been suppressed. A `>` at
/// the end of a line closes a generic argument list far more often than it
/// dangles off a comparison, and a comparison split there is still writable
/// with parentheses — a type ending in `>` is not avoidable at all.
#[test]
fn a_trailing_angle_bracket_does_not_continue_the_line() {
  let toks = kinds("a = x : Vec<u8>\nb = 1\n");
  assert!(
    toks.contains(&TokenKind::Newline),
    "the line must end: {toks:?}"
  );
}

/// `>>` likewise, for `Vec<Vec<u8>>`.
#[test]
fn a_trailing_double_angle_bracket_does_not_continue_the_line() {
  let toks = kinds("a = x : Vec<Vec<u8>>\nb = 1\n");
  assert!(
    toks.contains(&TokenKind::Newline),
    "the line must end: {toks:?}"
  );
}

/// A comment between two pipeline stages does not end the chain.
///
/// Plan 69's fourth finding, and the only one of that row's surface findings it
/// graded a **defect**: `next_line_continues` asked whether the line ahead
/// opened with `|>`, a comment line opens with `--`, so the chain was closed
/// and the `|>` below earned `RDX0102`. `AGENTS.md` calls comment preservation
/// load-bearing, and a trivium that stops its neighbour parsing is the
/// strongest form of discarding one.
///
/// It lived as `tests/ui/comment_between_pipeline_stages` while it was a
/// defect, with a note that fixing it had to fail there and be reviewed. Plan
/// 73 fixed it — after meeting it three times in one crate — so the record
/// moves here and becomes an assertion about what the lexer does rather than
/// about what it got wrong.
#[test]
fn a_comment_between_stages_does_not_end_the_pipeline() {
  let src = "x = xs\n  |> f\n  -- A comment between two stages.\n  |> g\n";
  assert_eq!(
    structural_count(src),
    structural_count("x = xs\n  |> f\n  |> g\n"),
    "the comment must not add a statement separator"
  );
}

/// A **doc** comment still ends it, which is the boundary the fix draws.
///
/// `-- |` documents the item below it and no item begins with `|>`, so
/// treating one as a continuation would attach documentation to an expression
/// and hide a real mistake.
#[test]
fn a_doc_comment_between_stages_still_ends_the_pipeline() {
  let src = "x = xs\n  |> f\n  -- | Documentation.\n  |> g\n";
  assert!(
    structural_count(src) > structural_count("x = xs\n  |> f\n  |> g\n"),
    "a doc comment is not a continuation"
  );
}
