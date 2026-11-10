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
    // Plan 55 leaf 5. `item_text` is reused rather than reimplemented, so a
    // local `data` round-trips through exactly the printer a module-scope one
    // does — which is the property `formatting_preserves_meaning` would
    // otherwise catch the hard way.
    //
    // Reached only through `expr_text`, which delegates every promoted form
    // here. `stmt_text` handles an item itself, before it writes the pad —
    // see the note there, and `emit_stmt`'s matching one. An item in a nested
    // *expression* position is refused by the emitter (`unlowerable`), so this
    // arm exists to keep the match exhaustive rather than to be used.
    Expr::Item { item, .. } => item_text(item, indent, source),
    Expr::Return { value: None, .. } => "return".to_string(),
    Expr::Return {
      value: Some(value), ..
    } => format!("return {}", expr_text(value, 0, indent, source)),
    // `with name = e`. Doc 30 gave `<-` to `await`, so the draw statement took
    // `with`, which is what `parser::try_draw` reads.
    Expr::Draw { name, value, .. } => format!(
      "with {} = {}",
      name.text,
      expr_text(value, 0, indent, source)
    ),
    // `<-` is `await` and nothing else, doc 30. With a pattern it binds;
    // without one it awaits for the effect, which `examples/async.sdx` writes
    // as a statement beginning with the arrow.
    Expr::Await { pattern, value, .. } => match pattern {
      Some(pattern) => format!(
        "{} <- {}",
        pattern_text(pattern),
        expr_text(value, 0, indent, source)
      ),
      None => format!("<- {}", expr_text(value, 0, indent, source)),
    },
    // Assignment. Plan 20 §10 recorded that **plain** reassignment had no
    // owner, so every `Assign` carried a real operator and there was no
    // bare-`=` case to print. Plan 54 gave it one, and it is spelled `:=`
    // rather than `=` — printing a bare `=` here would turn an assignment
    // back into a binding, which is the whole distinction the row exists to
    // make.
    Expr::Assign {
      target, op, value, ..
    } => format!(
      "{} {}= {}",
      expr_text(target, 0, indent, source),
      op.map_or(":", |op| op.redox_text()),
      expr_text(value, 0, indent, source)
    ),
    // `guard P = e else` and a diverging block — doc 03 §7.1's let-else.
    Expr::Guard {
      pattern,
      source: from,
      otherwise,
      ..
    } => format!(
      // Plan 60: scrutinee first here too.
      "guard {} is {} else\n{}",
      expr_text(from, 0, indent, source),
      pattern_text(pattern),
      block_text(otherwise, inner, source)
    ),
    Expr::If {
      condition,
      then_block,
      otherwise,
      ..
    } => format!(
      "if {}\n{}{}",
      condition_text(condition, indent, source),
      block_text(then_block, inner, source),
      else_text(otherwise, indent, source)
    ),
    // `until c` is `while !(c)`. The negation is the emitter's business; here
    // it is one keyword or the other, and the condition prints unchanged.
    Expr::While {
      condition,
      negated,
      body,
      label,
      ..
    } => format!(
      "{}{} {}\n{}",
      label_text(label),
      if *negated { "until" } else { "while" },
      condition_text(condition, indent, source),
      block_text(body, inner, source)
    ),
    Expr::For {
      binder,
      source: from,
      body,
      label,
      ..
    } => format!(
      "{}for {} in {}\n{}",
      label_text(label),
      binder.text,
      expr_text(from, 0, indent, source),
      block_text(body, inner, source)
    ),
    Expr::Loop { body, label, .. } => format!(
      "{}loop\n{}",
      label_text(label),
      block_text(body, inner, source)
    ),
    // A label goes **after** `break`, where a loop's goes before its keyword.
    // `examples/control.sdx` has both, two lines apart.
    Expr::Break { label, value, .. } => {
      let mut out = String::from("break");
      if let Some(name) = label {
        out.push_str(&format!(" '{name}"));
      }
      if let Some(value) = value {
        out.push(' ');
        out.push_str(&expr_text(value, 0, indent, source));
      }
      out
    }
    Expr::Continue { label, .. } => match label {
      Some(name) => format!("continue '{name}"),
      None => "continue".to_string(),
    },
    Expr::Assert { value, .. } => format!("assert {}", expr_text(value, 0, indent, source)),
    // Every other form is a value expression, and its canonical spelling is
    // the same whatever position it sits in. A wildcard rather than a list of
    // thirty-three, because `expr_text` is the exhaustive match: a new form
    // has to be considered there, and arriving here it is simply printed.
    value => expr_text(value, 0, indent, source),
  }
}
