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
}
