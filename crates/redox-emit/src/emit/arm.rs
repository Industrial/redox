use crate::emit::emit_body::emit_body;
use crate::emit::expr::emit_expr;
use crate::emit::inline_comment::emit_inline_comment;
use crate::emit::one_tail_expression::one_tail_expression;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::unparen::unparen;
use redox_ast::{Clause, StmtKind};

/// Emit one `match` arm for a clause, at `indent` spaces.
///
/// Two spellings of the same arm. A clause whose body is a single tail
/// expression becomes `pat => expr,`; anything else becomes a braced block.
///
/// The distinction is not cosmetic. Plan 04's guarantee is that generated Rust
/// is already rustfmt-clean, and rustfmt *collapses* `0 => { 0 }` to
/// `0 => 0,`, so emitting the braced form for a one-expression arm made
/// `examples/guards.sdx` the first example rustfmt would have rewritten. The
/// braced form is kept for every other body, because that is what rustfmt
/// leaves alone — a guard ladder in an arm stays `n => { if … }`.
///
/// Found by adding an example, not by a unit test: the clause tests asserted
/// `0 => {` and passed, because nothing in them ran rustfmt over the result.
pub(crate) fn emit_arm(out: &mut Sink<'_>, clause: &Clause, arm: &str, indent: usize, tail: bool) {
  if let Some(stmt) = one_tail_expression(clause, tail) {
    out.mark(stmt.span);
    pad(out, indent);
    out.push_str(arm);
    out.push_str(" => ");
    let StmtKind::Expr(e) = &stmt.kind else {
      unreachable!("`one_tail_expression` returns only an expression statement")
    };
    out.push_str(&emit_expr(unparen(e)));
    out.push(',');
    emit_inline_comment(out, stmt.trailing.as_ref());
    out.push('\n');
    return;
  }

  pad(out, indent);
  out.push_str(arm);
  out.push_str(" => {\n");
  // Every arm is a tail position: a `match` arm's value is the arm's value,
  // whether or not the function returns anything.
  emit_body(out, &clause.body, indent + RUST_INDENT, tail);
  pad(out, indent);
  out.push_str("}\n");
}
