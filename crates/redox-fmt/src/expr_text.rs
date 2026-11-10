pub(crate) mod argument;

pub(crate) mod can_drop_paren;

pub(crate) mod comma_list;

pub(crate) mod comprehension_text;

pub(crate) mod dollar_text;

pub(crate) mod lambda_text;

pub(crate) mod literal_text;

pub(crate) mod macro_call_text;

pub(crate) mod operand;

pub(crate) mod pipeline_text;

pub(crate) mod power_of;

pub(crate) mod receiver;

pub(crate) mod record_text;

pub(crate) mod rust_hatch_text;

pub(crate) mod section_text;

use crate::expr_text::argument::argument;

use crate::expr_text::can_drop_paren::can_drop_paren;

use crate::expr_text::comma_list::comma_list;

use crate::expr_text::comprehension_text::comprehension_text;

use crate::expr_text::dollar_text::dollar_text;

use crate::expr_text::lambda_text::lambda_text;

use crate::expr_text::literal_text::literal_text;

use crate::expr_text::macro_call_text::macro_call_text;

use crate::expr_text::operand::operand;

use crate::expr_text::pipeline_text::pipeline_text;

use crate::expr_text::receiver::receiver;

use crate::expr_text::record_text::record_text;

use crate::expr_text::rust_hatch_text::rust_hatch_text;

use crate::expr_text::section_text::section_text;

use crate::stmt_text::block_text::block_text;

use crate::stmt_text::case_arm_text::case_arm_text;

use crate::stmt_text::code_text::code_text;

use redox_ast::{BinaryOp, Expr};

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
/// before a method's arguments (row 6), a `:` ascription (row 8), and
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
    Expr::Call {
      callee,
      args,
      is_dollar: true,
      ..
    } if args.len() == 1 => dollar_text(callee, &args[0], indent, source),
    Expr::Call { callee, args, .. } => {
      let mut out = argument(callee, indent, source);
      for arg in args {
        out.push(' ');
        out.push_str(&argument(arg, indent, source));
      }
      out
    }
    Expr::MethodRef { name, .. } => format!(".{}", name.text),
    Expr::Field { recv, name, .. } => {
      let head = receiver(recv, indent, source);
      let gap = if head.ends_with('?') { " " } else { "" };
      format!("{}{}.{}", head, gap, name.text)
    }
    Expr::Index { recv, index, .. } => format!(
      "{}[{}]",
      receiver(recv, indent, source),
      expr_text(index, 0, indent, source)
    ),
    Expr::Binary {
      op: BinaryOp::Pipeline,
      lhs,
      rhs,
      ..
    } => pipeline_text(lhs, rhs, indent, source),
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
    Expr::Unary {
      op, operand: inner, ..
    } => {
      // Look **through** a parenthesis before asking whether the operand is
      // itself a unary. Asking the node instead gave `redox fmt` a two-cycle:
      // this arm parenthesised `*x` to keep `**x` from lexing as one token,
      // `can_drop_paren` judged that parenthesis redundant because its content
      // binds at `u8::MAX`, and the two rules undid each other forever —
      // `**inner` -> `*(*inner)` -> `**inner`, measured over four passes of
      // this very file. `*(*x)` is now a fixed point and `**x` reaches it in
      // one pass.
      //
      // `Expr::Field` above records the same lesson in the same words: the
      // question is about the **printed text**, not the tree. This arm asked
      // the tree, and its comment said no corpus case reached it — plan 69's
      // port is that case.
      let naked = match inner.as_ref() {
        Expr::Paren { inner: within, .. } => within,
        _ => inner,
      };
      let printed = match naked.as_ref() {
        Expr::Unary { .. } => format!("({})", expr_text(inner, 0, indent, source)),
        _ => operand(inner, u8::MAX, indent, source),
      };
      format!("{}{}", op.rust_text(), printed)
    }
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
    Expr::MacroCall { name, args, .. } => macro_call_text(name, args, indent, source),
    Expr::RustHatch { code, .. } => rust_hatch_text(code, indent),
    Expr::Ascribe { inner, ty, .. } => {
      format!("{} : {}", operand(inner, 0, indent, source), ty.text)
    }
    Expr::StrLit { .. }
    | Expr::RawLit { .. }
    | Expr::Num { .. }
    | Expr::Char { .. }
    | Expr::StrInterp { .. } => literal_text(expr, source),
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
    Expr::Try { inner, .. } => format!("{}?", receiver(inner, indent, source)),
    Expr::Unit { .. } => String::from("()"),
    Expr::Section { section, .. } => section_text(section, indent, source),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::SelfField { name, .. } => format!("self.{}", name.text),
    Expr::SelfType { name, .. } => format!("Self::{}", name.text),
    Expr::Ident(ident) => ident.text.clone(),
    Expr::Block { label, body, .. } => match label {
      Some(name) => format!("'{}\n{}", name, block_text(body, indent + 1, source)),
      None => block_text(body, indent + 1, source),
    },
    Expr::AsyncBlock { body, .. } => format!("async\n{}", block_text(body, indent + 1, source)),
    Expr::UnsafeBlock { body, .. } => format!("unsafe\n{}", block_text(body, indent + 1, source)),
    Expr::TryBlock { body, .. } => format!("try\n{}", block_text(body, indent + 1, source)),
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
    Expr::If { .. }
    | Expr::Binding(_)
    | Expr::Item { .. }
    | Expr::Return { .. }
    | Expr::Draw { .. }
    | Expr::Await { .. }
    | Expr::Assign { .. } => code_text(expr, indent, source),
    Expr::Guard { .. }
    | Expr::While { .. }
    | Expr::For { .. }
    | Expr::Loop { .. }
    | Expr::Break { .. }
    | Expr::Continue { .. } => code_text(expr, indent, source),
  }
}
