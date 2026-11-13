# -*- coding: utf-8 -*-
"""Plan 78 batch 1: six of `voltaic-lsp`'s twenty bodies.

**`cast` had to change shape, and the reason is worth stating.** Its Rust is
generic over `R: RequestTrait` and mentions `R` *only* as `R::Params` and
`R::METHOD`, so callers pick it with a turbofish — `cast::<Formatting>(request)`.
Voltaic has no turbofish, and ascription cannot stand in: `R` is not inferable
from `R::Params`, which is not injective. `type_vars` would not even register
`r` as a variable, because it skips an identifier adjacent to `::` — the rule
that keeps `fmt` in `fmt::Result` from becoming a type parameter.

So the discriminator becomes a **value**: `cast : DeserializeOwned p => &Request
-> &str -> Result<p, …>`, and the caller ascribes the params type it wants.
That is more explicit than the turbofish and no longer than it, and it is the
general answer — **a function generic only through an associated type must take
its discriminator as an argument.**

Two turbofishes elsewhere become ascriptions, which is the documented
replacement: `sum::<usize>()` and `extract::<P>(…)`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('serve/diagnostics/path_of', '''
export(crate) path_of : &Uri -> std::path::PathBuf
path_of uri =
  text = .as_str uri
  .to_path_buf (Path::new (.unwrap_or text (.strip_prefix "file://" text)))
''')

write('serve/cast', '''
import serde::de::DeserializeOwned

-- | **The method is a value, where the Rust made it a turbofish.**
-- |
-- | `cast::<Formatting>(request)` picks the request type at the call site, and
-- | the Rust's `R` appears only as `R::Params` and `R::METHOD`. Voltaic has no
-- | turbofish, and ascription cannot stand in — `R` is not inferable from
-- | `R::Params`, and `type_vars` would not register `r` at all, because it
-- | skips an identifier adjacent to `::` so that `fmt` in `fmt::Result` is not
-- | a type parameter.
-- |
-- | So the discriminator is passed: the caller writes the method and ascribes
-- | the params type. More explicit than the turbofish, no longer, and the
-- | general answer — a function generic only through an associated type has to
-- | take its discriminator as an argument.
export(crate) cast : DeserializeOwned p => &Request -> &str -> Result<p, ExtractError<Request>>
cast request method =
  extracted = .extract method (.clone request)
  .map (\\pair -> pair.1) extracted
''')

write('serve/positions/range_of', '''
export(crate) range_of : &str -> (usize, usize) -> Range
range_of source span = Range {
  start = position_of source span.0,
  end = position_of source span.1,
}
''')

write('capabilities/token_types', '''
export(crate) token_types : Vec<SemanticTokenType>
token_types = vec! \\-> SemanticTokenType::KEYWORD \\-> SemanticTokenType::TYPE \\-> SemanticTokenType::FUNCTION \\-> SemanticTokenType::VARIABLE \\-> SemanticTokenType::NUMBER \\-> SemanticTokenType::STRING \\-> SemanticTokenType::OPERATOR \\-> SemanticTokenType::COMMENT
''')

write('serve/positions/position_of', '''
export(crate) position_of : &str -> usize -> Position
position_of source offset =
  -- `at`, not a rebinding of `offset`: `RDX1003` reports a shadow.
  at = .min (.len source) offset
  -- Closed ranges: a parenthesised half-open range is an operator section.
  before = .unwrap_or "" (.get (0..at) source)
  line = .count (.matches '\\n' before)
  line_start = .map_or 0 (\\a -> a + 1) (.rfind '\\n' before)
  -- An ascription where the Rust had `sum::<usize>()`: doc 03 §18 has no
  -- turbofish, and a binding's type says the same thing.
  character : usize = .unwrap_or "" (.get (line_start..at) source)
    |> .chars
    |> .map char::len_utf16
    |> .sum
  Position {
    line = .unwrap_or u32::MAX (.ok (u32::try_from line)),
    character = .unwrap_or u32::MAX (.ok (u32::try_from character)),
  }
''')

write('serve/reply', '''
import serde::Serialize

export(crate) reply : Serialize t => RequestId -> Result<p, ExtractError<Request>> -> (impl FnOnce(p) -> t) -> Response
reply id params handle = case params of
  Ok got -> Response {
    id,
    response_result = Ok (.unwrap_or serde_json::Value::Null (serde_json::to_value (handle got))),
  }
  Err _ -> Response {
    id,
    response_result = Ok serde_json::Value::Null,
  }
''')

# `RequestTrait` bounded the type parameter that is gone, so the import the
# translator carried over is now unused — and `-D warnings` makes that an
# error. Removed after `write()`, because `write()` *merges* the head's imports
# and would carry it back on every rebuild.
import io  # noqa: E402

P = '/home/tom/Code/rust/voltaic/crates/voltaic-lsp/sdx/serve/cast.sdx'
s = io.open(P, encoding='utf-8').read()
DROP = 'import lsp_types::request::Request as RequestTrait\n'
if DROP in s:
    io.open(P, 'w', encoding='utf-8').write(s.replace(DROP, '', 1))
    print('cast.sdx: the import its type parameter needed is gone with it')
else:
    print('cast.sdx: already without RequestTrait')

print('plan 78 batch 1: six files')
