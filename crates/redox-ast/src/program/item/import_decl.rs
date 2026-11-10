use crate::Ident;

use crate::program::item::visibility::Visibility;

use redox_lexer::{Span, Trivia};

/// An `import` declaration — doc 03 §13.1, plan 14.
///
/// ```text
/// import std::collections::HashMap          use std::collections::HashMap;
/// import std::io::{self, Read, Write}       use std::io::{self, Read, Write};
/// import std::collections::HashMap as Map   use std::collections::HashMap as Map;
/// import crate::models::*               →   use crate::models::*;
/// export import crate::models::User         pub use crate::models::User;
/// ```
///
/// Doc 03 §13.1 corrected doc 01 §22's three inconsistent path conventions —
/// `::` in expressions, `.` in module paths, `.` in imports — to `::`
/// everywhere, *"so `.` unambiguously means field or method access"*. The
/// `::` half shipped with doc 18 §5.2; this is the half named second.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDecl {
  /// The path, as **verbatim source text** including any `{…}` tree or `*`.
  ///
  /// Doc 03 §14.1's discipline, applied where it applies equally: the
  /// lowering is `use` plus this text, so a nested tree Redox parsed would be
  /// a nested tree Redox could lag behind.
  pub path: String,
  /// `as Map` — the one token understood rather than copied.
  pub alias: Option<Ident>,
  /// `export import …` becomes `pub use …`, which is doc 03 §13.1's
  /// re-export and needs no form of its own.
  pub visibility: Visibility,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
