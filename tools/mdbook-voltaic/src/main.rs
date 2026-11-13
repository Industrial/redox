//! An mdBook preprocessor that highlights `.sdx` blocks with Voltaic's lexer.
//!
//! Identity and documentation site
//! (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 11.
//!
//! **This builds no grammar.** highlight.js ships forty-odd languages and
//! Voltaic is not among them, so before this existed a ```voltaic block
//! rendered flat — measured: zero `<span>` elements, code at 8.8px in one
//! colour, in a book whose entire value is its code samples.
//!
//! The obvious fix is a highlight.js language definition. That would be a
//! *third* grammar — after the hand-written parser and the tree-sitter one —
//! and the repository already records what happens to the second: `grammar.js`
//! was green before it knew `>>>`, because a flat alternation accepted `>>`
//! then `>`. A third would drift the same way, and silently, because nothing
//! compiles a highlighter.
//!
//! So this links `voltaic-lexer` and highlights the real token stream. If the
//! lexer's idea of a keyword changes, the highlighting changes with it, because
//! there is no second opinion to keep in step.
//!
//! Two consequences worth stating. Highlighting happens at **build time**, so
//! the published HTML carries the spans and needs no JavaScript — which is what
//! a crawler and a language model see too. And a block that does not lex is
//! reported on stderr and emitted as plain text rather than failing the build:
//! `examples/lexical.sdx` is deliberately unlexable, and the book quotes it.

use std::io::{Read, Write};

use voltaic_lexer::{TokenKind, TriviaKind, lex};

/// The CSS classes, which are `design/tokens/tokens.json`'s syntax classes.
/// `design/build.sh` emits `www/theme/syntax.css` from the same ten names, so
/// a class here with no colour there is a visible gap rather than a silent one.
const KEYWORD: &str = "keyword";
const TYPE: &str = "type";
const FUNCTION: &str = "function";
const CONSTANT: &str = "constant";
const STRING: &str = "string";
const NUMBER: &str = "number";
const COMMENT: &str = "comment";
const OPERATOR: &str = "operator";
const VARIABLE: &str = "variable";
const PUNCTUATION: &str = "punctuation";

