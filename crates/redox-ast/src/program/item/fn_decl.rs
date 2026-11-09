use crate::{Clause, Ident, Signature};
use redox_lexer::{Span, Trivia};

/// A signature line and the equations that define it.
///
/// Doc 18 §§2.2–2.3:
///
/// ```text
/// fib :: u64 -> u64        -- the signature
/// fib 0 = 0                -- clause 1
/// fib n = fib (n - 1)      -- clause 2
/// ```
///
/// **One or more clauses.** A single clause whose patterns are all
/// irrefutable is the ordinary function doc 03 had, and lowers to exactly the
/// Rust it lowered to before — which is what keeps doc 18 §1's rule true.
/// More than one clause, or a refutable pattern, lowers to one `fn` whose body
/// is a `match`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
  pub name: Ident,
  pub sig: Signature,
  pub clauses: Vec<Clause>,
  pub span: Span,
  pub leading: Vec<Trivia>,
  pub trailing: Option<Trivia>,
}
