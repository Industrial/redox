//! Enforces `docs/06 - Redox Code Architecture.md` §1: one construct per file.
//!
//! A rule that is only written down drifts. A rule with a test attached either
//! holds or the build is red, which is why this exists rather than a review
//! checklist item (doc 06 §9.1).
//!
//! The failure this prevents is not hypothetical: Emerald has 34,429 lines in
//! one `lib.rs` and 22,589 in another. Nothing about that was decided — it is
//! what happens when a file has no rule telling it when to stop growing.

use std::path::{Path, PathBuf};

/// Items that count as constructs. Matched at column 0 only, so an `fn` inside
/// an `impl` block or a `mod tests` is indented and therefore not counted.
const CONSTRUCT_KEYWORDS: [&str; 8] = [
  "struct", "enum", "union", "trait", "type", "fn", "const", "static",
];

/// Module roots hold declarations and re-exports, not constructs (doc 06 §1.2).
/// `main.rs` additionally holds `fn main`, which has no other legal home.
const MODULE_ROOTS: [&str; 2] = ["lib.rs", "main.rs"];

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries {
    let path = entry.expect("should read a directory entry").path();
    if path.is_dir() {
      sources(&path, out);
    } else if path.extension().is_some_and(|e| e == "rs") {
      out.push(path);
    }
  }
}

/// The top-level construct declarations in `text`, by name.
///
/// Deliberately line-based rather than a real parse: the check must not need a
/// Rust parser to run, and column-0 matching is sufficient because `rustfmt`
/// guarantees indentation of nested items.
fn constructs(text: &str) -> Vec<String> {
  text
    .lines()
    .filter_map(|line| {
      // Column 0 only. Anything nested is indented by rustfmt.
      if line.starts_with(char::is_whitespace) {
        return None;
      }
      // Comments and attributes are not declarations, and a comment can
      // legitimately contain the word `fn`.
      if line.starts_with("//") || line.starts_with("#[") || line.starts_with("#!") {
        return None;
      }

      // Scan words left to right for the first construct keyword, so leading
      // modifiers — `pub`, `pub(crate)`, `const fn`, `unsafe fn`, `async fn` —
      // are passed over rather than stripped in a fixed order. Stripping
      // `const` first is what previously made `pub(crate) const USAGE`
      // register as no construct at all.
      let words: Vec<&str> = line.split_whitespace().collect();
      let at = words.iter().position(|w| CONSTRUCT_KEYWORDS.contains(w))?;

      // `static mut NAME` carries a modifier between keyword and name.
      let mut rest = &words[at + 1..];
      if rest.first() == Some(&"mut") {
        rest = &rest[1..];
      }
      let name: String = rest
        .first()?
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
      (!name.is_empty()).then_some(name)
    })
    .collect()
}

#[test]
fn every_file_declares_exactly_one_construct() {
  let mut files = Vec::new();
  sources(&workspace_root().join("crates"), &mut files);
  files.sort();
  assert!(
    !files.is_empty(),
    "should find the workspace's Rust sources"
  );

  let mut offenders = Vec::new();
  for path in &files {
    let name = path
      .file_name()
      .expect("a file name")
      .to_string_lossy()
      .to_string();
    // Integration tests under `tests/` are many `#[test] fn`s by design.
    let is_integration_test = path.components().any(|c| c.as_os_str() == "tests");
    if MODULE_ROOTS.contains(&name.as_str()) || is_integration_test || name == "build.rs" {
      continue;
    }

    let text = std::fs::read_to_string(path).expect("should read a source file");
    let found = constructs(&text);
    if found.len() != 1 {
      offenders.push(format!(
        "{} declares {} constructs ({})",
        path.display(),
        found.len(),
        if found.is_empty() {
          "none".to_string()
        } else {
          found.join(", ")
        }
      ));
    }
  }

  assert!(
    offenders.is_empty(),
    "doc 06 §1 requires exactly one construct per file:\n  {}",
    offenders.join("\n  ")
  );
}

/// The file name must be the construct's name in `snake_case` (doc 06 §1.4),
/// so a construct can always be found without searching.
#[test]
fn every_file_is_named_after_its_construct() {
  let mut files = Vec::new();
  sources(&workspace_root().join("crates"), &mut files);
  files.sort();

  let mut offenders = Vec::new();
  for path in &files {
    let name = path
      .file_name()
      .expect("a file name")
      .to_string_lossy()
      .to_string();
    let is_integration_test = path.components().any(|c| c.as_os_str() == "tests");
    if MODULE_ROOTS.contains(&name.as_str()) || is_integration_test || name == "build.rs" {
      continue;
    }

    let text = std::fs::read_to_string(path).expect("should read a source file");
    let Some(construct) = constructs(&text).into_iter().next() else {
      continue; // reported by the test above
    };
    let stem = name.trim_end_matches(".rs");
    if to_snake_case(&construct) != stem {
      offenders.push(format!(
        "{} declares `{construct}`, so it should be named `{}.rs`",
        path.display(),
        to_snake_case(&construct)
      ));
    }
  }

  assert!(
    offenders.is_empty(),
    "doc 06 §1.4 requires the file name to be the construct's name:\n  {}",
    offenders.join("\n  ")
  );
}

/// `TokenKind` → `token_kind`; `RUST_INDENT` → `rust_indent`; `lex` → `lex`.
fn to_snake_case(name: &str) -> String {
  if name
    .chars()
    .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
  {
    return name.to_lowercase();
  }
  let mut out = String::new();
  for (i, ch) in name.char_indices() {
    if ch.is_uppercase() {
      if i > 0 {
        out.push('_');
      }
      out.extend(ch.to_lowercase());
    } else {
      out.push(ch);
    }
  }
  out
}

#[cfg(test)]
mod snake_case_tests {
  use super::to_snake_case;

  #[test]
  fn converts_the_names_this_workspace_uses() {
    assert_eq!(to_snake_case("TokenKind"), "token_kind");
    assert_eq!(to_snake_case("Token"), "token");
    assert_eq!(to_snake_case("RUST_INDENT"), "rust_indent");
    assert_eq!(to_snake_case("USAGE"), "usage");
    assert_eq!(to_snake_case("lex"), "lex");
    assert_eq!(to_snake_case("is_ident_start"), "is_ident_start");
    assert_eq!(to_snake_case("ParseFailure"), "parse_failure");
  }
}
