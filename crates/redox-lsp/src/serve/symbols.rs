/// One item as a document symbol.
pub(crate) mod symbol_of;

use crate::serve::symbols::symbol_of::symbol_of;
use lsp_types::{DocumentSymbol, DocumentSymbolResponse};

/// The document's items, as a symbol tree.
///
/// Plan 40 §1: document symbols **are** the item list. Emerald's server builds
/// a symbol table to answer completion and hover as well; Redox answers
/// neither, so there is nothing to build and this is a map over
/// `program.items`.
///
/// A `module` with a body nests, and that is the only nesting. A `class`'s
/// members are deliberately **not** children: they are not `Item`s, and giving
/// each one a symbol would mean inventing a kind and a range for a construct
/// the AST models differently — which is the kind of guess §8 declines on
/// Emerald's behalf.
///
/// Returns nothing for a document that does not parse, which is criterion 7.
/// An editor renders that as an unchanged outline rather than an empty one,
/// and while the user is mid-keystroke an outline that lags beats one that
/// empties itself on every half-typed line.
pub(crate) fn symbols(source: &str) -> DocumentSymbolResponse {
  let Ok(program) = redox_parser::parse_source(source) else {
    return DocumentSymbolResponse::Nested(Vec::new());
  };
  let found: Vec<DocumentSymbol> = program
    .items
    .iter()
    .map(|item| symbol_of(item, source))
    .collect();
  DocumentSymbolResponse::Nested(found)
}
