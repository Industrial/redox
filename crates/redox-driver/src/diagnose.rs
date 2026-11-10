pub(crate) mod io_diagnostic;

use redox_diag::Diagnostic;
use redox_parser::ParseFailure;
use std::path::Path;

/// Turn a [`ParseFailure`] into the diagnostics that represent it.
///
/// Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 4 wrote this in the CLI, on
/// the reasoning that `redox-diag` may not mention `redox-parser` — true, and
/// the CLI was the only caller. Plan 09 gave it a second caller, `compile_dir`,
/// which a build script uses and which must not depend on the CLI (plan 09
/// §2.1). So it moved **down** to `redox-driver`, which sits above both stages
/// and below the CLI.
///
/// That is doc 06 §3 rule 6 applied before the duplicate existed rather than
/// after: the alternative was two copies of the same conversion, and the copies
/// would have drifted the first time a stage error gained a field.
///
/// Every stage error already carries its own code and span, so this function
/// decides nothing — it only attaches the file name and the source text that
/// rendering needs, and copies the rest across.
pub fn diagnose(path: &Path, source: &str, failure: &ParseFailure) -> Vec<Diagnostic> {
  let path = path.display().to_string();
  match failure {
    ParseFailure::Lex(e) => vec![Diagnostic {
      code: e.code.into(),
      severity: redox_diag::Severity::Error,
      message: e.message.clone(),
      span: e.span,
      help: e.help.clone(),
      path,
      source: source.to_string(),
    }],
    ParseFailure::Parse(errors) => errors
      .iter()
      .map(|e| Diagnostic {
        code: e.code.into(),
        severity: redox_diag::Severity::Error,
        message: e.message.clone(),
        span: e.span,
        help: None,
        path: path.clone(),
        source: source.to_string(),
      })
      .collect(),
  }
}

#[cfg(test)]
mod tests {
  use super::diagnose;
  use std::path::Path;

  #[test]
  fn carries_a_lex_errors_code_and_help_across() {
    let source = "main : ()\nmain =\n\tprint 1\n";
    let failure = redox_parser::parse_source(source).expect_err("a tab is an error");
    let diagnostics = diagnose(Path::new("a.sdx"), source, &failure);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code.text(), "RDX0001");
    assert_eq!(diagnostics[0].path, "a.sdx");
    assert!(diagnostics[0].help.is_some(), "the lexer's help survives");
  }

  /// Recovered parse errors arrive as a batch and all of them are reported —
  /// one diagnostic per error, which is what plan 05 criterion 5 asserts end
  /// to end.
  #[test]
  fn reports_every_recovered_parse_error() {
    // Both errors are bounded to their own statement, which is what keeps
    // them *independent*: an error that runs off the end of its line lets
    // statement recovery swallow the following signature line, and the two
    // collapse into one. That is true of doc 18's grammar and not of doc
    // 03's, because a signature line is now something recovery can land in
    // the middle of.
    //
    // `print (1 +)` was the first of them until doc 18 §4 made it a **left
    // section**, which parses. A statement that begins with a binary
    // operator has nowhere left to go, and ends at its own line break.
    let source = "a : ()\na =\n  + 1\n\nb : ()\nb =\n  * \n";
    let failure = redox_parser::parse_source(source).expect_err("should fail");
    let diagnostics = diagnose(Path::new("a.sdx"), source, &failure);
    assert!(
      diagnostics.len() >= 2,
      "expected a diagnostic per error, got {}",
      diagnostics.len()
    );
    assert!(diagnostics.iter().all(|d| d.source == source));
  }
}
