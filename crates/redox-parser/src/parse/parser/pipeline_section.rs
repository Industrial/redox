use crate::ParseError;
use redox_diag::Code;
use redox_lexer::Span;

/// Refuse doc 18 §4's `(|> name)` method reference — `RDX0116`, plan 57 leaf
/// 7.
///
/// A bare `.name` is a method awaiting its receiver and lowers to the same
/// `|__rdx_1| __rdx_1.name()` this spelling emitted, so the two were one
/// construct with two surfaces. Plan 65's rule decides which row removes it:
/// **whoever builds the replacement retires what it replaces**, so there is
/// never a window in which neither spelling works.
///
/// It also removes a reinterpretation plan 58 would otherwise inherit. Under
/// reverse application `(|> f)` would read as `\x -> f x`, which is just `f` —
/// a section that sections nothing.
///
/// `(.name)` is **not** refused: it is the field selector, doc 18 §4 retired
/// `&:name` to get it, and a `)` immediately after the name is what tells the
/// two apart.
pub(crate) fn pipeline_section(name: &str, span: Span) -> ParseError {
  ParseError::with_help(
    Code::PipelineSection,
    "the `(|> name)` method reference is retired",
    format!(
      "write `.{name}` — a method is a function awaiting its receiver, and it lowers to the \
       closure this spelled. `(.{name})` still selects a *field* (plan 57 §1.3)"
    ),
    span,
  )
}
