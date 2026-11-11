# -*- coding: utf-8 -*-
"""Plan 76 batch 1: `redox-bench`'s six smaller files, and a field called `rust`.

**`rust` is a keyword** — plan 64 gave it the layout-delimited escape hatch — so
`Pair`'s `rust: PathBuf` field cannot keep its name. `r#rust` is the spelling,
and it is transparent to the one thing that reads the field: `tests/zero_cost.rs`
is hand-written Rust and stays Rust, where `r#rust` and `rust` are the same
identifier.

That makes two keyword collisions in two crates — `data` in plan 75, `rust`
here — on fields and parameters the Rust named freely. The list is long enough
(`type`, `module`, `test`, `case`, `of`, `is`, `in`, `or`, `not`, `do`, `with`,
`where`, `try`, `catch`, `guard`, `pure`, `errors`, `class`, `instance`,
`implements`) that the remaining seven crates will meet more, and the
translator could prefix `r#` itself rather than leaving a parse error — owed to
a successor row.
"""
import io
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

ROOT = '/home/tom/Code/rust/redox/crates/redox-bench/sdx/'

# --- the keyword collision ----------------------------------------------
p = ROOT + 'pair.sdx'
s = io.open(p, encoding='utf-8').read()
OLD = '  export rust : PathBuf,\n'
NEW = ('  -- | `r#rust`, because **`rust` is a keyword** — plan 64 gave it the\n'
       '  -- | layout-delimited escape hatch. The raw form is the same identifier\n'
       '  -- | in Rust, so `tests/zero_cost.rs` still reads `pair.rust`.\n'
       '  export r#rust : PathBuf,\n')
if OLD in s:
    io.open(p, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
    print('pair.sdx: r#rust')
else:
    assert 'export r#rust : PathBuf,' in s, 'pair.sdx: neither spelling'
    print('pair.sdx: already r#rust')

# --- the bodies ----------------------------------------------------------
write('throughput/transpile', '''
export(crate) transpile : &str -> Option<String>
transpile source =
  program = (.ok (redox_parser::parse_source source))?
  Some (redox_emit::emit (&program))
''')

write('ir/function_ir/strip_debug', '''
export(crate) strip_debug : &str -> &str
strip_debug line = case .find ", !dbg" line of
  -- `.get` and a fallback where the Rust sliced `&line[..at]`: a range index
  -- is `Index`, which doc 03 grades unreachable. `at` came from `find`, so the
  -- fallback cannot fire.
  Some at -> .unwrap_or line (.get (0..at) line)
  None -> line
''')

write('pair/pairs/first_public_function', '''
export(crate) first_public_function : &str -> Option<String>
first_public_function text =
  for line in .lines text
    rest = case .strip_prefix "pub fn " (.trim_start line) of
      Some r -> r
      None ->
        continue
    name : String = rest
      |> .chars
      |> .take_while (\\c -> .is_alphanumeric c || *c == '_')
      |> .collect
    if !(.is_empty name)
      return Some name
  None
''')

write('throughput', '''
export throughput : &str -> u32 -> (Duration, Duration)
throughput source runs =
  -- One warm run first, discarded. The first transpile in a process pays for
  -- lazy initialisation that has nothing to do with the compiler's speed, and
  -- folding it into the mean is the commonest way a benchmark overstates a
  -- cost.
  _warm = transpile source

  started = Instant::now ()
  for _run in (0..runs)
    _dropped = transpile source
  total = .elapsed started
  mean = total / .max 1 runs
  (total, mean)
''')

write('ir/function_ir/mark_target', '''
export(crate) mark_target : &str -> &str -> String
mark_target source name =
  signature = format! \\-> "fn #{name}("
  pub_signature = format! \\-> "pub #{signature}"
  -- A `for` loop where the Rust had a `map` over a multi-statement closure.
  -- The closure's body binds twice and branches, which reads better as
  -- statements than as a lambda inside a pipeline — and it avoids a
  -- block-bodied lambda in argument position entirely.
  mut parts : Vec<String> = Vec::new ()
  for line in .lines source
    trimmed = .trim_start line
    if .starts_with (&signature) trimmed || .starts_with (&pub_signature) trimmed
      body = if .starts_with "pub " trimmed
        .to_string line
      else
        format! \\-> "pub #{line}"
      -- Piped: the line above binds a block, and a `.name` statement after one
      -- is read as field access on it.
      parts |> .push (format! \\-> "#[no_mangle]\\n#[inline(never)]\\n#{body}")
    else
      .push (.to_string line) parts
  .join "\\n" parts
''')

write('pair/pairs', '''
export pairs : &Path -> Vec<Pair>
pairs root =
  dir = .join "benchmarks/pairs" root
  entries = case std::fs::read_dir (&dir) of
    Ok e -> e
    Err _ ->
      return Vec::new ()
  mut stems : Vec<String> = entries
    |> .filter_map Result::ok
    |> .map (\\entry -> .path entry)
    |> .filter (\\path -> .is_some_and (\\x -> x == "sdx") (.extension path))
    |> .filter_map (\\path -> .map (\\s -> .into_owned (.to_string_lossy s)) (.file_stem path))
    |> .collect
  .sort stems

  mut found = Vec::new ()
  for stem in stems
    sdx_path = .join (format! \\-> "#{stem}.sdx") dir
    rust_path = .join (format! \\-> "#{stem}.rs") dir
    if !(.is_file rust_path)
      continue
    read = .ok (std::fs::read_to_string (&rust_path))
    function = case .and_then (\\text -> first_public_function (&text)) read of
      Some f -> f
      None ->
        continue
    found |> .push (Pair {
      stem,
      function,
      sdx = sdx_path,
      r#rust = rust_path,
    })
  found
''')

print('plan 76 batch 1: six files')
