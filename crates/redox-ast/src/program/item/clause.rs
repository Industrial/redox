use crate::Pattern;
use crate::program::item::clause_body::ClauseBody;
use redox_lexer::{Span, Trivia};

/// One equation of a function.
///
/// Doc 18 §2.3. A function is a signature line plus **one or more** clauses,
/// each a row of patterns and either a body or a list of guards:
///
/// ```text
/// fib :: u64 -> u64            tier :: u64 -> &'static str
/// fib 0 = 0                    tier spend
/// fib 1 = 1                      | spend > 10_000 = "platinum"
/// fib n = fib (n - 1) + …        | otherwise      = "basic"
/// ```
///
/// Doc 03 spelled each clause as a closure with a repeated signature, which is
/// why `RDX0045` existed to catch the clauses disagreeing. With the signature
/// hoisted the disagreement reduces to arity, which `signature_from` reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
  /// One pattern per parameter, in order.
  pub patterns: Vec<Pattern>,
  /// The clause's body: one block, or a guard ladder.
  pub body: ClauseBody,
  pub span: Span,
  pub trailing: Option<Trivia>,
}
