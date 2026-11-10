//! A cache hit is an optimization, never a correctness dependency.
//!
//! Plan 10 (`docs/15 - 10 - Incremental Cache.md`) leaf 6. Doc 02 §7.1 quotes
//! the rule this whole plan rests on, from Emerald's own cache:
//!
//! > A cache MISS always falls back to calling the real function verbatim — a
//! > cache hit is strictly an optimization, never a correctness dependency.
//!
//! Every test here is written so it would still pass with the cache ripped
//! out — except the three that assert the cache *is* consulted, which is the
//! only honest way to test an optimization: prove the behaviour is unchanged,
//! then prove separately that work was skipped.

use redox_driver::CacheReporter;
use std::cell::RefCell;
use std::path::PathBuf;

/// A reporter that collects rather than prints.
///
/// Doc 02 §7.1 requires the cache to have *"no opinion on stdout"*, and this
/// is what that buys: the same events a build script would print to stderr,
/// available to a test as data.
struct Collect(RefCell<Vec<(String, bool)>>);

impl CacheReporter for Collect {
  fn report(&self, label: &str, hit: bool) {
    self.0.borrow_mut().push((label.to_string(), hit));
  }
}

impl Collect {
  fn new() -> Self {
    Self(RefCell::new(Vec::new()))
  }

  fn hits(&self) -> Vec<bool> {
    self.0.borrow().iter().map(|(_, hit)| *hit).collect()
  }
}

/// A source directory and an output directory, removed on drop.
struct Project {
  src: PathBuf,
  out: PathBuf,
  root: PathBuf,
}

impl Project {
  fn new(tag: &str) -> Self {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("redox-cache-{tag}-{}-{n}", std::process::id()));
    let src = root.join("src");
    let out = root.join("out");
    std::fs::create_dir_all(&src).expect("should create src");
    std::fs::create_dir_all(&out).expect("should create out");
    Self { src, out, root }
  }

  fn write(&self, name: &str, text: &str) {
    std::fs::write(self.src.join(name), text).expect("should write the source");
  }

  /// Compile, returning the generated Rust and whether each file was a hit.
  fn compile(&self) -> (String, Vec<bool>) {
    let reporter = Collect::new();
    let compiled = redox_driver::compile_dir_with(&self.src, &self.out, &reporter)
      .expect("the sources should compile");
    let rust = std::fs::read_to_string(&compiled.outputs[0]).expect("should read the output");
    (rust, reporter.hits())
  }

  fn wipe_cache(&self) {
    std::fs::remove_dir_all(self.out.join(".redox-cache")).ok();
  }
}

impl Drop for Project {
  fn drop(&mut self) {
    std::fs::remove_dir_all(&self.root).ok();
  }
}

const SOURCE: &str = "-- A cached program.\nmain : ()\nmain =\n  print 1\n  ()\n";

/// Criterion 1: the second compile of an unchanged file is a hit, and produces
/// byte-identical output.
///
/// Both halves matter. A hit that produced different bytes would be worse than
/// no cache at all, because the difference would appear only on a second run.
#[test]
fn an_unchanged_file_hits_and_produces_identical_output() {
  let project = Project::new("hit");
  project.write("a.sdx", SOURCE);

  let (first, hits) = project.compile();
  assert_eq!(hits, vec![false], "the first compile cannot be a hit");

  let (second, hits) = project.compile();
  assert_eq!(hits, vec![true], "the second compile should be a hit");
  assert_eq!(first, second, "a hit must produce identical output");
}

/// Criterion 2: one changed byte is a miss.
#[test]
fn a_changed_source_misses() {
  let project = Project::new("changed");
  project.write("a.sdx", SOURCE);
  let (first, _) = project.compile();

  project.write("a.sdx", &SOURCE.replace("print 1", "print 2"));
  let (second, hits) = project.compile();

  assert_eq!(hits, vec![false], "changed content must miss");
  assert_ne!(first, second, "and must produce the new output");
  assert!(second.contains("println!(\"{}\", 2)"), "{second}");
}

