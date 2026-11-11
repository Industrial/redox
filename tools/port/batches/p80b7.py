# -*- coding: utf-8 -*-
"""Plan 80 batch 7: three more, and `errors` collides again.

`diagnose` binds `ParseFailure::Parse(errors)` in Rust, and `errors` is doc 03
§14's declaration keyword — the same collision `compile_dir/with` hit one batch
ago, on a name the Rust chose freely both times. `of` is the other: a path
segment spelled `Fingerprint::of` is `case … of`'s keyword, so the method is
declared and called as `r#of`.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('lint/comment_pragma', '''
export(crate) comment_pragma : &Trivia -> Option<Lint>
comment_pragma trivium = case trivium.kind of
  -- A `case` where the Rust negated `matches!`: the macro's second argument
  -- is a pattern, so a `case` is the spelling, and it reads better the
  -- right way round.
  TriviaKind::LineComment -> Lint::from_allow_comment (.trim trivium.text)
  _ -> None

-- | `#[cfg(test)]`: the trivium every test below classifies.
#[cfg(test)]
trivium : TriviaKind -> &str -> Trivia
trivium kind text = Trivia { kind, text = .to_string text, span = (0, 0) }

test "the pragma suppresses"
  assert_eq! \\-> comment_pragma (&trivium TriviaKind::LineComment "redox: allow unwrap") \\-> Some Lint::Unwrap

-- | The second lint reaches the deprecated spelling too, because an author
-- | who learned the comment form has no reason to know it is per-lint.
test "each lint is suppressed by its own name"
  assert_eq! \\-> comment_pragma (&trivium TriviaKind::LineComment "redox: allow shadow") \\-> Some Lint::Shadow

-- | The `redox:` namespace, which is why the pragma is not a sentence.
test "an unrelated comment does not suppress"
  assert_eq! \\-> comment_pragma (&trivium TriviaKind::LineComment "allow unwrap here, it cannot fail") \\-> None

-- | A doc comment describes the item for a reader of the generated Rust;
-- | giving it a second job would let prose about `unwrap` silence a warning.
test "a doc comment does not suppress"
  assert_eq! \\-> comment_pragma (&trivium TriviaKind::DocComment "redox: allow unwrap") \\-> None
''')

write('cache/fingerprint', '''
instance Fingerprint
  -- | The fingerprint of the running compiler.
  -- |
  -- | If `current_exe` or the read fails, the fingerprint is derived from
  -- | something that differs every call, so every lookup misses. That is
  -- | the safe direction: a cache that cannot identify its compiler must not
  -- | claim its results are reusable.
  -- |
  -- | # Why the clock and not the pid
  -- |
  -- | This fallback used `std::process::id()`, which is the obvious thing and
  -- | **panics** on `wasm32-wasip1` — *"no pids on this platform"*, from
  -- | `std`'s `process::unsupported`. Both halves fail on that target:
  -- | `current_exe` is unsupported too, so the fallback is always taken, and
  -- | then it aborts. Plan 72 leaf 2 found it by running `redox-seed` under
  -- | `wasmtime`, 47 files into a 53-file tree.
  -- |
  -- | The wall clock exists wherever `std` does — WASI has `clock_time_get` —
  -- | and it satisfies the paragraph above better than a pid did: a pid is
  -- | reused between runs, so two different compilers could share a
  -- | fingerprint, where a nanosecond timestamp effectively cannot. If even
  -- | the clock fails there is nothing left to ask, and zero is as good a
  -- | constant as any: the cache is then useless rather than wrong, which is
  -- | the direction this function has always chosen.
  export current : Self
  current = case .and_then std::fs::read (std::env::current_exe ()) of
    Ok bytes -> Fingerprint::r#of (&bytes)
    Err _ -> Fingerprint::r#of (&.to_le_bytes (Fingerprint::now ()))

  -- | Nanoseconds since the epoch, or zero if the clock refuses.
  now : u128
  now =
    std::time::SystemTime::now ()
      |> .duration_since std::time::UNIX_EPOCH
      |> .map_or 0 (\\d -> .as_nanos d)

  -- | A fingerprint of arbitrary bytes.
  -- |
  -- | Public so a test can vary the compiler's identity without rebuilding the
  -- | compiler — plan 10 criterion 3 would otherwise be untestable in practice,
  -- | and an untested invalidation rule is the kind that quietly stops working.
  -- |
  -- | `r#of`, because `of` is `case … of`'s keyword.
  export r#of : &[u8] -> Self
  r#of bytes = Fingerprint (blake3::hash bytes)

  -- | The hash's 32 raw bytes, for mixing into a cache key.
  export(crate) as_bytes : &self -> &[u8; 32]
  as_bytes self = .as_bytes self.0

test "differs for different bytes"
  assert_ne! \\-> Fingerprint::r#of b"compiler-a" \\-> Fingerprint::r#of b"compiler-b"

test "is stable for the same bytes"
  assert_eq! \\-> Fingerprint::r#of b"same" \\-> Fingerprint::r#of b"same"

-- | The running compiler has *a* fingerprint, and asking twice agrees.
test "the current compiler has a stable fingerprint"
  assert_eq! \\-> Fingerprint::current () \\-> Fingerprint::current ()
''')

write('diagnose', '''
export diagnose : &Path -> &str -> &ParseFailure -> Vec<Diagnostic>
diagnose path source failure =
  -- `shown`, not a rebinding of `path`: `RDX1003` reports a shadow.
  shown = .to_string (.display path)
  case failure of
    ParseFailure::Lex e ->
      vec! \\-> (Diagnostic { code = .into e.code, severity = redox_diag::Severity::Error, message = .clone e.message, span = e.span, help = .clone e.help, path = shown, source = .to_string source })
    -- `problems`, where the Rust bound `errors`: `errors` is doc 03 §14's
    -- declaration keyword.
    ParseFailure::Parse problems ->
      problems
        |> .iter
        |> .map (\\e -> Diagnostic { code = .into e.code, severity = redox_diag::Severity::Error, message = .clone e.message, span = e.span, help = .clone e.help, path = .clone shown, source = .to_string source })
        |> .collect

test "carries a lex errors code and help across"
  source = "main : ()\\nmain =\\n\\tprint 1\\n"
  failure = .expect_err "a tab is an error" (redox_parser::parse_source source)
  diagnostics = diagnose (Path::new "a.sdx") source (&failure)
  assert_eq! \\-> .len diagnostics \\-> 1
  assert_eq! \\-> .text diagnostics[0].code \\-> "RDX0001"
  assert_eq! \\-> diagnostics[0].path \\-> "a.sdx"
  assert! \\-> .is_some diagnostics[0].help \\-> "the lexer's help survives"

-- | Recovered parse errors arrive as a batch and all of them are reported —
-- | one diagnostic per error, which is what plan 05 criterion 5 asserts end
-- | to end.
test "reports every recovered parse error"
  -- Both errors are bounded to their own statement, which is what keeps
  -- them *independent*: an error that runs off the end of its line lets
  -- statement recovery swallow the following signature line, and the two
  -- collapse into one. That is true of doc 18's grammar and not of doc
  -- 03's, because a signature line is now something recovery can land in
  -- the middle of.
  --
  -- `print (1 +)` was the first of them until doc 18 §4 made it a **left
  -- section**, which parses. A statement that begins with a binary
  -- operator has nowhere left to go, and ends at its own line break.
  source = "a : ()\\na =\\n  + 1\\n\\nb : ()\\nb =\\n  * \\n"
  failure = .expect_err "should fail" (redox_parser::parse_source source)
  diagnostics = diagnose (Path::new "a.sdx") source (&failure)
  count = .len diagnostics
  assert! \\-> count >= 2 \\-> "expected a diagnostic per error, got #{count}"
  assert! \\-> .all (\\d -> d.source == source) (.iter diagnostics)
''')

print('plan 80 batch 7: three files')
