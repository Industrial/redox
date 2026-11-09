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
pub(crate) mod verbatim;

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
use crate::expr_text::verbatim::verbatim;
use redox_ast::Expr;

/// One expression in canonical Redox — plan 34, leaf 3.
///
/// `required` is the binding power the surrounding context demands of this
/// expression: 0 at a statement's top level, an operator's power inside a
/// binary expression. It is what decides whether a parenthesis the author
/// wrote can be dropped, and it is threaded through rather than recomputed
/// because only the *parent* knows it.
///
/// # Why `source` is a parameter
///
/// A construct this leaf does not print is sliced out of the source and
/// returned verbatim, which is plan 34's criterion 7 — *"a construct the
/// formatter does not yet print is passed through verbatim rather than
/// mangled"*. Leaf 1 established that discipline for items; this is the same
/// thing one level down, and it is why leaf 3 can ship before leaf 4 exists.
///
/// The forms left verbatim are exactly the **block-bearing** ones — `if`,
/// `do`, `case`, `async` and `try` blocks — and §10 records why that is not an
/// arbitrary cut: their bodies are statement sequences, so printing them is
/// printing statements, which is leaf 4. §4's leaf split reads as
/// expressions / statements / items, and the real boundary is single-line
/// expressions / block-bearing constructs / items.
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
pub fn expr_text(expr: &Expr, required: u8, source: &str) -> String {
  match expr {
    // Doc 18 §2.1: juxtaposition is the call form, parentheses are grouping.
    // So the arguments are separated by spaces and each is parenthesised only
    // if it would otherwise read as more application.
    Expr::Call { callee, args, .. } => {
      let mut out = argument(callee, source);
      for arg in args {
        out.push(' ');
        out.push_str(&argument(arg, source));
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
      argument(recv, source),
      name.text,
      comma_list(args, source)
    ),
    Expr::Field { recv, name, .. } => format!("{}.{}", argument(recv, source), name.text),
    Expr::Index { recv, index, .. } => format!(
      "{}[{}]",
      argument(recv, source),
      expr_text(index, 0, source)
    ),
    Expr::Binary { op, lhs, rhs, .. } => {
      // Left-associative, so the right operand must bind one rung tighter for
      // its parentheses to be redundant.
      let power = op.power();
      format!(
        "{} {} {}",
        operand(lhs, power, source),
        op.redox_text(),
        operand(rhs, power + 1, source)
      )
    }
    // `&mut ` already carries its trailing space in `rust_text`, which is
    // also the Redox spelling.
    Expr::Unary {
      op, operand: inner, ..
    } => {
      format!("{}{}", op.rust_text(), argument(inner, source))
    }
    // §3 row 3, the one normalisation that can change meaning. `can_drop_paren`
    // is the whole decision and it is conservative by construction.
    Expr::Paren { inner, .. } => {
      if can_drop_paren(inner, required) {
        expr_text(inner, required, source)
      } else {
        format!("({})", expr_text(inner, 0, source))
      }
    }
    Expr::Path { text, .. } => text.clone(),
    Expr::Lambda {
      params,
      body,
      is_move,
      ..
    } => lambda_text(params, body, *is_move, source),
    Expr::OptionalChain { head, segments, .. } => {
      let mut out = argument(head, source);
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
      format!("{} :: {}", operand(inner, 0, source), ty.text)
    }
    // The lexer keeps a string's source text, quotes and escapes included, so
    // there is nothing to re-escape and no way to lose an escape.
    Expr::StrLit { value, .. } => value.clone(),
    Expr::RawLit { text, .. } => text.clone(),
    Expr::Num { text, .. } => text.clone(),
    Expr::StrInterp { parts, .. } => interp_text(parts, source),
    // §3 row 12: no trailing comma, except the one that makes `(a,)` a tuple
    // rather than a group — plan 33 §10.3.
    Expr::Tuple { items, .. } => {
      if items.len() == 1 {
        format!("({},)", expr_text(&items[0], 0, source))
      } else {
        format!("({})", comma_list(items, source))
      }
    }
    Expr::List { items, .. } => format!("[{}]", comma_list(items, source)),
    Expr::ArrayRepeat { value, count, .. } => format!(
      "[{}; {}]",
      expr_text(value, 0, source),
      expr_text(count, 0, source)
    ),
    Expr::Record { head, fields, .. } => record_text(head, fields, source),
    Expr::Comprehension {
      element,
      qualifiers,
      ..
    } => comprehension_text(element, qualifiers, source),
    Expr::Try { inner, .. } => format!("{}?", argument(inner, source)),
    Expr::Unit { .. } => "()".to_string(),
    Expr::Section { section, .. } => section_text(section, source),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::Char { value, .. } => char_text(*value),
    Expr::SelfField { name, .. } => format!("self.{}", name.text),
    Expr::SelfType { name, .. } => format!("Self::{}", name.text),
    Expr::Ident(ident) => ident.text.clone(),
    // The block-bearing forms. Leaf 4's, and verbatim until then.
    Expr::If { .. }
    | Expr::Do { .. }
    | Expr::Case { .. }
    | Expr::AsyncBlock { .. }
    | Expr::TryBlock { .. } => verbatim(expr, source),
  }
}