fn main() {
  let mut args = std::env::args().skip(1);
  // The protocol's handshake: `supports <renderer>`, answered by the exit code.
  if args.next().as_deref() == Some("supports") {
    // HTML only. Any other renderer gets the book unchanged, which is the
    // correct answer rather than a failure.
    let renderer = args.next().unwrap_or_default();
    std::process::exit(if renderer == "html" { 0 } else { 1 });
  }

  let mut input = String::new();
  if let Err(error) = std::io::stdin().read_to_string(&mut input) {
    eprintln!("mdbook-voltaic: could not read the book from stdin: {error}");
    std::process::exit(1);
  }

  // `[context, book]`, and only the book is returned.
  let parsed: serde_json::Value = match serde_json::from_str(&input) {
    Ok(value) => value,
    Err(error) => {
      eprintln!("mdbook-voltaic: stdin was not the preprocessor protocol: {error}");
      std::process::exit(1);
    }
  };
  let mut book = parsed.get(1).cloned().unwrap_or_else(|| parsed.clone());

  // mdBook 0.5 renamed this key from `sections` to `items`. Both are accepted
  // so the tool works against either, and a build against neither is a
  // **failure** rather than a quiet pass — see below.
  let mut blocks = 0;
  let mut chapters = 0;
  let found = ["items", "sections"].into_iter().find_map(|key| {
    book
      .get_mut(key)
      .and_then(|value| value.as_array_mut())
      .map(|array| {
        walk(array, &mut blocks, &mut chapters);
      })
  });

  // **Zero is an error, and this is why.** The first version of this read
  // `sections`, which is 0.4's name; against mdBook 0.5.4 it walked nothing,
  // printed "highlighted 0 Voltaic blocks", exited 0, and the book built
  // perfectly with every sample unhighlighted. A preprocessor that cannot find
  // the book has to say so loudly, because its output looks like success.
  if found.is_none() {
    eprintln!(
      "mdbook-voltaic: the book has neither an `items` nor a `sections` key, so \
       nothing could be highlighted. The preprocessor protocol has changed; \
       book keys were: {:?}",
      book.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    std::process::exit(1);
  }
  if chapters == 0 {
    eprintln!("mdbook-voltaic: walked the book and found no chapters at all");
    std::process::exit(1);
  }
  if blocks == 0 {
    eprintln!(
      "mdbook-voltaic: {chapters} chapters, and not one Voltaic block. Every \
       language chapter carries a sample, so this is a defect rather than a \
       thin book."
    );
    std::process::exit(1);
  }
  eprintln!("mdbook-voltaic: highlighted {blocks} blocks across {chapters} chapters");

  let out = serde_json::to_string(&book).expect("the book should serialise");
  if let Err(error) = std::io::stdout().write_all(out.as_bytes()) {
    eprintln!("mdbook-voltaic: could not write the book: {error}");
    std::process::exit(1);
  }
}

/// Rewrite every chapter in `items`, and every chapter beneath them.
fn walk(items: &mut [serde_json::Value], blocks: &mut usize, chapters: &mut usize) {
  for item in items {
    let Some(chapter) = item.get_mut("Chapter") else {
      continue;
    };
    *chapters += 1;
    if let Some(content) = chapter
      .get_mut("content")
      .and_then(|c| c.as_str())
      .map(String::from)
    {
      let (rewritten, n) = rewrite(&content);
      *blocks += n;
      chapter["content"] = serde_json::Value::String(rewritten);
    }
    if let Some(children) = chapter.get_mut("sub_items").and_then(|s| s.as_array_mut()) {
      walk(children, blocks, chapters);
    }
  }
}

/// Replace every Voltaic fence in `markdown` with highlighted HTML.
///
/// Scanned line by line rather than with a regular expression, because a fence
/// is a line-oriented construct and the body may contain anything at all —
/// including the triple quotes of Voltaic's own inline-Rust escape hatch.
fn rewrite(markdown: &str) -> (String, usize) {
  let mut out = String::with_capacity(markdown.len());
  let mut lines = markdown.lines().peekable();
  let mut count = 0;

  while let Some(line) = lines.next() {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let Some(info) = trimmed.strip_prefix("```") else {
      out.push_str(line);
      out.push('\n');
      continue;
    };
    let lang = info.trim();
    if lang != "voltaic" && lang != "sdx" {
      out.push_str(line);
      out.push('\n');
      continue;
    }

    // Collect to the closing fence at the same indentation.
    let mut body = String::new();
    let closing = format!("{indent}```");
    for inner in lines.by_ref() {
      if inner.trim_end() == closing.trim_end() || inner.trim() == "```" {
        break;
      }
      body.push_str(inner.strip_prefix(indent).unwrap_or(inner));
      body.push('\n');
    }

    out.push_str(&render(&body));
    out.push('\n');
    count += 1;
  }

  (out, count)
}

/// `<pre><code>` with one span per classified range.
fn render(source: &str) -> String {
  let mut html = String::from("<pre><code class=\"language-voltaic\">");
  for (text, class) in classify(source) {
    match class {
      Some(class) => {
        html.push_str("<span class=\"tok-");
        html.push_str(class);
        html.push_str("\">");
        escape(&text, &mut html);
        html.push_str("</span>");
      }
      None => escape(&text, &mut html),
    }
  }
  html.push_str("</code></pre>");
  html
}

/// The source split into runs, each either classified or plain.
///
/// Driven by walking the **original source** between spans rather than by
/// concatenating token text, so every byte survives — which matters more here
/// than in a braced language: Voltaic is off-side-rule, and indentation *is*
/// the block structure. Reassembling from tokens would lose it.
fn classify(source: &str) -> Vec<(String, Option<&'static str>)> {
  let Ok(tokens) = lex(source) else {
    // `examples/lexical.sdx` is the error-path example and is quoted in the
    // book on purpose, so an unlexable block is plain text and a note, not a
    // failed build.
    eprintln!("mdbook-voltaic: a block did not lex; emitted as plain text");
    return vec![(source.to_string(), None)];
  };

  let mut spans: Vec<(usize, usize, &'static str)> = Vec::new();
  for token in &tokens {
    for trivium in &token.leading {
      if let Some(class) = trivia_class(&trivium.kind) {
        spans.push((trivium.span.0, trivium.span.1, class));
      }
    }
    if let Some(class) = token_class(&token.kind) {
      spans.push((token.span.0, token.span.1, class));
    }
    if let Some(trivium) = &token.trailing {
      if let Some(class) = trivia_class(&trivium.kind) {
        spans.push((trivium.span.0, trivium.span.1, class));
      }
    }
  }
  spans.sort_by_key(|(start, end, _)| (*start, *end));

  let mut runs: Vec<(String, Option<&'static str>)> = Vec::new();
  let mut at = 0usize;
  for (start, end, class) in spans {
    // Zero-width and out-of-order spans are skipped rather than trusted; an
    // overlap would otherwise duplicate source text into the output.
    if start < at || end <= start || end > source.len() {
      continue;
    }
    if start > at {
      runs.push((source[at..start].to_string(), None));
    }
    runs.push((source[start..end].to_string(), Some(class)));
    at = end;
  }
  if at < source.len() {
    runs.push((source[at..].to_string(), None));
  }
  runs
}

/// A trivium's class. Whitespace carries none and falls through to plain text.
fn trivia_class(kind: &TriviaKind) -> Option<&'static str> {
  match kind {
    TriviaKind::LineComment
    | TriviaKind::DocComment
    | TriviaKind::InnerDocComment
    | TriviaKind::FencedComment => Some(COMMENT),
    // An attribute is a directive rather than a comment, and includes
    // Voltaic's own `#[voltaic::allow(...)]` lint suppressions.
    TriviaKind::Attribute | TriviaKind::InnerAttribute => Some(CONSTANT),
    TriviaKind::BlankLines => None,
  }
}

/// A token's class.
///
/// Exhaustive by hand rather than by a wildcard arm: a new `TokenKind` should
/// fail to compile here and be classified deliberately, not inherit whatever
/// the catch-all happened to say.
fn token_class(kind: &TokenKind) -> Option<&'static str> {
  match kind {
    TokenKind::Kw(_) => Some(KEYWORD),

    // The one lexical inference in this file. The lexer has a single `Ident`,
    // and `tree-sitter-voltaic/grammar.js` distinguishes `type_identifier` by
    // an uppercase initial — the same rule Rust itself uses by convention. So
    // `Terminal` is a type and `potential` is a value, decided from the first
    // character and nothing else. It is a convention rather than a parse, and
    // is wrong only for a deliberately miscapitalised name.
    TokenKind::Ident(name) => Some(match name.chars().next() {
      Some(first) if first.is_uppercase() => TYPE,
      _ => VARIABLE,
    }),

    TokenKind::Label(_) => Some(FUNCTION),

    TokenKind::Int(_) | TokenKind::Float(_) => Some(NUMBER),

    // `Str` and `RawStr` are struct variants: `Str { parts, raw }` carries the
    // interpolation parts and `RawStr { text, hashes }` the hash count.
    // Neither is needed here — the span is what gets wrapped.
    TokenKind::Str { .. }
    | TokenKind::RawStr { .. }
    | TokenKind::ByteStr(_)
    | TokenKind::CStr(_)
    | TokenKind::Char(_)
    | TokenKind::Byte(_) => Some(STRING),

    // Doc 01 §23.1's escape hatch. Its body is Rust, emitted verbatim and
    // judged by rustc rather than by Voltaic, so it is marked as one opaque
    // run instead of being highlighted as if it were Voltaic.
    TokenKind::RustBlock(_) => Some(STRING),

    TokenKind::UserOp(_)
    | TokenKind::Eq
    | TokenKind::Arrow
    | TokenKind::DrawArrow
    | TokenKind::FatArrow
    | TokenKind::Pipe
    | TokenKind::Compose
    | TokenKind::ComposeBack
    | TokenKind::Coalesce
    | TokenKind::CoalesceEq
    | TokenKind::Question
    | TokenKind::DotDotDot
    | TokenKind::DotDot
    | TokenKind::DotDotEq
    | TokenKind::DotParen
    | TokenKind::DotPlus
    | TokenKind::DotMinus
    | TokenKind::DotStar
    | TokenKind::DotSlash
    | TokenKind::DotPercent
    | TokenKind::AmpColon
    | TokenKind::Backslash
    | TokenKind::Dollar
    | TokenKind::At
    | TokenKind::AtAt
    | TokenKind::ColonEq
    | TokenKind::Plus
    | TokenKind::Minus
    | TokenKind::Star
    | TokenKind::Slash
    | TokenKind::Percent
    | TokenKind::PlusEq
    | TokenKind::MinusEq
    | TokenKind::StarEq
    | TokenKind::SlashEq
    | TokenKind::PercentEq
    | TokenKind::EqEq
    | TokenKind::Ne
    | TokenKind::Lt
    | TokenKind::Gt
    | TokenKind::Le
    | TokenKind::Ge
    | TokenKind::AndAnd
    | TokenKind::OrOr
    | TokenKind::Bang
    | TokenKind::Amp
    | TokenKind::Or
    | TokenKind::Caret
    | TokenKind::Tilde
    | TokenKind::Shl
    | TokenKind::Shr => Some(OPERATOR),

    TokenKind::LParen
    | TokenKind::RParen
    | TokenKind::LBracket
    | TokenKind::RBracket
    | TokenKind::LBrace
    | TokenKind::RBrace
    | TokenKind::ColonColon
    | TokenKind::Colon
    | TokenKind::Comma
    | TokenKind::Dot
    | TokenKind::Semi => Some(PUNCTUATION),

    // Layout and the end marker. These are the off-side rule's own tokens;
    // their text is whitespace, which is preserved by walking the source.
    TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => None,
  }
}

/// HTML-escape into `out`. Four characters, because the output goes inside
/// `<code>` and nothing else needs it.
fn escape(text: &str, out: &mut String) {
  for ch in text.chars() {
    match ch {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      '"' => out.push_str("&quot;"),
      other => out.push(other),
    }
  }
}
