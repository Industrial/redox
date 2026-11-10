use crate::ParseError;
use crate::parse::parser::Parser;
use redox_ast::Expr;

/// An indented block as an expression, with the label it may carry — plan 55
/// leaf 7.
///
/// ```text
/// inner =                 found = 'search
///   n = 2                   for c in candidates
///   n * 10                    if c.matches q
///                               break 'search Some c
///                           None
/// ```
///
/// This is `do_block` with the keyword taken off. §1.7's argument is that a
/// keyword announcing a block is redundant once an indented block is an
/// expression, and the node is unchanged — so what leaf 7 removed is the
/// `expect` for `Kw(Do)` and nothing else. §10.10 predicted exactly that:
/// *"leaf 7 removes a keyword rather than a form"*.
///
/// The cursor is on the `Newline` that ends the line the block hangs from,
/// which is what [`Parser::statement_block`] expects. Three positions reach
/// here — a binding's value, a `case` arm body and a labelled block in
/// statement position — and all three arrive that way.
pub(crate) fn block_expr(
  p: &mut Parser,
  label: Option<String>,
  start: usize,
) -> Result<Expr, ParseError> {
  let body = p.statement_block("expected an indented block")?;
  Ok(Expr::Block {
    label,
    body,
    span: (start, p.prev_span().1),
  })
}
