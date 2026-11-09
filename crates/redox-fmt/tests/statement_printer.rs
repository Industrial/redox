//! Plan 34 leaf 4, second half: `stmt_text` prints a statement, its trivia and
//! its nested blocks.
//!
//! Three assertions, matching the two printers before it.
//!
//! 1. **The canonical form of every `StmtKind`**, as a fixture pair. The
//!    fixture is one function whose body uses all fourteen kinds plus the
//!    trivia forms, and the expectation is the printed body — so the pair
//!    doubles as the readable record of what two-space re-indentation (§3
//!    row 11) produces.
//!
//! 2. **Idempotence**, over every function body in the repository.
//!
//! 3. **Meaning preservation**, likewise: a printed statement re-parses to the
//!    same statement. `Paren` is stripped from both sides, for the reason
//!    leaf 3's §10.12 gives — the printer legitimately adds and removes
//!    grouping, and precedence lives in the tree's shape rather than in its
//!    `Paren` nodes.
//!
//! # Trivia makes this the first printer that can lose text
//!
//! The two before it printed a single line from a node that carries none. A
//! statement carries `leading` and `trailing` trivia, so criterion 4 — *"every
//! comment and attribute in the input appears in the output"* — becomes a live
//! assertion here rather than a vacuous one, and it is asserted by **counting
//! markers**, because a comment that survives into the wrong place is still a
//! comment the count finds.

use redox_ast::{Block, Item, Program, Stmt, StmtKind};
use std::path::PathBuf;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fmt")
}

const NOT_PARSEABLE: [&str; 1] = ["lexical.sdx"];

fn sources() -> Vec<PathBuf> {
  let mut found = Vec::new();
  for dir in [
    root().join("examples"),
    root().join("crates/redox-cli/tests/ui"),
    root().join("crates/redox-cli/tests/lint"),
  ] {
    let Ok(entries) = std::fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries {
      let path = entry.expect("should read an entry").path();
      let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
      if path.extension().is_some_and(|x| x == "sdx") && !NOT_PARSEABLE.contains(&name) {
        found.push(path);
      }
    }
  }
  found.sort();
  assert!(found.len() > 20, "expected the corpora, found {found:?}");
  found
}

/// Every plain function body in a program, in order.
///
/// Guarded clauses are skipped: a guard ladder's body is leaf 2's to print, and
/// its statements are reached through the same `Block` once that lands.
fn bodies(program: &Program) -> Vec<&Block> {
  let mut found = Vec::new();
  for item in &program.items {
    match item {
      Item::Fn(decl) => {
        for clause in &decl.clauses {
          found.extend(clause.body.plain());
        }
      }
      Item::Test(decl) => found.push(&decl.body),
      _ => {}
    }
  }
  found
}

/// A statement's debug form with spans blanked and `Paren` nodes removed.
///
/// The span scan matches the **shape** of a span — a parenthesised pair of
/// bare integers — rather than the words `span: (`, because `Pattern` and
/// `Expr` between them have both named and tuple-variant spans. Leaf 4's first
/// half found that the hard way (§10.20).
fn meaning(stmt: &Stmt) -> String {
  strip_parens(&blank_spans(&format!("{stmt:?}")))
}

fn blank_spans(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut rest = text;
  while let Some(at) = find_span(rest) {
    out.push_str(&rest[..at]);
    out.push('_');
    rest = &rest[at..];
    let close = rest.find(')').expect("find_span matched a closing paren");
    rest = &rest[close + 1..];
  }
  out.push_str(rest);
  out
}

/// Where the next `(N, M)` starts, if there is one.
fn find_span(text: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  for at in 0..bytes.len() {
    if bytes[at] != b'(' {
      continue;
    }
    let mut i = at + 1;
    if !digits(bytes, &mut i) || !text[i..].starts_with(", ") {
      continue;
    }
    i += 2;
    if digits(bytes, &mut i) && bytes.get(i) == Some(&b')') {
      return Some(at);
    }
  }
  None
}

fn digits(bytes: &[u8], at: &mut usize) -> bool {
  let start = *at;
  while *at < bytes.len() && bytes[*at].is_ascii_digit() {
    *at += 1;
  }
  *at > start
}

/// Every `Paren` node removed, at every depth — leaf 3's scanner.
fn strip_parens(text: &str) -> String {
  const NEEDLE: &str = "Paren { inner: ";
  let mut out = text.to_string();
  while let Some(at) = out.find(NEEDLE) {
    let open = at + "Paren ".len();
    let Some(close) = matching_brace(&out, open) else {
      break;
    };
    let inner_start = at + NEEDLE.len();
    let Some(inner_end) = out[..close].rfind(", span: _") else {
      break;
    };
    let inner = out[inner_start..inner_end].to_string();
    out.replace_range(at..=close, &inner);
  }
  out
}

fn matching_brace(text: &str, open: usize) -> Option<usize> {
  let chars: Vec<char> = text.chars().collect();
  let offsets: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
  let start = offsets.iter().position(|&o| o == open)?;
  let mut depth = 0usize;
  let mut quote: Option<char> = None;
  let mut escaped = false;
  for i in start..chars.len() {
    let c = chars[i];
    if escaped {
      escaped = false;
      continue;
    }
    match quote {
      Some(q) => match c {
        '\\' => escaped = true,
        _ if c == q => quote = None,
        _ => {}
      },
      None => match c {
        '"' | '\'' => quote = Some(c),
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            return Some(offsets[i]);
          }
        }
        _ => {}
      },
    }
  }
  None
}

