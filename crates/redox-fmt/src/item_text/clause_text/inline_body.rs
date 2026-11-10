use crate::stmt_text::indent_text::indent_text;

use crate::stmt_text::stmt_text;

use redox_ast::Block;

/// A body that sits on the equation line, with no indentation of its own.
///
/// An inline body is one statement — that is what being on the line means —
/// so it is printed at depth 0 and the leading pad trimmed off. A
/// block-bearing expression inside it still indents its *own* body, which is
/// what makes
///
/// ```text
/// describe e = case e of
///   Close -> "close"
/// ```
///
/// come out right: the `case` is inline and its arms are not.
///
/// The join is defensive. A block with more than one statement cannot be
/// inline, and if one ever is, joining on a newline produces something that
/// fails to re-parse — which the round-trip property reports — rather than
/// quietly dropping the rest.
pub(crate) fn inline_body(block: &Block, indent: usize, source: &str) -> String {
  let pad = indent_text(indent);
  let parts: Vec<String> = block
    .stmts
    .iter()
    .map(|stmt| {
      let text = stmt_text(stmt, indent, source);
      // The first line follows `" = "` on the caller's line, so it must not be
      // padded; every continuation line must be. `stmt_text` threads one
      // indent for both, so the pad is written and then taken back off the
      // front — the same trade `code_text` makes, and for the same reason.
      //
      // This argument used to be made by passing **0**, which suppressed the
      // first pad and flattened the continuations with it. At module scope the
      // two are the same and nothing noticed; inside a local `instance` they
      // are not, and a `case`'s arms printed a level too shallow.
      text.strip_prefix(&pad).unwrap_or(&text).to_string()
    })
    .collect();
  parts.join("\n")
}
