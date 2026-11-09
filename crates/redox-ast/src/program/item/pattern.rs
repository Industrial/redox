use redox_lexer::Span;

/// A pattern in an equation's binder position.
///
/// Doc 18 §2.3: with the signature hoisted onto its own line, an equation's
/// head is a **row of patterns** rather than a repeated type annotation —
/// `fib 0 = 0` and `fib n = …` are two clauses of one function.
///
/// Deliberately small. The full pattern grammar — slices, tuples, struct
/// patterns, `@` — is doc 04's plan 33 and needs the `data` declarations of
/// §2.4 to be worth testing against. A pattern form that cannot be
/// constructed yet would be code with no way to exercise it.
///
/// Alternation and ranges are here because doc 18 §2.3's `case` example uses
/// them, and because §5.4's rule *depends* on alternation being spelled `or`:
/// the first `|` after a pattern is the guard, which is only unambiguous
/// while `|` is not also the alternation bar.
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
  /// `18..=64` and `18..64`, carried as verbatim endpoint text.
  Range {
    lo: String,
    hi: String,
    /// `..=` rather than `..`.
    inclusive: bool,
    span: Span,
  },
  /// A constructor applied to sub-patterns — `Some v`, `Ok raw`,
  /// `Move x _`, `Event::Close d` — plan 22.
  ///
  /// **No row owned this.** Row 21 shipped literal, range, alternation and
  /// *nullary* constructor patterns, and row 21's remainder is named "record,
  /// slice and as-patterns", which this is not: it is the tuple-variant form,
  /// and it is the one pattern every use of `Option` and `Result` needs. Plan
  /// 22 §10 records why it landed there — `if Some v <- x` cannot be written
  /// without it, so the row that introduced the position had to introduce the
  /// pattern.
  ///
  /// The constructor is carried as **verbatim path text**, the discipline doc
  /// 03 §14.1 applies to types and this enum already applies to literals: the
  /// emitter passes it through and Redox never has to agree with rustc about
  /// what `Event::Close` resolves to.
  ///
  /// Doc 18 §5.3's case rule is what makes this decidable from source text
  /// alone — an uppercase head is a constructor, a lowercase one binds — so
  /// no name resolution is involved, which is doc 03 §2's first admission
  /// test.
  Apply {
    path: String,
    args: Vec<Pattern>,
    span: Span,
  },
  /// `Teen or PreTeen`, doc 03's alternation, lowering to Rust's `|`.
  ///
  /// Keeps the `or` **keyword** rather than Rust's bar, which doc 18 §5.4
  /// says outright is the price of the guard rule: `Teen or PreTeen -> …`
  /// cannot be misread as a guard, and `Teen | PreTeen -> …` could.
  Or(Vec<Pattern>),
}

impl Pattern {
  pub fn span(&self) -> Span {
    match self {
      Self::Binding(ident) => ident.span,
      Self::Literal { span, .. }
      | Self::Wildcard(span)
      | Self::Range { span, .. }
      | Self::Apply { span, .. } => *span,
      // An alternation's span is its ends', and it is never empty: the
      // parser builds one only after reading a first pattern.
      Self::Or(alts) => (
        alts.first().map_or(0, |a| a.span().0),
        alts.last().map_or(0, |a| a.span().1),
      ),
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
      Self::Range {
        lo, hi, inclusive, ..
      } => format!("{lo}{}{hi}", if *inclusive { "..=" } else { ".." }),
      // `Some v` is `Some(v)`. A nullary constructor never reaches here: it
      // parses as a `Binding`, which emits the bare path — and that is right
      // either way, because doc 18 §2.4 emits `use T::*;` beside the enum.
      Self::Apply { path, args, .. } => format!(
        "{path}({})",
        args
          .iter()
          .map(Self::rust_text)
          .collect::<Vec<_>>()
          .join(", ")
      ),
      Self::Or(alts) => alts
        .iter()
        .map(Self::rust_text)
        .collect::<Vec<_>>()
        .join(" | "),
    }
  }
}
