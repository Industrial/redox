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
      Self::Coalesce | Self::Pipeline => return None,
    })
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
