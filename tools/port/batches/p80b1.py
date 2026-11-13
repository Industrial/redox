# -*- coding: utf-8 -*-
"""Plan 80 batch 1: the eight smallest of `voltaic-driver`'s 43 owed bodies.

Three of them are `const`s whose Rust value is written with a `\\`-at-end-of-line
continuation, so each paragraph is **one line** of runtime text wrapped across
several lines of source. Voltaic has no such continuation — plan 76 §2 found it
in `voltaic-bench`'s report literals — so each is one line here, un-escaped.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('lint/line_of', '''
export(crate) line_of : &str -> usize -> usize
line_of source offset =
  -- `source[0 .. end]` with `end` bound first, where the Rust wrote
  -- `source[..offset.min(source.len())]`. An **open range has no spelling**
  -- at either end: `..` demands both operands.
  end = .min (.len source) offset
  .count (.matches '\\n' source[0 .. end]) + 1
''')

write('lint/unwrap_message', '''
export(crate) const UNWRAP_MESSAGE : &str = "`unwrap` and `expect` panic — the ergonomic path is `?`, `??` or `catch` (doc 03 §12.7)"
''')

write('lint/shadow_help', '''
export(crate) const SHADOW_HELP : &str = "to reassign instead, write `:=` and `mut` on the first binding; if the shadow is meant, say so on the line above: `#[voltaic::allow(shadow)]`"
''')

write('lint/shadow_message', '''
export(crate) shadow_message : &str -> String
shadow_message name =
  format! \\-> "`#{name}` is already bound in this block — a bare `=` binds a second name rather than reassigning the first (plan 54 §1.1)"

-- | The name reaches the message, which is the reason it is a function.
test "it names the shadowed binding"
  assert! \\-> .starts_with "`cursor` is already bound" (shadow_message "cursor")
''')

write('lint/walk_program', '''
export(crate) walk_program : &Program -> &mut dyn ProgramWalk -> ()
walk_program program walk =
  for item in &program.items
    walk_item item walk
  ()
''')

write('lint/walk_member', '''
export(crate) walk_member : &ClassMember -> &mut dyn ProgramWalk -> ()
walk_member member walk =
  if member is ClassMember::Method decl
    walk_item (&Item::Fn (.clone decl)) walk
  ()
''')

write('cache/reporter/silent', '''
instance CacheReporter Silent
  report : &self -> &str -> bool -> ()
  report self _label _hit = ()

test "accepts everything and says nothing"
  .report "a.sdx" true Silent
  .report "a.sdx" false Silent
''')

write('lint/comment_pragma_sites', '''
export(crate) comment_pragma_sites : &str -> Vec<(Span, Lint)>
comment_pragma_sites source =
  guard voltaic_lexer::lex source is Ok tokens else
    -- Nothing to say about a file whose own lexical error is the diagnostic.
    return Vec::new ()

  tokens
    |> .iter
    |> .flat_map (\\t -> .chain (.as_ref t.trailing) (.iter t.leading))
    |> .filter_map (\\trivium -> .map (\\lint -> (trivium.span, lint)) (comment_pragma trivium))
    |> .collect
''')

print('plan 80 batch 1: eight files')
