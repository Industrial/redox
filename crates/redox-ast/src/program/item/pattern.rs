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
  /// `(a, b)` — doc 03 §6.5's tuple pattern, plan 33.
  ///
  /// Distinct from a parenthesised group, which `pattern` unwraps: a group
  /// holds one pattern and a tuple holds a comma. The comma is the whole
  /// difference, and it is read where the group's `)` would otherwise be.
  Tuple { items: Vec<Pattern>, span: Span },
  /// `[first, .., last]` — doc 03 §6.4's slice pattern, plan 33.
  Slice { items: Vec<Pattern>, span: Span },
  /// `Point { x = 0, y }` — doc 03 §6.3's **named** record pattern, plan 33.
  ///
  /// `=` and not §6.3's `:`, matching doc 18 §2.4's record *literal* — §5.2
  /// left `:` no job inside braces at all, and two spellings of one shape
  /// differing only by position is what §5.3 exists to remove. Lowers to
  /// Rust's `Point { x: 0, y }`.
  ///
  /// `rest` is `Point { x, .. }`. Doc 03 §6.3's *anonymous* form —
  /// `{active: true}` with no path — is **deleted** by doc 18 §2.3 along with
  /// `RDX0042` and `RDX0043`: it lowered to a field-access conjunction, which
  /// is shorter written directly.
  Record {
    path: String,
    fields: Vec<crate::RecordPatternField>,
    rest: bool,
    span: Span,
  },
  /// `whole @ Move x y` — doc 03 §6.4's as-pattern, plan 33.
  ///
  /// `@`'s **one remaining meaning**: doc 18 §4.3 dropped `@name` and
  /// `@@name` for `self.name` and `Self::name` and says that leaving `@` with
  /// exactly this is *"a cost worth paying twice over"*. Haskell's spelling
  /// too.
  As {
    name: crate::Ident,
    inner: Box<Pattern>,
    span: Span,
  },
  /// `..` in a slice or tuple pattern — doc 03 §6.5's rest, plan 33.
  ///
  /// Spelled `..` and not doc 03's `...rest`, because doc 18 §7 removed `...`
  /// from the language and both of Rust's tokens were already Redox's. Plan 33
  /// §1.3: one fewer token and one fewer lowering.
  Rest(Span),
  /// `rest @ ..` — a rest that binds, plan 33.
  ///
  /// Not an [`As`](Self::As) over a [`Rest`](Self::Rest), because Rust's
  /// grammar admits `name @ ..` only in a slice and the two emit the same text
  /// either way. One node keeps the parser from accepting `name @ ..` where
  /// Rust does not — rustc would reject it, but a node that cannot be built
  /// wrongly is better than a diagnostic.
  NamedRest { name: crate::Ident, span: Span },
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
      | Self::Apply { span, .. }
      | Self::Tuple { span, .. }
      | Self::Slice { span, .. }
      | Self::Record { span, .. }
      | Self::As { span, .. }
      | Self::Rest(span)
      | Self::NamedRest { span, .. } => *span,
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
    // **Only a bare name or `_`.** A destructuring pattern — `(a, b)`,
    // `[a, b]`, `Point { x }` — is treated as refutable even where it matches
    // everything, and plan 33 §10 records why: this answer decides whether an
    // equation's row needs a `match` *and* whether the row's bindings are in
    // scope in the body.
    //
    // `signature_from` lends a parameter its name only for a `Binding`, so a
    // tuple pattern's parameter is `__rdx_a0` and `a` and `b` come from the
    // `match` arm. Calling such a pattern irrefutable suppressed the `match`
    // and left the body referring to names nothing bound — which rustc caught
    // as `cannot find value \`a\``, against a line the author wrote.
    //
    // The cost is a `match` where a `let` would have done. rustc accepts one
    // with a single irrefutable arm, and one rule that cannot produce an
    // unbound name is worth more than a second lowering that could.
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
      // **A one-element tuple keeps Rust's trailing comma.** `(a)` is a
      // parenthesised group and `(a,)` is a tuple, so dropping the comma
      // turns a destructuring of a `(T,)` into a binding of the whole value —
      // which rustc catches, but only because the types differ. Caught here by
      // asserting the group and the tuple side by side.
      Self::Tuple { items, .. } => {
        let inner = items
          .iter()
          .map(Self::rust_text)
          .collect::<Vec<_>>()
          .join(", ");
        if items.len() == 1 {
          format!("({inner},)")
        } else {
          format!("({inner})")
        }
      }
      Self::Slice { items, .. } => format!(
        "[{}]",
        items
          .iter()
          .map(Self::rust_text)
          .collect::<Vec<_>>()
          .join(", ")
      ),
      // `=` becomes `:`, which is the one character this form changes.
      Self::Record {
        path, fields, rest, ..
      } => {
        let mut parts: Vec<String> = fields
          .iter()
          .map(crate::RecordPatternField::rust_text)
          .collect();
        if *rest {
          parts.push("..".to_string());
        }
        format!("{path} {{ {} }}", parts.join(", "))
      }
      Self::As { name, inner, .. } => format!("{} @ {}", name.text, inner.rust_text()),
      Self::Rest(_) => "..".to_string(),
      Self::NamedRest { name, .. } => format!("{} @ ..", name.text),
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

impl Pattern {
  /// Every name this pattern binds, in source order.
  ///
  /// Written by plan 62 leaf 3 as `emit::ascribed_arm::collect` and moved here
  /// by plan 61, which needs the same answer in the parser — for the
  /// parameter-shadow refusal and for the `where` dependency graph. A
  /// recursive walk over eleven forms is not a thing to keep two copies of,
  /// and unlike the five cases plan 34 §10.31 describes this really is **one
  /// question with one answer in every position**: a pattern's binders do not
  /// depend on where it stands.
  ///
  /// A *testing* pattern — a literal, a range, a wildcard, a rest — binds
  /// nothing and contributes nothing, which is why the catch-all arm is
  /// correct rather than lazy: those are the variants that have no name to
  /// give.
  pub fn binders(&self) -> Vec<String> {
    let mut names = Vec::new();
    self.collect_binders(&mut names);
    names
  }

  fn collect_binders(&self, into: &mut Vec<String>) {
    match self {
      Pattern::Binding(name) | Pattern::NamedRest { name, .. } => into.push(name.text.clone()),
      Pattern::As { name, inner, .. } => {
        into.push(name.text.clone());
        inner.collect_binders(into);
      }
      Pattern::Apply { args, .. } => {
        for arg in args {
          arg.collect_binders(into);
        }
      }
      Pattern::Tuple { items, .. } | Pattern::Slice { items, .. } => {
        for item in items {
          item.collect_binders(into);
        }
      }
      Pattern::Or(items) => {
        for item in items {
          item.collect_binders(into);
        }
      }
      Pattern::Record { fields, .. } => {
        for field in fields {
          match &field.pattern {
            Some(pattern) => pattern.collect_binders(into),
            None => into.push(field.name.text.clone()),
          }
        }
      }
      Pattern::Literal { .. } | Pattern::Wildcard(_) | Pattern::Range { .. } | Pattern::Rest(_) => {
      }
    }
  }
}
