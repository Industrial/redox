# -*- coding: utf-8 -*-
"""`line_col`'s test input loses the macro call it never needed.

The Rust test used `"main : ()\\nmain =\\n  println! \\\\-> \\"#{1}\\"\\n"`, and
`#{…}` is interpolation in **any** Redox string, so the literal cannot be
written directly. `r#"…"#` is not a multi-line raw string here either — the
lexer answers *"a single-quoted string may not span lines — use \\"\\"\\" for a
multi-line string"*.

The macro call is incidental: the test asserts that byte 19 resolves to line 3
column 3, and it needs a third line whose third byte is a letter. `  println 1`
is that, with the same 10 + 7 prefix, so both the slice assertion and the
offsets are unchanged and the comment explaining them stays true.
"""
import io

p = '/home/tom/Code/rust/redox/crates/redox-diag/sdx/render/line_col.sdx'
s = io.open(p, encoding='utf-8').read()

OLD = '''  src = r#"main : ()
main =
  println! \\-> "#{1}"
"#
'''
NEW = '''  -- `  println 1` where the Rust wrote `  println! \\-> "#{1}"`: `#{…}` is
  -- interpolation in any Redox string, so that literal cannot be written
  -- directly, and a multi-line raw string is not a form the lexer has. The
  -- macro call was never what this test is about — it needs a third line
  -- whose third byte is a letter, and the 10 + 7 byte prefix is unchanged, so
  -- every offset below still holds.
  src = "main : ()\\nmain =\\n  println 1\\n"
'''
assert s.count(OLD) == 1, s.count(OLD)
io.open(p, 'w', encoding='utf-8').write(s.replace(OLD, NEW, 1))
print('line_col.sdx: the test input drops the macro call')
