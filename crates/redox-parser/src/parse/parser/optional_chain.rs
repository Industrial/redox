use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Expr;
use redox_lexer::TokenKind;

/// Read a `?.` chain's segments, the cursor on the first `?.` — doc 01 §13.1,
/// plan 28.
///
/// ```text
/// user?.profile?.name   →   OptionalChain { head: user, segments: [profile, name] }
/// ```
///
/// **One chain node, not one per link**, because the lowering is one closure
/// over the whole run: `(|| Some(user.as_ref()?.profile.as_ref()?.name.as_ref()))()`.
/// Nesting a node per `?.` would produce a closure per link, which is three
/// closures where one does, and each inner one would wrap its result in a
/// second `Option` — doc 01 §13.1 requires *not* flattening, but it does not
/// ask for extra nesting either.
///
/// `?.` is **one token** (`QuestionDot`), which plan 15 §10.1 records: it is
/// why `x?.await` is a parse error and `x? .await` is two postfixes. This
/// reads the token it is given and re-decides nothing.
pub(crate) fn optional_chain(p: &mut Parser, head: Expr) -> Result<Expr, ParseError> {
  let start = head.span().0;
  let mut segments = Vec::new();

  while p.check(&TokenKind::QuestionDot) {
    p.advance();
    segments.push(p.ident("expected a field name after `?.`")?.text);
  }

  Ok(Expr::OptionalChain {
    head: Box::new(head),
    segments,
    span: (start, p.prev_span().1),
  })
}
