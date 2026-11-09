pub(crate) mod argument;
pub(crate) mod can_drop_paren;
pub(crate) mod char_text;
pub(crate) mod comma_list;
pub(crate) mod comprehension_text;
pub(crate) mod interp_text;
pub(crate) mod lambda_text;
pub(crate) mod operand;
pub(crate) mod power_of;
pub(crate) mod record_text;
pub(crate) mod section_text;

use crate::expr_text::argument::argument;
use crate::expr_text::can_drop_paren::can_drop_paren;
use crate::expr_text::char_text::char_text;
use crate::expr_text::comma_list::comma_list;
use crate::expr_text::comprehension_text::comprehension_text;
use crate::expr_text::interp_text::interp_text;
use crate::expr_text::lambda_text::lambda_text;
use crate::expr_text::operand::operand;
use crate::expr_text::record_text::record_text;
use crate::expr_text::section_text::section_text;
use crate::stmt_text::block_text::block_text;
use crate::stmt_text::case_arm_text::case_arm_text;
use crate::stmt_text::condition_text::condition_text;
use redox_ast::Expr;

/// One expression in canonical Redox — plan 34, leaf 3.
///
/// `required` is the binding power the surrounding context demands of this
/// expression: 0 at a statement's top level, an operator's power inside a
/// binary expression. It is what decides whether a parenthesis the author
/// wrote can be dropped, and it is threaded through rather than recomputed
/// because only the *parent* knows it.
///
/// `indent` is the statement depth this expression sits at, which only the
/// five block-bearing forms use — they print their bodies at `indent + 1`.
/// Threaded rather than defaulted, because a block printed at the wrong depth
/// is the one mistake the off-side rule turns into a different program.
///
/// # `source` is no longer read, and the parameter stays
///
/// Leaf 3 sliced the block-bearing forms verbatim out of the source, per
/// criterion 7. Leaf 4 prints them, so nothing here reads `source` any more —
/// every expression form is printed from the tree.
///
/// The parameter is kept because leaf 2 will need it the moment an item turns
/// out to have a form the printer does not cover, and because removing it
/// would churn every call site for no gain. That is a judgement, not an
/// oversight, so it is written down.
///
/// # What it normalises
///
/// §3's rows that live in an expression: juxtaposition for calls (row 1),
/// redundant parentheses dropped where precedence allows (row 3), one space
/// before a method's arguments (row 6), a spaced `::` ascription (row 8), and
/// no trailing comma except the one a single-element tuple needs (row 12).
///
/// Rows 2, 4 and 5 are **retired** by §10 rather than implemented. Row 2
/// (`f(x, y)` → `f x y`) changes the program: with juxtaposition as the call
/// form, `f (x, y)` applies `f` to a **tuple**. Rows 4 and 5 (a method chain
/// rewritten to `|>`) replace one construct with another, which is a refactor
/// and not a normalisation — and the property that guards this formatter
/// cannot guard it.
pub fn expr_text(expr: &Expr, required: u8, indent: usize, source: &str) -> String {
  match expr {
    // Doc 18 §2.1: juxtaposition is the call form, parentheses are grouping.
    // So the arguments are separated by spaces and each is parenthesised only
    // if it would otherwise read as more application.
    Expr::Call { callee, args, .. } => {
      let mut out = argument(callee, indent, source);
      for arg in args {
        out.push(' ');
        out.push_str(&argument(arg, indent, source));
      }
      out
    }
    // A method is **not** juxtaposition — `y.m (a, b)` parses as a
    // two-argument method call where `f (a, b)` parses as application to a
    // tuple. §3 row 6's one space, and the arguments comma-separated.
    Expr::MethodCall {
      recv, name, args, ..
    } => format!(
      "{}.{} ({})",
      argument(recv, indent, source),
      name.text,
      comma_list(args, indent, source)
    ),
    Expr::Field { recv, name, .. } => format!("{}.{}", argument(recv, indent, source), name.text),
    Expr::Index { recv, index, .. } => format!(
      "{}[{}]",
      argument(recv, indent, source),
      expr_text(index, 0, indent, source)
    ),
    Expr::Binary { op, lhs, rhs, .. } => {
      // Left-associative, so the right operand must bind one rung tighter for
      // its parentheses to be redundant.
      let power = op.power();
      format!(
        "{} {} {}",
        operand(lhs, power, indent, source),
        op.redox_text(),
        operand(rhs, power + 1, indent, source)
      )
    }
    // `&mut ` already carries its trailing space in `rust_text`, which is
    // also the Redox spelling.
    Expr::Unary {
      op, operand: inner, ..
    } => {
      format!("{}{}", op.rust_text(), argument(inner, indent, source))
    }
    // §3 row 3, the one normalisation that can change meaning. `can_drop_paren`
    // is the whole decision and it is conservative by construction.
    Expr::Paren { inner, .. } => {
      if can_drop_paren(inner, required) {
        expr_text(inner, required, indent, source)
      } else {
        format!("({})", expr_text(inner, 0, indent, source))
      }
    }
    Expr::Path { text, .. } => text.clone(),
    Expr::Lambda {
      params,
      body,
      is_move,
      ..
    } => lambda_text(params, body, *is_move, indent, source),
    Expr::OptionalChain { head, segments, .. } => {
      let mut out = argument(head, indent, source);
      for segment in segments {
        out.push_str("?.");
        out.push_str(segment);
      }
      out
    }
    // Doc 18 §5.2: a spaced `::` ascribes and a tight one is a path, so the
    // spaces here are load-bearing rather than cosmetic — §3 row 8 exists to
    // say the formatter must never tighten one.
    //
    // The inner expression is an **operand** and not an argument: `::` binds
    // looser than every binary operator, which the parser settles —
    // `1 + 2 :: u64` is `Ascribe { inner: Binary { .. } }` with no parenthesis
    // in the source. Printing it as an argument added one, and the corpus
    // property reported the program had changed. `operand` at power 0 keeps
    // the parentheses only around the forms `power_of` declines on, which is
    // the right set: a lambda or a second ascription does need them.
    Expr::Ascribe { inner, ty, .. } => {
      format!("{} :: {}", operand(inner, 0, indent, source), ty.text)
    }
    // The lexer keeps a string's source text, quotes and escapes included, so
    // there is nothing to re-escape and no way to lose an escape.
    Expr::StrLit { value, .. } => value.clone(),
    Expr::RawLit { text, .. } => text.clone(),
    Expr::Num { text, .. } => text.clone(),
    Expr::StrInterp { parts, .. } => interp_text(parts, indent, source),
    // §3 row 12: no trailing comma, except the one that makes `(a,)` a tuple
    // rather than a group — plan 33 §10.3.
    Expr::Tuple { items, .. } => {
      if items.len() == 1 {
        format!("({},)", expr_text(&items[0], 0, indent, source))
      } else {
        format!("({})", comma_list(items, indent, source))
      }
    }
    Expr::List { items, .. } => format!("[{}]", comma_list(items, indent, source)),
    Expr::ArrayRepeat { value, count, .. } => format!(
      "[{}; {}]",
      expr_text(value, 0, indent, source),
      expr_text(count, 0, indent, source)
    ),
    Expr::Record { head, fields, .. } => record_text(head, fields, indent, source),
    Expr::Comprehension {
      element,
      qualifiers,
      ..
    } => comprehension_text(element, qualifiers, indent, source),
    Expr::Try { inner, .. } => format!("{}?", argument(inner, indent, source)),
    Expr::Unit { .. } => "()".to_string(),
    Expr::Section { section, .. } => section_text(section, indent, source),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::Char { value, .. } => char_text(*value),
    Expr::SelfField { name, .. } => format!("self.{}", name.text),
    Expr::SelfType { name, .. } => format!("Self::{}", name.text),
    Expr::Ident(ident) => ident.text.clone(),
    // The block-bearing forms, which §10.15 moved to leaf 4 because their
    // bodies are statement sequences. The first line is unindented and every
    // later line sits at `indent + 1`, matching `kind_text`'s convention —
    // the caller has already written whatever this hangs from.
    //
    // `if c then a else b` is the **expression** `if`, told apart from the
    // statement by `then`, and it is the one of the five that stays on one
    // line.
    Expr::If {
      condition,
      then,
      otherwise,
      ..
    } => format!(
      "if {} then {} else {}",
      condition_text(condition, indent, source),
      expr_text(then, 0, indent, source),
      expr_text(otherwise, 0, indent, source)
    ),
    Expr::Do { label, body, .. } => format!(
      "do{}\n{}",
      label.as_ref().map_or(String::new(), |n| format!(" '{n}")),
      block_text(body, indent + 1, source)
    ),
    Expr::AsyncBlock { body, .. } => {
      format!("async\n{}", block_text(body, indent + 1, source))
    }
    Expr::TryBlock { body, .. } => {
      format!("try\n{}", block_text(body, indent + 1, source))
    }
    Expr::Case {
      scrutinee, arms, ..
    } => {
      let printed: Vec<String> = arms
        .iter()
        .map(|arm| case_arm_text(arm, indent + 1, source))
        .collect();
      format!(
        "case {} of\n{}",
        expr_text(scrutinee, 0, indent, source),
        printed.join("\n")
      )
    }
  }
}
