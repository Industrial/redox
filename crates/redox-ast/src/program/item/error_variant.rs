use crate::{Ident, TypeExpr};
use redox_lexer::Span;

/// One row of an `errors` declaration — doc 03 §12.5.
///
/// ```text
/// Missing String            a variant and its payload
/// Invalid String u32        several fields, juxtaposed (doc 18 §2.1)
/// Closed                    no payload
/// from io::Error as Io      a variant that also gets a `From` impl
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorVariant {
  pub name: Ident,
  /// The payload types, in order.
  pub fields: Vec<TypeExpr>,
  /// Set when the row was written `from T as Name`, which additionally
  /// generates `impl From<T> for TheError`.
  ///
  /// The type is also the variant's single field, so `fields` carries it too
  /// — one place for "what the variant holds" rather than two, which keeps
  /// the enum emitter from having to ask.
  pub from: bool,
  /// The `Display` message, with `#{0}` holes naming the fields by position.
  ///
  /// Kept as written, holes included. Rewriting it at parse time would make
  /// the formatter unable to print back what the author typed, and the
  /// rewrite is the emitter's business.
  pub message: String,
  pub span: Span,
}
