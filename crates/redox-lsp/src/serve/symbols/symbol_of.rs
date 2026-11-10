use crate::serve::positions::range_of::range_of as to_range;
use lsp_types::{DocumentSymbol, SymbolKind};
use redox_ast::Item;

/// One item as a document symbol.
///
/// The kind is the protocol's nearest equivalent, and two of them are
/// judgements worth recording rather than leaving to look obvious:
///
/// - **`errors` is a `SymbolKind::ENUM`**, because that is what it lowers to —
///   doc 03 §14's declaration becomes an enum with a `Display` and an `Error`
///   impl. Calling it a `STRUCT` would read better in an outline and would
///   disagree with the generated Rust.
/// - **`test` is a `SymbolKind::FUNCTION`**, because it is one in the output,
///   and its *name* is its description string. An outline of a file full of
///   tests is then a readable list of what the file asserts, which is the one
///   place a symbol name is more useful than an identifier.
///
/// A `module`'s items nest. Everything else is a leaf.
pub(crate) fn symbol_of(item: &Item, source: &str) -> DocumentSymbol {
  let range = to_range(source, item.span());
  let (name, kind, children) = match item {
    Item::Fn(decl) => (decl.name.text.clone(), SymbolKind::FUNCTION, None),
    Item::Data(decl) => (
      decl.name.text.clone(),
      // A `newtype` is one constructor over one field, which is a struct; a
      // `data` with alternatives is an enum. Plan 53 leaf 2 is what makes the
      // `newtype` half an invariant rather than a convention — before it the
      // keyword was unvalidated and `newtype Multi = A | B` reached here as
      // an enum. Reading the shape rather than the keyword was already the
      // right thing to do and is why this symbol was never wrong.
      //
      // Taken from the declaration's own
      // shape rather than from the keyword, because a single-constructor
      // `data` lowers to a struct too.
      if decl.constructors.len() > 1 {
        SymbolKind::ENUM
      } else {
        SymbolKind::STRUCT
      },
      None,
    ),
    Item::Type(decl) => (decl.name.text.clone(), SymbolKind::TYPE_PARAMETER, None),
    Item::Class(decl) => (decl.name.text.clone(), SymbolKind::INTERFACE, None),
    Item::Instance(decl) => (
      match &decl.class {
        // The class's own name only: `instance Convert f64 Meters` is
        // `Convert Meters` in the outline, because the symbol names the pair
        // a reader is looking for and the trait's arguments are not part of
        // it.
        Some(class) => format!("{} {}", class.head.text, decl.target.head.text),
        None => decl.target.head.text.clone(),
      },
      SymbolKind::OBJECT,
      None,
    ),
    Item::Errors(decl) => (decl.name.text.clone(), SymbolKind::ENUM, None),
    Item::Test(decl) => (decl.description.clone(), SymbolKind::FUNCTION, None),
    Item::Import(decl) => (decl.path.clone(), SymbolKind::NAMESPACE, None),
    Item::Module(decl) => (
      decl.name.text.clone(),
      SymbolKind::MODULE,
      decl.items.as_ref().map(|items| {
        items
          .iter()
          .map(|inner| symbol_of(inner, source))
          .collect::<Vec<_>>()
      }),
    ),
    Item::Const(decl) => (decl.name.text.clone(), SymbolKind::CONSTANT, None),
    // Plan 45 leaf 1. Inline Rust has no name of its own, so the outline
    // shows the construct — which is still worth a symbol: a reader scanning
    // a file wants to know where the un-analysed Rust is, and that is the
    // one thing this entry can honestly tell them.
    Item::InlineRust(_) => ("rust".to_string(), SymbolKind::OBJECT, None),
    // Plan 45 leaf 3. The ABI names the block, and its declarations nest —
    // the second item kind after `module` whose children are worth showing,
    // because an FFI block's whole content is the names it imports.
    Item::Extern(decl) => (
      format!("extern {}", decl.abi),
      SymbolKind::NAMESPACE,
      Some(
        decl
          .decls
          .iter()
          .map(|d| DocumentSymbol {
            name: d.name.text.clone(),
            detail: None,
            kind: SymbolKind::FUNCTION,
            tags: None,
            #[allow(deprecated, reason = "the field is required by the struct literal")]
            deprecated: None,
            range: to_range(source, d.span),
            selection_range: to_range(source, d.span),
            children: None,
          })
          .collect(),
      ),
    ),
  };
  DocumentSymbol {
    name,
    detail: None,
    kind,
    tags: None,
    // `range` is the whole item and `selection_range` is what the editor
    // highlights when you pick the symbol. Both are the item here: a
    // narrower selection would need the name's own span, and only some item
    // kinds carry one.
    #[allow(deprecated, reason = "the field is required by the struct literal")]
    deprecated: None,
    range,
    selection_range: range,
    children,
  }
}
