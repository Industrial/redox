# -*- coding: utf-8 -*-
r"""Plan 83 batch 1: all six of `redox-seed`'s bodies.

The thirteenth and last crate, and the smallest port since `redox-fuzz`.

Two things are worth naming.

**`main` returns `()` and ends in a diverging call.** `std::process::exit` has
type `!`, and plan 55 §1.6 wants a `()`-returning body to end with `()`,
`return`, `break` or `continue` — none of which a call is. It is legal because
the rule grades an **indented** body only, which is the carve-out §1.6 states:
`clause` asks `unit_tail` on the block path and not on the inline one. So
`main = std::process::exit (run::run ())` is one line, and a trailing `()`
after it would be `unreachable_code` rather than a fix.

**`run` is nullary**, so it is `run : i32` with no parameter row — the shape
`above_bitor` already has in `redox-parser` — and the call site is
`run::run ()`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('main', r'''
-- One line, deliberately: `std::process::exit` has type `!`, and plan 55
-- §1.6's unit rule grades an **indented** body only. A trailing `()` after a
-- diverging call is `unreachable_code`, which `-D warnings` makes an error.
--
-- **Above the signature line, not between it and the equation.** A comment in
-- that position parses and is then dropped by the emitter *and* the formatter
-- — §2.2 of this row's plan has the reproduction.
main : ()
main = std::process::exit (run::run ())
''')

write('run', r'''
export(crate) run : i32
run =
  -- `paths.0` is the source directory and `paths.1` the output one, where the
  -- Rust destructured the pair in its `let … else`.
  guard arguments () is Some paths else
    eprintln! \-> "usage: redox-seed <src-dir> <out-dir>"
    return 2
  src = paths.0
  out = paths.1
  -- Every directory, not just the root: `compile_dir` does not descend, and
  -- `walk`'s doc records why that is its own row's problem rather than this
  -- one's.
  mut diagnostics = Vec::new ()
  mut failed = false
  mut written : usize = 0
  for pair in walk (&src) (&out)
    from = pair.0
    into = pair.1
    case compile_dir_with (&from) (&into) (&Silent) of
      Ok compiled ->
        written += .len compiled.outputs
        report (&compiled)
      Err found ->
        -- **Collected, not returned.** A seed reporting the first directory's
        -- errors and stopping would make a clean clone take as many runs to
        -- diagnose as it has broken directories. Everything that compiles is
        -- still written, which is what `init.sh` wants: the failure is loud and
        -- the tree is as complete as it can be.
        --
        -- Inside the arm rather than above it: a comment between two `case`
        -- arms parses and is then dropped (§2.2).
        failed := true
        .extend found diagnostics
    -- The cache `compile_dir_with` just wrote into the destination. Right for
    -- plan 09's `OUT_DIR`, wrong for a crate's `src/` — `tidy`'s doc has the
    -- three reasons.
    tidy (&into)
  if failed
    -- `plain: true`, and not because stderr might not be a terminal. The
    -- graphical renderer is what reaches for `miette`, and the seed's entire
    -- reason for existing is to be the smallest binary that transpiles — see
    -- this crate's module doc.
    rendered = render (&diagnostics) true
    eprintln! \-> "#{rendered}"
    return 1
  -- **Writing nothing is a failure.** `compile_dir_with` returns `Ok` with an
  -- empty output list for a directory holding no `.sdx` — including one it
  -- could not read at all — so without this the seed exits 0 having produced
  -- an empty tree. Found by plan 72 leaf 4's first run, which handed a WASI
  -- binary an absolute path: `wasmtime` exited 0, the seed exited 0, and the
  -- output directory was empty.
  --
  -- For a bootstrap tool that is the worst available failure mode. On a clean
  -- clone the consequence is an empty `src/`, a green exit, and a cargo error
  -- minutes later naming a missing crate rather than a missing argument.
  if written == 0
    shown = .display src
    eprintln! \-> "error: no .sdx found under #{shown}, so nothing was written. If this is a WASI run, the path must be relative to the directory preopened with --dir."
    return 1
  0
''')

write('run/arguments', r'''
export(crate) arguments : Option<(PathBuf, PathBuf)>
arguments =
  mut args = .skip 1 (std::env::args_os ())
  src = (.next args)?
  out = (.next args)?
  if .is_some (.next args)
    return None
  Some (PathBuf::from src, PathBuf::from out)

-- | Inline because `arguments` is private and reads the real process
-- | arguments, which an integration test cannot set without spawning — and
-- | spawning is the one thing the seed's target cannot do. AGENTS.md's rule
-- | is that inline tests are for private constructs, which this is.
-- |
-- | Under `cargo test` the arguments are the harness's, so the only
-- | behaviour assertable here is that it does not panic on whatever it is
-- | handed. The real coverage is `tests/seed.rs`, which runs the binary.
test "it does not panic on the harness arguments"
  -- `_dropped`, where the Rust wrote `let _ = …`: `_` is not a Redox pattern,
  -- and the binding is still what satisfies `Option`'s `#[must_use]`.
  _dropped = arguments ()
''')

write('run/report', r'''
export(crate) report : &Compiled -> ()
report compiled =
  for path in &compiled.outputs
    -- `shown` bound out: an interpolation holds an expression and `println!`
    -- takes a template, so the `Display` adaptor needs a name.
    shown = .display path
    println! \-> "#{shown}"
  ()
''')

write('run/tidy', r'''
export(crate) tidy : &Path -> ()
tidy dir =
  -- Errors are deliberately dropped. The only ones possible are "it was not
  -- there", which is the desired state, and a permission failure on a
  -- directory the seed just wrote — which the caller will notice when the
  -- layout tests fail, with a far better message than this function could
  -- produce.
  --
  -- `_dropped`, where the Rust wrote `let _ = …`.
  _dropped = std::fs::remove_dir_all (.join ".redox-cache" dir)
  ()
''')

write('run/walk', r'''
export(crate) walk : &Path -> &Path -> Vec<(PathBuf, PathBuf)>
walk src out =
  mut pairs = vec! \-> (.to_path_buf src, .to_path_buf out)
  mut at = 0
  while at < .len pairs
    -- `from` and `into` projected from the pair, where the Rust destructured
    -- it: there is no tuple-destructuring binding.
    pair = .clone pairs[at]
    from = pair.0
    into = pair.1
    at += 1
    guard std::fs::read_dir (&from) is Ok entries else
      continue
    mut children : Vec<PathBuf> = entries
      |> .flatten
      |> .map (\e -> .path e)
      |> .filter (\p -> .is_dir p)
      |> .collect
    -- Sorted, so the file list the seed prints is the same on every machine.
    -- A bootstrap log that reorders itself is a diff nobody can read.
    .sort children
    for child in children
      guard .file_name child is Some name else
        continue
      -- Joined before the push, not inside it: `name` borrows `child`, and
      -- moving `child` into the tuple while that borrow is live is `E0505`.
      target = .join name into
      .push (child, target) pairs
  pairs

-- | Inline because `walk` is private, per AGENTS.md's rule that inline tests
-- | are for private constructs.
test "it mirrors a nested tree"
  root = std::path::Path::new "../redox-ast/sdx"
  if !.is_dir root
    -- The crate layout moved; `port.rs` is what grades that, not this.
    return ()
  pairs = walk root (std::path::Path::new "/out")
  assert! \-> .len pairs > 1 \-> "redox-ast's sdx tree is nested, so walking it must find more than the root"
  -- `pairs[1].1` read in place, where the Rust bound `(_, out)` from a
  -- reference to it.
  shown = .display pairs[1].1
  assert! \-> .starts_with "/out" pairs[1].1 \-> "every output directory mirrors under the root it was given, got #{shown}"
''')

print('plan 83 batch 1: six files')
