//! Plan 34 leaf 3: `expr_text` prints one expression in canonical Redox.
//!
//! Three assertions, in increasing order of what they would catch.
//!
//! 1. **The canonical form of each §3 row**, as a fixture pair — an accepted
//!    spelling in `fmt/expressions.sdx` and the expected output in
//!    `fmt/expressions.canonical.sdx`. The expectation is a *file* for
//!    AGENTS.md's reason: Emerald's one syntax cutover missed 113 fixtures that
//!    lived as string literals inside test modules, and that cost is paid once
//!    per syntax change, forever. `REDOX_BLESS=1` rewrites it and then fails,
//!    so a changed expectation is always reviewed.
//!
//! 2. **Idempotence** — printing the re-parsed output reproduces it. The same
//!    property leaf 5 asserts for `format`, asked of the printer directly,
//!    where a failure names the expression rather than the file.
//!
//! 3. **Meaning preservation, over every expression in the corpus** — every
//!    binding value in every `.sdx` in the repository re-parses to the same
//!    expression it was printed from. This is the one that would catch a
//!    wrongly dropped parenthesis, and §3 row 3 is the only normalisation in
//!    plan 34 that can change what a program means.
//!
//! # Why the comparison ignores `Paren`
//!
//! `Paren` is a node in the tree, so dropping a redundant parenthesis changes
//! the tree and a literal comparison would reject every correct run of §3
//! row 3. Stripping `Paren` from **both** sides is not a weakening: precedence
//! is recorded in the tree's *shape*, not in its `Paren` nodes, so a
//! parenthesis dropped where it was needed re-associates the operators and the
//! stripped trees differ. The property still catches exactly the mistake it
//! exists for.

use redox_ast::{Expr, Item, Program, Stmt, StmtKind};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fmt")
}

/// Plan 02's lexer fixture, which deliberately does not parse — the same
/// declared exemption `examples.rs` and leaf 5's properties carry.
const NOT_PARSEABLE: [&str; 1] = ["lexical.sdx"];

/// Every `.sdx` in the repository that is meant to parse.
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

/// Every statement in a program, including the ones nested in a block.
///
/// Flat rather than a visitor, because what the caller wants is the bindings
/// and a binding's value is reachable from its own statement.
fn statements(program: &Program) -> Vec<&Stmt> {
  let mut found = Vec::new();
  for item in &program.items {
    match item {
      Item::Fn(decl) => {
        for clause in &decl.clauses {
          if let Some(block) = clause.body.plain() {
            collect(&block.stmts, &mut found);
          }
          collect(&clause.wheres, &mut found);
        }
      }
      Item::Test(decl) => collect(&decl.body.stmts, &mut found),
      _ => {}
    }
  }
  found
}

fn collect<'a>(stmts: &'a [Stmt], into: &mut Vec<&'a Stmt>) {
  for stmt in stmts {
    into.push(stmt);
    for block in blocks_of(&stmt.kind) {
      collect(&block.stmts, into);
    }
  }
}

/// The blocks a statement kind holds, so nesting is not skipped.
///
/// A control-flow statement's body is where most of a program's expressions
/// live, and a traversal that stopped at the top level would make the corpus
/// property look thorough while reading almost nothing.
fn blocks_of(kind: &StmtKind) -> Vec<&redox_ast::Block> {
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
    // A `guard`'s block is the *else* branch — the one taken when the pattern
    // does not match — which is why it is named `otherwise` and not `body`.
    StmtKind::Guard { otherwise, .. } => vec![otherwise],
    _ => Vec::new(),
  }
}

/// The value of every binding in a program, in order.
fn binding_values(program: &Program) -> Vec<&Expr> {
  statements(program)
    .into_iter()
    .filter_map(|stmt| match &stmt.kind {
      StmtKind::Binding(binding) => Some(&binding.value),
      _ => None,
    })
    .collect()
}

/// An expression's debug form with spans blanked and `Paren` nodes removed.
///
/// The span scan is leaf 5's, for the reason its own comment records: a
/// pretty-printed span is four lines, so a line filter leaves the numbers
/// behind. `Paren` is stripped here for the reason the module doc gives.
fn meaning(expr: &Expr) -> String {
  let text = format!("{expr:?}");
  let mut out = String::with_capacity(text.len());
  let mut rest = text.as_str();
  while let Some(at) = rest.find("span: (") {
    out.push_str(&rest[..at]);
    out.push_str("span: _");
    rest = &rest[at + "span: (".len()..];
    match rest.find(')') {
      Some(close) => rest = &rest[close + 1..],
      None => break,
    }
  }
  out.push_str(rest);
  strip_parens(&out)
}

