# -*- coding: utf-8 -*-
"""Plan 78 batch 6: `symbol_of`, the largest file in the crate.

A twelve-arm `case` yielding a three-tuple, which is the Rust's shape kept
intact — the alternative, twelve `return`s, would lose the fact that every arm
answers the same three questions.

Two ascriptions stand in for `collect::<Vec<_>>()`. The Rust's
`#[allow(deprecated, …)]` sits on each `deprecated:` field, and **Voltaic cannot
carry an attribute there at all** — `FieldInit` has no `leading`, so the parser
discards it before the AST exists. The allow is on the function instead; batch
7 has the finding.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('serve/symbols/symbol_of', '''
-- **The allow is on the function, where the Rust had it on each field.**
-- An attribute on a record-field initialiser is **dropped**: `FieldInit`
-- has no `leading`, so the parser discards it before the AST exists and
-- neither the emitter nor `voltaic fmt` ever sees it. That contradicts doc
-- 02 §10.1, which attributes joined at plan 18 §1.3, and it is quiet —
-- the `.sdx` parses and clippy then warns about a field the author had
-- explicitly allowed. Carrying it is a feature for a successor row; this
-- is a wider scope with the same effect.
#[allow(deprecated, reason = "the field is required by the struct literal")]
-- **The allow is on the function, where the Rust had it on each field.**
-- An attribute on a record-field initialiser is **dropped**: `FieldInit`
-- has no `leading`, so the parser discards it before the AST exists and
-- neither the emitter nor `voltaic fmt` ever sees it. That contradicts doc
-- 02 §10.1, which attributes joined at plan 18 §1.3, and it is quiet —
-- the `.sdx` parses and clippy then warns about a field the author had
-- explicitly allowed. Carrying it is a feature for a successor row; this
-- is a wider scope with the same effect.
#[allow(deprecated, reason = "the field is required by the struct literal")]
export(crate) symbol_of : &Item -> &str -> DocumentSymbol
symbol_of item source =
  range = to_range source (.span item)
  described = case item of
    Item::Fn decl -> (.clone decl.name.text, SymbolKind::FUNCTION, None)
    Item::Data decl ->
      -- A `newtype` is one constructor over one field, which is a struct; a
      -- `data` with alternatives is an enum. Plan 53 leaf 2 is what makes the
      -- `newtype` half an invariant rather than a convention — before it the
      -- keyword was unvalidated and `newtype Multi = A | B` reached here as
      -- an enum. Reading the shape rather than the keyword was already the
      -- right thing to do and is why this symbol was never wrong.
      --
      -- Taken from the declaration's own shape rather than from the keyword,
      -- because a single-constructor `data` lowers to a struct too.
      kind = if .len decl.constructors > 1
        SymbolKind::ENUM
      else
        SymbolKind::STRUCT
      (.clone decl.name.text, kind, None)
    Item::Type decl -> (.clone decl.name.text, SymbolKind::TYPE_PARAMETER, None)
    Item::Class decl -> (.clone decl.name.text, SymbolKind::INTERFACE, None)
    Item::Instance decl ->
      -- The class's own name only: `instance Convert f64 Meters` is
      -- `Convert Meters` in the outline, because the symbol names the pair
      -- a reader is looking for and the trait's arguments are not part of
      -- it.
      -- `.r#class` and `Some implemented`, because **`class` is a keyword** —
      -- it cannot be read as a field *or* bound by a pattern. Plan 68 already
      -- spelled the AST field `export r#class`, so the convention is settled
      -- and only undocumented. Fifth keyword collision in five crates.
      name = case &decl.r#class of
        Some implemented -> format! \\-> "#{implemented.head.text} #{decl.target.head.text}"
        None -> .clone decl.target.head.text
      (name, SymbolKind::OBJECT, None)
    Item::Errors decl -> (.clone decl.name.text, SymbolKind::ENUM, None)
    Item::Test decl -> (.clone decl.description, SymbolKind::FUNCTION, None)
    Item::Import decl -> (.clone decl.path, SymbolKind::NAMESPACE, None)
    Item::Module decl ->
      -- Ascribed where the Rust wrote `collect::<Vec<_>>()`: doc 03 §18 has
      -- no turbofish and names ascription as the replacement.
      nested : Option<Vec<DocumentSymbol>> = .map (\\items -> items |> .iter |> .map (\\inner -> symbol_of inner source) |> .collect) (.as_ref decl.items)
      (.clone decl.name.text, SymbolKind::MODULE, nested)
    Item::Const decl -> (.clone decl.name.text, SymbolKind::CONSTANT, None)
    -- Plan 45 leaf 1. Inline Rust has no name of its own, so the outline
    -- shows the construct — which is still worth a symbol: a reader scanning
    -- a file wants to know where the un-analysed Rust is, and that is the
    -- one thing this entry can honestly tell them.
    Item::InlineRust _ -> (.to_string "rust", SymbolKind::OBJECT, None)
    -- Plan 45 leaf 3. The ABI names the block, and its declarations nest —
    -- the second item kind after `module` whose children are worth showing,
    -- because an FFI block's whole content is the names it imports.
    Item::Extern decl ->
      declared : Vec<DocumentSymbol> = decl.decls
        |> .iter
        |> .map (\\d -> DocumentSymbol {
          name = .clone d.name.text,
          detail = None,
          kind = SymbolKind::FUNCTION,
          tags = None,
          deprecated = None,
          range = to_range source d.span,
          selection_range = to_range source d.span,
          children = None,
        })
        |> .collect
      (format! \\-> "extern #{decl.abi}", SymbolKind::NAMESPACE, Some declared)
  name = described.0
  kind = described.1
  children = described.2
  DocumentSymbol {
    name,
    detail = None,
    kind,
    tags = None,
    -- `range` is the whole item and `selection_range` is what the editor
    -- highlights when you pick the symbol. Both are the item here: a
    -- narrower selection would need the name's own span, and only some item
    -- kinds carry one.
    deprecated = None,
    range,
    selection_range = range,
    children,
  }
''')

print('plan 78 batch 6: symbol_of')
