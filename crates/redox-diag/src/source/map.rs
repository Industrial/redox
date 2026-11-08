pub(crate) mod parse_map;
pub(crate) mod render_map;
pub(crate) mod version;

use crate::source::origin::Origin;

/// Which source position each line of generated Rust came from.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 1. Indexed by generated line,
/// 1-based, with `None` for a line the emitter synthesised and no source line
/// corresponds to — a closing brace, or the blank line between two items.
///
/// One record per *line* rather than per span: rustc reports positions by line
/// and column, so a line-indexed map is exactly the resolution the consumer
/// queries. A span-level map would be larger and more precise than anything
/// downstream can use (plan 06 §9).
///
/// Lives in `redox-diag` rather than `redox-emit` because the emitter produces
/// it and the driver consumes it; a type that is not about emitting should not
/// make its consumer depend on the emitter (doc 06 §1.2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
  /// The `.sdx` path this map is about, as the emitter was told it.
  pub source_path: String,
  /// One entry per generated line, in order.
  origins: Vec<Option<Origin>>,
}

impl SourceMap {
  pub fn new(source_path: impl Into<String>, origins: Vec<Option<Origin>>) -> Self {
    Self {
      source_path: source_path.into(),
      origins,
    }
  }

  /// How many generated lines this map covers.
  pub fn len(&self) -> usize {
    self.origins.len()
  }

  pub fn is_empty(&self) -> bool {
    self.origins.is_empty()
  }

  /// The origin of a 1-based generated line, if it has one.
  pub fn origin(&self, gen_line: usize) -> Option<&Origin> {
    self.origins.get(gen_line.checked_sub(1)?)?.as_ref()
  }

  /// Resolve a 1-based generated position to a 1-based source position.
  ///
  /// `None` means the line is synthetic, or past the end of the map — both of
  /// which a caller must handle by reporting against the generated file
  /// instead, never by guessing.
  pub fn resolve(&self, gen_line: usize, gen_col: usize) -> Option<(usize, usize)> {
    let origin = self.origin(gen_line)?;
    Some((origin.src_line, origin.map_col(gen_col)))
  }

  /// Every line with no origin, as 1-based line numbers.
  ///
  /// The totality test (plan 06 criterion 1) uses this to check the synthetic
  /// lines are the ones it expects, rather than counting them.
  pub fn synthetic_lines(&self) -> Vec<usize> {
    self
      .origins
      .iter()
      .enumerate()
      .filter(|(_, o)| o.is_none())
      .map(|(i, _)| i + 1)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::SourceMap;
  use crate::source::origin::Origin;

  fn map() -> SourceMap {
    SourceMap::new(
      "a.sdx",
      vec![
        Some(Origin {
          span: (0, 5),
          src_line: 1,
          src_col: 1,
          gen_col: 1,
        }),
        None,
        Some(Origin {
          span: (10, 15),
          src_line: 2,
          src_col: 3,
          gen_col: 5,
        }),
      ],
    )
  }

  #[test]
  fn resolves_a_mapped_line() {
    assert_eq!(map().resolve(3, 7), Some((2, 5)));
  }

  #[test]
  fn declines_a_synthetic_line() {
    assert_eq!(map().resolve(2, 1), None);
  }

  /// A line past the end must decline rather than panic: a caller holding a
  /// stale map would otherwise take down the compiler.
  #[test]
  fn declines_a_line_past_the_end() {
    assert_eq!(map().resolve(99, 1), None);
  }

  /// And a zero line, which a 1-based API will be handed sooner or later.
  #[test]
  fn declines_line_zero() {
    assert_eq!(map().resolve(0, 1), None);
  }

  #[test]
  fn reports_its_synthetic_lines() {
    assert_eq!(map().synthetic_lines(), vec![2]);
    assert_eq!(map().len(), 3);
  }
}
