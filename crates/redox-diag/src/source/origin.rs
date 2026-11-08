use crate::Span;

/// Where one line of generated Rust came from.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 1. Recorded by the emitter as
/// it writes, which is the only moment both positions are known at once.
///
/// Both columns are stored because mapping a column is a *shift*, not a
/// lookup: rustc reports a position inside a line, and the generated line and
/// the source line differ by their indentation (four spaces against two, say,
/// plus whatever wrapper the emitter added). Knowing where the construct's
/// text begins on each side turns that into subtraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Origin {
  /// The source span of the construct that produced the line.
  pub span: Span,
  /// 1-based source line of [`Origin::span`]'s start.
  pub src_line: usize,
  /// 1-based source column at which the construct's text begins.
  pub src_col: usize,
  /// 1-based generated column at which the same text begins.
  pub gen_col: usize,
}

impl Origin {
  /// Map a column on the generated line back to a source column.
  ///
  /// Floored at 1: a position *left* of where the construct begins — inside
  /// the emitter's own indentation — has no source column, and reporting the
  /// construct's first column is better than reporting zero or wrapping.
  pub fn map_col(&self, gen_col: usize) -> usize {
    self.src_col + gen_col.saturating_sub(self.gen_col)
  }
}

#[cfg(test)]
mod tests {
  use super::Origin;

  fn origin() -> Origin {
    // Source `  a + b` at line 2 column 3; generated `    a + b` at column 5.
    Origin {
      span: (10, 15),
      src_line: 2,
      src_col: 3,
      gen_col: 5,
    }
  }

  #[test]
  fn shifts_a_column_by_the_indentation_difference() {
    assert_eq!(origin().map_col(5), 3, "the first character");
    assert_eq!(origin().map_col(9), 7, "four characters in");
  }

  #[test]
  fn floors_at_the_constructs_first_column() {
    assert_eq!(origin().map_col(1), 3);
    assert_eq!(origin().map_col(4), 3);
  }
}
