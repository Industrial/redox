" Redox syntax highlighting for Vim. Plan 49 leaf 2.
"
" Regex only. This is the fallback for an editor with neither tree-sitter nor a
" built-in LSP client, so it is the one client here that highlights by pattern
" rather than by parse — and therefore the one that can be wrong about a
" keyword inside a string or a comment, which `syn region` mitigates and does
" not eliminate.
"
" The keyword list is checked against `tree-sitter-redox/grammar.js` by
" `editors.rs`'s `every_grammar_keyword_is_coloured`, so this file and the
" tree-sitter and TextMate highlighters cannot drift apart. Three highlighters
" that disagree are worse than one.

if exists('b:current_syntax')
  finish
endif

" Comments are **regions**, not matches, and that is forced rather than
" stylistic. In Vim a `syn keyword` outranks a `syn match` wherever both apply,
" so with `syn match` comments the word `data` inside `-- mentioning data`
" stays a keyword and the leading `--` is claimed by the operator pattern. A
" region's `contains` is what excludes an item from its interior, and only
" `@Spell` is listed here.
"
" Found by `editors.rs`, which asserts the group *inside* a comment and not
" only at its start — the assertion that would have passed either way.
"
" `-- |` and `-- !` are the doc forms (Haskell's); `--` alone is ordinary.
" Both start at the same column, and among items starting together Vim prefers
" the one defined last, so the doc region comes second.
syn region  sdxComment      start="--" end="$" oneline keepend contains=@Spell
syn region  sdxDocComment   start="--\s*[|!]" end="$" oneline keepend contains=@Spell

" Strings. The triple-quoted form is Redox's verbatim block (plan 45's inline
" Rust and plan 23's raw strings), and must be matched before the single form
" or the opening `"""` reads as an empty string followed by a quote.
syn region  sdxVerbatim     start=+"""+ end=+"""+ keepend
syn region  sdxString       start=+"+ skip=+\\\\\|\\"+ end=+"+ oneline
syn match   sdxChar         "'\([^'\\]\|\\.\)'"
syn match   sdxLifetime     "'[a-z_][a-zA-Z0-9_]*\>'\@!"

syn match   sdxNumber       "\<\d[0-9_]*\(\.\d[0-9_]*\)\?\([eE][-+]\?\d\+\)\?\(_\?[iuf]\(8\|16\|32\|64\|128\|size\)\)\?\>"
syn match   sdxNumber       "\<0[xX][0-9a-fA-F_]\+\>"
syn match   sdxNumber       "\<0[bB][01_]\+\>"
syn match   sdxNumber       "\<0[oO][0-7_]\+\>"
syn keyword sdxBoolean      true false
syn keyword sdxSelf         self Self

" Keywords. One list, derived from the grammar — see the header.
"
" `then` left at plan 55 leaf 7, which retired the construct it named and
" freed the word for `Ordering::then`. `do` stayed: the leaf retired its
" construct too and kept the word **reserved**, so it is still worth colouring
" and `editors.rs` excuses it from the grammar cross-check — the tree-sitter
" grammar declares it through `reserved` rather than as a literal in a rule.
syn keyword sdxKeyword      import module data newtype type class instance
syn keyword sdxKeyword      errors test const static deriving from as where
syn keyword sdxKeyword      otherwise if else while until for in loop
syn keyword sdxKeyword      break continue return assert guard is with case of
syn keyword sdxKeyword      do async try catch mut move or
syn keyword sdxKeyword      rust unsafe extern union
syn keyword sdxKeyword      export

" A type is any `UpperCamel` name; Redox has no type checker, so a syntax file
" certainly cannot tell a type from a constructor, and colouring both the same
" is the honest answer rather than a heuristic that is sometimes wrong.
syn match   sdxType         "\<\u[a-zA-Z0-9_]*\>"
syn match   sdxAttribute    "@[a-zA-Z_][a-zA-Z0-9_]*"
" Plan 52's two tokens. `:` introduces a type in every position that has one —
" a signature, a typed binding, an ascription, a field, a lambda's binder — and
" `::` separates path segments and nothing else. The `:\(:\)\@!` lookahead is
" the same idiom as the `-` match below, and for the same reason: it makes the
" result independent of which item Vim happens to prefer.
syn match   sdxTypeColon    ":\(:\)\@!"
syn match   sdxPathSep      "::"
" `-` is matched with a negative lookahead for a second `-`, and that is the
" second half of the comment fix above. Redox has no `--` operator, and among
" items starting at the same column Vim prefers the one defined **last** — so
" a `[-+*…]` class here outranks the comment region at column 1 and the whole
" line stops being a comment. Excluding `--` is both correct on its own terms
" and what makes the ordering stop mattering.
" Plan 66's `>>>` and `<<<` are listed **before** the shifts: Vim takes the
" first alternative that matches at a position, so `>>` first would colour two
" of the three characters and leave the last one to the `[<>]` class.
syn match   sdxOperator     "->\|=>\|\\\|??\||>\|>>>\|<<<\|<<\|>>\|[+*/%<>=!&|^~]\|-\(-\)\@!"

hi def link sdxComment      Comment
hi def link sdxDocComment   SpecialComment
hi def link sdxString       String
hi def link sdxVerbatim     String
hi def link sdxChar         Character
hi def link sdxLifetime     Label
hi def link sdxNumber       Number
hi def link sdxBoolean      Boolean
hi def link sdxSelf         Identifier
hi def link sdxKeyword      Keyword
hi def link sdxType         Type
hi def link sdxAttribute    PreProc
hi def link sdxTypeColon    Delimiter
hi def link sdxPathSep      Delimiter
hi def link sdxOperator     Operator

let b:current_syntax = 'sdx'
