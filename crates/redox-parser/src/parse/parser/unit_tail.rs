use crate::ParseError;
use redox_ast::{Block, Expr, Ident};
use redox_diag::Code;

/// Require a `()`-returning function's indented body to end in `()` —
/// `RDX0113`, plan 55 leaf 4.
///
/// Plan 55 §1.6's rule: *a block whose value would be unit must say `()`*. A
/// block's value is its last element, and a `()`-returning function's body has
/// no value to show for itself — so it writes one, and §9 decision 3 pays that
/// line visibly rather than leaving the value positional.
///
/// # Why the declared return type and not the shape of the tail
///
/// Both are syntactic facts, and only one of them is a defect worth naming.
///
/// A body whose last element has no value under a **non-unit** return type is
/// already `E0308` *at the definition*, which plan 07's remapping lands on the
/// author's own line; rustc's message is better than this one would be, and
/// this one's remedy — write `()` — is the wrong advice for a function that
/// promised a `u32`. AGENTS.md's rule applies directly: every check rustc can
/// make, rustc makes.
///
/// Under a **unit** return type rustc has nothing to say, at the definition or
/// anywhere else, because the body is correct. What it is not is *deliberate*:
/// the value arrived from whichever line happened to be last.
///
/// So `Expr::is_statement_form()` is deliberately **not** the predicate, even
/// though it looks like one. It answers *"is this written by a statement
/// printer"*, which includes `return`, `break` and `continue` — all three of
/// type `!` — so a block ending in one has no value and no defect either.
///
/// # The three exemptions
///
/// `return`, `break` and `continue` end a body legitimately: each states where
/// the function's value comes from, and each diverges, so a `()` written after
/// one is unreachable code that rustc warns about. Asking for `()` there would
/// trade a positional unit for a warning in the generated Rust, which plan
/// 04's *no rustc output* guarantee would then have to defend.
///
/// # An indented block only, which is what the criterion says
///
/// Criterion 6 is about *a block*, and a block is what an author writes by
/// indenting. `print_outcome r = print (r.is_ok ())` and a guard row's
/// `| c = print x` are **one expression wide**: that expression is visibly the
/// value, there is no last-line-wins for this rule to make explicit, and
/// §1.6's failure mode needs a statement list to occur in at all.
///
/// The practical half matters as much. A guard row has nowhere to put a second
/// element, so requiring `()` of one would not cost a line — it would force
/// `| c = do` and an indented block, changing a construct's shape rather than
/// adding to it, which is more than §9 decision 3 priced. Hence the call site
/// is the indented-body branch of `Parser::clause`, which is the one place
/// that knows a block was written.
pub(crate) fn unit_tail(name: &Ident, block: &Block) -> Result<(), ParseError> {
  // An empty block is `RDX0105`'s, raised by `statements` itself. Nothing to
  // say about the tail of a body that has none.
  let Some(last) = block.stmts.last() else {
    return Ok(());
  };
  if matches!(
    last.expr,
    Expr::Unit { .. } | Expr::Return { .. } | Expr::Break { .. } | Expr::Continue { .. }
  ) {
    return Ok(());
  }

  Err(ParseError::with_help(
    Code::UnitTail,
    format!(
      "`{}` returns `()`, so its body must end with `()`",
      name.text
    ),
    "write `()` as the body's last line — plan 55 §1.6 spells a unit value rather than \
     implying it, so a body ending in a loop, a binding or an assignment cannot become `()` \
     by arriving last",
    last.span,
  ))
}
