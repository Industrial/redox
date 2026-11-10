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
  /// `RDX0109` — the input nests brackets deeper than the parser's stack
  /// can hold.
  ///
  /// Found by plan 42's fuzz sweep, which is the first thing that ever
  /// handed the parser three hundred nested parentheses. A recursive-descent
  /// parser with no limit overflows its stack and **aborts the process** —
  /// not a diagnostic, not a panic that can be caught, an abort. Plan 40's
  /// language server runs the parser on every keystroke, so that is a crash
  /// in front of a user who typed a bracket too many.
  ///
  /// Measured rather than guessed: 24 levels parse and 32 abort, in a debug
  /// build. The limit is 16, which is well below the first failure and far
  /// above anything a human writes — the deepest nesting in the whole corpus
  /// is a handful.
  NestingTooDeep,

  /// A `::` with whitespace before it — plan 52 criterion 2.
  ///
  /// `::` joins a path, and a path is written tight in both Rust and Redox.
  /// Before plan 52 a *spaced* `::` ascribed a type, which made this the one
  /// place in the language where a space changed meaning; plan 52 gave
  /// ascription its own token and retired the rule.
  ///
  /// What it did not retire was the old spelling in anybody's fingers. Left
  /// alone, `y = x :: u64` is consumed as the path `x::u64` — `u64` is a
  /// perfectly good segment — and the first sign of trouble is rustc
  /// reporting an unresolved path against a line the author did not write.
  /// That is the exact failure mode doc 18 §5.2's whitespace rule existed to
  /// prevent, so retiring the rule has to come with this diagnostic rather
  /// than with silence.
  ///
  /// Detected from spans alone: a gap between the previous token's end and
  /// the `::`'s start is whitespace. The rule is positional, not a guess
  /// about what the author meant.
  SpacedPathSeparator,

  /// `RDX0111` — a `newtype` that is not one constructor over one field.
  ///
  /// Plan 53 leaf 2. `newtype` is `data` narrowed to Haskell's shape, and
  /// before this code the narrowing was nowhere: `data_decl.rs` set
  /// `is_newtype` and never read it again, so `newtype Bad = Bad f64 f64`
  /// emitted a two-field tuple struct and `newtype Multi = A | B` emitted an
  /// **enum**, both silently and both exit 0. A keyword that constrains
  /// nothing is doc 02 §5.2's failure mode spelled as vocabulary.
  ///
  /// **One code, three messages** — no field, two-or-more fields,
  /// two-or-more constructors. A code names a *rule*, and there is one rule
  /// here: the shape `newtype` promises. The remedy is also one word in all
  /// three cases, `data`, so three codes would publish three rules a reader
  /// then has to work out are the same one. `UnterminatedString` and
  /// `UnterminatedRawString` are two codes because they are two constructs
  /// with two different closing delimiters; these are three readings of one
  /// arity.
  ///
  /// **rustc is not a backstop for most of it.** Plan 53 §6 claimed a bad
  /// payload was *"rejected by rustc, not by Redox"*; §11.3 measured it and
  /// only the multi-field case gets `E0690` —
  /// `#[repr(transparent)] pub struct Unit;` compiles clean. So for
  /// `newtype Foo = Foo` this check is the only thing between the author and
  /// a keyword that quietly meant nothing.
  NewtypeShape,

  /// `RDX0112` — a `test` declaration inside a body.
  ///
  /// Plan 55 leaf 6, and the one carve-out from §1.4's *everything in a body
  /// is an expression*. It is refused because **rustc does not collect a
  /// nested `#[test]`**: the `unnameable_test_items` lint warns
  /// `cannot test inner items`, the harness runs one test where two were
  /// written, and the build exits 0 either way (§1.5, measured).
  ///
  /// So lowering a local `test` would ship a construct that compiles and
  /// then silently does nothing — worse than one that is refused, because a
  /// refusal costs a minute and a test that never runs costs whatever it was
  /// guarding. §9 decision 2 declined the alternative, hoisting to module
  /// scope, on the ground that AGENTS.md already puts tests elsewhere.
  ///
  /// **Not a widening of the ground.** A declaration keyword in a body
  /// already failed, as `RDX0102` *"expected an expression"* from `prefix`'s
  /// default arm — twice over, because the indented body line raised a second
  /// one. This names the rule instead of the token, which is the same reason
  /// [`Self::ModuleScopeBinding`] and [`Self::EllipsisRestPattern`] exist.
  ///
  /// The number is the one plan 55 leaf 1 was briefed to take and did not:
  /// §10.2 found the position rule it was meant for had no trigger.
  LocalTest,

  /// `RDX0113` — a `()`-returning function whose body does not end in `()`.
  ///
  /// Plan 55 §1.6's rule, and §9 decision 3 priced it before it was built:
  /// *"The cost is one line per `()`-returning function and it is paid
  /// visibly."* What it buys is the removal of positional silence — a body
  /// whose last line is a loop, a binding or an assignment has **no value**,
  /// and today it becomes `()` by arriving last rather than by saying so.
  ///
  /// **rustc does catch the mistake, at the call site rather than the
  /// definition.** A function that falls off the end of a `for` type-checks as
  /// `()` where it stands; the complaint arrives wherever somebody tried to
  /// use the value, which is a different file as often as not. This code moves
  /// it to the line that is wrong, which is doc 05 §4's rule and the whole
  /// argument for Redox naming anything rustc also names.
  ///
  /// **Decidable without a type checker**, which is why it may exist at all
  /// (AGENTS.md: *every check rustc can make, rustc makes* — and this is one
  /// rustc makes elsewhere). Two syntactic facts settle it: the **declared**
  /// return type, which doc 18 §2.2 makes mandatory, and whether the body's
  /// last element is the literal `()`. Nothing is inferred — `print x` is not
  /// *known* to be unit, and does not have to be, because the signature
  /// already said so.
  ///
  /// **Scoped to a declared `()`, and not to "the last element has no
  /// value".** The wider reading was measured and declined twice over: for a
  /// non-unit return type a valueless tail is already `E0308` *at the
  /// definition*, with a better message than a suggestion to write `()` —
  /// which is the wrong remedy there — and `Expr::is_statement_form()` is not
  /// the predicate it looks like, since `return`, `break` and `continue` are
  /// `!` and a block ending in one has no value *and no defect*. Those three
  /// are therefore accepted as a unit body's last element: they state the
  /// function's value outright, and a `()` after one is unreachable code.
  UnitTail,

  // ---- lints: RDX1000+ ----
  /// `unwrap` or `expect` used — doc 03 §12.7, plan 38.
  ///
  /// The **first lint**, and the first diagnostic in the project that is not
  /// an error: §12.7 makes these *"lintable rather than prohibited"*, so the
  /// build succeeds and the author decides. Suppressible per line with
  /// `-- redox: allow unwrap`.
  ///
  /// Detected lexically — a `.` then the name — so a user's own method called
  /// `unwrap` warns too. Plan 38 §1.5 takes that trade: the warning is
  /// suppressible, and the alternative needs a type checker AGENTS.md forbids.
  UnwrapUsed,
  /// `RDX1002` — `RDX1001` suppressed by the **comment** pragma, which is
  /// deprecated in favour of `#[redox::allow(unwrap)]`.
  ///
  /// Plan 53 leaf 5. Plan 38 §1.3 put the pragma in a comment on the
  /// argument that a comment *"reaches nothing downstream"*; plan 53 §1.3
  /// reverses it on evidence plan 38 did not have — plan 48 found `redox fmt`
  /// **deleting** four doc comments from a `.sdx`, so a comment is not safe
  /// *upstream*. A suppression a formatter run can silently remove turns a
  /// deliberate pragma into a surprise `RDX1001`, or removes the record of
  /// why a panic was acceptable.
  ///
  /// A **warning about a warning's spelling**, so it is in the lint band and
  /// not the syntactic one: the comment still suppresses, the build still
  /// succeeds, and the author is told what to write instead. Removal is plan
  /// 64's, with the rest of the comment-carried semantics.
  CommentAllowPragma,
  /// `RDX1003` — a bare `=` rebinding a name already bound in the **same
  /// statement list**, which binds a second name rather than reassigning.
  ///
  /// Plan 54 leaf 3, and the one defect §1.1 measured that nothing caught:
  /// of sixteen constructs that refuse or reinterpret, fifteen were caught by
  /// a Redox diagnostic or by rustc, and `i = j` after `mut i = 0` was caught
  /// by neither. It emits `let i = j;`, which compiles, and the only signal is
  /// rustc's `unused variable` on the *outer* binding — a warning pointing at
  /// the declaration rather than at the line that went wrong.
  ///
  /// **A warning, because shadowing is legal Rust and sometimes meant.**
  /// Doc 03 §1.4 keeps `=` binding on purpose, so this names a suspicion and
  /// not a rule, and `#[redox::allow(shadow)]` is how an author says it was
  /// deliberate.
  ///
  /// **One statement list, deliberately.** Plan 54 decision 4: the lint sees
  /// a list and never a scope, so a nested block rebinding an outer name is
  /// silent. Widening it is the first step toward the scope table AGENTS.md
  /// forbids Redox growing and §1.3 refused on exactly that ground — which is
  /// why the boundary is asserted by criterion 5 rather than left to drift.
  ShadowedBinding,

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
      Self::NestingTooDeep => 109,
      Self::SpacedPathSeparator => 110,
      Self::NewtypeShape => 111,
      Self::LocalTest => 112,
      Self::UnitTail => 113,
      // Doc 03 §17's published number, and the first in the lint band.
      Self::UnwrapUsed => 1001,
      Self::CommentAllowPragma => 1002,
      Self::ShadowedBinding => 1003,
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
      Self::NestingTooDeep,
      Self::SpacedPathSeparator,
      Self::NewtypeShape,
      Self::LocalTest,
      Self::UnitTail,
      Self::UnwrapUsed,
      Self::CommentAllowPragma,
      Self::ShadowedBinding,
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
          (40..=99).contains(&n) || (100..=199).contains(&n) || n >= 1000,
          "{code:?} is {n}, outside RDX0040-0099, RDX0100-0199 and the RDX1000+ lint band"
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
