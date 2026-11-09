use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Ident;
use redox_lexer::TokenKind;

/// Parse an equation's binders: the names between the head and the `=`.
///
/// Doc 18 §2.3. `load p =` binds one; `fib n =` binds one; `main =` binds
/// none. Each is a plain lowercase name, and the signature line (doc 18 §2.2)
/// supplies its type — which is the whole point of hoisting the signature: a
/// clause becomes a pattern row instead of a repeated type annotation, and
/// `RDX0045` reduces from "the clauses disagree" to "the arity is wrong".
///
/// Patterns in binder position — `fib 0 = 0` — are doc 18 §2.3's full form and
/// are **not** parsed here: doc 18 §11.1 converts the signature line and the
/// equation head, and multi-clause pattern matching is specified for a later
/// plan. A literal where a binder belongs is reported as wanting a name, which
/// names the right thing even though the eventual answer is to accept it.
pub(crate) fn binders(p: &mut Parser) -> Result<Vec<Ident>, ParseError> {
  let mut found = Vec::new();
  while !p.check(&TokenKind::Eq) && !p.at_eof() {
    found.push(p.ident("expected a parameter name or `=`")?);
  }
  Ok(found)
}
