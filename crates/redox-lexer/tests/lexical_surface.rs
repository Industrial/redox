//! Plan 02's concrete proof: `examples/lexical.sdx` exercises the whole
//! lexical surface and its tokens are asserted directly.
//!
//! `docs/07 - 02 - Lexical Surface.md` §3. The tokens are asserted rather than
//! the program's behaviour because no parser yet accepts most of them — this
//! plan decides what tokens exist, not what they mean.

use redox_lexer::{Keyword, StrPart, TokenKind, TriviaKind, lex};
use std::path::PathBuf;

fn fixture() -> String {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/lexical.sdx");
  std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("should read {}: {e}", path.display()))
}

/// Every token on the line whose first `Ident` is `name`, up to its `Newline`.
fn line_of(src: &str, name: &str) -> Vec<TokenKind> {
  let toks = lex(src).expect("the fixture should lex");
  let start = toks
    .iter()
    .position(|t| t.kind == TokenKind::Ident(name.to_string()))
    .unwrap_or_else(|| panic!("no binding named `{name}`"));
  toks[start..]
    .iter()
    .take_while(|t| t.kind != TokenKind::Newline)
    .map(|t| t.kind.clone())
    .collect()
}

fn ident(name: &str) -> TokenKind {
  TokenKind::Ident(name.to_string())
}

/// Criterion 1: the fixture lexes with no error.
#[test]
fn the_whole_lexical_surface_lexes() {
  let toks = lex(&fixture()).expect("the fixture should lex");
  assert_eq!(toks.last().map(|t| &t.kind), Some(&TokenKind::Eof));
}

/// Criterion 3: every token's span indexes the original source.
#[test]
fn every_token_span_indexes_the_source() {
  let src = fixture();
  for t in lex(&src).expect("should lex") {
    assert!(
      t.span.0 <= t.span.1 && t.span.1 <= src.len(),
      "{:?} has span {:?} outside a {}-byte source",
      t.kind,
      t.span,
      src.len()
    );
  }
}

#[test]
fn lexes_every_numeric_literal() {
  let src = fixture();
  assert_eq!(
    line_of(&src, "count"),
    vec![
      ident("count"),
      TokenKind::Eq,
      TokenKind::Int("1_000_000".into())
    ]
  );
  assert_eq!(
    line_of(&src, "hexed"),
    vec![
      ident("hexed"),
      TokenKind::Eq,
      TokenKind::Int("0xDEAD_beef".into())
    ]
  );
  assert_eq!(
    line_of(&src, "ratio"),
    vec![
      ident("ratio"),
      TokenKind::Eq,
      TokenKind::Float("2.5e-3".into())
    ]
  );
}

#[test]
fn lexes_every_quoted_literal_kind() {
  let src = fixture();
  assert_eq!(
    line_of(&src, "ch"),
    vec![ident("ch"), TokenKind::Eq, TokenKind::Char('x')]
  );
  assert_eq!(
    line_of(&src, "byte"),
    vec![ident("byte"), TokenKind::Eq, TokenKind::Byte(b'\n')]
  );
  assert_eq!(
    line_of(&src, "raw"),
    vec![
      ident("raw"),
      TokenKind::Eq,
      TokenKind::RawStr {
        text: "no \\escapes here".into(),
        hashes: 1
      }
    ]
  );
  assert_eq!(
    line_of(&src, "cstr"),
    vec![
      ident("cstr"),
      TokenKind::Eq,
      TokenKind::CStr("nul terminated".into())
    ]
  );
}

/// Doc 03 §10.4's one surviving collection literal — plan 29. `%w[` and `#{`
/// are **deleted** by doc 18 §7, and their tokens went with them: a variant
/// nothing produces weakens every exhaustive match that carries it, which is
/// plan 18 §10.2's rule.
#[test]
fn lexes_the_array_repeat_delimiters() {
  let src = fixture();
  let repeated = line_of(&src, "repeated");
  assert_eq!(repeated[2], TokenKind::LBracket, "[ opens an array repeat");
  assert!(
    repeated.contains(&TokenKind::Semi),
    "the `;` is what tells a repeat from a list"
  );
  assert_eq!(repeated.last(), Some(&TokenKind::RBracket));
}

