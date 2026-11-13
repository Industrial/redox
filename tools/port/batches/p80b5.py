# -*- coding: utf-8 -*-
"""Plan 80 batch 5: four more of `voltaic-driver`'s bodies.

`scratch_dir`'s test is the first ported body to need a **block as an
expression** — the Rust scopes a `let` inside `{ … }` so the `Drop` runs before
the assertion. Plan 55 leaf 7 retired the `do` keyword rather than the
construct: an indented block after `=` *is* the expression, so the shape
carries over with nothing to announce it.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

# `check/scratch_dir` is written here and **rewritten by `p80b12`**, which
# moves its hoisted `static` into a file of its own — one construct per file,
# doc 06 §1. Kept as written so the sequence reads as it happened.
write('check/scratch_dir', '''
-- | The counter, at module scope where the Rust declared it inside
-- | `ScratchDir::new`. Doc 03 §13.4's `const`/`static` is module-scope, and a
-- | function-local `static` has static lifetime anyway, so the only change is
-- | that the module can see it.
static NEXT : std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new 0

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

write('compile_dir/compile_file', '''
export(crate) compile_file : &Path -> &Path -> &Store -> &Fingerprint -> Result<Outcome, Vec<Diagnostic>>
compile_file source out store fingerprint =
  path = .to_string (.display source)
  text = (.map_err (\\e -> vec! \\-> (io_diagnostic (&path) (format! \\-> "cannot read #{path}: #{e}"))) (std::fs::read_to_string source))?

  key = cache_key (&text) fingerprint
  if .get (&key) store is Some hit
    cached = (write_outputs source out (&path) (&hit))?
    return Ok (Outcome { written = cached, was_cached = true })

  program = (.map_err (\\failure -> diagnose source (&text) (&failure)) (voltaic_parser::parse_source (&text)))?
  -- `emitted.0` is the Rust and `emitted.1` the map, where the Rust
  -- destructured the pair. `r#rust` because `rust` is a Voltaic keyword.
  emitted = voltaic_emit::emit_with_map (&program) (&text) (&path)
  fresh = Cached { r#rust = emitted.0, map = voltaic_diag::render_map (&emitted.1) }

  written = (write_outputs source out (&path) (&fresh))?
  .put (&key) (&fresh) store
  Ok (Outcome { written, was_cached = false })
''')

write('rustc_path', '''
export rustc_path : Option<PathBuf>
rustc_path =
  from_path = std::env::var_os "PATH"
    |> .into_iter
    |> .flat_map (\\paths ->
    dirs : Vec<PathBuf> = std::env::split_paths (&paths) |> .map (\\dir -> .join "rustc" dir) |> .collect
    dirs)
  baked = std::iter::once (PathBuf::from (env! \\-> "VOLTAIC_RUSTC"))
  -- `RUSTC` last. It is blaze's own shim under `blaze build`, so it is the
  -- least trustworthy of the three and is here only so that a toolchain
  -- reachable by no other means is still reachable.
  from_env = std::env::var_os "RUSTC" |> .map PathBuf::from |> .into_iter

  from_path
    |> .chain baked
    |> .chain from_env
    |> .find (\\candidate -> is_real_rustc candidate)

-- | The environment this suite runs in has a real rustc, since the suite was
-- | compiled by one.
test "finds a rustc"
  found = .expect "the toolchain should provide a rustc" (rustc_path ())
  shown = .display found
  assert! \\-> is_real_rustc (&found) \\-> "#{shown} is not rustc"
''')

write('lint/walk_item', '''
export(crate) walk_item : &Item -> &mut dyn ProgramWalk -> ()
walk_item item walk =
  .item item walk

  case item of
    Item::Fn decl ->
      for clause in &decl.clauses
        case &clause.body of
          ClauseBody::Plain block -> walk_stmts (&block.stmts) walk
          ClauseBody::Guarded rows ->
            for row in rows
              walk_stmts (&row.body.stmts) walk
        -- `where` locals are statements of the clause's own scope, so a
        -- pragma leads one exactly as it leads a body statement — and a
        -- binding shadows within them on the same terms.
        walk_stmts (&clause.wheres) walk
    Item::Class decl ->
      for member in &decl.members
        walk_member member walk
    Item::Instance decl ->
      for member in &decl.members
        walk_member member walk
    Item::Test decl -> walk_stmts (&decl.body.stmts) walk
    -- The one item that contains items, so the one that recurses.
    Item::Module decl ->
      for inner in .flatten (.iter decl.items)
        walk_item inner walk
    -- No statement blocks. Inline Rust is never parsed, so it holds no call
    -- site Voltaic could lint, no binding Voltaic could see and no pragma Voltaic
    -- could read.
    Item::Data _ or Item::Type _ or Item::Errors _ or Item::Import _ or Item::Const _ or Item::InlineRust _ or Item::Extern _ -> ()
  ()
''')

print('plan 80 batch 5: four files')
