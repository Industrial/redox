use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::atom::atom;
use crate::parse::parser::starts_argument::starts_argument;
use redox_ast::Expr;

/// Parse an application: a head followed by its arguments, juxtaposed.
///
/// Doc 18 §2.1, and the whole of doc 03's call grammar replaced by one
/// sentence: **application binds tighter than every operator and associates
/// left.** Doc 03 decision 49 paid for `f x, y` with what its §20 calls the
/// parser's most intricate region — eight interacting rules, the comma below
/// every operator, a four-member terminator set, six required-parens cases,
/// and `RDX0055` for the nested call it could not read. All of that is this
/// function.
///
/// **`f a b` is one saturated call, not two.** Doc 18 §8 declines currying
/// because Rust functions are not curried, so the arguments accumulate into a
/// single `Call` with two arguments rather than nesting — `add 10 20` is
/// `add(10, 20)`, and `f g x` is `f(g, x)`, which doc 18 §2.1 notes has
/// exactly one reading and so needs no diagnostic.
///
/// An operator token can never begin an argument, which is what lets the loop
/// stop without a terminator set. [`starts_argument`] is plan 03's own
/// predicate, written for statement-position calls and now the rule
/// everywhere — the lifting plan 19 was reserved for, arriving with doc 18
/// instead and needing no adjacency rule, because juxtaposition removed the
/// ambiguity rather than resolving it.
pub(crate) fn application(p: &mut Parser) -> Result<Expr, ParseError> {
  let head = atom(p)?;
  if !starts_argument(&p.peek().kind) {
    return Ok(head);
  }

  let mut args = Vec::new();
  while starts_argument(&p.peek().kind) {
    args.push(atom(p)?);
  }

  let span = (head.span().0, p.prev_span().1);
  Ok(Expr::Call {
    callee: Box::new(head),
    args,
    span,
  })
}
