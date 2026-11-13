# -*- coding: utf-8 -*-
"""Plan 80 batch 8: the two pragma readers.

`unwrap_call_sites` is the first ported body whose `filter_map` closure is a
`match` — so the first to need a **multi-line lambda holding a `case`**, with
the closing parenthesis on the `_ -> None` arm. Plan 68 established the shape
in `where_edges.sdx`; this is where it earns its keep.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lint/attribute_pragma', '''
export(crate) attribute_pragma : &Trivia -> Option<Lint>
attribute_pragma trivium = case trivium.kind of
  -- A `case` where the Rust negated `matches!`: the macro's second argument
  -- is a pattern, so a `case` is the spelling.
  TriviaKind::Attribute -> Lint::from_allow_attribute (.trim trivium.text)
  _ -> None

-- | `#[cfg(test)]`: the trivium every test below classifies.
#[cfg(test)]
trivium : TriviaKind -> &str -> Trivia
trivium kind text = Trivia { kind, text = .to_string text, span = (0, 0) }

test "the attribute suppresses"
  assert_eq! \\-> attribute_pragma (&trivium TriviaKind::Attribute "voltaic::allow(unwrap)") \\-> Some Lint::Unwrap

-- | The second lint, which is the whole of what parametrising bought: one
-- | predicate, and the name decides which warning is silenced.
test "each lint is suppressed by its own name"
  assert_eq! \\-> attribute_pragma (&trivium TriviaKind::Attribute "voltaic::allow(shadow)") \\-> Some Lint::Shadow

-- | A name no lint has is **not** a pragma, so the emitter does not consume
-- | it either and rustc names it — which is the loud failure the
-- | one-spelling rule relies on.
test "an unknown lint name is not a pragma"
  assert_eq! \\-> attribute_pragma (&trivium TriviaKind::Attribute "voltaic::allow(unwarp)") \\-> None

-- | The namespace, which is what keeps `#[allow(unwrap)]` rustc's.
test "an unnamespaced allow is rustc's"
  assert_eq! \\-> attribute_pragma (&trivium TriviaKind::Attribute "allow(unwrap)") \\-> None

-- | The whitespace decision, pinned rather than left to a reader: one
-- | spelling, and a generous one reaches rustc as `E0433` rather than being
-- | silently ignored.
test "whitespace inside the brackets is not normalised"
  assert_eq! \\-> attribute_pragma (&trivium TriviaKind::Attribute "voltaic :: allow( unwrap )") \\-> None

-- | An *inner* attribute applies to the enclosing module, not to the next
-- | construct, so it is not this pragma.
test "an inner attribute does not suppress"
  assert_eq! \\-> attribute_pragma (&trivium TriviaKind::InnerAttribute "voltaic::allow(unwrap)") \\-> None
''')

write('lint/unwrap_call_sites', '''
export(crate) unwrap_call_sites : &str -> Option<&Program> -> Vec<Span>
unwrap_call_sites source program =
  guard voltaic_lexer::lex source is Ok tokens else
    -- A file that does not lex has no call sites worth reporting: the lexical
    -- error is the diagnostic, and this pass only ever runs after a successful
    -- parse anyway.
    return Vec::new ()

  -- Lines carrying the pragma, from the trivia on every token.
  suppressed : Vec<usize> = tokens
    |> .iter
    |> .flat_map (\\t -> .chain (.as_ref t.trailing) (.iter t.leading))
    |> .filter (\\trivium -> allow_pragma trivium Lint::Unwrap)
    |> .map (\\trivium -> line_of source trivium.span.0)
    |> .collect

  -- Spans of the constructs a pragma leads, which the lines above cannot
  -- reach.
  --
  -- `p`, where the Rust's closure reused `program`: `RDX1003` reports a
  -- shadow.
  ranges = program
    |> .map (\\p -> suppressed_spans p Lint::Unwrap)
    |> .unwrap_or_default

  tokens
    |> .windows 2
    |> .filter (\\pair -> pair[0].kind == TokenKind::Dot)
    |> .filter_map (\\pair ->
    case &pair[1].kind of
      TokenKind::Ident name | name == "unwrap" || name == "expect" -> Some pair[1].span
      _ -> None)
    |> .filter (\\span -> !.contains (&line_of source span.0) suppressed)
    |> .filter (\\span -> !.any (\\r -> r.0 <= span.0 && span.1 <= r.1) (.iter ranges))
    |> .collect
''')

print('plan 80 batch 8: two files')
