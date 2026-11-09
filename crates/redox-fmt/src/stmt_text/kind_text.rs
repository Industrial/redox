use crate::expr_text::expr_text;
use crate::pattern_text::pattern_text;
use crate::stmt_text::binding_text::binding_text;
use crate::stmt_text::block_text::block_text;
use crate::stmt_text::condition_text::condition_text;
use crate::stmt_text::else_text::else_text;
use crate::stmt_text::label_text::label_text;
use redox_ast::StmtKind;

/// A statement's code, with its **first line unindented**.
///
/// That convention is what lets `else if` work: `else_text` puts this after
/// `else ` on a line it has already indented. Every later line carries its own
/// full indentation, computed from `indent` — so a nested block is at
/// `indent + 1` and a caller never has to re-indent anything this returns.
///
/// All fourteen statement kinds. The spellings are `examples/control.sdx`'s
/// and `examples/draw_condition.sdx`'s, which §1.5 makes the style guide.
pub(crate) fn kind_text(kind: &StmtKind, indent: usize, source: &str) -> String {
  // Every nested block is one level deeper than the statement that owns it,
  // which is the whole of §3 row 11 as far as statements are concerned.
  let inner = indent + 1;
  match kind {
    StmtKind::Expr(expr) => expr_text(expr, 0, indent, source),
    StmtKind::Binding(binding) => binding_text(binding, indent, source),
    StmtKind::Return(None) => "return".to_string(),
    StmtKind::Return(Some(value)) => format!("return {}", expr_text(value, 0, indent, source)),
    // `with name = e`. Doc 30 gave `<-` to `await`, so the draw statement took
    // `with`, which is what `parser::try_draw` reads.
    StmtKind::Draw { name, value } => format!(
      "with {} = {}",
      name.text,
      expr_text(value, 0, indent, source)
    ),
    // `<-` is `await` and nothing else, doc 30. With a pattern it binds;
    // without one it awaits for the effect, which `examples/async.sdx` writes
    // as a statement beginning with the arrow.
    StmtKind::Await { pattern, value } => match pattern {
      Some(pattern) => format!(
        "{} <- {}",
        pattern_text(pattern),
        expr_text(value, 0, indent, source)
      ),
      None => format!("<- {}", expr_text(value, 0, indent, source)),
    },
    // Compound assignment. Plan 20 §10 records that **plain** reassignment has
    // no owner, so every `Assign` carries a real operator and there is no
    // bare-`=` case to print.
    StmtKind::Assign { target, op, value } => format!(
      "{} {}= {}",
      expr_text(target, 0, indent, source),
      op.redox_text(),
      expr_text(value, 0, indent, source)
    ),
    // `guard P = e else` and a diverging block — doc 03 §7.1's let-else.
    StmtKind::Guard {
      pattern,
      source: from,
      otherwise,
    } => format!(
      "guard {} = {} else\n{}",
      pattern_text(pattern),
      expr_text(from, 0, indent, source),
      block_text(otherwise, inner, source)
    ),
    StmtKind::If {
      condition,
      then_block,
      otherwise,
    } => format!(
      "if {}\n{}{}",
      condition_text(condition, indent, source),
      block_text(then_block, inner, source),
      else_text(otherwise, indent, source)
    ),
    // `until c` is `while !(c)`. The negation is the emitter's business; here
    // it is one keyword or the other, and the condition prints unchanged.
    StmtKind::While {
      condition,
      negated,
      body,
      label,
    } => format!(
      "{}{} {}\n{}",
      label_text(label),
      if *negated { "until" } else { "while" },
      condition_text(condition, indent, source),
      block_text(body, inner, source)
    ),
    StmtKind::For {
      binder,
      source: from,
      body,
      label,
    } => format!(
      "{}for {} in {}\n{}",
      label_text(label),
      binder.text,
      expr_text(from, 0, indent, source),
      block_text(body, inner, source)
    ),
    StmtKind::Loop { body, label } => format!(
      "{}loop\n{}",
      label_text(label),
      block_text(body, inner, source)
    ),
    // A label goes **after** `break`, where a loop's goes before its keyword.
    // `examples/control.sdx` has both, two lines apart.
    StmtKind::Break { label, value } => {
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
    StmtKind::Continue { label } => match label {
      Some(name) => format!("continue '{name}"),
      None => "continue".to_string(),
    },
    StmtKind::Assert(expr) => format!("assert {}", expr_text(expr, 0, indent, source)),
  }
}
