use crate::emit::emit_stmts::emit_stmts;
use crate::emit::expr::emit_expr;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::two_armed::two_armed;
use redox_ast::Guard;

/// Emit one guard ladder as an `if`/`else if`/`else` chain, at `indent` spaces.
///
/// Doc 18 §2.3:
///
/// ```text
/// tier spend                       if spend > 10_000 {
///   | spend > 10_000 = "platinum"      "platinum"
///   | otherwise      = "basic"     } else {
///                                      "basic"
///                                  }
/// ```
///
/// `otherwise` carries no condition and so closes the chain with a bare
/// `else`. Lowering it to `if true` would read the same to a human and not to
/// rustc: a chain whose last arm is conditional has an implicit `else` of
/// `()`, so every guarded function with a return type would stop compiling.
/// Doc 18 §1 keeps rustc the authority on types, which only works if what
/// reaches it says what the author meant.
///
/// A ladder with no `otherwise` is left as a conditional chain, and rustc
/// reports the mismatch against the signature. Same division of labour:
/// Redox does not decide whether a ladder is total, and AGENTS.md forbids it
/// growing a checker that could.
pub(crate) fn emit_guards(
  out: &mut Sink<'_>,
  guards: &[Guard],
  indent: usize,
  tail: bool,
  joinable: bool,
) {
  // A two-armed ladder narrow enough goes on one line, because that is what
  // rustfmt does to it — see `SINGLE_LINE_IF_ELSE_MAX_WIDTH`. An `else if`
  // in the middle disowns the rule, which is why doc 18 §2.3's three-row
  // `tier` example was unaffected and a two-row ladder found it.
  //
  // `joinable` is the second half of the rule, and it was found the same
  // way: rustfmt joins an `if`/`else` that is a **function body's** tail and
  // expands the identical expression inside a `match` arm. Only the caller
  // knows which position this is, so only the caller can say.
  if joinable && let Some(one_line) = two_armed(guards, indent) {
    pad(out, indent);
    out.push_str(&one_line);
    out.push('\n');
    return;
  }

  for (i, guard) in guards.iter().enumerate() {
    match (&guard.condition, i) {
      (Some(cond), 0) => {
        pad(out, indent);
        out.push_str(&format!("if {} {{\n", emit_expr(cond, indent)));
      }
      (Some(cond), _) => {
        pad(out, indent);
        out.push_str(&format!("}} else if {} {{\n", emit_expr(cond, indent)));
      }
      // `otherwise` in first position is a ladder of one arm, which is not a
      // conditional at all — emit the body with no wrapper.
      (None, 0) => {
        emit_stmts(out, &guard.body.stmts, indent, tail);
        return;
      }
      (None, _) => {
        pad(out, indent);
        out.push_str("} else {\n");
      }
    }
    // Every arm is a tail position, for the same reason a `match` arm is.
    emit_stmts(out, &guard.body.stmts, indent + RUST_INDENT, tail);
  }
  pad(out, indent);
  out.push_str("}\n");
}
