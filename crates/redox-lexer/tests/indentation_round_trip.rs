//! Plan 43's one real property: the lexer's block structure agrees with the
//! text it came from.
//!
//! Redox is off-side-rule. A block is opened by indentation, and the lexer
//! turns that into `Indent` and `Dedent` tokens by keeping a stack of widths.
//! **Every later stage trusts that stack without checking it** — the parser
//! reads `Indent` to open a block, the emitter never sees the text at all, and
//! plan 34's leaf 4 will re-indent from exactly these tokens.
//!
//! So this is the assertion that the stack and the text say the same thing. If
//! they ever disagree, the formatter will change block structure silently,
//! which is the one failure a formatter may not have.
//!
//! Plan 43 §1.1: the other three properties row 43 names already exist, each
//! built by the row whose claim needed it — plan 04's determinism and plan 34
//! leaf 5's two. This is the one that was outstanding.

use redox_lexer::{TokenKind, lex};
use std::path::PathBuf;

fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.sdx` in the repository.
///
/// **`lexical.sdx` is included**, unlike in every other corpus sweep. It is
/// exempt elsewhere because it does not *parse*; this property is about
/// lexing, which it must do — so it is the most valuable input here rather
/// than the one to skip. Plan 43's criterion 3.
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
      if path.extension().is_some_and(|x| x == "sdx") {
        found.push(path);
      }
    }
  }
  found.sort();
  assert!(found.len() > 20, "expected the corpora, found {found:?}");
  found
}

/// The indentation width of the line a byte offset falls on, or `None` if the
/// line is blank or a comment-only line.
///
/// Blank lines carry no depth — the lexer's `dedent_block` makes the same
/// exclusion for the same reason, and a blank line's width would otherwise
/// force every comparison to zero.
fn line_indent(source: &str, offset: usize) -> Option<usize> {
  let start = source[..offset.min(source.len())]
    .rfind('\n')
    .map_or(0, |n| n + 1);
  let end = source[start..]
    .find('\n')
    .map_or(source.len(), |n| start + n);
  let line = &source[start..end];
  if line.trim().is_empty() {
    return None;
  }
  Some(line.len() - line.trim_start_matches(' ').len())
}

/// Criterion 1: the depth the tokens imply equals the depth the text implies.
///
/// Walking the stream, `Indent` pushes a level and `Dedent` pops one. The
/// first token on each new line should then sit at a width consistent with the
/// token depth — monotone in the same direction as the text, which is the
/// strongest statement that holds without assuming a fixed indent unit.
#[test]
fn token_depth_tracks_the_texts_indentation() {
  for path in sources() {
    let source = std::fs::read_to_string(&path).expect("should read");
    let Ok(tokens) = lex(&source) else {
      // A file that does not lex is `RDX0002`'s business, asserted separately.
      continue;
    };

    let mut depth: i32 = 0;
    // The text width seen at each token depth, so a later line at the same
    // depth can be checked against it.
    let mut width_at: Vec<usize> = Vec::new();

    for token in &tokens {
      match token.kind {
        TokenKind::Indent => {
          depth += 1;
          width_at.truncate(depth as usize);
        }
        TokenKind::Dedent => {
          depth = (depth - 1).max(0);
          width_at.truncate(depth as usize + 1);
        }
        TokenKind::Newline | TokenKind::Eof => {}
        _ => {
          let Some(width) = line_indent(&source, token.span.0) else {
            continue;
          };
          let level = depth as usize;
          match width_at.get(level) {
            // The same token depth must mean the same text width, within one
            // file — which is exactly what the lexer's width stack claims.
            Some(&known) => assert_eq!(
              known,
              width,
              "{}: token depth {level} is text width {known} elsewhere but {width} at byte {}",
              path.display(),
              token.span.0
            ),
            None => {
              while width_at.len() < level {
                width_at.push(width);
              }
              width_at.push(width);
            }
          }
        }
      }
    }
  }
}

/// Criterion 2: the property's negative case.
///
/// A property test over a corpus that all passes cannot distinguish *the
/// property holds* from *the property is vacuous*. Doc 03 §3.1's `RDX0002`
/// exists because an indentation width matching no enclosing level is an
/// error, and this is the assertion that it is one rather than a silently
/// chosen depth.
#[test]
fn inconsistent_indentation_is_rejected() {
  // Opens at four, then **dedents to two**, which matches neither 0 nor 4.
  //
  // The first version of this case used a *deeper* second line — two spaces
  // then three — and it lexed fine, correctly: a deeper line opens a block.
  // `RDX0002` is about a **dedent** landing between two levels, which is the
  // only way the width stack can be asked for a level it does not hold.
  let source = "main :: ()\nmain =\n    print 1\n  print 2\n";
  let failed = lex(source);
  assert!(
    failed.is_err(),
    "a dedent matching no enclosing level must be RDX0002, got {:?}",
    failed.map(|t| t.len())
  );
}

/// And two positive counterparts, so the negative case is not passing for
/// some unrelated reason.
#[test]
fn consistent_indentation_is_accepted() {
  let flat = "main :: ()\nmain =\n  print 1\n  print 2\n";
  assert!(lex(flat).is_ok(), "two lines at one level should lex");

  // A deeper line opens a block, and dedenting back to a level the stack
  // holds closes it. This is the case the first negative attempt used, and it
  // is *correct* Redox — which is why it belongs here instead.
  let nested = "main :: ()\nmain =\n  if c\n    print 1\n  print 2\n";
  assert!(lex(nested).is_ok(), "a nested block should lex");
}
