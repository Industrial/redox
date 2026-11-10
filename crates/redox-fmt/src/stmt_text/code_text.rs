use crate::expr_text::expr_text;

use crate::item_text::item_text;

use crate::pattern_text::pattern_text;

use crate::stmt_text::binding_text::binding_text;

use crate::stmt_text::block_text::block_text;

use crate::stmt_text::condition_text::condition_text;

use crate::stmt_text::else_text::else_text;

use crate::stmt_text::label_text::label_text;

use redox_ast::Expr;

/// A block element's code, with its **first line unindented**.
///
/// That convention is what lets `else if` work: `else_text` puts this after
/// `else ` on a line it has already indented. Every later line carries its own
/// full indentation, computed from `indent` — so a nested block is at
/// `indent + 1` and a caller never has to re-indent anything this returns.
///
/// # This is the statement position, and that is a choice now rather than a type
///
/// Plan 55 leaf 1 deleted `StmtKind`, so a block element and a subexpression
/// are the same type. The thirteen forms `Expr::is_statement_form` names are
/// printed here and everything else falls through to `expr_text` — which is
/// the same split the two printers always had, with the *position* deciding
/// where it used to be decided by which enum a node lived in.
///
/// `if` used to be the form where the choice showed: reached here it was doc
/// 01 §8's **indented** spelling and reached through `expr_text` it was the
/// one-line `if c then a else b`. Plan 55 leaf 7 retired the second, so
/// `expr_text` delegates every `if` here and the two printers cannot disagree
/// about which one the author wrote.
pub(crate) fn code_text(expr: &Expr, indent: usize, source: &str) -> String {
  // Every nested block is one level deeper than the element that owns it,
  // which is the whole of §3 row 11 as far as statements are concerned.
  let inner = indent + 1;
  match expr {
    Expr::Binding(binding) => binding_text(binding, indent, source),
    Expr::Item { item, .. } => item_text(item, indent, source),
    Expr::Return { value: None, .. } => String::from("return"),
    Expr::Return {
      value: Some(value), ..
    } => format!("return {}", expr_text(value, 0, indent, source)),
    Expr::Draw { name, value, .. } => format!(
      "with {} = {}",
      name.text,
      expr_text(value, 0, indent, source)
    ),
    Expr::Await { pattern, value, .. } => match pattern {
      Some(pattern) => format!(
        "{} <- {}",
        pattern_text(pattern),
        expr_text(value, 0, indent, source)
      ),
      None => format!("<- {}", expr_text(value, 0, indent, source)),
    },
    Expr::Assign {
      target, op, value, ..
    } => {
      let sigil = op.map_or(":", |o| o.redox_text());
      format!(
        "{} {}= {}",
        expr_text(target, 0, indent, source),
        sigil,
        expr_text(value, 0, indent, source)
      )
    }
    Expr::Guard {
      pattern,
      source: r#from,
      r#otherwise,
      ..
    } => format!(
      "guard {} is {} else\n{}",
      expr_text(r#from, 0, indent, source),
      pattern_text(pattern),
      block_text(r#otherwise, inner, source)
    ),
    Expr::If {
      condition,
      then_block,
      r#otherwise,
      ..
    } => format!(
      "if {}\n{}{}",
      condition_text(condition, indent, source),
      block_text(then_block, inner, source),
      else_text(r#otherwise, indent, source)
    ),
    Expr::While {
      condition,
      negated,
      body,
      label,
      ..
    } => {
      let keyword = if *negated { "until" } else { "while" };
      format!(
        "{}{} {}\n{}",
        label_text(label),
        keyword,
        condition_text(condition, indent, source),
        block_text(body, inner, source)
      )
    }
    Expr::For {
      binder,
      source: r#from,
      body,
      label,
      ..
    } => format!(
      "{}for {} in {}\n{}",
      label_text(label),
      binder.text,
      expr_text(r#from, 0, indent, source),
      block_text(body, inner, source)
    ),
    Expr::Loop { body, label, .. } => format!(
      "{}loop\n{}",
      label_text(label),
      block_text(body, inner, source)
    ),
    Expr::Break { label, value, .. } => {
      let mut out = String::from("break");
      if let Some(name) = label {
        out.push_str(&format!(" '{}", name));
      }
      if let Some(value) = value {
        out.push(' ');
        out.push_str(&expr_text(value, 0, indent, source));
      }
      out
    }
    Expr::Continue { label, .. } => match label {
      Some(name) => format!("continue '{}", name),
      None => String::from("continue"),
    },
    value => expr_text(value, 0, indent, source),
  }
}