/// Every `Paren` node removed, at every depth.
///
/// A top-level strip is not enough, and the shape of why is the point: the
/// printer **adds** parentheses where a form would otherwise be misread —
/// `-(a + b)`, `f (g x)`, `(\x -> x) :: T` — and those land at whatever depth
/// the form sits at. So the two trees legitimately differ by `Paren` nodes
/// anywhere, and a comparison that tolerated them only at the root reported
/// the printer had changed the program when it had not.
///
/// Done on the debug string rather than by rebuilding the tree, because
/// rebuilding means thirty-one match arms in a test file that would then have
/// to be kept in step with the AST — a second copy of the AST is a worse
/// liability than a scanner.
fn strip_parens(text: &str) -> String {
  const NEEDLE: &str = "Paren { inner: ";
  let mut out = text.to_string();
  while let Some(at) = out.find(NEEDLE) {
    let open = at + "Paren ".len();
    let Some(close) = matching_brace(&out, open) else {
      break;
    };
    // `Paren`'s fields are `inner` then `span`, so the last `, span: _` before
    // the closing brace is the node's own and everything before it is `inner`.
    let inner_start = at + NEEDLE.len();
    let Some(inner_end) = out[..close].rfind(", span: _") else {
      break;
    };
    let inner = out[inner_start..inner_end].to_string();
    out.replace_range(at..=close, &inner);
  }
  out
}

/// The index of the `}` that closes the `{` at `open`.
///
/// Quoted text is skipped: a string literal's debug form can hold a brace —
/// `StrLit { value: "a{b}" }` — and counting it would close the wrong node.
fn matching_brace(text: &str, open: usize) -> Option<usize> {
  let bytes: Vec<char> = text.chars().collect();
  // The scan is over chars, so the byte index has to be rebuilt at the end.
  let mut offsets = Vec::with_capacity(bytes.len());
  for (i, _) in text.char_indices() {
    offsets.push(i);
  }
  let start = offsets.iter().position(|&o| o == open)?;
  let mut depth = 0usize;
  let mut quote: Option<char> = None;
  let mut escaped = false;
  for i in start..bytes.len() {
    let c = bytes[i];
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

/// Plan 34 §3's rows, as a fixture pair.
#[test]
fn each_normalisation_reaches_its_canonical_form() {
  let input = fixtures().join("expressions.sdx");
  let expected_path = fixtures().join("expressions.canonical.sdx");
  let source = std::fs::read_to_string(&input).expect("the fixture should exist");
  let program = redox_parser::parse_source(&source).expect("the fixture should parse");

  let mut lines = Vec::new();
  for value in binding_values(&program) {
    lines.push(redox_fmt::expr_text(value, 0, 0, &source));
  }
  let printed = format!("{}\n", lines.join("\n"));

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
    "the printed expressions differ from {}",
    expected_path.display()
  );
}

/// Printing what was printed changes nothing.
#[test]
fn printing_is_idempotent() {
  for path in sources() {
    let source = std::fs::read_to_string(&path).expect("should read");
    let Ok(program) = redox_parser::parse_source(&source) else {
      continue;
    };
    for value in binding_values(&program) {
      let once = redox_fmt::expr_text(value, 0, 0, &source);
      // A block-bearing form is a verbatim slice and spans more than one
      // line, so it cannot be re-parsed as a binding's value. Leaf 4 prints
      // those, and will assert them.
      if once.contains('\n') {
        continue;
      }
      let Some((reparsed, probe)) = reparse(&once) else {
        panic!("{}: `{once}` did not re-parse", path.display());
      };
      let twice = redox_fmt::expr_text(&reparsed, 0, 0, &probe);
      assert_eq!(
        once,
        twice,
        "{}: printing is not idempotent",
        path.display()
      );
    }
  }
}

/// The property §3 row 3 needs: printing an expression does not change it.
#[test]
fn printing_preserves_meaning() {
  let mut checked = 0;
  for path in sources() {
    let source = std::fs::read_to_string(&path).expect("should read");
    let Ok(program) = redox_parser::parse_source(&source) else {
      continue;
    };
    for value in binding_values(&program) {
      let printed = redox_fmt::expr_text(value, 0, 0, &source);
      if printed.contains('\n') {
        continue;
      }
      let Some((reparsed, _)) = reparse(&printed) else {
        panic!("{}: `{printed}` did not re-parse", path.display());
      };
      assert_eq!(
        meaning(value),
        meaning(&reparsed),
        "{}: printing `{printed}` changed the expression",
        path.display()
      );
      checked += 1;
    }
  }
  // A corpus property that read nothing would pass. Leaf 5's §10.8 records
  // that an all-green corpus proves less than it looks, so the count is part
  // of the assertion.
  assert!(
    checked > 40,
    "only {checked} expressions were checked, which is too few to mean anything"
  );
}

/// Parse one expression by putting it where a binding's value goes, and hand
/// back **the text its spans refer to** alongside it.
///
/// The probe text is returned because a re-parsed node's spans are offsets
/// into the probe, not into the printed expression. Printing the re-parse
/// against the printed text was the first version, and it was fine until
/// `literal_text` began slicing the source: then a string literal's span
/// pointed at the wrong bytes and the second pass produced
/// `name =  ()),.to_string ()` — a harness defect reported as a printer
/// defect, which is the third time in this plan (§10.5, §10.20).
fn reparse(text: &str) -> Option<(Expr, String)> {
  let whole = format!("__rdx_probe :: ()\n__rdx_probe =\n  __rdx_it = {text}\n");
  let program = redox_parser::parse_source(&whole).ok()?;
  let found = binding_values(&program).first().map(|e| (*e).clone())?;
  Some((found, whole))
}

/// The fixture is reachable, so a rename cannot silently skip the test above.
#[test]
fn the_fixture_directory_exists() {
  assert!(
    Path::new(&fixtures()).is_dir(),
    "{} should hold the fixture pair",
    fixtures().display()
  );
}
