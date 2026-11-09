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
pub(crate) fn inline_body(block: &Block, source: &str) -> String {
  block
    .stmts
    .iter()
    .map(|stmt| stmt_text(stmt, 0, source))
    .collect::<Vec<_>>()
    .join("\n")
}