/// The retired spellings no longer lex. Asserted because *nothing else would
/// notice*: `%w[` now scans as `%`, `w`, `[` and fails in the parser, and `#{`
/// as `#` and `{` — both reaching an error, but by a different route than
/// before, and neither producing a token.
#[test]
fn the_retired_collection_delimiters_no_longer_lex() {
  for src in ["x = %w[a b]\n", "x = #{1, 2}\n"] {
    let kinds: Vec<TokenKind> = redox_lexer::lex(src)
      .map(|toks| toks.into_iter().map(|t| t.kind).collect())
      .unwrap_or_default();
    assert!(
      !kinds
        .iter()
        .any(|k| format!("{k:?}").contains("PercentW") || format!("{k:?}").contains("HashBrace")),
      "{src} should not produce a retired token: {kinds:?}"
    );
  }
}

/// Criterion 5's first two clauses, on the fixture's own interpolations.
#[test]
fn lexes_interpolation_with_and_without_a_format_spec() {
  let src = fixture();
  let line = line_of(&src, "greeting");
  let TokenKind::Str { parts, .. } = &line[2] else {
    panic!("expected a string, got {:?}", line[2]);
  };

  assert_eq!(parts.len(), 4, "text, interp, text, interp");
  assert_eq!(parts[0], StrPart::Text("hi ".into()));

  let StrPart::Interp { tokens, spec } = &parts[1] else {
    panic!("expected an interpolation");
  };
  assert_eq!(tokens.len(), 1);
  assert_eq!(tokens[0].kind, ident("name"));
  assert_eq!(*spec, None, "`#{{name}}` carries no spec");

  let StrPart::Interp { tokens, spec } = &parts[3] else {
    panic!("expected a second interpolation");
  };
  assert_eq!(tokens[0].kind, ident("amount"));
  assert_eq!(spec.as_deref(), Some(".2"), "`#{{amount:.2}}` carries `.2`");
}

/// Criterion 3, for the hardest case: a token *inside* an interpolation must
/// carry a span indexing the original file, not the interpolation body.
#[test]
fn interpolated_tokens_carry_original_file_spans() {
  let src = fixture();
  let line = line_of(&src, "greeting");
  let TokenKind::Str { parts, .. } = &line[2] else {
    panic!("expected a string");
  };
  let StrPart::Interp { tokens, .. } = &parts[1] else {
    panic!("expected an interpolation");
  };
  assert_eq!(
    &src[tokens[0].span.0..tokens[0].span.1],
    "name",
    "the span should index the real source text"
  );
}

/// Criterion 4: a `"""` block is dedented, and the line after it is correct.
#[test]
fn dedents_a_multiline_block_and_keeps_line_numbers() {
  let src = fixture();
  let line = line_of(&src, "block");
  let TokenKind::Str { parts, .. } = &line[2] else {
    panic!("expected a string, got {:?}", line[2]);
  };
  assert_eq!(
    parts,
    &vec![StrPart::Text("indented\n  relative".into())],
    "common indentation stripped, relative indentation kept"
  );

  // The binding after the block must still report its true line. `chain` is
  // on line 22 of the fixture, and this number is written out so that moving
  // the fixture is a visible decision rather than a silent one.
  //
  // It has moved twice and the mechanism worked both times: doc 18's surface
  // added two header lines, and plan 29 replaced the `%w` and `#{` fixture
  // lines — two deleted spellings — with one array repeat, netting minus one.
  let toks = lex(&src).expect("should lex");
  let chain = toks
    .iter()
    .position(|t| t.kind == ident("chain"))
    .expect("the chain binding");
  let before = &src[..toks[chain].span.0];
  assert_eq!(
    before.matches('\n').count() + 1,
    22,
    "the line after a \"\"\" block must be numbered correctly"
  );
}

