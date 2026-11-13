//! The corpus as token streams, and what it takes to witness a construct.
//!
//! Split out of `construct_coverage.rs` at plan 67, which had to add three
//! witnesses to a file sitting at `line_cap`'s 1,500-line ceiling exactly.
//! The division follows what the pieces **are** rather than where the middle
//! fell: this module is the haystack and the shape of a claim about it,
//! [`shape`](super::shape) is the claims a token run cannot express, and the
//! root keeps the index, the witness table and the assertions.

use voltaic_lexer::{Token, TokenKind, TriviaKind, lex};

/// One corpus file, lexed once.
pub(crate) struct Source {
  pub(crate) name: String,
  pub(crate) text: String,
  pub(crate) tokens: Vec<Token>,
  /// One spelling per token, in step with `tokens`.
  pub(crate) spellings: Vec<String>,
}

impl Source {
  pub(crate) fn new(name: String, text: String) -> Option<Self> {
    // A `.sdx` the lexer rejects contributes nothing, and several UI fixtures
    // exist precisely to be rejected. Skipping them is correct; panicking
    // would make this test fail for the lexer's reasons rather than its own.
    let tokens = lex(&text).ok()?;
    let spellings = tokens.iter().map(|t| spell(t, &text)).collect();
    Some(Self {
      name,
      text,
      tokens,
      spellings,
    })
  }

  /// Whether `pattern` occurs as a contiguous run of tokens.
  pub(crate) fn has(&self, pattern: &[&str]) -> bool {
    if pattern.is_empty() || self.tokens.len() < pattern.len() {
      return false;
    }
    (0..=self.tokens.len() - pattern.len()).any(|start| {
      pattern.iter().enumerate().all(|(offset, want)| {
        matches(
          want,
          &self.tokens[start + offset],
          &self.spellings[start + offset],
        )
      })
    })
  }

  /// The source text a token spans.
  pub(crate) fn span_text(&self, token: &Token) -> &str {
    self.text.get(token.span.0..token.span.1).unwrap_or("")
  }

  /// Every trivium attached to any token of this file.
  pub(crate) fn trivia(&self) -> impl Iterator<Item = &TriviaKind> {
    self.tokens.iter().flat_map(|token| {
      token
        .leading
        .iter()
        .chain(token.trailing.iter())
        .map(|trivium| &trivium.kind)
    })
  }
}

/// How a token is spelled for matching.
///
/// Operators, delimiters, identifiers and keywords are spelled by the **source
/// text they span**, so the lexer stays the authority on what a construct looks
/// like and this file holds no second copy of the operator table. Literals are
/// spelled by a placeholder instead, which is what keeps the match honest: a
/// string's span covers its whole body, so spelling a `Str` by its text would
/// put `$` inside a string back into the haystack this test exists to keep it
/// out of.
fn spell(token: &Token, text: &str) -> String {
  let class = match &token.kind {
    TokenKind::Int(_) => "«int»",
    TokenKind::Float(_) => "«float»",
    TokenKind::Str { .. } => "«str»",
    TokenKind::RawStr { .. } => "«rawstr»",
    TokenKind::ByteStr(_) => "«bytestr»",
    TokenKind::CStr(_) => "«cstr»",
    TokenKind::Char(_) => "«char»",
    TokenKind::Byte(_) => "«byte»",
    TokenKind::Label(_) => "«label»",
    TokenKind::Newline => "«nl»",
    TokenKind::Indent => "«indent»",
    TokenKind::Dedent => "«dedent»",
    TokenKind::Eof => "«eof»",
    _ => {
      return text
        .get(token.span.0..token.span.1)
        .unwrap_or("")
        .to_owned();
    }
  };
  class.to_owned()
}

/// Whether one pattern element matches one token.
///
/// `«any»` is any single token and `«id»` is any identifier; everything else
/// compares against the spelling.
fn matches(pattern: &str, token: &Token, spelling: &str) -> bool {
  match pattern {
    "«any»" => true,
    "«id»" => matches!(token.kind, TokenKind::Ident(_)),
    _ => pattern == spelling,
  }
}

/// The three corpora plan 51 §1.2 names, lexed.
///
/// Note the paths: the `.sdx` side of the golden corpus is `examples/`, not
/// `tests/golden/` — that directory holds only the expected `.xrs` — and the
/// UI and run corpora live under `crates/voltaic-cli/tests/`, not at the
/// repository root.
pub(crate) fn corpus() -> Vec<Source> {
  let root = crate::workspace_root();
  let dirs = [
    root.join("examples"),
    root.join("crates/voltaic-cli/tests/ui"),
    root.join("crates/voltaic-cli/tests/run"),
  ];

  let mut paths = Vec::new();
  for dir in &dirs {
    let entries = std::fs::read_dir(dir)
      .unwrap_or_else(|e| panic!("should read the corpus at {}: {e}", dir.display()));
    for entry in entries {
      let path = entry.expect("should read a corpus entry").path();
      if path.extension().is_some_and(|ext| ext == "sdx") {
        paths.push(path);
      }
    }
  }
  paths.sort();

  assert!(
    paths.len() >= 50,
    "the corpus should hold the examples and both fixture sets; found {}",
    paths.len()
  );

  paths
    .into_iter()
    .filter_map(|path| {
      let name = path
        .strip_prefix(&root)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();
      let text = std::fs::read_to_string(&path).expect("should read a corpus file");
      Source::new(name, text)
    })
    .collect()
}

/// What it takes for the corpus to witness one construct.
pub(crate) enum Witness {
  /// A contiguous run of token spellings, in any one corpus file.
  Toks(&'static [&'static str]),
  /// Every part, each in some corpus file — not necessarily the same one.
  All(&'static [Witness]),
  /// At least one part.
  Any(&'static [Witness]),
  /// Something a token run cannot express, with a description of what it is.
  Shape(&'static str, fn(&Source) -> bool),
}

impl Witness {
  /// `None` when the corpus witnesses this, otherwise what was looked for.
  pub(crate) fn unmet(&self, corpus: &[Source]) -> Option<String> {
    match self {
      Self::Toks(pattern) => {
        (!corpus.iter().any(|file| file.has(pattern))).then(|| self.describe())
      }
      Self::All(parts) => parts.iter().find_map(|part| part.unmet(corpus)),
      Self::Any(parts) => (!parts.iter().any(|part| part.unmet(corpus).is_none())).then(|| {
        parts
          .iter()
          .map(Self::describe)
          .collect::<Vec<_>>()
          .join(" or ")
      }),
      Self::Shape(what, test) => (!corpus.iter().any(test)).then(|| (*what).to_owned()),
    }
  }

  pub(crate) fn describe(&self) -> String {
    match self {
      Self::Toks(pattern) => format!("the tokens `{}`", pattern.join(" ")),
      Self::All(parts) => parts
        .iter()
        .map(Self::describe)
        .collect::<Vec<_>>()
        .join(" and "),
      Self::Any(parts) => parts
        .iter()
        .map(Self::describe)
        .collect::<Vec<_>>()
        .join(" or "),
      Self::Shape(what, _) => (*what).to_owned(),
    }
  }
}

/// A graded row, the Voltaic spelling that witnesses it, and the pattern.
pub(crate) struct Covers {
  /// The `docs/GRAMMAR.md` area cell, normalised by `normalise`.
  pub(crate) row: &'static str,
  /// The Voltaic spelling, for a reader of the failure message.
  pub(crate) spelling: &'static str,
  pub(crate) witness: Witness,
}
