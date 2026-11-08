/// One diagnostic as rustc's `--error-format=short` prints it.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 2. The format, measured
/// rather than assumed (plan 07 §2):
///
/// ```text
/// main.xrs:6:24: error[E0308]: mismatched types: expected `i32`, found `&str`
/// main.xrs:3:9: warning: unused variable: `x`
/// error: aborting due to 1 previous error
/// ```
///
/// A path, a line, a column, a level, an optional bracketed code, and the
/// message. The last line carries no position: it is rustc's summary, not a
/// diagnostic, and [`RustcMessage::parse`] declines it so Redox can print its
/// own count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustcMessage {
  pub(crate) path: String,
  /// 1-based, in the **generated** file.
  pub(crate) line: usize,
  /// 1-based, in the generated file.
  pub(crate) col: usize,
  /// `error` or `warning`, rustc's own word.
  pub(crate) level: String,
  /// `E0308`, or a lint name, or `None` where rustc gave none.
  pub(crate) code: Option<String>,
  /// rustc's message, verbatim — never reworded (plan 07 §1.1).
  pub(crate) message: String,
}

impl RustcMessage {
  /// Parse one line, or `None` if it is not a positioned diagnostic.
  ///
  /// Splitting is done from the **left on `:`** for the three position fields
  /// and then left alone, because the message routinely contains colons
  /// (`expected \`i32\`, found \`&str\`` does not, but `cannot add \`&str\` to
  /// \`i32\`: no implementation` does). A greedy split on the whole line would
  /// truncate the message at rustc's own punctuation.
  ///
  /// A Windows path (`C:\…`) would defeat a naive left split too, which is why
  /// the line and column are found by taking the *last two* colon-separated
  /// fields before the level rather than the second and third overall.
  pub(crate) fn parse(line: &str) -> Option<Self> {
    // `path:line:col: level...` — find the `: ` that ends the position.
    let (position, rest) = line.split_once(": ")?;

    // The position's last two `:` fields are the line and column; everything
    // before them is the path, colons and all.
    let (path, line_col) = position.rsplit_once(':')?;
    let col: usize = line_col.parse().ok()?;
    let (path, line_no) = path.rsplit_once(':')?;
    let line_no: usize = line_no.parse().ok()?;

    // `level[code]: message`, or `level: message`.
    let (label, message) = rest.split_once(": ")?;
    let (level, code) = match label.split_once('[') {
      Some((level, bracketed)) => (level, Some(bracketed.trim_end_matches(']').to_string())),
      None => (label, None),
    };
    if level != "error" && level != "warning" {
      return None;
    }

    Some(Self {
      path: path.to_string(),
      line: line_no,
      col,
      level: level.to_string(),
      code,
      message: message.to_string(),
    })
  }

  pub(crate) fn is_error(&self) -> bool {
    self.level == "error"
  }
}

#[cfg(test)]
mod tests {
  use super::RustcMessage;

  /// The exact line rustc produced for `plan 07 §2`'s measurement.
  #[test]
  fn parses_a_coded_error() {
    let m = RustcMessage::parse(
      ".tmp/typeerr.xrs:6:24: error[E0308]: mismatched types: expected `i32`, found `&str`",
    )
    .expect("should parse");
    assert_eq!(m.path, ".tmp/typeerr.xrs");
    assert_eq!((m.line, m.col), (6, 24));
    assert_eq!(m.level, "error");
    assert_eq!(m.code.as_deref(), Some("E0308"));
    assert_eq!(
      m.message, "mismatched types: expected `i32`, found `&str`",
      "the message keeps rustc's own colons"
    );
    assert!(m.is_error());
  }

  /// The other measured line, whose message contains a colon.
  #[test]
  fn keeps_a_message_containing_a_colon() {
    let m = RustcMessage::parse(
      ".tmp/tail.xrs:2:7: error[E0277]: cannot add `&str` to `i32`: no implementation for `i32 + &str`",
    )
    .expect("should parse");
    assert_eq!(
      m.message,
      "cannot add `&str` to `i32`: no implementation for `i32 + &str`"
    );
  }

  #[test]
  fn parses_an_uncoded_warning() {
    let m =
      RustcMessage::parse("main.xrs:3:9: warning: unused variable: `x`").expect("should parse");
    assert_eq!(m.level, "warning");
    assert_eq!(m.code, None);
    assert_eq!(m.message, "unused variable: `x`");
    assert!(!m.is_error());
  }

  /// rustc's summary line is not a diagnostic.
  #[test]
  fn declines_the_summary_line() {
    assert_eq!(
      RustcMessage::parse("error: aborting due to 1 previous error"),
      None
    );
  }

  #[test]
  fn declines_an_unpositioned_note() {
    assert_eq!(
      RustcMessage::parse("note: run with `RUST_BACKTRACE=1`"),
      None
    );
    assert_eq!(RustcMessage::parse(""), None);
  }

  /// A path containing a colon must not be mistaken for a position.
  #[test]
  fn survives_a_colon_in_the_path() {
    let m = RustcMessage::parse("C:\\out\\main.xrs:6:24: error[E0308]: mismatched types")
      .expect("should parse");
    assert_eq!(m.path, "C:\\out\\main.xrs");
    assert_eq!((m.line, m.col), (6, 24));
  }

  /// A level rustc does not use is not a diagnostic Redox should relay.
  #[test]
  fn declines_an_unknown_level() {
    assert_eq!(RustcMessage::parse("main.xrs:1:1: help: try this"), None);
  }
}
