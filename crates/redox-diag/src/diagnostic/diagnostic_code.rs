use crate::Code;

/// A diagnostic's code: Redox's own, or another compiler's passed through.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 1. `redox check` reports
/// rustc's errors against the `.sdx`, changing **only the position** — the
/// message, the severity and the code stay rustc's (plan 07 §1.1). So a
/// diagnostic has to be able to carry `E0308`.
///
/// Why this is a separate type rather than a `Code::Rustc(String)` variant:
/// [`Code`] is a closed, numbered catalogue whose value is precisely that it
/// is closed — plan 05's `code_coverage.rs` fails if any variant is
/// unreachable from the UI corpus, which is what makes `RDX0105` a promise
/// rather than a hope. A string variant would be in the catalogue without
/// being *of* it. A foreign code is explicitly outside, and the type says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCode {
  /// Redox's own, from the `RDX` catalogue.
  Redox(Code),
  /// Another compiler's, verbatim — `E0308`, `unused_variables`.
  ///
  /// Never parsed, never renumbered. A user who searches for it must find
  /// Rust's documentation, not Redox's.
  Foreign(String),
}

impl DiagnosticCode {
  /// The code as it is printed.
  pub fn text(&self) -> String {
    match self {
      Self::Redox(code) => code.text(),
      Self::Foreign(text) => text.clone(),
    }
  }

  /// Whether this code comes from Redox's catalogue.
  ///
  /// Plan 05's corpus checks read `.stderr` files for `RDX` codes; this is how
  /// a later `check` UI case can be excluded from that count without the check
  /// having to pattern-match on the text.
  pub fn is_redox(&self) -> bool {
    matches!(self, Self::Redox(_))
  }
}

impl From<Code> for DiagnosticCode {
  fn from(code: Code) -> Self {
    Self::Redox(code)
  }
}

#[cfg(test)]
mod tests {
  use super::DiagnosticCode;
  use crate::Code;

  #[test]
  fn renders_a_redox_code_from_the_catalogue() {
    let code: DiagnosticCode = Code::ExpectedToken.into();
    assert_eq!(code.text(), "RDX0101");
    assert!(code.is_redox());
  }

  #[test]
  fn renders_a_foreign_code_untouched() {
    let code = DiagnosticCode::Foreign("E0308".to_string());
    assert_eq!(code.text(), "E0308");
    assert!(!code.is_redox());
  }

  /// A lint name is not numbered, and must survive anyway.
  #[test]
  fn carries_a_foreign_code_that_is_not_a_number() {
    let code = DiagnosticCode::Foreign("unused_variables".to_string());
    assert_eq!(code.text(), "unused_variables");
  }
}
