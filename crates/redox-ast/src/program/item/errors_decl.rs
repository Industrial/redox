use crate::Ident;
use crate::program::item::error_variant::ErrorVariant;
use redox_lexer::{Span, Trivia};

/// An `errors` declaration — doc 03 §12.5, kept unchanged by doc 18 §8.
///
/// ```text
/// errors ConfigError                    #[derive(Debug)]
///   Missing String  "missing: #{0}"     pub enum ConfigError {
///   from io::Error as Io "io: #{0}"  →      Missing(String),
///                                           Io(io::Error),
///                                       }
///                                       // + Display, Error, From
/// ```
///
/// Generates the enum together with its `Display`, `Error` and `From`
/// implementations — "the boilerplate every Rust project either writes by
/// hand or pulls a derive crate for", naming nothing outside `std`.
///
/// Doc 18 §8 lists this among the four constructs **kept** despite having no
/// Haskell analogue, because deleting it would cost ergonomics with nothing
/// to show. The field list is juxtaposed rather than parenthesised, which is
/// the one thing doc 18 changes about it (§2.1).
///
/// Doc 03 §12.5 is careful that this "generates items for a user-declared
/// type, as `derive` does; it is not runtime scaffolding in the sense of
/// §7.3" — which is what admits it at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorsDecl {
  pub name: Ident,
  pub variants: Vec<ErrorVariant>,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
