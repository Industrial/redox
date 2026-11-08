//! `redox-lexer` — the off-side-rule lexer.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 2. Hand-written rather
//! than built on `logos`, because a regex-dispatch lexer cannot express an
//! off-side rule and Redox additionally needs trivia and whitespace-adjacency
//! on every token (doc 02 §5.1).
//!
//! Three properties are mandatory and must not be weakened by later plans:
//!
//! 1. **Every token carries a byte span.**
//! 2. **Comments and blank-line runs are emitted as [`Trivia`] attached to the
//!    following token, never discarded.** Doc 02 §10.1 corrects doc 01 §4.1 on
//!    this point: the generated `.xrs` is a human-facing artifact, so comments
//!    must reach it. Emerald's formatter loses every non-doc comment because
//!    its lexer treats them as whitespace; that is the defect this property
//!    exists to prevent.
//! 3. **Every token records whether whitespace preceded it** (`ws_before`).
//!    Unused by this slice, and free to collect here. Plan 19's call grammar
//!    and plan 27's `(-1)`-versus-`(- 1)` rule both depend on it
//!    (doc 03 §5.1, §5.4).
//!
//! The token set is deliberately the minimum this slice needs. Continuation
//! suppression inside delimiters, the `#` longest-match table, interpolation
//! and `"""` strings are plan 02.

/// A half-open byte range into the source text.
pub type Span = (usize, usize);

/// Non-semantic source text that must survive to the emitter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriviaKind {
  /// A `#` comment, either on its own line or trailing a line of code. The
  /// stored text excludes the `#` and one optional following space.
  LineComment,
  /// A run of one or more blank lines. `text` is empty; the run's length is
  /// not recorded, because the emitter collapses any run to a single blank
  /// line and nothing in the language depends on the count.
  BlankLines,
}

/// One piece of trivia, with the span it occupied in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
  pub kind: TriviaKind,
  pub text: String,
  pub span: Span,
}

impl Trivia {
  fn comment(text: String, span: Span) -> Self {
    Self {
      kind: TriviaKind::LineComment,
      text,
      span,
    }
  }

  fn blank(span: Span) -> Self {
    Self {
      kind: TriviaKind::BlankLines,
      text: String::new(),
      span,
    }
  }
}

/// The token set for plan 01's slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
  Ident(String),
  /// A string literal, stored **raw, including its delimiters**. Escape
  /// decoding and interpolation are plan 23; storing the source text verbatim
  /// lets the emitter reproduce it byte-for-byte and keeps this slice free of
  /// an escape table it would only have to redo.
  StrLit(String),
  Eq,
  Arrow,
  /// A logical line break terminating a statement.
  Newline,
  Indent,
  Dedent,
  Eof,
}

/// A token, its span, the trivia preceding it, and whether whitespace
/// immediately preceded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
  pub ws_before: bool,
  pub leading: Vec<Trivia>,
  /// Trivia on the same line, after this token. Only ever set on the
  /// [`TokenKind::Newline`] that closes the line.
  pub trailing: Option<Trivia>,
}

/// A lexical error, carrying enough to report against the `.sdx` source.
/// `miette` rendering is plan 05; a plain message with a position is what
/// plan 01 needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
  pub message: String,
  pub span: Span,
  pub line: usize,
  pub col: usize,
}

// Indentation *width* is deliberately not fixed here. Any increase opens a
// block and the matching decrease closes it, so two-space and four-space
// sources both lex, and only inconsistency is an error. This is worth stating
// because doc 01 §3.1 prescribes four spaces while every example in docs 01
// and 05 uses two — see `docs/COMPILER.md`. Enforcing one canonical width is
// a formatter concern (plan 34), not a lexer one.

struct Lexer<'a> {
  src: &'a str,
  toks: Vec<Token>,
  indents: Vec<usize>,
  pending: Vec<Trivia>,
  line: usize,
}

/// Tokenize `src`.
///
/// Returns every token in order, ending with [`TokenKind::Eof`]. Trivia is
/// attached to the token that follows it; trivia at the end of the file is
/// attached to `Eof`.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
  let mut lx = Lexer {
    src,
    toks: Vec::new(),
    indents: vec![0],
    pending: Vec::new(),
    line: 0,
  };
  lx.run()?;
  Ok(lx.toks)
}

impl<'a> Lexer<'a> {
  fn run(&mut self) -> Result<(), LexError> {
    let mut offset = 0usize;
    for raw_line in self.src.split('\n') {
      self.line += 1;
      let line_start = offset;
      // `split('\n')` drops the separator; advance past the line and it.
      offset += raw_line.len() + 1;
      // A trailing '\r' from CRLF input is not indentation and not content.
      let text = raw_line.strip_suffix('\r').unwrap_or(raw_line);
      self.line_pass(text, line_start)?;
    }
    self.finish();
    Ok(())
  }