#[test]
fn lexes_the_operator_surface() {
  let src = fixture();
  assert!(
    line_of(&src, "chain").contains(&TokenKind::Pipe),
    "the pipeline operator"
  );
  assert!(
    line_of(&src, "maybe").contains(&TokenKind::Coalesce),
    "the coalesce operator"
  );
  assert!(
    line_of(&src, "guarded").contains(&TokenKind::Kw(Keyword::Catch)),
    "`catch` is a keyword"
  );
  assert!(
    line_of(&src, "spread").contains(&TokenKind::DotDotDot),
    "the spread operator"
  );
  assert!(
    line_of(&src, "range").contains(&TokenKind::DotDot),
    "`..` is exclusive, matching Rust (doc 03 §4.2)"
  );
  assert!(
    line_of(&src, "closed").contains(&TokenKind::DotDotEq),
    "`..=` is inclusive"
  );
  // Doc 18 §2.5 retires `@name` and `@@name` for `self.name` and
  // `Self::name`, which leaves `@` with exactly one meaning — the as-pattern.
  assert!(
    line_of(&src, "field").contains(&TokenKind::Dot),
    "self-access is a field access now, not a sigil"
  );
  assert!(
    line_of(&src, "static_call").contains(&TokenKind::ColonColon),
    "an associated item is a path now, not a sigil"
  );
  // Doc 18 §4 retired `&:name` for `(|> name)`, a *section* of the pipeline,
  // and plan 57 retired that in turn: a **method is a function** whose last
  // argument is its receiver, so a bare `.name` is a method awaiting it and
  // lowers to the closure the section spelled. Two surfaces for one
  // construct, and the row that built the replacement removed the other.
  //
  // So the token is a `Dot` and the assertion is about *position*: the `.`
  // opens the line rather than following a receiver, which is the whole
  // method-versus-field rule and the only thing a lexer can see of it.
  let method_ref = line_of(&src, "method_ref");
  assert!(
    method_ref
      .windows(2)
      .any(|pair| pair == [TokenKind::Eq, TokenKind::Dot]),
    "a method reference is a `.` opening the value now, not a `|>` section or \
     a sigil: {method_ref:?}"
  );
  assert!(
    !method_ref.contains(&TokenKind::Pipe),
    "`(|> name)` is retired — plan 57"
  );
}

/// Criterion 8: `'x'` is a char and `'outer` a label, by lookahead alone.
#[test]
fn distinguishes_a_char_literal_from_a_label() {
  let src = fixture();
  assert_eq!(
    line_of(&src, "label"),
    vec![
      ident("label"),
      TokenKind::Eq,
      TokenKind::Label("outer".into())
    ]
  );
  assert!(line_of(&src, "ch").contains(&TokenKind::Char('x')));
}

/// Criterion 10: every comment form survives as trivia.
#[test]
fn every_comment_form_survives_as_trivia() {
  let toks = lex(&fixture()).expect("should lex");
  let kinds: Vec<TriviaKind> = toks
    .iter()
    .flat_map(|t| t.leading.iter().map(|v| v.kind.clone()))
    .collect();

  assert!(
    kinds.contains(&TriviaKind::FencedComment),
    "### fence: {kinds:?}"
  );
  assert!(
    kinds.contains(&TriviaKind::DocComment),
    "## doc comment: {kinds:?}"
  );
  assert!(
    kinds.contains(&TriviaKind::LineComment),
    "-- line comment: {kinds:?}"
  );
}

/// Criterion 9: a reserved word never lexes as an identifier.
#[test]
fn reserved_words_never_lex_as_identifiers() {
  let toks = lex(&fixture()).expect("should lex");
  for t in &toks {
    if let TokenKind::Ident(name) = &t.kind {
      assert!(
        Keyword::from_text(name).is_none(),
        "`{name}` is reserved but lexed as an identifier"
      );
    }
  }
}
