# -*- coding: utf-8 -*-
"""Batch 11: `Keyword`'s two mapping tables, generated from the Rust.

Fifty-seven arms each way. Typed out they would be 118 chances to transcribe a
spelling wrongly — and `round_trips_every_spelling` only checks 21 of them, so
a slip in either table could survive the suite. So both are **read out of
`keyword.rs`** and rewritten, which makes the port of this file a
transformation rather than a transcription.

`from_text` also loses a wrapper. The Rust is `Some(match text { … _ => return
None })`, which exists because a `match` cannot produce `Option` and `None` in
the same breath without one; a `case` whose arms are `Some X` and whose default
is `None` says it directly.
"""
import io
import re
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

SRC = '/home/tom/Code/rust/redox/crates/redox-lexer/src/token/keyword.rs'
text = io.open(SRC, encoding='utf-8').read()

# `"mut" => Self::Mut,`
from_text = re.findall(r'^\s*"([^"]+)" => Self::([A-Za-z0-9_]+),$', text, re.M)
# `Self::Mut => "mut",`
to_text = re.findall(r'^\s*Self::([A-Za-z0-9_]+) => "([^"]+)",$', text, re.M)

# **Checked against the enum, not against a number I counted.** The first
# version of this asserted 59 arms and there are 57, which is exactly the
# transcription error the generation exists to avoid — made while writing the
# guard against it. The enum's variant list is the real invariant: every
# variant must appear in both tables, and the tables must be inverses.
enum = text[text.index('pub enum Keyword {'):text.index('impl Keyword')]
variants = set(re.findall(r'^\s{2}([A-Z][A-Za-z0-9_]*),$', enum, re.M))
assert variants, 'no variants found — did the enum move?'
assert {c for _, c in from_text} == variants, \
    'from_text misses %s' % sorted(variants - {c for _, c in from_text})
assert {c for c, _ in to_text} == variants, \
    'text misses %s' % sorted(variants - {c for c, _ in to_text})
# The property `round_trips_every_spelling` checks for 21 of them, checked
# here for all of them.
assert dict(from_text) == {k: v for v, k in to_text}, 'the tables disagree'

arms_from = '\n'.join('    "%s" -> Some %s' % (word, ctor) for word, ctor in from_text)
arms_to = '\n'.join('    %s -> "%s"' % (ctor, word) for ctor, word in to_text)

RESERVED = ['mut', 'if', 'do', 'case', 'of', 'otherwise', 'data', 'newtype',
            'deriving', 'instance', 'module', 'not', 'or', 'guard', 'catch',
            'loop', 'Self', 'self', 'true', 'errors', 'implements']
ORDINARY = ['main', 'print', 'iff', 'switched', 'island', 'notable', 'Selfish',
            'and', 'use', 'defer', 'cond', 'into', 'derive', 'mod', 'unless',
            'switch', 'withhold', 'then']


def listing(words, indent):
    return '\n'.join('%s"%s",' % (indent, w) for w in words)


BODY = '''
instance Keyword
  -- | The reserved word `text` names, if it is one.
  -- |
  -- | A `case` producing `Some`, where the Rust wrote `Some(match text { … _ =>
  -- | return None })`. The wrapper exists in Rust because one `match` cannot
  -- | yield both an `Option` and `None` without it; arms that each say `Some`
  -- | and a default that says `None` need no wrapper at all.
  export from_text : &str -> Option<Self>
  from_text text = case text of
%s
    _ -> None

  -- | The source spelling, for diagnostics and for the formatter.
  export text : self -> &'static str
  text self = case self of
%s

test "round trips every spelling"
  for word in [
%s
  ]
    kw = .expect "should be reserved" (Keyword::from_text word)
    assert_eq! \\-> (.text kw) \\-> word

-- | The words doc 18 §7 retires are ordinary identifiers again, and that is
-- | the point of checking them here: a retired keyword that stayed reserved
-- | would reject a program that is now legal, and nothing else in the suite
-- | would notice.
test "leaves ordinary identifiers alone"
  for word in [
    -- `is` left this list in plan 60, which restored it as the pattern-test
    -- keyword — doc 03 §5.5's original spelling. `island` and `notable`
    -- stay, and they are the cases that matter: a keyword must not swallow
    -- an identifier that merely begins with it.
    --
    -- `print` is still an ordinary identifier here. Plan 64 retires the
    -- spelling with a **parser** diagnostic, `RDX0118`, rather than with a
    -- keyword, so what the lexer answers is unchanged.
%s
  ]
    assert! \\-> (.is_none (Keyword::from_text word)) \\-> "#{word} should not reserve"
''' % (arms_from, arms_to, listing(RESERVED, '    '), listing(ORDINARY, '    '))

write('token/keyword', BODY)
print('batch 11: keyword — %d arms each way, read out of the Rust' % len(from_text))