  /// Handle one physical line: indentation, then its tokens.
  fn line_pass(&mut self, text: &str, line_start: usize) -> Result<(), LexError> {
    let width = self.measure_indent(text, line_start)?;
    let body = &text[width..];

    // Blank line: trivia, and the indent stack is untouched. An indentation
    // change is only ever signalled by a line with content, so a blank line
    // inside a block does not close it.
    if body.is_empty() {
      self.push_blank(line_start, line_start + text.len());
      return Ok(());
    }

    // Comment-only line: trivia, indent stack untouched for the same reason.
    if let Some(rest) = body.strip_prefix('#') {
      let span = (line_start + width, line_start + text.len());
      self.pending.push(Trivia::comment(trim_comment(rest), span));
      return Ok(());
    }

    self.sync_indent(width, line_start)?;
    self.tokenize(body, line_start + width)?;
    Ok(())
  }

  /// Count leading spaces, rejecting tabs outright.
  ///
  /// Doc 01 §3.1 permits either rejection or one documented normalisation
  /// rule. Rejection is one branch and removes a class of indentation bug
  /// that otherwise reports against the wrong line.
  fn measure_indent(&self, text: &str, line_start: usize) -> Result<usize, LexError> {
    let mut width = 0usize;
    for (i, ch) in text.char_indices() {
      match ch {
        ' ' => width += 1,
        '\t' => {
          return Err(LexError {
            message: "tab in indentation; Redox indents with spaces".to_string(),
            span: (line_start + i, line_start + i + 1),
            line: self.line,
            col: i + 1,
          });
        }
        _ => break,
      }
    }
    Ok(width)
  }

  /// Reconcile this line's indentation width against the indent stack,
  /// emitting `Indent`, `Newline` or one or more `Dedent`s.
  fn sync_indent(&mut self, width: usize, line_start: usize) -> Result<(), LexError> {
    let top = *self.indents.last().expect("indent stack is never empty");
    let at = (line_start + width, line_start + width);

    if width > top {
      self.indents.push(width);
      self.push(TokenKind::Indent, at, false);
      return Ok(());
    }

    if width == top {
      // A statement separator, but only between statements — not before the
      // first token in the file.
      if !self.toks.is_empty() {
        self.push(TokenKind::Newline, at, false);
      }
      return Ok(());
    }

    while width < *self.indents.last().expect("indent stack is never empty") {
      self.indents.pop();
      self.push(TokenKind::Dedent, at, false);
    }
    if width != *self.indents.last().expect("indent stack is never empty") {
      return Err(LexError {
        message: format!(
          "inconsistent dedent: {width} spaces matches no enclosing indentation level"
        ),
        span: at,
        line: self.line,
        col: width + 1,
      });
    }
    self.push(TokenKind::Newline, at, false);
    Ok(())
  }

  /// Tokenize the content of one line, which has already had its indentation
  /// consumed. Emits a closing `Newline`.
  fn tokenize(&mut self, body: &str, base: usize) -> Result<(), LexError> {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut ws = false;
    let mut trailing = None;

    while i < bytes.len() {
      let ch = bytes[i];
      if ch == b' ' {
        i += 1;
        ws = true;
        continue;
      }

      // A `#` after content on the same line: the rest of the line is trivia.
      if ch == b'#' {
        let span = (base + i, base + body.len());
        trailing = Some(Trivia::comment(trim_comment(&body[i + 1..]), span));
        break;
      }

      let start = base + i;

      if ch == b'"' {
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] != b'"' {
          j += 1;
        }
        if j >= bytes.len() {
          return Err(LexError {
            message: "unterminated string literal".to_string(),
            span: (start, base + body.len()),
            line: self.line,
            col: i + 1,
          });
        }
        let raw = body[i..=j].to_string();
        self.push(TokenKind::StrLit(raw), (start, base + j + 1), ws);
        i = j + 1;
        ws = false;
        continue;
      }

      if ch == b'-' && bytes.get(i + 1) == Some(&b'>') {
        self.push(TokenKind::Arrow, (start, start + 2), ws);
        i += 2;
        ws = false;
        continue;
      }

      if ch == b'=' {
        self.push(TokenKind::Eq, (start, start + 1), ws);
        i += 1;
        ws = false;
        continue;
      }

      if is_ident_start(ch) {
        let mut j = i;
        while j < bytes.len() && is_ident_continue(bytes[j]) {
          j += 1;
        }
        let text = body[i..j].to_string();
        self.push(TokenKind::Ident(text), (start, base + j), ws);
        i = j;
        ws = false;
        continue;
      }

      return Err(LexError {
        message: format!("unexpected character {:?}", ch as char),
        span: (start, start + 1),
        line: self.line,
        col: i + 1,
      });
    }

