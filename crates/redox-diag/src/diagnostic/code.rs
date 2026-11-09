/// A stable diagnostic code.
///
/// Plan 05 (`docs/10 - 05 - Diagnostics.md`) leaf 1. Every diagnostic is
/// citable, and a code never changes meaning once published.
///
/// Ranges are chosen to respect what `docs/03 - Redox Surface Design.md` §17
/// already published rather than renumbering it:
///
/// | Range | Holds |
/// |---|---|
/// | `RDX0001`–`RDX0039` | lexical errors |
/// | `RDX0040`–`RDX0099` | **reserved** for doc 03 §17's surface codes (plans 19–28) |
/// | `RDX0100`–`RDX0199` | syntactic errors |
/// | `RDX1000`+ | lints, from doc 03 §17's `RDX1001` |
///
/// A code is assigned where its error is **constructed**, never matched from
/// message text at render time — otherwise improving a message would silently
/// change a code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Code {
  // ---- lexical: RDX0001–RDX0039 ----
  /// A tab in leading whitespace. Doc 01 §3.1 permits rejection, and one
  /// branch removes a class of bug that otherwise reports the wrong line.
  TabIndent,
  /// An indentation width matching no enclosing level.
  InconsistentDedent,
  /// A string literal with no closing delimiter.
  UnterminatedString,
  /// A raw string with no closing delimiter.
  UnterminatedRawString,
  /// An interpolation with no closing `}`.
  UnterminatedInterpolation,
  /// A `###` fence with no closing fence.
  UnterminatedFence,
  /// An attribute with no closing `]`.
  UnterminatedAttribute,
  /// A byte or C-string literal with no closing delimiter.
  UnterminatedByteLiteral,
  /// An escape sequence Redox does not define.
  UnknownEscape,
  /// A numeric literal with a radix prefix and no digits, or a bad exponent.
  MalformedNumber,
  /// A character that cannot begin any token.
  UnexpectedCharacter,
  /// An opening delimiter that is never closed.
  UnbalancedDelimiter,
  /// A `'` that opens neither a character literal nor a label.
  BadQuote,

  // ---- syntactic: RDX0100–RDX0199 ----
  /// A token appeared where the grammar required a different one.
  ExpectedToken,
  /// An expression was required and something else appeared.
  ExpectedExpression,
  /// A name was required.
  ExpectedName,
  /// A type was required after `:`.
  ExpectedType,
  /// A declaration body with no statements.
  EmptyBody,
  /// A module-scope binding written doc 01 §4.3's way — `PI = 3.14159`.
  ///
  /// Doc 03 §13.4 corrected that spelling out of existence: Rust forbids
  /// `let` at module scope and requires a type on a `const`. The generic
  /// "expected `::`" is true and unhelpful here, because the author followed
  /// a **published** specification — so this code exists to name the
  /// correction rather than the token.
  ModuleScopeBinding,
  /// `until Some v <- x` — a pattern test under a keyword that negates.
  ///
  /// `until c` lowers to `while !(c)`, and a pattern test cannot be negated:
  /// `while !(let Some(v) = x)` is not Rust. The nearest Rust that compiles,
  /// `while !matches!(x, Some(_))`, **binds nothing** — so it cannot be what
  /// a binding construct meant, and lowering to it would be doc 02 §5.2's
  /// failure mode. Plan 22 §1.3 reports instead.
  PatternUntil,
  /// `...rest` in a pattern — doc 03 §6.5's published spelling, plan 33.
  ///
  /// Doc 18 §7 removed `...` from the language, and both of Rust's own tokens
  /// were already Redox's: `..` to discard and `rest @ ..` to bind. An author
  /// who wrote `...rest` followed a **published** specification, so the
  /// message names the correction rather than the token — the same reason
  /// `ModuleScopeBinding` exists for doc 01 §4.3.
  EllipsisRestPattern,

  // ---- doc 03 §17's published surface codes: RDX0040-RDX0099 ----
  /// `return` inside a `with` callback's tail — doc 03 §7.6, plan 30.
  ///
  /// Spelled `x <- f` when this was written and `with x = f` since doc 30
  /// §3.3; the diagnostic is about the **lowering** — a callback whose body is
  /// the rest of the block — so only the word in its message moved.
  ///
  /// §7.6 calls this a *"disclosed trap, rejected rather than documented
  /// away"*: the tail becomes a **closure**, so a `return` there leaves the
  /// closure and not the enclosing function. The result is valid Rust meaning
  /// something else — doc 02 §5.2's failure mode — and nothing in the source
  /// hints that the block became a closure.
  ///
  /// `?` is deliberately **not** this diagnostic. It propagates out of the
  /// closure, which is the construct's purpose and why §7.6's own example
  /// type-checks.
  ///
  /// **44 is doc 03 §17's own number for this**, which is what the reserved
  /// band is for. The first code to use it, twenty-nine plans after the band
  /// was reserved.
  ReturnInCallbackTail,
}

