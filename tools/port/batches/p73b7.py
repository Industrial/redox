# -*- coding: utf-8 -*-
"""voltaic-diag batch 7 — parse_map, the crate's most intricate control flow.

Three shapes first met here:

* **No turbofish.** `f.parse::<u32>()` becomes a binding with an ascription,
  `v : u32 = .parse f`, which is the form plan 69 already used for
  `.collect`.
* **A slice pattern with a literal** — `[_, "-"]` — which Voltaic writes the
  same way.
* **A local closure used with `?`.** `num` captures `expected` and returns a
  `Result`, so each call site keeps its own `?`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

BODY = '''-- | Read a `.xrs.map` sidecar back.
-- |
-- | Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 4. Every failure is an `Err`
-- | with a sentence, never a partial map: a map that silently dropped records
-- | would resolve some lines and decline others, and the declined ones are
-- | indistinguishable from genuinely synthetic lines. Refusing the whole file
-- | makes the caller fall back to reporting against the `.xrs`, which is worse
-- | output but not *wrong* output.
-- |
-- | The span is not stored in the sidecar — the consumer (plan 07) needs line
-- | and column, and a byte range it would have to recompute the text for adds
-- | a field that can disagree with the other three. A parsed origin therefore
-- | carries an empty span, which is the one way a parsed map differs from the
-- | map the emitter produced.
export parse_map : &str -> Result<SourceMap, String>
parse_map text =
  mut lines = .lines text
  -- The `?` outside the `.ok_or`, not inside its receiver: the inner form
  -- unwraps first and then calls `Option::ok_or` on the value.
  header = (.ok_or "the map is empty" (.next lines))?

  mut parts = .split ' ' header
  if .next parts != Some "voltaic-map"
    return Err (.to_string "not a Voltaic source map: the header does not begin `voltaic-map`")
  -- `version : u32 = .parse …` where the Rust wrote `.parse::<u32>()`: Voltaic
  -- has no turbofish, and an ascription says the same thing.
  raw = (.ok_or "the map header has no version" (.next parts))?
  version : Result<u32, _> = .parse raw
  version = .map_err (\\_ -> .to_string "the map header's version is not a number") version?
  if version != VERSION
    return Err (format! \\-> "this map is version #{version}; this Voltaic writes version #{VERSION}")
  collected : Vec<&str> = .collect parts
  source_path = .join " " collected
  if .is_empty source_path
    return Err (.to_string "the map header names no source file")

  mut origins = Vec::new ()
  for pair in .enumerate lines
    i = pair.0
    line = pair.1
    if .is_empty line
      continue
    fields : Vec<&str> = .collect (.split ' ' line)
    expected = i + 1
    -- `.ok_or` with the message built eagerly, where the Rust wrote
    -- `.ok_or_else(|| …)`. A **nullary lambda in argument position makes
    -- `voltaic fmt` diverge**: `power_of` declines on a lambda so `argument`
    -- parenthesises it, and the parenthesised form is an argument again on the
    -- next pass — one layer per run, without bound. Plan 73 measured it and
    -- left the fix to its own row.
    --
    -- Eager costs nothing here: the `?` above has already established the
    -- record exists, so the message is formatted exactly as often either way.
    gen_line : usize = (fields
      |> .first
      |> .and_then (\\f -> .ok (.parse f))
      |> .ok_or (format! \\-> "record #{expected} does not begin with a line number"))?
    if gen_line != expected
      return Err (format! \\-> "record #{expected} claims to be line #{gen_line}; the records must be in order with no gaps")

    case .as_slice fields of
      [_, "-"] -> .push None origins
      [_, src_line, src_col, gen_col] ->
        -- Three ascribed bindings where the Rust had one annotated closure.
        -- `|f: &str, what: &str|` annotated its parameters for inference, and
        -- a **lambda parameter in Voltaic is a bare name** — neither a pattern
        -- nor an annotation, which is plan 69's third finding wider than it
        -- was recorded. Without the annotation the closure cannot be written,
        -- and `Result<usize, _>` on a binding says the same thing with the
        -- field named in its own message.
        parsed_src_line : Result<usize, _> = .parse src_line
        parsed_src_col : Result<usize, _> = .parse src_col
        parsed_gen_col : Result<usize, _> = .parse gen_col
        origin = Origin {
          span = (0, 0),
          src_line = (.map_err (\\_ -> format! \\-> "record #{expected}'s source line is not a number") parsed_src_line)?,
          src_col = (.map_err (\\_ -> format! \\-> "record #{expected}'s source column is not a number") parsed_src_col)?,
          gen_col = (.map_err (\\_ -> format! \\-> "record #{expected}'s generated column is not a number") parsed_gen_col)?,
        }
        if origin.src_line == 0 || origin.src_col == 0 || origin.gen_col == 0
          return Err (format! \\-> "record #{expected} has a zero line or column; both are 1-based")
        .push (Some origin) origins
      _ ->
        return Err (format! \\-> "record #{expected} has #{(.len fields)} fields; expected 2 (`n -`) or 4 (`n src_line src_col gen_col`)")

  Ok (SourceMap::new source_path origins)
'''

print('%-40s written' % write('source/map/parse_map', BODY))