/// Criterion 4: with the cache wiped, the output is identical.
///
/// This is the criterion that makes the cache safe to have. If it ever fails,
/// the cache has become load-bearing and plan 10 §1's rule is broken.
#[test]
fn wiping_the_cache_changes_nothing_but_speed() {
  let project = Project::new("wipe");
  project.write("a.sdx", SOURCE);

  let (cached, _) = project.compile();
  assert_eq!(project.compile().1, vec![true], "warm");

  project.wipe_cache();
  let (cold, hits) = project.compile();

  assert_eq!(hits, vec![false], "a wiped cache must miss");
  assert_eq!(cached, cold, "and must recompute the same bytes");
}

/// Criterion 5: a corrupt entry is a miss, not a failure.
///
/// The cache lives in `OUT_DIR`, which an interrupted build or an unrelated
/// tool can leave in any state. A compiler that fails because its *cache* is
/// damaged has turned an optimization into a liability.
#[test]
fn a_corrupt_cache_entry_is_a_miss() {
  let project = Project::new("corrupt");
  project.write("a.sdx", SOURCE);
  let (good, _) = project.compile();

  // Truncate and garble every entry.
  let cache_dir = project.out.join(".redox-cache");
  for entry in std::fs::read_dir(&cache_dir).expect("the cache should exist") {
    let path = entry.expect("an entry").path();
    std::fs::write(&path, "garbage").expect("should corrupt");
  }

  let (after, hits) = project.compile();
  assert_eq!(hits, vec![false], "a corrupt entry must miss");
  assert_eq!(good, after, "and must recompute correctly");
}

/// Criterion 3: a different compiler fingerprint is a miss.
///
/// Asserted at the key rather than by rebuilding the compiler — plan 10 §1.1
/// is why this matters at all: without the fingerprint in the key, generated
/// Rust from an *older emitter* survives a rebuild, compiles cleanly, runs,
/// and is wrong. Nothing downstream can detect that, so the invalidation rule
/// has to be tested directly.
#[test]
fn a_different_compiler_invalidates_the_cache() {
  use redox_driver::Fingerprint;
  assert_ne!(
    Fingerprint::of(b"emitter v1"),
    Fingerprint::of(b"emitter v2"),
    "a changed compiler must change the fingerprint that keys every entry"
  );
  assert_eq!(Fingerprint::current(), Fingerprint::current());
}

/// Criterion 7: the reporter decides where its output goes, and the cache has
/// no opinion.
///
/// Asserted by the fact that this whole file observes the cache without a
/// single line reaching stdout — which for a build script is not cosmetic:
/// stdout is Cargo's directive channel.
#[test]
fn the_cache_reports_through_the_caller_not_stdout() {
  let project = Project::new("reporter");
  project.write("a.sdx", SOURCE);
  project.write("b.sdx", SOURCE.replace("print 1", "print 2").as_str());

  let reporter = Collect::new();
  redox_driver::compile_dir_with(&project.src, &project.out, &reporter).expect("should compile");

  let seen = reporter.0.borrow().clone();
  assert_eq!(seen.len(), 2, "one event per file: {seen:?}");
  let labels: Vec<&str> = seen.iter().map(|(l, _)| l.as_str()).collect();
  assert!(
    labels.contains(&"a.sdx") && labels.contains(&"b.sdx"),
    "{labels:?}"
  );
}

/// Criterion 6: `compile_dir` kept plan 09's signature, cache and all.
#[test]
fn compile_dir_still_takes_two_paths() {
  let project = Project::new("signature");
  project.write("a.sdx", SOURCE);

  let compiled =
    redox_driver::compile_dir(&project.src, &project.out).expect("should compile unchanged");
  assert_eq!(compiled.inputs.len(), 1);
  assert!(compiled.outputs[0].ends_with("a.sdx".replace(".sdx", ".rs")));
}

/// Two files with identical contents share a cache entry.
///
/// The key covers content and the compiler, not the path — so the second file
/// is a hit on its first compile. Worth asserting because it is a consequence
/// of the key's design that a reader might not expect.
#[test]
fn identical_contents_share_an_entry_across_names() {
  let project = Project::new("shared");
  project.write("a.sdx", SOURCE);
  project.write("b.sdx", SOURCE);

  let reporter = Collect::new();
  redox_driver::compile_dir_with(&project.src, &project.out, &reporter).expect("should compile");

  assert_eq!(
    reporter.hits(),
    vec![false, true],
    "the second file has the same bytes, so it hits on its first compile"
  );
}
