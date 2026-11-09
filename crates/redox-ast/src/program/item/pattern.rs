use redox_lexer::Span;

/// A pattern in an equation's binder position.
///
/// Doc 18 §2.3: with the signature hoisted onto its own line, an equation's
/// head is a **row of patterns** rather than a repeated type annotation —
/// `fib 0 = 0` and `fib n = …` are two clauses of one function.
///
/// Deliberately small. Doc 18 §2.3's own examples use exactly these three
/// shapes, and the full pattern grammar — slices, tuples, struct patterns,
/// `or`, `@` — is doc 04's plan 33 and needs the `data` declarations of §2.4
/// to be worth testing against. A pattern form that cannot be constructed yet
/// would be code with no way to exercise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
  /// A name, which binds. `fib n`.
  Binding(crate::Ident),
  /// A literal, which tests. `fib 0`, `greet "hi"`, `f true`.
  ///
  /// Carried as its verbatim source text, the same discipline doc 03 §14.1
  /// applies to types: the emitter passes it through and Redox never has to
  /// agree with rustc about what a literal means.
  Literal { text: String, span: Span },
  /// `_`, which matches and discards.
  Wildcard(Span),
}

impl Pattern {
  pub fn span(&self) -> Span {
    match self {
      Self::Binding(ident) => ident.span,
      Self::Literal { span, .. } | Self::Wildcard(span) => *span,
    }
  }

  /// Whether this pattern is irrefutable — it matches anything.
  ///
  /// A clause row of only irrefutable patterns needs no `match`, which is
  /// what keeps the common one-clause function emitting exactly the Rust it
  /// emitted before doc 18 §2.3 existed.
  pub fn is_irrefutable(&self) -> bool {
    matches!(self, Self::Binding(_) | Self::Wildcard(_))
  }

  /// The text to write in a `match` arm.
  pub fn rust_text(&self) -> String {
    match self {
      Self::Binding(ident) => ident.text.clone(),
      Self::Literal { text, .. } => text.clone(),
      Self::Wildcard(_) => "_".to_string(),
    }
  }
}
