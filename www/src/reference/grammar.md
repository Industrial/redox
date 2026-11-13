# Grammar

The grammar has two independent statements, and they are checked against each
other.

## The parser

`crates/voltaic-parser` is hand-written recursive descent. Not generated — a
hand-written parser is what produces diagnostics worth reading, and
diagnostics are the thing a source-to-source compiler lives or dies by.

## The tree-sitter grammar

`tree-sitter-voltaic/` carries a second grammar, used by editors for
highlighting and structural selection. It is generated from `grammar.js` with
a pinned `tree-sitter`, because a generated parser is an artifact of its
generator and an unpinned one cannot be regenerated identically.

## They are checked against each other

`docs/GRAMMAR.md` is the row-by-row grading of every Rust construct, and
`grammar_inventory.rs` asserts that the `.sdx` paths it cites exist. A test
also asks the *lexer* — not `grammar.js` — whether every highlighted keyword is
still in the grammar, because `grammar.js` is the thing most likely to be
behind.

## Two traps, both measured

**A flat `repeat1(choice(...))` hides a missing token.** The grammar gates were
green before `grammar.js` knew `>>>`, because a flat alternation accepts `>>`
then `>` with no error node.

**A precedence can intercept a rule you read correctly.** `instance Convert f64
Meters` plainly parses by the instance rule, and is still rejected, because the
constraint rule's `prec(1)` matches `Convert f64` first and then fails wanting
`=>`. Declaring a conflict does not fix that — `tree-sitter generate` reports
such a conflict as *"unnecessary"* when a precedence already decides, which
reads like reassurance and is the opposite.

For anything involving precedence or a flat alternation: **run the input.**
`tree-sitter parse` on two lines costs a second and is the only thing that
answers.
