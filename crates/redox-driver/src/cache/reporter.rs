/// Reports nothing.
pub(crate) mod silent;

/// Where the cache reports hits and misses.
///
/// Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 4. Doc 02 §7.1 names
/// this trait and the property that matters: the cache reports
/// `[cache] <query> <label> HIT|MISS` **with no opinion on stdout.**
///
/// That is not fastidiousness. A build script's stdout is Cargo's directive
/// channel — a stray line there is interpreted, not printed — so a cache that
/// decided to `println!` would corrupt the build it was speeding up. A test
/// wants the same events in a `Vec`. Neither belongs in the cache, so neither
/// is there.
pub trait CacheReporter {
  /// `label` names the file; `hit` is whether its result was reused.
  fn report(&self, label: &str, hit: bool);
}

#[cfg(test)]
mod tests {
  use super::CacheReporter;
  use std::cell::RefCell;

  /// The shape a test uses: collect, do not print.
  struct Collect(RefCell<Vec<String>>);

  impl CacheReporter for Collect {
    fn report(&self, label: &str, hit: bool) {
      self
        .0
        .borrow_mut()
        .push(format!("{label} {}", if hit { "HIT" } else { "MISS" }));
    }
  }

  #[test]
  fn a_reporter_sees_both_outcomes() {
    let collect = Collect(RefCell::new(Vec::new()));
    collect.report("a.sdx", false);
    collect.report("a.sdx", true);
    assert_eq!(collect.0.borrow().as_slice(), ["a.sdx MISS", "a.sdx HIT"]);
  }
}
