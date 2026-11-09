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
}

impl BinaryOp {
  /// The Rust spelling, for operators that pass straight through.
  ///
  /// `None` for the two that need a lowering rather than a spelling: `??`
  /// becomes a `match` and `|>` a receiver call, both in later plans.
  pub fn rust_text(self) -> Option<&'static str> {
    Some(match self {
      Self::Add => "+",
      Self::Sub => "-",
      Self::Mul => "*",
      Self::Div => "/",
      Self::Rem => "%",
      Self::Eq => "==",
      Self::Ne => "!=",
      Self::Lt => "<",
      Self::Gt => ">",
      Self::Le => "<=",
      Self::Ge => ">=",
      Self::And => "&&",
      Self::Or => "||",
      Self::BitAnd => "&",
      Self::BitOr => "|",
      Self::BitXor => "^",
      Self::Shl => "<<",
      Self::Shr => ">>",
      Self::Range => "..",
      Self::RangeInclusive => "..=",
      Self::Coalesce | Self::Catch | Self::Pipeline => return None,
    })
  }

  /// The Redox spelling — every operator has one, unlike `rust_text`.
  ///
  /// `rust_text` returns `None` for `??`, `catch` and `|>` because those need
  /// a *lowering* rather than a spelling. A formatter printing Redox needs the
  /// opposite: the surface spelling of all of them, including the three that
  /// have no Rust equivalent.
  pub fn redox_text(self) -> &'static str {
    match self {
      Self::Coalesce => "??",
      Self::Catch => "catch",
      Self::Pipeline => "|>",
      // The remaining operators are spelled as Rust spells them, which is
      // doc 02 §5.2's rule restated: an expression that reads like Rust and
      // binds differently is the worst available failure mode, so neither the
      // spelling nor the precedence diverges.
      other => other
        .rust_text()
        .expect("every operator but the three above has a Rust spelling"),
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
      Self::Pipeline => 1,
      // Doc 03 §4.5 places `catch` *at `??`'s level*, so they share a rung.
      Self::Coalesce | Self::Catch => 3,
      Self::Range | Self::RangeInclusive => 5,
      Self::Or => 7,
      Self::And => 9,
      // Comparison is non-associative in Rust. Equal powers let `a < b < c`
      // parse so that rustc rejects it — doc 02 §6.2's division of labour.
      Self::Eq | Self::Ne | Self::Lt | Self::Gt | Self::Le | Self::Ge => 11,
      Self::BitOr => 13,
      Self::BitXor => 15,
      Self::BitAnd => 17,
      Self::Shl | Self::Shr => 19,
      Self::Add | Self::Sub => 21,
      Self::Mul | Self::Div | Self::Rem => 23,
    }
  }

  /// Whether this is `..` or `..=`.
  ///
  /// Those two are written with no surrounding spaces, which is Rust's style
  /// and therefore rustfmt's — it rewrites `1 .. 4` to `1..4`. Every other
  /// operator takes its spaces, so the emitter asks here rather than
  /// carrying two spelling tables.
  pub fn is_range(self) -> bool {
    matches!(self, Self::Range | Self::RangeInclusive)
  }

  /// The `PartialOrd`/`PartialEq` method this comparison is, if it is one.
  ///
  /// Doc 18 §4 lowers a comparison section to the **method** form —
  /// `(> 0)` becomes `|__rdx_1| __rdx_1.gt(&0)` rather than
  /// `|__rdx_1| __rdx_1 > 0` — because §1.1's auto-ref point applies: the
  /// closure an iterator adaptor receives is handed a reference, and
  /// `&i32 > i32` does not compile while `(&i32).gt(&0)` does.
  pub fn compare_method(self) -> Option<&'static str> {
    Some(match self {
      Self::Eq => "eq",
      Self::Ne => "ne",
      Self::Lt => "lt",
      Self::Gt => "gt",
      Self::Le => "le",
      Self::Ge => "ge",
      _ => return None,
    })
  }

  /// This comparison with its operands exchanged.
  ///
  /// `(0 <)` and `(> 0)` are the same predicate written from either side, and
  /// doc 18 §4 gives them the same lowering — which this is what makes
  /// possible: flip the operator and the left section becomes a right one.
  pub fn flipped(self) -> Option<Self> {
    Some(match self {
      Self::Eq => Self::Eq,
      Self::Ne => Self::Ne,
      Self::Lt => Self::Gt,
      Self::Gt => Self::Lt,
      Self::Le => Self::Ge,
      Self::Ge => Self::Le,
      _ => return None,
    })
  }
}
