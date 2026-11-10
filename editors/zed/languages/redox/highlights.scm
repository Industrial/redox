; Redox highlighting — plan 41 leaf 4.
;
; A query colours a **token**, which is why §1.2's flat expression rule costs
; nothing here: nothing below asks about a tree shape inside an expression.
;
; Criterion 5 pairs this file with plan 40's semantic-token table — the two are
; the same mapping for two consumers, and two highlighters that disagree are
; worse than one.

; ------------------------------------------------------------------ keywords

; Plan 49 added the last four. They were in `grammar.js` from plan 45 and in
; no highlighter, because nothing compared the two lists — `editors.rs`'s
; `every_grammar_keyword_is_coloured` now does, for this file and for the Vim
; and TextMate copies of the same list.
[
  "import" "module" "data" "newtype" "type" "class" "instance" "errors"
  "test" "const" "static" "deriving" "from" "as" "where" "otherwise"
  "if" "then" "else" "while" "until" "for" "in" "loop" "break" "continue"
  "return" "assert" "guard" "with" "case" "of" "do" "async" "try" "catch"
  "mut" "move" "or"
  "rust" "unsafe" "extern" "union"
] @keyword

(visibility) @keyword

; `self` and `Self` are one token, not two keywords.
(self_expression) @variable.builtin

; ---------------------------------------------------------------- literals

(number) @number
(boolean) @constant.builtin
(string) @string
(raw_string) @string
(char) @character
(comment) @comment
(lifetime) @label
(label) @label

; ------------------------------------------------------------------- names

(type_identifier) @type
(path) @namespace

; A declaration's own name, which an editor shows in bold or in its outline.
(signature name: (identifier) @function)
(equation name: (identifier) @function)
(data_declaration name: (type_identifier) @type.definition)
(type_alias name: (type_identifier) @type.definition)
(class_declaration name: (type_identifier) @type.definition)
(errors_declaration name: (type_identifier) @type.definition)
(module name: (identifier) @namespace)
(const_declaration name: (type_identifier) @constant)
(const_declaration name: (identifier) @constant)
(field_declaration name: (identifier) @property)
(record_literal_field name: (identifier) @property)
(record_pattern_field name: (identifier) @property)
(test_declaration description: (string) @string.special)

(identifier) @variable

; --------------------------------------------------------------- operators

(operator) @operator
(ascription) @operator
(attribute) @attribute

[ "(" ")" "[" "]" "{" "}" ] @punctuation.bracket
; `:` and not `::` since plan 52: `:` introduces a type in every position that
; has one, and `::` is a path separator that never stands alone — a path lexes
; as one `(path)` token and the `_type_punctuation` run inside a signature is
; hidden, so `"::"` is no longer a node this query could name. A query naming a
; node the grammar does not have is an error rather than a no-op, which is what
; `the_highlight_query_loads_and_matches` catches.
[ "," ";" "|" "=" ":" "->" "=>" "\\" ] @punctuation.delimiter
