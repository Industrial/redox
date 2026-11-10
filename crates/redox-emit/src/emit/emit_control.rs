use crate::emit::condition_head::condition_head;
use crate::emit::emit_condition::emit_condition;
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
/// for x in xs     for x in xs {   loop            loop {
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
///
/// # `tail` is the `if`'s, and only the `if`'s
///
/// `tail` marks this statement as the block's final one, whose value is the
/// function's. It is forwarded to an `if`'s branch bodies and to **nothing
/// else**, because nothing else here can carry a value:
///
/// - a `while` or `for` loop is `()` in Rust whatever its body ends with;
/// - a `loop`'s value arrives through `break`, which carries it explicitly —
///   which is the whole reason `loop` already read as an expression while
///   `if` did not;
/// - a `guard`'s else block must diverge, so rustc never reads it for a value.
///
/// An `if` **without** an `else` is also not forwarded: `if c { v }` as a
/// function's tail is `()` by Rust's own rule, so its branch stays a
/// statement. Only `if … else …` is an expression, and that is plan 55 leaf 0.
///
/// Before it, this function took no `tail` at all and passed a literal `false`
/// at all six body sites. That single omission is what made plan 22 record
/// `if` as the one control form that is not an expression: a branch written as
/// a bare value emitted `"big";` and rustc answered `E0308`.
pub(crate) fn emit_control(out: &mut Sink<'_>, kind: &StmtKind, indent: usize, tail: bool) {
  match kind {
    StmtKind::If {
      condition,
      then_block,
      otherwise,
    } => {
      pad(out, indent);
      out.push_str(&condition_head("if", condition, indent));
      // `tail && otherwise.is_some()`: see the note above — an `if` with no
      // `else` is `()`, so its branch is never the function's value.
      emit_stmts(
        out,
        &then_block.stmts,
        indent + RUST_INDENT,
        tail && otherwise.is_some(),
      );

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
        emit_control_inline(out, &only.kind, indent, tail);
        return;
      }
      out.push_str("} else {\n");
      emit_stmts(out, &otherwise.stmts, indent + RUST_INDENT, tail);
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
      // A pattern test can never be negated here — the parser rejects
      // `until P <- e` outright (plan 22 §1.3) — so `until` keeps the simple
      // text wrap, and only `while` can need rustfmt's chain breaking.
      if *negated {
        let text = emit_condition(condition, indent);
        out.push_str(&format!("{}while !({text}) {{\n", label_prefix(label)));
      } else {
        out.push_str(&label_prefix(label));
        out.push_str(&condition_head("while", condition, indent));
      }
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

    // Doc 03 §7.1's `guard`, lowering to let-else — plan 22. Its else body is
    // a statement block like any other, so a `return`, a `break` or a panic
    // form all reach it, and rustc is what insists one of them diverges.
    StmtKind::Guard {
      pattern,
      source,
      otherwise,
    } => {
      pad(out, indent);
      out.push_str(&format!(
        "let {} = {} else {{\n",
        pattern.rust_text(),
        emit_expr(unparen(source), indent)
      ));
      emit_stmts(out, &otherwise.stmts, indent + RUST_INDENT, false);
      out.unmark();
      pad(out, indent);
      // The `;` is **required**: let-else is a `let` statement, and omitting
      // it is a parse error rustc reports against this line.
      out.push_str("};\n");
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
