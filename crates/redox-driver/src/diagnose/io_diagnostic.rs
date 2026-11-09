use redox_diag::{Diagnostic, DiagnosticCode};

/// A diagnostic for something that failed outside the source text.
///
/// Plan 09 (`docs/14 - 09 - Cargo Integration.md`) leaf 1. Grouped with
/// [`super::diagnose`] because both construct a [`Diagnostic`], which is what
/// makes `diagnose.rs` a module head rather than a lone root-level file
/// (doc 06 §2.1).
///
/// A missing file or an
/// unwritable `OUT_DIR` is not a language error and has no span, but a build
/// script should have exactly **one** thing to render. A second error type for
/// I/O would make every caller match on two shapes to print one message.
///
/// The code is `DiagnosticCode::Foreign("io")` rather than an `RDX` number:
/// the `RDX` catalogue is closed and test-enforced (plan 05 leaf 6), and an
/// I/O failure is not a property of the language. `Foreign` is exactly the
/// escape hatch plan 07 added for codes Redox does not own.
pub(crate) fn io_diagnostic(path: &str, message: String) -> Diagnostic {
  Diagnostic {
    code: DiagnosticCode::Foreign("io".to_string()),
    severity: redox_diag::Severity::Error,
    message,
    span: (0, 0),
    help: None,
    path: path.to_string(),
    source: String::new(),
  }
}

#[cfg(test)]
mod tests {
  use super::io_diagnostic;

  #[test]
  fn renders_without_a_snippet() {
    let d = io_diagnostic("a.sdx", "cannot read a.sdx: missing".to_string());
    assert_eq!(d.code.text(), "io");
    assert_eq!(d.path, "a.sdx");
    // An empty source means the renderer has no snippet to draw, which it must
    // survive rather than panic on.
    let out = redox_diag::render(std::slice::from_ref(&d), true);
    assert!(out.contains("cannot read"), "{out}");
    assert!(!out.contains("  |"), "no snippet should be drawn:\n{out}");
  }
}
