use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::atom::atom;
use crate::parse::parser::method_call_syntax::method_call_syntax;
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
/// **`f a b` is one `Call` with two arguments, not two nested calls.** The
/// arguments accumulate into a single node — `add 10 20` is `add(10, 20)`, and
/// `f g x` is `f(g, x)`, which doc 18 §2.1 notes has exactly one reading and so
/// needs no diagnostic.
///
/// That paragraph used to read *"doc 18 §8 declines currying because Rust
/// functions are not curried"*, and plan 56 (`docs/50 - 56 - Currying.md`)
/// retired the decline — so it is corrected here rather than left to read as
/// false, which plan 56 §11.7 asked for. What changed is **not** this
/// function: the tree is still flat, and the parser still counts arity
/// nowhere. An under-applied call becomes a closure in the *emitter*, which
/// consults the arity its signatures already state, and it is in the emitter
/// precisely so that `redox fmt` keeps printing the author's `add 1` as
/// `add 1`.
///
/// An operator token can never begin an argument, which is what lets the loop
/// stop without a terminator set. [`starts_argument`] is plan 03's own
/// predicate, written for statement-position calls and now the rule
/// everywhere — the lifting plan 19 was reserved for, arriving with doc 18
/// instead and needing no adjacency rule, because juxtaposition removed the
/// ambiguity rather than resolving it.
pub(crate) fn application(p: &mut Parser) -> Result<Expr, ParseError> {
  let head = atom(p)?;
  // A head that ended with a `Dedent` closed an indented block, so whatever
  // follows is on a *later line* and cannot be its argument. Doc 18 §2.3's
  // `case … of` is the first expression with layout of its own, and without
  // this `label age = case age of …` swallowed the next item's name as an
  // argument — the application loop reads tokens, and the token after a
  // dedent looks like any other.
  if p.just_dedented() || !starts_argument(&p.peek().kind) {
    return Ok(head);
  }

  // The parenless half of plan 57 leaf 3's retirement — `RDX0115`. §11.2:
  // `x.f a` is an application over an `Expr::Field`, because there is no
  // paren-call step above, so `postfix`'s refusal of `x.f (a)` would
  // otherwise be escaped by dropping the parentheses. A `.` is field access
  // and a field is not applied; the call is `.f a x`.
  if let Expr::Field { name, .. } = &head {
    return Err(method_call_syntax(&name.text, head.span()));
  }

  let mut args = Vec::new();
  while starts_argument(&p.peek().kind) {
    args.push(atom(p)?);
  }

  let span = (head.span().0, p.prev_span().1);
  Ok(Expr::Call {
    callee: Box::new(head),
    args,
    is_dollar: false,
    span,
  })
}
