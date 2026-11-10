/// A binary operator, named rather than spelled.
///
/// Naming the operator rather than carrying its token keeps the emitter from
/// re-deciding what a token meant, and keeps doc 03's respellings — `..`
/// exclusive, `??` coalescing — in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
  Add,
  Sub,
  Mul,
  Div,
  Rem,
  Eq,
  Ne,
  Lt,
  Gt,
  Le,
  Ge,
  And,
  Or,
  BitAnd,
  BitOr,
  BitXor,
  Shl,
  Shr,
  /// `..` — exclusive, matching Rust (doc 03 §4.2 corrects doc 01 §21).
  Range,
  /// `..=` — inclusive.
  RangeInclusive,
  /// `??` — `Option` coalescing (doc 03 §12.1).
  Coalesce,
  /// `catch` — doc 03 §12.2's `Result` fallback, plan 28.
  ///
  /// At `??`'s precedence level (doc 03 §4.5) and lowered the same way, to a
  /// `match`, for §12.1's reasons: `unwrap_or_else`'s closure arity differs
  /// between `Option` and `Result`, so a method lowering would be
  /// type-dependent and Redox has no types.
  ///
  /// `??` serves `Option` and `catch` serves `Result` — doc 03 §12 keeps the
  /// two *"visually distinct and non-interchangeable"*, which is why this is a
  /// second operator rather than one that inspects its operand.
  Catch,
  /// `|>` — the receiver pipeline (doc 03 §5.3).
  Pipeline,
  /// `>>>` — forward composition: `f >>> g` applies `f` and then `g`.
  ///
  /// Haskell's `Control.Category` spelling, which is the one choice in this
  /// milestone that is borrowed rather than argued (plan 66 §1.3). Lowered
  /// rather than spelled — Rust has no composition operator — so
  /// [`Self::rust_text`] declines and `emit/compose.rs` writes the closure.
  Compose,
  /// `<<<` — backward composition: `f <<< g` applies `g` and then `f`.
  ///
  /// The same function as `g >>> f`, which is what plan 66 §6 discloses: the
  /// only thing this direction buys is a Haskell reader's muscle memory, and
  /// a later row may drop it on evidence.
  ComposeBack,
}
#[allow(unused_imports)]
use BinaryOp::*;

impl BinaryOp {
  /// The Rust spelling, for operators that pass straight through.
  ///
  /// `None` for the two that need a lowering rather than a spelling: `??`
  /// becomes a `match` and `|>` a receiver call, both in later plans.
  ///
  /// The `None` arm comes **first**, where the Rust wrote
  /// `Some(match self { … _ => return None })`. One arm per answer rather
  /// than an early `return` inside a constructor call — the same function,
  /// and it puts the operators this is about at the top.
  pub fn rust_text(self) -> Option<&'static str> {
    match self {
      Coalesce | Catch | Pipeline | Compose | ComposeBack => None,
      Add => Some("+"),
      Sub => Some("-"),
      Mul => Some("*"),
      Div => Some("/"),
      Rem => Some("%"),
      Eq => Some("=="),
      Ne => Some("!="),
      Lt => Some("<"),
      Gt => Some(">"),
      Le => Some("<="),
      Ge => Some(">="),
      And => Some("&&"),
      Or => Some("||"),
      BitAnd => Some("&"),
      BitOr => Some("|"),
      BitXor => Some("^"),
      Shl => Some("<<"),
      Shr => Some(">>"),
      Range => Some(".."),
      RangeInclusive => Some("..="),
    }
  }

  /// The Redox spelling — every operator has one, unlike `rust_text`.
  ///
  /// `rust_text` returns `None` for `??`, `catch` and `|>` because those need
  /// a *lowering* rather than a spelling. A formatter printing Redox needs the
  /// opposite: the surface spelling of all of them, including the three that
  /// have no Rust equivalent.
  pub fn redox_text(self) -> &'static str {
    match self {
      Coalesce => "??",
      Catch => "catch",
      Pipeline => "|>",
      Compose => ">>>",
      ComposeBack => "<<<",
      other => match other.rust_text() {
        Some(text) => text,
        None => unreachable!(),
      },
    }
  }

  /// This operator's left binding power — **the** precedence table.
  ///
  /// Doc 02 §5.2 fixes it as Rust's unchanged, and doc 03 §4.5 places the
  /// three operators Rust lacks: `??` and `catch` below `||`, `|>` below `??`.
  ///
  /// # Why this lives on the operator and not in the parser
  ///
  /// It was the parser's, keyed by `TokenKind` and `pub(crate)`, which was
  /// right while the parser was the only consumer. Plan 34 leaf 3 made the
  /// formatter a second one: dropping a redundant parenthesis is a precedence
  /// decision, and a formatter working from a *copy* of the table would drop a
  /// needed parenthesis the moment the two drifted — silently, since both
  /// would still compile.
  ///
  /// So the table is here, on the operator the AST already names, and
  /// `binary_power` in the parser maps token to operator and then asks. One
  /// table, two readers.
  ///
  /// Only the relative order is meaningful; the numbers are the parser's
  /// original odd-numbered rungs, kept so left-associativity is `power() + 1`
  /// with no collisions.
  pub fn power(self) -> u8 {
    match self {
      Pipeline => 1,
      Coalesce | Catch => 3,
      Range | RangeInclusive => 5,
      Or => 7,
      And => 9,
      Eq | Ne | Lt | Gt | Le | Ge => 11,
      BitOr => 13,
      BitXor => 15,
      BitAnd => 17,
      Shl | Shr => 19,
      Add | Sub => 21,
      Mul | Div | Rem => 23,
      Compose | ComposeBack => 25,
    }
  }

  /// Whether this is `..` or `..=`.
  ///
  /// Those two are written with no surrounding spaces, which is Rust's style
  /// and therefore rustfmt's — it rewrites `1 .. 4` to `1..4`. Every other
  /// operator takes its spaces, so the emitter asks here rather than
  /// carrying two spelling tables.
  pub fn is_range(self) -> bool {
    match self {
      Range | RangeInclusive => true,
      _ => false,
    }
  }

  /// The `PartialOrd`/`PartialEq` method this comparison is, if it is one.
  ///
  /// Doc 18 §4 lowers a comparison section to the **method** form —
  /// `(> 0)` becomes `|__rdx_1| __rdx_1.gt(&0)` rather than
  /// `|__rdx_1| __rdx_1 > 0` — because §1.1's auto-ref point applies: the
  /// closure an iterator adaptor receives is handed a reference, and
  /// `&i32 > i32` does not compile while `(&i32).gt(&0)` does.
  pub fn compare_method(self) -> Option<&'static str> {
    match self {
      Eq => Some("eq"),
      Ne => Some("ne"),
      Lt => Some("lt"),
      Gt => Some("gt"),
      Le => Some("le"),
      Ge => Some("ge"),
      _ => None,
    }
  }

  /// This comparison with its operands exchanged.
  ///
  /// `(0 <)` and `(> 0)` are the same predicate written from either side, and
  /// doc 18 §4 gives them the same lowering — which this is what makes
  /// possible: flip the operator and the left section becomes a right one.
  pub fn flipped(self) -> Option<Self> {
    match self {
      Eq => Some(Eq),
      Ne => Some(Ne),
      Lt => Some(Gt),
      Gt => Some(Lt),
      Le => Some(Ge),
      Ge => Some(Le),
      _ => None,
    }
  }
}
