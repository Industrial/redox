# -*- coding: utf-8 -*-
"""Plan 81 batch 12: the two tables and the escape translator.

`rust_escapes`'s tests are the second place raw literals earn their keep —
every fixture is a backslash sequence, and `r"\\n"` is the only way to write
one that reaches the function as two characters rather than a newline.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/redox/tools/port')
from write import write  # noqa: E402

write('emit/arity_table', '''
instance ArityTable
  -- | Index the signatures `items` declares, inline `module` bodies included.
  export(crate) from_items : &[Item] -> Self
  from_items items =
    mut table = ArityTable::default ()
    mut ambiguous = Vec::new ()
    .gather items (&mut ambiguous) table
    for name in ambiguous
      .remove (&name) table.arities
    table

  -- | Index one item list, recording any name whose arity clashes.
  gather : &[Item] -> &mut Vec<String> -> &mut self -> ()
  gather items ambiguous self =
    for item in items
      case item of
        Item::Fn decl ->
          arity = .len decl.sig.params
          clash = .is_some_and (\\previous -> previous != arity) (.insert (.clone decl.name.text) arity self.arities)
          if clash
            .push (.clone decl.name.text) ambiguous
        Item::Module decl ->
          if &decl.items is Some inner
            .gather inner ambiguous self
        _ -> ()
    ()

  -- | The declared arity of `name`, when this file declares exactly one
  -- | function by that name.
  export(crate) arity_of : &str -> &self -> Option<usize>
  arity_of name self = .copied (.get name self.arities)
''')

write('emit/binding_table', '''
instance BindingTable
  -- | Index the parameterless bindings `items` declares, inline `module`
  -- | bodies included.
  export(crate) from_items : &[Item] -> Self
  from_items items =
    mut table = BindingTable::default ()
    mut ambiguous = Vec::new ()
    .gather items (&[]) (&mut ambiguous) table
    for name in ambiguous
      .remove (&name) table.kinds
    table

  -- | Index one item list under a module prefix, recording any name whose kind clashes.
  gather : &[Item] -> &[String] -> &mut Vec<String> -> &mut self -> ()
  gather items prefix ambiguous self =
    for item in items
      case item of
        Item::Fn decl ->
          guard classify decl is Some kind else
            continue
          bare = .clone decl.name.text
          .insert (.clone bare) kind ambiguous self
          if !.is_empty prefix
            mut path = .join "::" prefix
            .push_str "::" path
            .push_str (&bare) path
            .insert path kind ambiguous self
        Item::Module decl ->
          if &decl.items is Some inner
            mut nested = .to_vec prefix
            .push (.clone decl.name.text) nested
            .gather inner (&nested) ambiguous self
        _ -> ()
    ()

  -- | Record one name's kind, noting it as ambiguous if it already had another.
  insert : String -> BindingKind -> &mut Vec<String> -> &mut self -> ()
  insert key kind ambiguous self =
    if .is_some_and (\\previous -> previous != kind) (.insert (.clone key) kind self.kinds)
      .push key ambiguous
    ()

  -- | What `name` means in the file being emitted, when it declares exactly one
  -- | parameterless binding by that name.
  export(crate) kind_of : &str -> &self -> Option<BindingKind>
  kind_of name self = .copied (.get name self.kinds)
''')

write('emit/rust_escapes', '''
export(crate) rust_escapes : &str -> String
rust_escapes raw =
  mut out = String::with_capacity (.len raw)
  mut chars = .chars raw
  while .next chars is Some ch
    if ch != '\\\\'
      .push ch out
      continue
    case .next chars of
      -- The one escape Rust does not have. `\\#` means `#`, and `#` needs no
      -- escaping in Rust, so the backslash is dropped.
      Some '#' -> .push '#' out
      -- Every other escape is Rust's too; pass both characters through. This
      -- includes `\\\\`, which must consume its second backslash here or a
      -- following `#` would be read as an escape it is not part of.
      Some other ->
        .push '\\\\' out
        .push other out
      -- A trailing backslash cannot occur in a token the lexer accepted, but
      -- emitting it unchanged is better than dropping a character.
      None -> .push '\\\\' out
  out

-- | Every fixture below is a **raw** literal — doc 03 §11.2's `r"…"`, which
-- | reaches this function as the characters it names rather than as the
-- | escapes they would otherwise spell.
test "translates the one escape rust lacks"
  assert_eq! \\-> rust_escapes r#""a\\#{b""# \\-> r#""a#{b""#

test "leaves every shared escape alone"
  for escape in [r"\\n", r"\\t", r"\\r", r"\\0", r#"\\""#, r"\\'", r"\\x41", r"\\u{1F600}"]
    assert_eq! \\-> rust_escapes escape \\-> escape \\-> "#{escape} should be untouched"

-- | The case a naive `replace("\\\\#", "#")` gets wrong: a literal backslash
-- | followed by a `#` is not a `\\#` escape, and must keep its backslash.
test "does not mistake an escaped backslash for an escaped hash"
  assert_eq! \\-> rust_escapes r"a\\\\#b" \\-> r"a\\\\#b"
  assert_eq! \\-> rust_escapes r"a\\\\\\#b" \\-> r"a\\\\#b"

test "leaves text with no escapes alone"
  assert_eq! \\-> rust_escapes "hello world" \\-> "hello world"
  assert_eq! \\-> rust_escapes "" \\-> ""

test "survives a trailing backslash"
  assert_eq! \\-> rust_escapes r"a\\" \\-> r"a\\"
''')

print('plan 81 batch 12: three files')
