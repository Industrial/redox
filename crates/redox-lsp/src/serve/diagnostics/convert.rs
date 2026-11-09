use crate::serve::positions::range_of::range_of;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

/// One Redox diagnostic as the protocol's.
///
/// The mapping is field for field, which is what made this row small:
/// `redox_diag::Diagnostic` already carries a code, a severity, a span, a
/// message and a help string.
///
/// `help` becomes part of the message rather than a `relatedInformation`
/// entry. A related-information item needs a location of its own, and a help
/// string has none — it is advice about the diagnostic, not a second place to
/// look.
pub(crate) fn convert(diagnostic: &redox_diag::Diagnostic, source: &str) -> Diagnostic {
  let message = match &diagnostic.help {
    Some(help) => format!("{}\nhelp: {help}", diagnostic.message),
    None => diagnostic.message.clone(),
  };
  Diagnostic {
    range: range_of(source, diagnostic.span),
    // `is_fatal` rather than a match on the variant: plan 38 made `Severity`
    // the thing that knows, and an integer in a protocol is exactly where an
    // off-by-one would be invisible.
    severity: Some(if diagnostic.severity.is_fatal() {
      DiagnosticSeverity::ERROR
    } else {
      DiagnosticSeverity::WARNING
    }),
    code: Some(NumberOrString::String(diagnostic.code.text())),
    code_description: None,
    source: Some("redox".to_string()),
    message,
    related_information: None,
    tags: None,
    data: None,
  }
}
