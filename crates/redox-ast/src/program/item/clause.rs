use crate::{Block, Pattern};
use redox_lexer::{Span, Trivia};

/// One equation of a function.
///
/// Doc 18 §2.3. A function is a signature line plus **one or more** clauses,
/// each a row of patterns and a body:
///
/// ```text
/// fib :: u64 -> u64
/// fib 0 = 0
/// fib 1 = 1
/// fib n = fib (n - 1) + fib (n - 2)
/// ```
///
/// Doc 03 spelled each clause as a closure with a repeated signature, which is
/// why `RDX0045` existed to catch the clauses disagreeing. With the signature
/// hoisted the disagreement reduces to arity, which `signature_from` reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
  /// One pattern per parameter, in order.
  pub patterns: Vec<Pattern>,
  pub body: Block,
  pub span: Span,
  pub trailing: Option<Trivia>,
}
