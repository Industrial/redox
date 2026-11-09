use crate::{BinaryOp, Expr, Ident};

/// A **section**: an operator with one side supplied — doc 18 §4.
///
/// ```text
/// (+ 1)       |__rdx_1| __rdx_1 + 1          right section
/// (1 +)       |__rdx_1| 1 + __rdx_1          left section
/// (> 0)       |__rdx_1| __rdx_1.gt(&0)       comparison, in method form
/// (0 <)       |__rdx_1| __rdx_1.gt(&0)       flipped, the same lowering
/// (.name)     |__rdx_1| __rdx_1.name         field selector
/// (|> name)   |__rdx_1| __rdx_1.name()       method reference
/// ```
///
/// All four shapes become a one-parameter closure over the reserved name
/// `__rdx_1`, so a section is a *value* and needs no special call site.
///
/// Doc 18 §4 notes that the last two rows each delete something. A section of
/// `|>` is a method reference **derived from the rule** rather than added to
/// it, which retires doc 03 §8.5's `&:name` sigil — and unlike `&:` it tells a
/// field from a method, because `(.name)` selects and `(|> name)` calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Section {
  /// `(+ 1)` — the operator's **right** operand is supplied.
  Right { op: BinaryOp, operand: Box<Expr> },
  /// `(1 +)` — the operator's **left** operand is supplied.
  ///
  /// New in doc 18 §4, and it is what closes doc 03 §5.4's gap: with only
  /// right sections, `\x -> 1 - x` had no section spelling at all.
  Left { op: BinaryOp, operand: Box<Expr> },
  /// `(.name)` — GHC's record dot, selecting a field.
  Field(Ident),
  /// `(|> name)` — a section of the pipeline, calling a method.
  Method(Ident),
}