    let end = base + body.len();
    self.push(TokenKind::Newline, (end, end), false);
    if let Some(t) = trailing {
      self
        .toks
        .last_mut()
        .expect("a Newline was just pushed")
        .trailing = Some(t);
    }
    Ok(())
  }

  /// Close every open block and emit `Eof`, carrying any trailing trivia.
  fn finish(&mut self) {
    let end = self.src.len();
    while self.indents.len() > 1 {
      self.indents.pop();
      self.push(TokenKind::Dedent, (end, end), false);
    }
    self.push(TokenKind::Eof, (end, end), false);
  }

  fn push(&mut self, kind: TokenKind, span: Span, ws_before: bool) {
    // Structural tokens never claim trivia. They sit at positions the parser
    // does not read trivia from, so letting a `Dedent` absorb a pending
    // comment loses it silently — which is exactly how a comment between two
    // top-level items, or at end of file, disappeared before this distinction
    // existed. Trivia flows past them to the next real token; `Eof` counts as
    // real here, so a trailing comment has somewhere to land.
    let structural = matches!(
      kind,
      TokenKind::Indent | TokenKind::Dedent | TokenKind::Newline
    );
    let leading = if structural {
      Vec::new()
    } else {
      std::mem::take(&mut self.pending)
    };
    self.toks.push(Token {
      kind,
      span,
      ws_before,
      leading,
      trailing: None,
    });
  }

  /// Record a blank line, merging it into a contiguous preceding run so the
  /// emitter sees one `BlankLines` per run rather than one per line.
  fn push_blank(&mut self, start: usize, end: usize) {
    match self.pending.last_mut() {
      Some(t) if t.kind == TriviaKind::BlankLines => t.span.1 = end,
      _ => self.pending.push(Trivia::blank((start, end))),
    }
  }
}

/// Strip the single optional space after a comment's `#`, so `# text` and
/// `#text` both yield `text` and the emitter can write `// text` uniformly.
fn trim_comment(rest: &str) -> String {
  rest.strip_prefix(' ').unwrap_or(rest).to_string()
}

fn is_ident_start(b: u8) -> bool {
  b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
  b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
  use super::*;

  fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src)
      .expect("should lex")
      .into_iter()
      .map(|t| t.kind)
      .collect()
  }

  #[test]
  fn lexes_the_hello_slice() {
    let toks = kinds("main = ->\n  print \"hello\"\n");
    assert_eq!(
      toks,
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

  /// Acceptance criterion 6: every token carries a real byte span. `main`
  /// occupies bytes 0..4 and the literal on line 2 starts at byte 18.
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
    assert_eq!(toks[0].leading.len(), 1);
    assert_eq!(toks[0].leading[0].kind, TriviaKind::LineComment);
    assert_eq!(toks[0].leading[0].text, "a note");
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

  /// Acceptance criterion 8.
  #[test]
  fn rejects_a_tab_in_indentation() {
    let err = lex("main = ->\n\tprint \"x\"\n").expect_err("a tab is an error");
    assert!(err.message.contains("tab"), "{}", err.message);
    assert_eq!(err.line, 2);
    assert_eq!(err.col, 1);
  }

  /// Acceptance criterion 9.
  #[test]
  fn rejects_an_inconsistent_dedent() {
    let src = "main = ->\n    print \"a\"\n  print \"b\"\n";
    let err = lex(src).expect_err("a mismatched dedent is an error");
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
    // Not `.last()`: the file's own final newline arrives as a `BlankLines`
    // trivium after the comment, so the comment is found by kind.
    assert_eq!(
      eof
        .leading
        .iter()
        .find(|t| t.kind == TriviaKind::LineComment)
        .map(|t| t.text.as_str()),
      Some("the end")
    );
  }

  /// Regression: found by running the real binary, not by inspection. A
  /// comment between two top-level items used to attach to the `Dedent`
  /// closing the first item's body, where the parser never reads it, so it
  /// vanished from the generated Rust.
  #[test]
  fn a_comment_between_items_attaches_to_the_next_item() {
    let src = "a = ->\n  print \"1\"\n# between\nb = ->\n  print \"2\"\n";
    let toks = lex(src).expect("should lex");
    let b = toks
      .iter()
      .find(|t| t.kind == TokenKind::Ident("b".into()))
      .expect("the second item's name");
    assert_eq!(
      b.leading
        .iter()
        .find(|t| t.kind == TriviaKind::LineComment)
        .map(|t| t.text.as_str()),
      Some("between"),
      "the comment should reach `b`, not the preceding Dedent"
    );
  }

  /// Regression, same cause: structural tokens must not absorb trivia or a
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
}
