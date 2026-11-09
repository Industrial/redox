use crate::Diagnostic;
use crate::render::anchor_span::anchor_span;

/// A [`Diagnostic`] in the shape `miette` reports.
///
/// Two reasons this wrapper exists rather than `impl miette::Diagnostic for
/// Diagnostic`:
///
/// 1. `miette::Diagnostic::source_code` returns a *borrowed* `&dyn
///    SourceCode`, so the [`miette::NamedSource`] that puts the file name on
///    the snippet has to be owned by the reported value. Without it `miette`
///    renders a nameless `╭────`, and a diagnostic that does not say which
///    file it is about is a bug report waiting to happen.
/// 2. It keeps `miette` out of the stage-facing data type. `Diagnostic` is
///    plain data that any renderer can consume (doc 06 §6 rule 2); the
///    coupling to one renderer lives here, inside `render`.
pub(crate) struct Reportable {
  message: String,
  code: String,
  help: Option<String>,
  span: (usize, usize),
  source: miette::NamedSource<String>,
  severity: crate::Severity,
}

impl Reportable {
  pub(crate) fn new(d: &Diagnostic) -> Self {
    Self {
      message: d.message.clone(),
      code: d.code.text(),
      help: d.help.clone(),
      span: anchor_span(&d.source, d.span),
      source: miette::NamedSource::new(&d.path, d.source.clone()).with_language("Rust"),
      severity: d.severity,
    }
  }
}

impl std::fmt::Display for Reportable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::fmt::Debug for Reportable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for Reportable {}

impl miette::Diagnostic for Reportable {
  fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
    Some(Box::new(&self.code))
  }

  /// What decides whether the rendered line reads `error[RDX…]` or
  /// `warning[RDX…]` — doc 03 §12.7, plan 38.
  fn severity(&self) -> Option<miette::Severity> {
    Some(self.severity.to_miette())
  }

  fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
    self
      .help
      .as_ref()
      .map(|h| Box::new(h) as Box<dyn std::fmt::Display>)
  }

  fn source_code(&self) -> Option<&dyn miette::SourceCode> {
    Some(&self.source)
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
    // A zero-width span would render no caret at all, so widen it to one byte.
    let len = (self.span.1.saturating_sub(self.span.0)).max(1);
    Some(Box::new(std::iter::once(miette::LabeledSpan::new(
      Some("here".to_string()),
      self.span.0,
      len,
    ))))
  }
}
