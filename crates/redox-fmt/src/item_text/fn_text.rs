use crate::item_text::clause_text::clause_text;
use crate::item_text::signature_line::signature_line;
use crate::stmt_text::indent_text::indent_text;
use redox_ast::FnDecl;

/// A function — its signature line, then one line per equation.
///
/// The signature is sliced verbatim and `signature_line` records why at
/// length: the parser rewrites a signature's type variables into Rust's
/// uppercase convention in place, so the AST cannot say what the author wrote.
///
/// The visibility is **not** printed here. It is part of the signature line,
/// which is sliced, so printing it as well would double it — `export export
/// hello : …`. For a declaration with no signature line the marker is in the
/// equation line instead, which the parser also puts inside the clause's own
/// text, so neither shape needs it from the tree. Written down because the
/// asymmetry with every other item kind looks like an omission.
pub(crate) fn fn_text(decl: &FnDecl, indent: usize, source: &str) -> String {
  let pad = indent_text(indent);
  let mut lines: Vec<String> = Vec::new();
  if let Some(signature) = signature_line(decl, source) {
    lines.push(signature);
  }
  for clause in &decl.clauses {
    lines.push(clause_text(clause, &decl.name.text, indent, source));
  }
  // Every line but the first carries the item's own indentation; the first is
  // placed by the caller, which is `item_text`'s convention throughout.
  lines.join(&format!("\n{pad}"))
}