/// Parse one printed statement back, by putting it in a function body.
///
/// The printed text is already indented two spaces per level from depth 1, so
/// it drops straight under an equation's `=` with nothing to re-indent — which
/// is the convention `stmt_text` exists to hold up.
/// # A statement has two homes, and the probe has to try both
///
/// The indented form is the usual one. The other is an equation's **tail** —
/// `tier n = if n > 9 then "big" else "small"` — whose body is a block holding
/// one statement, exactly as an indented body is.
///
/// They are not interchangeable, and that is a parser asymmetry rather than a
/// formatter one: the expression `if` is told apart from the statement `if` by
/// `then`, but only in expression position. At the start of an indented
/// statement the parser commits to the statement form and the `then` is a
/// syntax error. So `if n > 9 then "big" else "small"` — real, from
/// `examples/control.sdx` — re-parses as a tail and not as a statement.
///
/// **This is a constraint on leaf 2**, recorded in §10: an equation whose body
/// is a single expression must stay on the equation line. Moving it to an
/// indented block is a change `redox fmt` would make silently and rustc would
/// never see, because the file would stop parsing as Redox first.
fn reparse(text: &str) -> Option<Stmt> {
  let indented = format!("__rdx_probe :: ()\n__rdx_probe =\n{text}\n");
  let tail = format!("__rdx_probe :: ()\n__rdx_probe = {}\n", text.trim_start());
  for whole in [indented, tail] {
    let Ok(program) = redox_parser::parse_source(&whole) else {
      continue;
    };
    if let Some(stmt) = bodies(&program)
      .first()
      .and_then(|block| block.stmts.first())
    {
      return Some(stmt.clone());
    }
  }
  None
}

/// Every statement kind's canonical form, as a fixture pair.
#[test]
fn every_statement_reaches_its_canonical_form() {
  let input = fixtures().join("statements.sdx");
  let expected_path = fixtures().join("statements.canonical.sdx");
  let source = std::fs::read_to_string(&input).expect("the fixture should exist");
  let program = redox_parser::parse_source(&source).expect("the fixture should parse");

  let body = bodies(&program)
    .first()
    .copied()
    .expect("the fixture has one function");
  let printed = format!(
    "{}\n",
    body
      .stmts
      .iter()
      .map(|stmt| redox_fmt::stmt_text(stmt, 1, &source))
      .collect::<Vec<_>>()
      .join("\n")
  );

  if std::env::var_os("REDOX_BLESS").is_some() {
    std::fs::write(&expected_path, &printed).expect("should write the expectation");
    panic!("blessed {} — review the diff", expected_path.display());
  }

  let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| {
    panic!(
      "{} is missing; run with REDOX_BLESS=1 to create it",
      expected_path.display()
    )
  });
  assert_eq!(
    printed,
    expected,
    "the printed statements differ from {}",
    expected_path.display()
  );
}

/// Printing a statement changes neither what it means nor, on a second pass,
/// what it looks like.
#[test]
fn printing_preserves_the_statement() {
  let mut checked = 0;
  let mut paths = sources();
  paths.push(fixtures().join("statements.sdx"));
  for path in paths {
    let source = std::fs::read_to_string(&path).expect("should read");
    let Ok(program) = redox_parser::parse_source(&source) else {
      continue;
    };
    for block in bodies(&program) {
      for stmt in &block.stmts {
        let printed = redox_fmt::stmt_text(stmt, 1, &source);
        let Some(reparsed) = reparse(&printed) else {
          panic!("{}: did not re-parse:\n{printed}", path.display());
        };
        assert_eq!(
          meaning(stmt),
          meaning(&reparsed),
          "{}: printing this changed it:\n{printed}",
          path.display()
        );
        let twice = redox_fmt::stmt_text(&reparsed, 1, &printed);
        assert_eq!(
          printed,
          twice,
          "{}: printing this is not idempotent:\n{printed}",
          path.display()
        );
        checked += 1;
      }
    }
  }
  assert!(
    checked > 100,
    "only {checked} statements were checked, which is too few to mean anything"
  );
}

/// Criterion 4, which only becomes a real assertion once trivia is printed.
#[test]
fn printing_loses_no_comment_or_attribute() {
  for path in sources() {
    let source = std::fs::read_to_string(&path).expect("should read");
    let Ok(program) = redox_parser::parse_source(&source) else {
      continue;
    };
    for block in bodies(&program) {
      for stmt in &block.stmts {
        let printed = redox_fmt::stmt_text(stmt, 1, &source);
        // Counted on the trivia the statement carries, against the markers in
        // what came out — so a trivium the printer dropped is caught even if
        // no other statement would have shown it.
        let expected = count_trivia(stmt);
        let found = printed.matches("--").count()
          + printed.matches("{-").count()
          + printed.matches("#[").count();
        assert!(
          found >= expected,
          "{}: {expected} trivia went in and {found} markers came out:\n{printed}",
          path.display()
        );
      }
    }
  }
}

/// How many trivia a statement and its nested blocks carry, excluding blank
/// lines — which have no marker to count.
fn count_trivia(stmt: &Stmt) -> usize {
  let mut n = stmt
    .leading
    .iter()
    .filter(|t| !matches!(t.kind, redox_lexer::TriviaKind::BlankLines))
    .count();
  if stmt.trailing.is_some() {
    n += 1;
  }
  for block in blocks_of(&stmt.kind) {
    for inner in &block.stmts {
      n += count_trivia(inner);
    }
  }
  n
}

fn blocks_of(kind: &StmtKind) -> Vec<&Block> {
  match kind {
    StmtKind::If {
      then_block,
      otherwise,
      ..
    } => {
      let mut found = vec![then_block];
      found.extend(otherwise.iter());
      found
    }
    StmtKind::While { body, .. } | StmtKind::For { body, .. } | StmtKind::Loop { body, .. } => {
      vec![body]
    }
    StmtKind::Guard { otherwise, .. } => vec![otherwise],
    _ => Vec::new(),
  }
}