impl Code {
  /// The numeric code, which is what users cite and what UI tests assert.
  pub fn number(self) -> u16 {
    match self {
      // Lexical.
      Self::TabIndent => 1,
      Self::InconsistentDedent => 2,
      Self::UnterminatedString => 3,
      Self::UnterminatedRawString => 4,
      Self::UnterminatedInterpolation => 5,
      Self::UnterminatedFence => 6,
      Self::UnterminatedAttribute => 7,
      Self::UnterminatedByteLiteral => 8,
      Self::UnknownEscape => 9,
      Self::MalformedNumber => 10,
      Self::UnexpectedCharacter => 11,
      Self::UnbalancedDelimiter => 12,
      Self::BadQuote => 13,
      // Syntactic.
      Self::ExpectedToken => 101,
      Self::ExpectedExpression => 102,
      Self::ExpectedName => 103,
      Self::ExpectedType => 104,
      Self::EmptyBody => 105,
      Self::ModuleScopeBinding => 106,
      Self::PatternUntil => 107,
      Self::EllipsisRestPattern => 108,
      // Doc 03 §17's published assignment, not the next free number.
      Self::ReturnInCallbackTail => 44,
    }
  }

  /// The rendered form, `RDX0001`.
  pub fn text(self) -> String {
    format!("RDX{:04}", self.number())
  }

  /// Every code, for the uniqueness and range checks.
  pub fn all() -> &'static [Code] {
    &[
      Self::TabIndent,
      Self::InconsistentDedent,
      Self::UnterminatedString,
      Self::UnterminatedRawString,
      Self::UnterminatedInterpolation,
      Self::UnterminatedFence,
      Self::UnterminatedAttribute,
      Self::UnterminatedByteLiteral,
      Self::UnknownEscape,
      Self::MalformedNumber,
      Self::UnexpectedCharacter,
      Self::UnbalancedDelimiter,
      Self::BadQuote,
      Self::ExpectedToken,
      Self::ExpectedExpression,
      Self::ExpectedName,
      Self::ExpectedType,
      Self::EmptyBody,
      Self::ModuleScopeBinding,
      Self::PatternUntil,
      Self::EllipsisRestPattern,
      Self::ReturnInCallbackTail,
    ]
  }

  /// Whether this code is a lexical one, for the range check.
  #[cfg(test)]
  fn is_lexical(self) -> bool {
    self.number() <= 39
  }
}

#[cfg(test)]
mod tests {
  use super::Code;
  use std::collections::BTreeSet;

  /// Criterion 2: codes are unique.
  #[test]
  fn every_code_is_unique() {
    let numbers: BTreeSet<u16> = Code::all().iter().map(|c| c.number()).collect();
    assert_eq!(
      numbers.len(),
      Code::all().len(),
      "two codes share a number; a code must never have two meanings"
    );
  }

  /// Criterion 2: codes are inside their declared range.
  #[test]
  fn every_code_is_in_its_declared_range() {
    for code in Code::all() {
      let n = code.number();
      if code.is_lexical() {
        assert!(
          (1..=39).contains(&n),
          "{code:?} is {n}, outside RDX0001-0039"
        );
      } else {
        assert!(
          (40..=99).contains(&n) || (100..=199).contains(&n),
          "{code:?} is {n}, outside RDX0040-0099 and RDX0100-0199"
        );
      }
    }
  }

  /// Criterion 3: nothing collides with doc 03 §17's published assignments.
  ///
  /// Doc 03 published `RDX0042`-`RDX0063` for surface features belonging to
  /// plans 19-28. A code in that band must be **one §17 published, at the
  /// number §17 gave it** — otherwise one number ships two meanings.
  ///
  /// This test used to assert that *no* code was in the band at all, which is
  /// the opposite of what its own comment said and would have blocked every
  /// plan 19-28 surface code. Plan 30 §1.5 found it by being the first plan to
  /// need one: `RDX0044` is §17's assignment for `return` in a `use` tail, and
  /// using it is the reservation working rather than a collision.
  #[test]
  fn every_reserved_code_is_one_doc_03_published() {
    // Doc 03 §17's table, transcribed. A code in the band and absent here is
    // a number taken from a plan that has not claimed it yet.
    const PUBLISHED: &[(u16, &str)] = &[(44, "ReturnInCallbackTail")];

    for code in Code::all() {
      let n = code.number();
      if !(40..=99).contains(&n) {
        continue;
      }
      let name = format!("{code:?}");
      assert!(
        PUBLISHED.iter().any(|(num, who)| *num == n && *who == name),
        "{code:?} is {n}, inside the band doc 03 §17 reserved, and is not \
         the diagnostic §17 assigned that number to"
      );
    }
  }

  #[test]
  fn renders_as_a_four_digit_code() {
    assert_eq!(Code::TabIndent.text(), "RDX0001");
    assert_eq!(Code::ExpectedToken.text(), "RDX0101");
  }
}
