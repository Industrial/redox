# -*- coding: utf-8 -*-
"""Plan 78 batch 5: `semantic_tokens` and `token_index`.

`token_index` is `Some(match … _ => return None)` — the wrapper Rust needs
because one `match` cannot yield both an `Option` and `None`. A `case` whose
arms say `Some` and whose default says `None` needs no wrapper, which is the
same simplification `Keyword::from_text` got in plan 74.

`data` is a field name again, so `r#data` twice more.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('serve/semantic_tokens/token_index', '''
export(crate) token_index : &TokenKind -> Option<u32>
token_index kind = case kind of
  -- 0: KEYWORD
  TokenKind::Kw _ -> Some 0
  -- 1: TYPE — a lifetime or loop label. Neither is a type, and `TYPE` is
  -- the legend entry whose usual colour suits them: both are a short
  -- `'name` that readers scan past rather than read.
  TokenKind::Label _ -> Some 1
  -- 3: VARIABLE — every identifier. Telling a function name from a
  -- variable needs to know what the name *is*, which is name resolution,
  -- which Redox does not have (plan 40 §6). So `FUNCTION`, index 2, is in
  -- the legend and unused: it is what a future row with a symbol table
  -- would reach for, and removing it would renumber everything after it.
  TokenKind::Ident _ -> Some 3
  -- 4: NUMBER
  TokenKind::Int _ or TokenKind::Float _ or TokenKind::Byte _ -> Some 4
  -- 5: STRING
  TokenKind::Str { .. } or TokenKind::RawStr { .. } or TokenKind::ByteStr _ or TokenKind::CStr _ or TokenKind::Char _ -> Some 5
  -- 7: COMMENT is in the legend and unreachable from here, because a
  -- comment is **trivia** and never a token. Doc 02 §10.1 made that choice
  -- and this is where it shows: an editor colours comments from its own
  -- syntax file, and a server that wanted to own them would have to publish
  -- trivia as tokens.
  --
  -- No `Some(match …)` wrapper: Rust needs one because a single `match`
  -- cannot yield both an `Option` and a bare `None`, and arms that each say
  -- `Some` do not.
  TokenKind::Newline or TokenKind::Indent or TokenKind::Dedent or TokenKind::Eof -> None
  -- 6: OPERATOR — everything else, which is punctuation.
  _ -> Some 6
''')

write('serve/semantic_tokens', '''
export(crate) semantic_tokens : &str -> SemanticTokensResult
semantic_tokens source =
  tokens = case redox_lexer::lex source of
    Ok lexed -> lexed
    Err _ ->
      return SemanticTokensResult::Tokens (SemanticTokens {
        result_id = None,
        r#data = Vec::new (),
      })

  mut collected : Vec<SemanticToken> = Vec::new ()
  mut last_line : u32 = 0
  mut last_start : u32 = 0
  for token in &tokens
    kind = case token_index (&token.kind) of
      Some index -> index
      None ->
        continue
    start = to_position source token.span.0
    end = to_position source token.span.1
    -- A token spanning a line break cannot be delta-encoded as one token, and
    -- the protocol has no shape for it. Multi-line strings are the real case
    -- — `examples/strings.sdx`'s block string — and they are skipped rather
    -- than truncated, because a token whose length ran past the end of its
    -- line would shift every later token on that line.
    if end.line != start.line
      continue
    delta_line = start.line - last_line
    delta_start = if delta_line == 0
      start.character - last_start
    else
      start.character
    collected |> .push (SemanticToken {
      delta_line,
      delta_start,
      length = end.character - start.character,
      token_type = kind,
      token_modifiers_bitset = 0,
    })
    last_line := start.line
    last_start := start.character

  SemanticTokensResult::Tokens (SemanticTokens {
    result_id = None,
    r#data = collected,
  })
''')

print('plan 78 batch 5: two files')
