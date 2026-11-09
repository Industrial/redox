use crate::expr_text::expr_text;
use crate::item_text::visibility_text::visibility_text;
use redox_ast::ConstDecl;

/// A `const` or `static` item — `const MAX :: usize = 1024`.
///
/// **This** is the form §3 row 8's example named. The separator is `::` and it
/// is spaced, where a local binding takes a single `:` — three similar forms,
/// §10.22's table.
///
/// `is_static` picks the keyword. The value is an expression at power 0, and
/// the type is printed from the tree: a `const`'s type is concrete by
/// construction, so it is not touched by the type-variable rewrite that makes
/// a function's signature unprintable.
pub(crate) fn const_text(decl: &ConstDecl, source: &str) -> String {
  let keyword = if decl.is_static { "static" } else { "const" };
  format!(
    "{}{keyword} {} :: {} = {}",
    visibility_text(&decl.visibility),
    decl.name.text,
    decl.ty.text,
    // Indent 0: a `const` sits at the top of a file, so there is no enclosing
    // indentation for a block-bearing expression to hang from — and doc 03
    // does not allow one in a `const` anyway.
    //
    // The **source** is passed, unlike the indentation. An empty string here
    // was the first version and it defeats `literal_text`'s span slice, so a
    // `static` holding a block string would have been rewritten into a
    // single-line literal — the same defect leaf 2 had just fixed, reopened
    // one file away by a parameter that looked unused.
    expr_text(&decl.value, 0, 0, source)
  )
}
