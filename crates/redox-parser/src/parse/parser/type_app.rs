use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::TypeApp;
use redox_lexer::TokenKind;

/// Parse a juxtaposed type: `Counter`, `Wrapper t`, `(Pair k v)`.
///
/// Doc 18 §2.1's reading of application, applied to types — so an `instance`
/// target needs no notation of its own. The parentheses in §2.5's
/// `instance Display t => Describe (Wrapper t)` are grouping, exactly as they
/// are in an expression: they separate the target from the class name, and
/// carry no meaning beyond that.
pub(crate) fn type_app(p: &mut Parser) -> Result<TypeApp, ParseError> {
  let start = p.peek_span().0;
  let grouped = p.check(&TokenKind::LParen);
  if grouped {
    p.advance();
  }

  let head = p.ident("expected a type name")?;
  let mut args = Vec::new();
  while let Some(arg) = p.try_ident() {
    args.push(arg);
  }

  if grouped {
    p.expect(&TokenKind::RParen, "expected `)` to close the type")?;
  }

  Ok(TypeApp {
    head,
    args,
    span: (start, p.prev_span().1),
  })
}
