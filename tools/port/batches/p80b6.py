# -*- coding: utf-8 -*-
"""Plan 80 batch 6: four more, including the crate's one `as` cast.

`cache_key` is `docs/CONFORMANCE.md`'s single `redox-driver` entry: the Rust
writes `(source.len() as u64).to_le_bytes()`, and an `as` cast is graded
`UNREACHABLE` in `docs/GRAMMAR.md`. `u64::try_from` plus `unwrap_or` expresses
it, which is the treatment plan 76 §2 gave `redox-bench`'s `as f64` — and the
fallback cannot mislead, because a length that does not fit a `u64` cannot
exist and `u64::MAX` would still be a distinct prefix if one did.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('compile_dir/with', '''
export compile_dir_with : &Path -> &Path -> &dyn CacheReporter -> Result<Compiled, Vec<Diagnostic>>
compile_dir_with src out reporter =
  store = Store::new out
  fingerprint = Fingerprint::current ()

  mut compiled = Compiled::default ()
  mut diagnostics = Vec::new ()

  for source in sdx_in src
    label = .map_or_else String::new (\\n -> .to_string (.to_string_lossy n)) (.file_name source)

    case compile_file (&source) out (&store) (&fingerprint) of
      Ok outcome ->
        .report (&label) outcome.was_cached reporter
        .push source compiled.inputs
        .push outcome.written compiled.outputs
      -- A file that failed to compile was not served from cache, and saying
      -- so keeps the reported events in step with the files attempted.
      --
      -- `reported`, where the Rust bound `errors`: `errors` is doc 03 §14's
      -- declaration keyword. And `.extend`, where the Rust wrote
      -- `.append(&mut errors)`: a pattern binder cannot be `mut`, and
      -- consuming the vector leaves the same diagnostics in the same order.
      Err reported ->
        .report (&label) false reporter
        .extend reported diagnostics

  if .is_empty diagnostics
    Ok compiled
  else
    Err diagnostics
''')

write('remap/narrow', '''
export(crate) narrow : Span -> &str -> usize -> usize -> Span
narrow span source line col =
  guard char_offset source line col is Some offset else
    return span
  if offset < span.0 || offset >= span.1
    return span
  -- `source[offset .. n]` with `n` bound first, where the Rust wrote
  -- `source[offset..]`. An **open range has no spelling**.
  n = .len source
  width = .map_or 1 char::len_utf8 (.next (.chars source[offset .. n]))
  (offset, offset + width)

-- | `add = …\\n  a + "x"\\n` — the statement spans bytes 33..40.
#[cfg(test)]
const SRC : &str = "add = (a: i32, b: i32): i32 ->\\n  a + \\"x\\"\\n"

#[cfg(test)]
const SPAN : (usize, usize) = (33, 40)

test "narrows to one character inside the span"
  got = narrow SPAN SRC 2 5
  assert_eq! \\-> &SRC[got.0 .. got.1] \\-> "+"

test "keeps the span when the column is past the line"
  assert_eq! \\-> narrow SPAN SRC 2 99 \\-> SPAN

-- | A position on another line is outside the span, even though it exists.
test "keeps the span when the position is elsewhere"
  assert_eq! \\-> narrow SPAN SRC 1 1 \\-> SPAN

test "keeps the span when the line does not exist"
  assert_eq! \\-> narrow SPAN SRC 99 1 \\-> SPAN

-- | The narrowed span must cover a whole character, not a byte of one.
test "narrows to a whole multibyte character"
  -- `x : ()\\n` is 7 bytes and `x =\\n` is 4, so the body's two spaces are
  -- at 11 and 12 and `é` — two bytes — is 13..15, on line 3 column 3.
  --
  -- Every number in those two lines moved by one when plan 52 made the
  -- type marker `:` rather than `::`, which is the hazard of deriving an
  -- offset in a comment: the arithmetic is only checked by the assertion
  -- two lines down, and that assertion is the one that caught it.
  src = "x : ()\\nx =\\n  é\\n"
  span = (13, 15)
  assert_eq! \\-> &src[span.0 .. span.1] \\-> "é" \\-> "the premise of this test"
  got = narrow span src 3 3
  assert_eq! \\-> &src[got.0 .. got.1] \\-> "é"
''')

write('cache/cache_key', '''
export(crate) cache_key : &str -> &Fingerprint -> blake3::Hash
cache_key source fingerprint =
  mut hasher = blake3::Hasher::new ()
  -- A length prefix keeps the two fields from running together: without it,
  -- sources differing only in where the fingerprint appears to begin could
  -- collide. Cheap, and the alternative is a subtle one.
  --
  -- `u64::try_from` plus `unwrap_or`, where the Rust wrote `as u64`: an `as`
  -- cast is `UNREACHABLE` in `docs/GRAMMAR.md`, and this is the crate's one
  -- entry in `docs/CONFORMANCE.md`. The fallback cannot mislead — a length
  -- that does not fit a `u64` cannot exist, and `u64::MAX` would still be a
  -- distinct prefix if one did.
  length = .unwrap_or u64::MAX (u64::try_from (.len source))
  .update (&.to_le_bytes length) hasher
  .update (.as_bytes source) hasher
  .update (.as_bytes fingerprint) hasher
  .finalize hasher

-- | `#[cfg(test)]`: the compiler fingerprint every test below keys against.
#[cfg(test)]
fp : Fingerprint
fp = Fingerprint::r#of b"compiler"

test "the same source and compiler give the same key"
  assert_eq! \\-> cache_key "main : ()\\nmain =\\n" (&fp ()) \\-> cache_key "main : ()\\nmain =\\n" (&fp ())

-- | Criterion 2's mechanism: one byte changes the key.
test "one changed byte changes the key"
  assert_ne! \\-> cache_key "print 1\\n" (&fp ()) \\-> cache_key "print 2\\n" (&fp ())

-- | Criterion 3's mechanism: a new compiler invalidates everything.
test "a different compiler changes the key"
  a = cache_key "main : ()\\nmain =\\n" (&Fingerprint::r#of b"old")
  b = cache_key "main : ()\\nmain =\\n" (&Fingerprint::r#of b"new")
  assert_ne! \\-> a \\-> b

-- | The length prefix exists to prevent this: two inputs whose concatenations
-- | would be identical must still key differently.
test "the boundary between fields is unambiguous"
  assert_ne! \\-> cache_key "ab" (&Fingerprint::r#of b"c") \\-> cache_key "a" (&Fingerprint::r#of b"bc")

-- | A key is usable as a filename, which is what the bare `blake3::Hash`
-- | buys and what the store relies on.
test "renders as hex"
  hex = .to_string (cache_key "x" (&fp ()))
  assert_eq! \\-> .len hex \\-> 64
  assert! \\-> .all (\\c -> .is_ascii_hexdigit c) (.chars hex)
''')

write('compile_dir/sdx_in', '''
export(crate) sdx_in : &Path -> Vec<PathBuf>
sdx_in dir =
  guard std::fs::read_dir dir is Ok entries else
    return Vec::new ()
  mut found : Vec<PathBuf> = entries
    |> .filter_map Result::ok
    |> .map (\\e -> .path e)
    |> .filter (\\p -> .is_file p && .is_some_and (\\x -> x == "sdx") (.extension p))
    |> .collect
  .sort found
  found

#[cfg(test)]
static NEXT : std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new 0

-- | A scratch directory holding the named files, removed on drop.
-- |
-- | `#[cfg(test)]`, and at file scope with its counter and its `Drop`: Redox
-- | puts nothing but `#[test] fn`s inside `mod __rdx_tests` (plan 73 §2.9).
#[cfg(test)]
data Dir = Dir std::path::PathBuf

#[cfg(test)]
instance Dir
  -- | `r#with`, because `with` is a Redox keyword.
  r#with : &str -> &[&str] -> Self
  r#with tag names =
    n = .fetch_add 1 std::sync::atomic::Ordering::Relaxed NEXT
    pid = std::process::id ()
    dir = .join (format! \\-> "redox-sdx-in-#{tag}-#{pid}-#{n}") (std::env::temp_dir ())
    .expect "should create" (std::fs::create_dir_all (&dir))
    for name in names
      .expect "should write" (std::fs::write (.join name dir) "main : ()\\nmain =\\n  println! \\\\-> \\"\\#{1}\\"\\n")
    Dir dir

#[cfg(test)]
instance Drop Dir
  drop : &mut self -> ()
  drop self =
    _dropped = .ok (std::fs::remove_dir_all (&self.0))
    ()

test "finds sdx files and sorts them"
  dir = Dir::r#with "sorted" (&["b.sdx", "a.sdx", "c.sdx"])
  found : Vec<String> = sdx_in (&dir.0)
    |> .iter
    |> .map (\\p -> .to_string (.to_string_lossy (.expect "a name" (.file_name p))))
    |> .collect
  assert_eq! \\-> found \\-> vec! \\-> "a.sdx" \\-> "b.sdx" \\-> "c.sdx"

test "ignores other extensions"
  dir = Dir::r#with "mixed" (&["keep.sdx", "skip.rs", "skip.xrs", "skip.md"])
  assert_eq! \\-> .len (sdx_in (&dir.0)) \\-> 1

test "yields nothing for a missing directory"
  found = sdx_in (std::path::Path::new "/nonexistent/redox")
  assert! \\-> .is_empty found

test "yields nothing for an empty directory"
  dir = Dir::r#with "empty" (&[])
  found = sdx_in (&dir.0)
  assert! \\-> .is_empty found
''')

print('plan 80 batch 6: four files')
