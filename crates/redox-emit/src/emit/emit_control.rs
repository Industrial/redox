use crate::emit::emit_control_inline::emit_control_inline;
use crate::emit::emit_stmts::emit_stmts;
use crate::emit::expr::emit_expr;
use crate::emit::label_prefix::label_prefix;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::unparen::unparen;
use redox_ast::StmtKind;

/// Emit a control-flow statement at `indent` spaces — doc 01 §8 and doc 03
/// §7.7, plan 20.
///
/// ```text
/// if c            if c {          while c         while c {
///   …                 …             …         →       …
/// else            } else {        until c         while !(c) {
///   …                 …             …                 …
///                 }
/// for x <- xs     for x in xs {   loop            loop {
///   …                 …             …                 …
/// ```
///
/// **`until` parenthesises its condition.** `until a && b` is
/// `while !(a && b)`, and `while !a && b` is a different loop that differs on
/// the second iteration and nowhere else — doc 02 §5.2's worst failure mode,
/// so the parentheses are not optional.
///
/// Every body is a statement block, so `return`, a nested `if` and a `break`
/// all work inside one without this knowing about them.
pub(crate) fn emit_control(out: &mut Sink<'_>, kind: &StmtKind, indent: usize) {
  match kind {
    StmtKind::If {
      condition,
      then_block,
      otherwise,
    } => {
      pad(out, indent);
      out.push_str(&format!(
        "if {} {{\n",
        emit_expr(unparen(condition), indent)
      ));
      emit_stmts(out, &then_block.stmts, indent + RUST_INDENT, false);

      let Some(otherwise) = otherwise else {
        out.unmark();
        pad(out, indent);
        out.push_str("}\n");
        return;
      };

      // `else if` is an `else` whose block holds one `If`, which is how the
      // parser builds it — so the chain writes itself and needs no third
      // node. Recognising it here is what keeps `} else if c {` on one line
      // rather than opening a nested block rustfmt would flatten.
      out.unmark();
      pad(out, indent);
      if let [only] = otherwise.stmts.as_slice()
        && matches!(only.kind, StmtKind::If { .. })
      {
        // The nested `if` writes its own `if … {`, so its leading pad is
        // already on this line.
        out.push_str("} else ");
        emit_control_inline(out, &only.kind, indent);
        return;
      }
      out.push_str("} else {\n");
      emit_stmts(out, &otherwise.stmts, indent + RUST_INDENT, false);
      out.unmark();
      pad(out, indent);
      out.push_str("}\n");
    }

    StmtKind::While {
      condition,
      negated,
      body,
      label,
    } => {
      pad(out, indent);
      let text = emit_expr(unparen(condition), indent);
      let test = if *negated { format!("!({text})") } else { text };
      out.push_str(&format!("{}while {test} {{\n", label_prefix(label)));
      emit_stmts(out, &body.stmts, indent + RUST_INDENT, false);
      out.unmark();
      pad(out, indent);
      out.push_str("}\n");
    }

    StmtKind::For {
      binder,
      source,
      body,
      label,
    } => {
      pad(out, indent);
      out.push_str(&format!(
        "{}for {} in {} {{\n",
        label_prefix(label),
        binder.text,
        emit_expr(unparen(source), indent)
      ));
      emit_stmts(out, &body.stmts, indent + RUST_INDENT, false);
      out.unmark();
      pad(out, indent);
      out.push_str("}\n");
    }

    StmtKind::Loop { body, label } => {
      pad(out, indent);
      out.push_str(&format!("{}loop {{\n", label_prefix(label)));
      emit_stmts(out, &body.stmts, indent + RUST_INDENT, false);
      out.unmark();
      pad(out, indent);
      out.push_str("}\n");
    }

    StmtKind::Break { label, value } => {
      pad(out, indent);
      out.push_str("break");
      if let Some(name) = label {
        out.push_str(&format!(" '{name}"));
      }
      if let Some(v) = value {
        out.push_str(&format!(" {}", emit_expr(unparen(v), indent)));
      }
      out.push_str(";\n");
    }

    StmtKind::Continue { label } => {
      pad(out, indent);
      out.push_str("continue");
      if let Some(name) = label {
        out.push_str(&format!(" '{name}"));
      }
      out.push_str(";\n");
    }

    _ => unreachable!("`emit_control` is called only for a control-flow statement"),
  }
}
