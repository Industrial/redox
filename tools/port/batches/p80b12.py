# -*- coding: utf-8 -*-
"""Plan 80 batch 12: the hoisted `static` gets its own file.

`ScratchDir::new` declares its counter as a `static` **inside the function**.
Voltaic's `const`/`static` is module-scope (doc 03 §13.4), so the port hoists it
— and `one_construct_per_file.rs` then reports `scratch_dir.rs` declaring two
constructs, which is doc 06 §1 working rather than an obstacle.

So the counter becomes `check/scratch_dir/next.sdx`, and `scratch_dir.rs`
becomes a **module head** with a sibling directory. The two `#[cfg(test)]`
counters in `cache/store` and `compile_dir/sdx_in` need no such move: plan 79
taught `constructs()` not to count a `#[cfg(test)]` declaration, which is
exactly the distinction — a test's helper is not part of the file's surface.

**A second `write()` over the same file must carry the declaration too.** The
first one consumed the `-- TODO` marker, so `write.py` then cuts at the first
*construct* — the type — and a body holding only the impls drops it.
`E0425: cannot find type ScratchDir` is how that reads.
"""
import io
import os
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

ROOT = '/home/tom/Code/rust/voltaic/crates/voltaic-driver/sdx/'

NEXT = '''-- | The scratch-directory counter.
-- |
-- | Its own file because doc 06 §1 is one construct per file, and the Rust
-- | declared it **inside** `ScratchDir::new` — a function-local `static`,
-- | which Voltaic has no spelling for: doc 03 §13.4's `const`/`static` is
-- | module-scope. A function-local `static` already has static lifetime, so
-- | hoisting changes nothing but who can see it — and `export(crate)` it must
-- | be, because the only thing that reads it now lives one file up.
-- |
-- | The counter exists so two directories made in the same process, for the
-- | same tag, in the same nanosecond cannot collide — which
-- | `two_directories_do_not_collide` asserts.
export(crate) static NEXT : std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new 0
'''

os.makedirs(ROOT + 'check/scratch_dir', exist_ok=True)
io.open(ROOT + 'check/scratch_dir/next.sdx', 'w', encoding='utf-8').write(NEXT)
print('  created check/scratch_dir/next.sdx')

write('check/scratch_dir', '''
import crate::check::scratch_dir::next::NEXT

export(crate) module next

-- | A temporary directory that removes itself.
-- |
-- | Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 4. `voltaic check` needs a
-- | file on disk to hand rustc, and the user did not ask for one — so it goes
-- | somewhere temporary and is cleaned up, rather than littering the source
-- | tree with a `.xrs` nobody asked for.
-- |
-- | Keyed by process id *and* a counter, which is the lesson plan 04 learned
-- | the hard way: keying on a name alone let two concurrent tests share a
-- | directory, and one test's cleanup deleted a file the other was still using.
export(crate) data ScratchDir = ScratchDir PathBuf

instance ScratchDir
  -- | A fresh directory, named for the tag, this process and a counter.
  export(crate) new : &str -> Result<Self, String>
  new tag =
    n = .fetch_add 1 std::sync::atomic::Ordering::Relaxed NEXT
    pid = std::process::id ()
    dir = .join (format! \\-> "voltaic-#{tag}-#{pid}-#{n}") (std::env::temp_dir ())
    shown = .to_string (.display dir)
    (.map_err (\\e -> format! \\-> "error: cannot create #{shown}: #{e}") (std::fs::create_dir_all (&dir)))?
    Ok (ScratchDir dir)

  -- | The directory itself.
  export(crate) path : &self -> &Path
  path self = &self.0

instance Drop ScratchDir
  drop : &mut self -> ()
  drop self =
    -- A failure here is not worth reporting: the directory is temporary, and
    -- the user came for a diagnostic, not for housekeeping news.
    _dropped = .ok (std::fs::remove_dir_all (&self.0))
    ()

test "creates and removes itself"
  -- An indented block as the binding's value, where the Rust wrote `{ … }`:
  -- the point of the scope is that `dir` drops before the assertion below.
  gone =
    dir = .expect "should create" (ScratchDir::new "test")
    live = .to_path_buf (.path dir)
    assert! \\-> .is_dir live
    live
  assert! \\-> !.exists gone \\-> "the directory should be gone after the drop"

-- | Two live scratch directories must not be the same directory.
test "two directories do not collide"
  a = .expect "should create" (ScratchDir::new "test")
  b = .expect "should create" (ScratchDir::new "test")
  assert_ne! \\-> .path a \\-> .path b
''')

print('plan 80 batch 12: one new file, one rewritten')
