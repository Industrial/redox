pub(crate) mod argument;
pub(crate) mod can_drop_paren;
pub(crate) mod comma_list;
pub(crate) mod comprehension_text;
pub(crate) mod dollar_text;
pub(crate) mod lambda_text;
pub(crate) mod literal_text;
pub(crate) mod operand;
pub(crate) mod pipeline_text;
pub(crate) mod power_of;
pub(crate) mod receiver;
pub(crate) mod record_text;
pub(crate) mod section_text;

use crate::expr_text::argument::argument;
use crate::expr_text::can_drop_paren::can_drop_paren;
use crate::expr_text::comma_list::comma_list;
use crate::expr_text::comprehension_text::comprehension_text;
use crate::expr_text::dollar_text::dollar_text;
use crate::expr_text::lambda_text::lambda_text;
use crate::expr_text::literal_text::literal_text;
use crate::expr_text::operand::operand;
use crate::expr_text::pipeline_text::pipeline_text;
use crate::expr_text::receiver::receiver;
use crate::expr_text::record_text::record_text;
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
    // Doc 18 §2.1's `$`, which is the same `Call` spelled without the
    // parenthesis — `dollar_text` is where the spelling is put back, and the
    // arity guard is the parser's invariant restated rather than a case it
    // can produce.
    Expr::Call {
      callee,
      args,
      is_dollar: true,
      ..
    } if args.len() == 1 => dollar_text(callee, &args[0], indent, source),
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
    // Plan 57's method-as-function. A leaf, like an identifier: the arguments
    // and the receiver are the surrounding `Call`'s, printed by the
    // application arm above, which is §11.6's *"it should be an
    // application"* — `.name a b` canonical, with the redundant parenthesis of
    // `(.name a) b` dropped by associativity.
    //
    // `(.name)` is **not** this node — it is `Section::Field`, printed with
    // its parentheses by `section_text`, and that is the whole
    // selector-versus-method distinction in the printer.
    Expr::MethodRef { name, .. } => format!(".{}", name.text),
    // A field on a `?`, written `x? .name` with the space. `?.` was one token
    // until plan 57 deleted optional chaining, and the glued spelling is now
    // `RDX0117` — so printing it would turn a program this formatter parsed
    // into one it cannot, which is the one thing §3 row 3's conservatism is
    // there to prevent. Tested by `formatting_preserves_meaning` over
    // `examples/fallible.sdx`, which is where it was found.
    //
    // The question is about the **printed text** and not the tree: a `Paren`
    // around the `?` is dropped by `can_drop_paren`, so asking the node would
    // miss `(x?).name`.
    Expr::Field { recv, name, .. } => {
      let head = receiver(recv, indent, source);
      let gap = if head.ends_with('?') { " " } else { "" };
      format!("{head}{gap}.{}", name.text)
    }
    Expr::Index { recv, index, .. } => format!(
      "{}[{}]",
      receiver(recv, indent, source),
      expr_text(index, 0, indent, source)
    ),
    // The pipeline is the one operator with a layout of its own, because it is
    // the one the lexer continues across lines — `pipeline_text` records why,
    // and owns the width decision for the whole chain rather than per node.
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
    // `&mut ` already carries its trailing space in `rust_text`, which is
    // also the Redox spelling.
    //
    // The operand is **not** an argument. A prefix operator binds tighter
    // than every binary one and looser than application, so it takes a whole
    // application without parentheses: `examples/errors.sdx` writes
    // `&ConfigError::Missing (String::from "retries")`, and printing the
    // operand as an argument parenthesised the application inside it.
    //
    // `u8::MAX` as the required power states that exactly: only the forms
    // binding tighter than any operator stay bare, so a binary expression
    // gets its parentheses and `-(a + b)` survives.
    //
    // A nested unary is the one case precedence does not cover — `-(-x)`
    // would print as `--x`, which lexes as one token. No corpus case reaches
    // it, so it is guarded here rather than found later.
    Expr::Unary {
      op, operand: inner, ..
    } => {
      let printed = if matches!(**inner, Expr::Unary { .. }) {
        format!("({})", expr_text(inner, 0, indent, source))
      } else {
        operand(inner, u8::MAX, indent, source)
      };
      format!("{}{printed}", op.rust_text())
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
    // Doc 18 §5.2 made a spaced `::` ascribe and a tight one a path; plan 52
    // retired that rule and `:` is the marker, so the
    // spaces here are load-bearing rather than cosmetic — §3 row 8 exists to
    // say the formatter must never tighten one.
    //
    // The inner expression is an **operand** and not an argument: `::` binds
    // looser than every binary operator, which the parser settles —
    // `1 + 2 : u64` is `Ascribe { inner: Binary { .. } }` with no parenthesis
    // in the source. Printing it as an argument added one, and the corpus
    // property reported the program had changed. `operand` at power 0 keeps
    // the parentheses only around the forms `power_of` declines on, which is
    // the right set: a lambda or a second ascription does need them.
    Expr::Ascribe { inner, ty, .. } => {
      format!("{} : {}", operand(inner, 0, indent, source), ty.text)
    }
    // Every literal is sliced verbatim, and `literal_text` records why the
    // stored text is not enough: a block string is kept **cooked**, so
    // printing it rewrote one literal form into another, and a byte string's
    // hex escapes are uppercased on the way in.
    Expr::StrLit { .. }
    | Expr::RawLit { .. }
    | Expr::Num { .. }
    | Expr::Char { .. }
    | Expr::StrInterp { .. } => literal_text(expr, source),
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
    Expr::Try { inner, .. } => format!("{}?", receiver(inner, indent, source)),
    Expr::Unit { .. } => "()".to_string(),
    Expr::Section { section, .. } => section_text(section, indent, source),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::SelfField { name, .. } => format!("self.{}", name.text),
    Expr::SelfType { name, .. } => format!("Self::{}", name.text),
    Expr::Ident(ident) => ident.text.clone(),
    // The block-bearing forms, which §10.15 moved to leaf 4 because their
    // bodies are statement sequences. The first line is unindented and every
    // later line sits at `indent + 1`, matching `code_text`'s convention —
    // the caller has already written whatever this hangs from.
    //
    // An indented block, as a value — plan 55 leaf 7, and `do` / `do 'label`
    // before it. The unlabelled form has **nothing on its own first line**,
    // so the text begins with the block's first statement at `indent + 1`;
    // `binding_text` is the caller that has to know, because `= ` in front of
    // a newline is trailing whitespace.
    Expr::Block { label, body, .. } => match label {
      Some(name) => format!("'{name}\n{}", block_text(body, indent + 1, source)),
      None => block_text(body, indent + 1, source),
    },
    Expr::AsyncBlock { body, .. } => {
      format!("async\n{}", block_text(body, indent + 1, source))
    }
    Expr::UnsafeBlock { body, .. } => {
      format!("unsafe\n{}", block_text(body, indent + 1, source))
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
    // The forms plan 55 leaf 1 promoted out of `StmtKind`. Each has exactly
    // one Redox spelling, which `code_text` writes, and none of them can
    // reach an expression position through today's grammar — `prefix` has no
    // production for any. Printed rather than declined because a formatter
    // that drops a form rewrites the program, which is the one failure
    // `formatting_preserves_meaning` exists to catch.
    Expr::If { .. }
    | Expr::Binding(_)
    | Expr::Item { .. }
    | Expr::Return { .. }
    | Expr::Draw { .. }
    | Expr::Await { .. }
    | Expr::Assign { .. }
    | Expr::Guard { .. }
    | Expr::While { .. }
    | Expr::For { .. }
    | Expr::Loop { .. }
    | Expr::Break { .. }
    | Expr::Continue { .. }
    | Expr::Assert { .. } => code_text(expr, indent, source),
  }
}
