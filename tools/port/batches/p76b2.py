# -*- coding: utf-8 -*-
"""Plan 76 batch 2: `function_ir` and its two helpers.

`alpha_renamed` is the first file in any port to slice a `&str` **four times in
one loop** — `&rest[..at]`, `&rest[at + 1..]`, `&rest[..end]`, `&rest[end..]` —
so it is where the closed-range rule earns its keep: a parenthesised half-open
range is an operator section, and every one of these has to name both ends.

`function_ir` is the first to build a `Command`. Ten chained `.arg`s nested as
juxtaposition would be unreadable; a pipeline is the shape, and `Command::arg`
returning `&mut Command` is what makes it work.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('ir/function_ir/body_of', '''
export(crate) body_of : &str -> &str -> String
body_of text name =
  mut out = String::new ()
  mut inside = false
  for line in .lines text
    if .starts_with "define" line && .contains name line
      inside := true
      -- The `define` line itself carries the mangled symbol, which includes a
      -- hash of the source path. Recorded as a marker rather than compared.
      .push_str "define <symbol>\\n" out
      continue
    if inside
      if line == "}"
        break
      trimmed = .trim line
      if .starts_with '!' trimmed
        continue
      .push_str (strip_debug trimmed) out
      .push '\\n' out
  out
''')

write('ir/function_ir/alpha_renamed', '''
export(crate) alpha_renamed : &str -> String
alpha_renamed body =
  mut names : Vec<String> = Vec::new ()
  mut out = String::with_capacity (.len body)
  mut rest = body
  -- A `loop` with a `case` where the Rust had `while let Some(at) = …`.
  loop
    at = case .find '%' rest of
      Some a -> a
      None ->
        break
    -- **Closed ranges throughout.** A parenthesised half-open range is an
    -- operator section — `(at..)` is `|x| at .. x` — so each of the four
    -- slices in this loop names both ends. None of the fallbacks can fire:
    -- `at` came from `find` and `end` is clamped to the length.
    -- Piped: `at` above binds a block, and a `.name` statement after one is
    -- read as field access on it.
    out |> .push_str (.unwrap_or "" (.get (0..at) rest))
    rest := .unwrap_or "" (.get ((at + 1)..(.len rest)) rest)
    -- **`.char_indices` where the Rust wrote `find(|c: char| …)`.** A Voltaic
    -- lambda parameter is a bare name, so the annotation that call needs has
    -- no spelling — and `char_indices` removes the need for one: `p.1` is a
    -- `char`, and `p.0` is the **byte** index `str::find` would have returned,
    -- so this is the same answer rather than a near one.
    found = rest
      |> .char_indices
      |> .find (\\p -> !(.is_alphanumeric p.1 || p.1 == '_' || p.1 == '.' || p.1 == '-'))
      |> .map (\\p -> p.0)
    end = .unwrap_or (.len rest) found
    name = .unwrap_or "" (.get (0..end) rest)
    -- A purely numeric local is already canonical and keeps its own name:
    -- renumbering it would renumber it to something else and make two
    -- identical bodies differ.
    if .all (\\c -> .is_ascii_digit c) (.chars name)
      .push '%' out
      .push_str name out
    else
      index = case .position (\\seen -> seen == name) (.iter names) of
        Some i -> i
        None ->
          .push (.to_string name) names
          .len names - 1
      -- Piped: the line above binds a block.
      out |> .push_str (&format! \\-> "%v#{index}")
    rest := .unwrap_or "" (.get (end..(.len rest)) rest)
  out |> .push_str rest
  out
''')

write('ir/function_ir', '''
export function_ir : &str -> &str -> &Path -> Option<String>
function_ir source name scratch =
  -- The `?` outside the whole call: `rustc_path ()?` is
  -- `rustc_path(()?)`, because a parenthesised argument is where `?`
  -- binds — plan 73 §2.11.
  rustc = (rustc_path ())?
  (.ok (std::fs::create_dir_all scratch))?
  input = .join (format! \\-> "#{name}_probe.rs") scratch
  output = .join (format! \\-> "#{name}_probe.ll") scratch
  (.ok (std::fs::write (&input) (mark_target source name)))?

  -- A pipeline where the Rust chained ten `.arg`s. Nested juxtaposition would
  -- read inside-out; `Command::arg` returns `&mut Command`, which is exactly
  -- what a pipeline wants.
  --
  -- One codegen unit. With `-O` rustc splits the module across several,
  -- writes one `.ll` per unit, and `-o` then names only one of them — so a
  -- function can land in a file this never reads. The symptom is an
  -- **empty** body, which compares equal to another empty body: a vacuous
  -- pass, which the non-empty assertion in the test is there to catch.
  --
  -- It also makes the output deterministic, which a comparison needs more
  -- than it needs whatever parallelism buys.
  result = (std::process::Command::new (&rustc)
    |> .arg "-O"
    |> .arg "-Ccodegen-units=1"
    |> .arg "--emit=llvm-ir"
    |> .arg "--crate-type=lib"
    |> .arg "--crate-name"
    |> .arg "probe"
    |> .arg "-o"
    |> .arg (&output)
    |> .arg (&input)
    |> .output
    |> .ok)?
  if !(.success result.status)
    return None

  text = (.ok (std::fs::read_to_string (&output)))?
  Some (alpha_renamed (&body_of (&text) name))
''')

print('plan 76 batch 2: three files')
