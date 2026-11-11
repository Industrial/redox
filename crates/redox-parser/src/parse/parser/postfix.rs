use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::method_call_syntax::method_call_syntax;
use crate::parse::parser::optional_chaining::optional_chaining;
use crate::parse::parser::parse_expr::parse_expr;
use crate::parse::parser::record_literal::record_literal;
use redox_ast::{Expr, Ident};
use redox_lexer::TokenKind;

/// One postfix step: an index, a record literal, a field, or `?`.
///
/// `Ok` means the expression grew; `Err` hands it back unchanged so the Pratt
/// loop can move on to infix operators. The nested `Result` is the price of
/// expressing "no step applied" without a sentinel.
#[allow(clippy::result_large_err)]
pub(crate) fn postfix(p: &mut Parser, lhs: Expr) -> Result<Result<Expr, Expr>, ParseError> {
  let start = lhs.span().0;

  // **No paren-call step.** Doc 18 §2.1 makes parentheses grouping only:
  // `f(x)`, `f (x)` and `f x` are one expression, and `application` builds
  // the call from juxtaposition. A postfix call here would take the
  // parenthesised group as an argument list and leave any following argument
  // to form a *second* call — `map (\n -> n * n) xs` emitted
  // `map(|n| n * n)(xs)` until this was removed.
  //
  // And a method call is not a postfix step at all since plan 57: a method is
  // a function whose last argument is its receiver, so `.name` is read in
  // *prefix* position and the receiver arrives through `application`.

  // `e?` — Rust's try operator. A postfix step like any other, and the
  // reason doc 03 §12.5's generated `From` impl has a caller.
  if p.check(&TokenKind::Question) {
    // `?.` is retired — plan 57 leaf 4, `RDX0117`. The token is gone, so this
    // is an **adjacency** check rather than a branch: `x? .name` is two
    // ordinary postfix steps and stays legal, and only the glued spelling is
    // refused, because it used to mean something else and would otherwise
    // silently become a different program that compiles.
    if p
      .peek_at(1)
      .is_some_and(|t| t.kind == TokenKind::Dot && !t.ws_before)
    {
      return Err(optional_chaining((p.peek_span().0, p.peek_span().1 + 1)));
    }
    p.advance();
    return Ok(Ok(Expr::Try {
      inner: Box::new(lhs),
      span: (start, p.prev_span().1),
    }));
  }

  if p.check(&TokenKind::LBracket) {
    p.advance();
    p.skip_layout();
    let index = parse_expr(p, 0)?;
    p.skip_layout();
    p.expect(&TokenKind::RBracket, "expected `]` to close the index")?;
    return Ok(Ok(Expr::Index {
      recv: Box::new(lhs),
      index: Box::new(index),
      span: (start, p.prev_span().1),
    }));
  }

  // Doc 18 §2.4's record literal. A `{` after an expression is unambiguous:
  // Redox has no brace-delimited block in expression position — layout does
  // that job — so nothing else a `{` could open is in the grammar.
  if p.check(&TokenKind::LBrace) {
    return Ok(Ok(record_literal(p, lhs)?));
  }

  if p.check(&TokenKind::Dot) {
    p.advance();

    // `m.0` — a tuple field. Doc 18 §2.4's `newtype Meters = Meters f64`
    // lowers to a tuple struct, and a tuple struct whose field cannot be read
    // is a declaration with no use. The index is an `Ident` carrying digits
    // rather than a variant of its own, because `Expr::Field` already holds
    // exactly "a name after a dot" and the emitter writes it through
    // unchanged.
    if let TokenKind::Int(digits) = &p.peek().kind {
      let name = Ident {
        text: digits.clone(),
        span: p.peek_span(),
      };
      p.advance();
      return Ok(Ok(Expr::Field {
        recv: Box::new(lhs),
        name,
        span: (start, p.prev_span().1),
      }));
    }

    let name = p.ident("expected a field name or a tuple index after `.`")?;
    // A `(` **adjacent** to the name was Rust's method-call syntax, which plan
    // 57 leaf 3 retires — `RDX0115`. Half of the retirement is here and half
    // is in `application`, because §11.2 measured **two** AST shapes for one
    // form: this built `Expr::MethodCall`, and the parenless `x.f a` is an
    // application over the `Expr::Field` built below. Refusing only this one
    // would leave `x.f (a)` re-parsing as the other and emitting the
    // identical `x.f(a)` silently.
    //
    // **Adjacency is what separates the two halves, and testing for any `(`
    // at all refused a legal program.** `.contains p.name (&text)` is the
    // ordinary receiver-last call, and its `(` opens the *enclosing*
    // application's next argument rather than a call on the field — but this
    // runs inside `atom`, which cannot see that there is an enclosing
    // application. Plan 79 found it in `run/usage.sdx`, where the formatter
    // then round-tripped the parentheses off a working program.
    //
    // A *spaced* `x.f (a)` is still refused, and by the other half: it
    // becomes the `Expr::Field` below, and `application` refuses a field as
    // the head of an application. The two halves now split on whitespace,
    // which is exactly how the two spellings differ.
    if p.check(&TokenKind::LParen) && p.peek_span().0 == name.span.1 {
      return Err(method_call_syntax(&name.text, (start, name.span.1)));
    }
    return Ok(Ok(Expr::Field {
      recv: Box::new(lhs),
      name,
      span: (start, p.prev_span().1),
    }));
  }

  Ok(Err(lhs))
}
