use crate::cache::reporter::CacheReporter;

/// Reports nothing.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 4. The default for
/// [`crate::compile_dir`], which plan 09 §1.1 promised would keep its
/// signature when the cache arrived — so the cache had to have a way of
/// saying nothing, and this is it. A caller that wants the events uses
/// [`crate::compile_dir_with`].
pub struct Silent;

impl CacheReporter for Silent {
  fn report(&self, _label: &str, _hit: bool) {}
}

#[cfg(test)]
mod tests {
  use super::Silent;
  use crate::cache::reporter::CacheReporter;

  #[test]
  fn accepts_everything_and_says_nothing() {
    Silent.report("a.sdx", true);
    Silent.report("a.sdx", false);
  }
}
