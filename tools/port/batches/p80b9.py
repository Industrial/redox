# -*- coding: utf-8 -*-
"""Plan 80 batch 9: `check` and the cache store.

`Store::paths` returns a pair that both callers destructure, so this is the
third and fourth tuple-binding site in the crate — bound once and projected,
and the local the Rust called `rust` is `rust_path` because `rust` is a Redox
keyword.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('cache/store', '''
instance Store
  -- | The cache for an output directory.
  -- |
  -- | Under `OUT_DIR` for a build script, so the cache is per-crate,
  -- | per-profile, and emptied by `cargo clean` — it inherits Cargo's lifetime
  -- | rules rather than inventing its own, and so needs no eviction policy.
  export(crate) new : &Path -> Self
  new out = Store { dir = .join ".redox-cache" out }

  -- | The `<hash>.rs` and `<hash>.rs.map` paths one key names.
  -- |
  -- | `rust_path`, where the Rust named the local `rust`: `rust` is a Redox
  -- | keyword.
  paths : &blake3::Hash -> &self -> (PathBuf, PathBuf)
  paths key self =
    rust_path = .join (format! \\-> "#{key}.rs") self.dir
    map_path = .join (format! \\-> "#{key}.rs.map") self.dir
    (rust_path, map_path)

  -- | The cached result for `key`, if both halves are present and readable.
  export(crate) get : &blake3::Hash -> &self -> Option<Cached>
  get key self =
    found = .paths key self
    text = (.ok (std::fs::read_to_string found.0))?
    map = (.ok (std::fs::read_to_string found.1))?
    -- A map that does not parse means the entry was written by something other
    -- than this format, or was truncated. Either way: miss.
    (.ok (redox_diag::parse_map (&map)))?
    Some (Cached { r#rust = text, map })

  -- | Record a result. A failure to write is silently a future miss.
  export(crate) put : &blake3::Hash -> &Cached -> &self -> ()
  put key cached self =
    if .is_err (std::fs::create_dir_all (&self.dir))
      return ()
    found = .paths key self
    -- The map is written *first*. If the process dies between the two writes,
    -- the next `get` finds a map with no Rust beside it and treats the entry
    -- as absent — whereas the other order could leave Rust with no map, and a
    -- caller that read only the Rust would use an entry with no positions.
    if .is_err (std::fs::write (&found.1) (&cached.map))
      return ()
    if .is_err (std::fs::write (&found.0) (&cached.r#rust))
      -- Remove the orphan so the directory does not accumulate half-entries.
      _dropped = .ok (std::fs::remove_file (&found.1))
    ()

#[cfg(test)]
static NEXT : std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new 0

-- | `#[cfg(test)]`: a scratch directory, removed on drop, and one cache entry.
-- | Redox puts nothing but `#[test] fn`s inside `mod __rdx_tests` (plan 73
-- | §2.9), so both live here at file scope.
#[cfg(test)]
data Dir = Dir std::path::PathBuf

#[cfg(test)]
instance Dir
  new : &str -> Self
  new tag =
    n = .fetch_add 1 std::sync::atomic::Ordering::Relaxed NEXT
    pid = std::process::id ()
    dir = .join (format! \\-> "redox-store-#{tag}-#{pid}-#{n}") (std::env::temp_dir ())
    .expect "should create" (std::fs::create_dir_all (&dir))
    Dir dir

#[cfg(test)]
instance Drop Dir
  drop : &mut self -> ()
  drop self =
    _dropped = .ok (std::fs::remove_dir_all (&self.0))
    ()

#[cfg(test)]
entry : Cached
entry = Cached { r#rust = .to_string "fn main() {}\\n", map = .to_string "redox-map 1 a.sdx\\n1 1 1 1\\n" }

test "a stored entry comes back"
  dir = Dir::new "roundtrip"
  store = Store::new (&dir.0)
  key = blake3::hash b"k"
  assert_eq! \\-> .get (&key) store \\-> None \\-> "nothing is cached yet"
  .put (&key) (&entry ()) store
  assert_eq! \\-> .get (&key) store \\-> Some (entry ())

test "an unknown key is a miss"
  dir = Dir::new "unknown"
  store = Store::new (&dir.0)
  .put (&blake3::hash b"a") (&entry ()) store
  assert_eq! \\-> .get (&blake3::hash b"b") store \\-> None

-- | Criterion 5: a corrupt entry is a miss, not a failure.
test "a corrupt map is a miss"
  dir = Dir::new "corrupt"
  store = Store::new (&dir.0)
  key = blake3::hash b"k"
  .put (&key) (&entry ()) store

  found = .paths (&key) store
  .expect "should overwrite" (std::fs::write (&found.1) "not a redox map at all\\n")
  assert_eq! \\-> .get (&key) store \\-> None

-- | A half-entry — the Rust missing — is absent rather than partly usable.
test "a half written entry is a miss"
  dir = Dir::new "half"
  store = Store::new (&dir.0)
  key = blake3::hash b"k"
  .put (&key) (&entry ()) store

  found = .paths (&key) store
  .expect "should remove" (std::fs::remove_file (&found.0))
  assert_eq! \\-> .get (&key) store \\-> None

-- | An unwritable cache directory must not fail anything.
test "an unwritable store is simply always a miss"
  store = Store::new (std::path::Path::new "/proc/nonexistent-redox")
  key = blake3::hash b"k"
  .put (&key) (&entry ()) store
  assert_eq! \\-> .get (&key) store \\-> None
''')

write('check', '''
export check : &Path -> &str -> Result<CheckOutcome, String>
check path source =
  sdx_path = .to_string (.display path)

  -- A syntax error is Redox's own, and never reaches rustc.
  program = case redox_parser::parse_source source of
    Ok parsed -> parsed
    Err failure ->
      return Ok (CheckOutcome { diagnostics = diagnose path source (&failure), failed = true })

  -- Doc 03 §12.7's lints — plan 38. Reported and **not** fatal: §12.7 makes
  -- `unwrap` *"lintable rather than prohibited"*.
  --
  -- After the parse, because a warning about a program nobody can compile is
  -- noise, and before rustc runs, so Redox's own warnings are not buried under
  -- rustc's.
  mut diagnostics = crate::lint path source (Some (&program))

  -- `emitted.0` is the Rust and `emitted.1` the map, where the Rust
  -- destructured the pair.
  emitted = redox_emit::emit_with_map (&program) source (&sdx_path)

  rustc = (.ok_or_else (\\-> .to_string "error: no rustc found. `redox check` compiles the generated Rust, so it needs the Rust toolchain on PATH.") (rustc_path ()))?

  dir = (ScratchDir::new "check")?
  xrs = .join "check.xrs" (.path dir)
  xrs_path = .to_string (.display xrs)
  (.map_err (\\e -> format! \\-> "error: cannot write #{xrs_path}: #{e}") (std::fs::write (&xrs) (&emitted.0)))?

  rustc_shown = .to_string (.display rustc)
  invoked = Command::new (&rustc)
    |> .args (["--edition", "2024", "--error-format=short", "--emit=metadata"])
    |> .arg "-o"
    |> .arg (.join "check.rmeta" (.path dir))
    |> .arg (&xrs)
    |> .output
  out = (.map_err (\\e -> format! \\-> "error: cannot run #{rustc_shown}: #{e}") invoked)?

  stderr = String::from_utf8_lossy (&out.stderr)
  messages : Vec<RustcMessage> = .collect (.filter_map RustcMessage::parse (.lines stderr))
  failed = .any RustcMessage::is_error (.iter messages)
  remapped = .map (\\m -> remap m (&emitted.1) (&sdx_path) source (&xrs_path) (&emitted.0)) (.iter messages)
  .extend remapped diagnostics

  if .is_empty messages && !.success out.status
    -- rustc failed for a reason it gave no positioned diagnostic for — a
    -- missing linker, say. Passing its own words through is more use than a
    -- message of Redox's invention.
    trimmed = .trim_end stderr
    return Err (format! \\-> "error: rustc failed:\\n#{trimmed}")

  Ok (CheckOutcome { diagnostics, failed })
''')

print('plan 80 batch 9: two files')
