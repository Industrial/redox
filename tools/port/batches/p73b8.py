# -*- coding: utf-8 -*-
"""redox-diag batch 8 — diagnostic/lint, including its five tests.

New shapes: `&'static [Lint]` as a return type with a named lifetime that
*should* be declared nowhere (it is `'static`, which `type_vars` already
excludes), and `.iter().copied().find(|…| …)` over a static slice.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

BODY = '''instance Lint
  -- | Every suppressible lint, which is what makes recognition total.
  -- A nullary signature is the return type alone — `() -> T` would declare
  -- one unit argument, as `examples/hello.sdx`'s `main : ()` shows.
  export all : &'static [Lint]
  all = &[Lint::Unwrap, Lint::Shadow]

  -- | The name an author writes inside the pragma.
  -- |
  -- | Short and not the code, because a pragma is read far more often than it
  -- | is written and `#[redox::allow(RDX1003)]` says nothing to a reader who
  -- | does not already know the catalogue.
  export name : self -> &'static str
  name self = case self of
    Lint::Unwrap -> "unwrap"
    Lint::Shadow -> "shadow"

  -- | The code this lint raises, for a diagnostic that has to cite it.
  export code : self -> Code
  code self = case self of
    Lint::Unwrap -> Code::UnwrapUsed
    Lint::Shadow -> Code::ShadowedBinding

  -- | The suppression attribute as the **lexer stores it**.
  -- |
  -- | The source spelling is `#[redox::allow(unwrap)]`; an attribute trivium
  -- | carries the text *between* the brackets, which is this.
  -- |
  -- | The `redox::` namespace is load-bearing: `#[allow(unwrap)]` goes on
  -- | meaning rustc's lint and nothing else, so Redox's own attributes can
  -- | never collide with a Rust one.
  export allow_attribute : self -> String
  allow_attribute self = format! \\-> "redox::allow(#{(.name self)})"

  -- | The **deprecated** comment spelling, as the lexer stores it.
  -- |
  -- | The source spelling is `-- redox: allow unwrap`; a comment trivium
  -- | carries the text after the dashes. Still a suppression, and
  -- | additionally an `RDX1002` — plan 53 §1.3 reverses plan 38 §1.3 on
  -- | evidence plan 38 did not have, because plan 48 found `redox fmt`
  -- | deleting comments from a `.sdx` and a suppression a formatter can remove
  -- | is worse than no suppression.
  -- |
  -- | The `redox:` namespace is load-bearing here too: `allow shadow` alone
  -- | would turn an ordinary English sentence into a pragma by accident.
  export allow_comment : self -> String
  allow_comment self = format! \\-> "redox: allow #{(.name self)}"

  -- | The lint an attribute trivium's text suppresses, if it is one of these.
  -- |
  -- | The emitter's half of the shared spelling: an attribute that answers
  -- | `Some` is Redox's own and is **consumed**, and one that answers `None`
  -- | is the author's and is passed through.
  export from_allow_attribute : &str -> Option<Self>
  from_allow_attribute text = Lint::all ()
    |> .iter
    |> .copied
    |> .find (\\lint -> text == .allow_attribute lint)

  -- | The lint a comment trivium's text suppresses, if it is one of these.
  export from_allow_comment : &str -> Option<Self>
  from_allow_comment text = Lint::all ()
    |> .iter
    |> .copied
    |> .find (\\lint -> text == .allow_comment lint)

-- | The spellings plan 53 published, which must not move: a program in
-- | outside code carries them as text.
test "the published spellings are unchanged"
  assert_eq! \\-> (.allow_attribute Lint::Unwrap) \\-> "redox::allow(unwrap)"
  assert_eq! \\-> (.allow_comment Lint::Unwrap) \\-> "redox: allow unwrap"

-- | Recognition is the inverse of the spelling, for every lint — which is
-- | the property hard-coded strings could not have.
test "every lint round trips through both spellings"
  for lint in Lint::all ()
    assert_eq! \\-> (Lint::from_allow_attribute (&(.allow_attribute lint))) \\-> (Some (*lint))
    assert_eq! \\-> (Lint::from_allow_comment (&(.allow_comment lint))) \\-> (Some (*lint))

-- | Two names would mean one pragma suppressing two lints.
test "every name is unique"
  mut names : Vec<&str> = Lint::all () |> .iter |> .map (\\l -> .name l) |> .collect
  .sort_unstable names
  before = .len names
  .dedup names
  assert_eq! \\-> before \\-> (.len names) \\-> "two lints share a pragma name"

-- | A name is one lowercase word, because it is written inside brackets and
-- | compared as exact text.
test "every name is one lowercase word"
  for lint in Lint::all ()
    name = .name lint
    assert! \\-> (.all (\\c -> .is_ascii_lowercase c || c == '_') (.chars name)) \\-> "#{name} is not a lowercase word"

-- | The cross-spelling confusion the enum exists to prevent: a comment's
-- | text is not an attribute's and never suppresses as one.
test "the two spellings do not cross"
  assert_eq! \\-> (Lint::from_allow_attribute "redox: allow unwrap") \\-> None
  assert_eq! \\-> (Lint::from_allow_comment "redox::allow(unwrap)") \\-> None
'''

print('%-40s written' % write('diagnostic/lint', BODY))
