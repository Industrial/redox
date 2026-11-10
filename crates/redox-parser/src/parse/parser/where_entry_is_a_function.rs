use crate::parse::parser::Parser;
use redox_lexer::TokenKind;

/// Whether the `where` entry at the cursor is a **function declaration**
/// rather than a binding — plan 61 leaf 0.
///
/// ```text
/// go : u64 -> u64     a signature line  — a function
/// go 0 = 0            an equation       — a function
/// scaled = base * 2   a binding
/// (a, b) = p          a binding
/// ```
///
/// # Nothing is invented, and that is §11.1a's whole argument
///
/// Both discriminators already exist and are already asserted:
///
/// - a **signature line** is `Ident` then `:` with no depth-zero `=` before
///   the line break, which is plan 52's [`Parser::reaches_a_binding_eq`]
///   asked the other way round — the test that tells `(a, b) = p` from the
///   expression `(a, b)`;
/// - an **equation** is `Ident` then a pattern then `=`, which is a name
///   whose next token is none of `=`, `:` or `{` and which still reaches a
///   depth-zero `=`. That is exactly the gap
///   [`Parser::looks_like_a_binding`] leaves: it tests `peek_at(1) ∈ {Eq,
///   Colon, LBrace}` and never asks the question it already has an answer
///   for, which is why `go 0` parsed as a call and the `=` was unexpected.
///
/// # An equation with no signature is the same diagnostic as at module scope
///
/// Reading an equation as a function entry is what gives it that: a bare
/// `go 0 = 0` routes to [`Parser::function`], whose `signature_head` reports
/// *"expected `:` after the name, to begin the signature"* — the message a
/// module-scope `f n = n * 2` already gets. Detecting only the signature line
/// would have left it as `RDX0101` *"expected a line break after the
/// statement"*, which names the symptom and not the omission.
///
/// A signature is **required**, and that is consistent rather than an
/// exception: it is already required at module scope, and it is precisely
/// what makes a nested `fn`'s parameter and return types available without
/// the inference Redox does not have and may never grow.
pub(crate) fn where_entry_is_a_function(p: &Parser) -> bool {
  if !matches!(p.peek().kind, TokenKind::Ident(_)) {
    return false;
  }
  match p.peek_at(1).map(|t| &t.kind) {
    // A signature line, or plan 52's ascription — told apart by whether a
    // depth-zero `=` follows, which is the one question that separates them.
    Some(TokenKind::Colon) => !p.reaches_a_binding_eq(),
    // `name = …` is a binding and `name { … } = …` is a destructuring one;
    // neither can be a function.
    Some(TokenKind::Eq) | Some(TokenKind::LBrace) => false,
    // A name followed by anything else, with a depth-zero `=` still to come:
    // the tokens between them are the equation's pattern row.
    _ => p.reaches_a_binding_eq(),
  }
}
