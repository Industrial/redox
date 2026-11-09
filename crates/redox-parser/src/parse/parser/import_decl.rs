use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::import_path::import_path;
use redox_ast::{ImportDecl, Visibility};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 03 §13.1's `import` declaration — plan 14.
///
/// ```text
/// import std::collections::HashMap
/// import std::io::{self, Read, Write}
/// import std::collections::HashMap as Map
/// import crate::models::*
/// export import crate::models::User
/// ```
///
/// Five forms, one grammar: a path and an optional alias. The path is
/// verbatim text (§1.3), and `export import` needs nothing of its own —
/// `visibility` is read before the dispatch, so the re-export form is the
/// marker plus this declaration.
pub(crate) fn import_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
  visibility: Visibility,
) -> Result<ImportDecl, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Import), "expected `import`")?;
  let path = import_path(p)?;

  let alias = if p.check(&TokenKind::Kw(Keyword::As)) {
    p.advance();
    Some(p.ident("expected a name after `as`")?)
  } else {
    None
  };

  Ok(ImportDecl {
    path,
    alias,
    visibility,
    span: (start, p.prev_span().1),
    leading,
  })
}
