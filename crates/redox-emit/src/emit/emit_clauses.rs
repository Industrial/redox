use crate::emit::arm::emit_arm;
use crate::emit::emit_body::emit_body;
use crate::emit::emit_stmts::emit_stmts;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use redox_ast::FnDecl;

/// Emit a function's body from its clauses.
///
/// Doc 18 §2.3. Two shapes, and which one applies is decided by the source
/// text alone — doc 03 §2's admission test 1.
///
/// **One clause, all patterns irrefutable** emits the body directly, which is
/// exactly the Rust doc 03's single-equation form emitted. That is what keeps
/// doc 18 §1's rule — *the lowerings do not move* — true for every function
/// written before §2.3 existed, and it is why the goldens did not change when
/// clauses arrived.
///
/// **Anything else** emits one `match` over the parameters:
///
/// ```text
/// fib :: u64 -> u64            fn fib(__rdx_a0: u64) -> u64 {
/// fib 0 = 0                        match __rdx_a0 {
/// fib 1 = 1                            0 => 0,
/// fib n = fib (n - 1) + …              1 => 1,
///                                      n => fib(n - 1) + …,
///                                  }
///                              }
/// ```
///
/// A parameter whose pattern *tests* rather than binds has no name to take,
/// so `signature_from` gives it `__rdx_a0` — the generated-identifier prefix
/// doc 02 reserves — and the pattern moves into the arm.
///
/// With more than one parameter the scrutinee is a tuple, `match (a, b)`,
/// which is Rust's own way of matching several values at once and needs no
/// temporary.
/// `base` is the column the enclosing `fn` starts at — zero for a top-level
/// function, one level in for a `trait` or `impl` member. The body goes one
/// level inside that.
pub(crate) fn emit_clauses(out: &mut Sink<'_>, f: &FnDecl, base: usize) {
  let tail = f.sig.ret.is_some();

  let simple = f.clauses.len() == 1
    && f.clauses[0]
      .patterns
      .iter()
      .all(redox_ast::Pattern::is_irrefutable);
  if simple {
    // `where` locals come **first**, which is the only order Rust allows: a
    // `let` has to precede its use. Haskell writes them after the expression
    // that uses them, and that reordering is the whole of doc 18 §2.2's
    // lowering.
    emit_stmts(out, &f.clauses[0].wheres, base + RUST_INDENT, false);
    // The body is the function's own, so a narrow two-armed guard ladder may
    // go on one line — unless a `where` local precedes it, which is a
    // position rustfmt's behaviour has not been measured at and so is not
    // claimed.
    let joinable = f.clauses[0].wheres.is_empty();
    emit_body(out, &f.clauses[0].body, base + RUST_INDENT, tail, joinable);
    return;
  }

  let names: Vec<&str> = f.sig.params.iter().map(|p| p.name.text.as_str()).collect();
  let scrutinee = match names.as_slice() {
    [one] => (*one).to_string(),
    many => format!("({})", many.join(", ")),
  };

  pad(out, base + RUST_INDENT);
  out.push_str(&format!("match {scrutinee} {{\n"));

  for clause in &f.clauses {
    let row: Vec<String> = clause
      .patterns
      .iter()
      .map(redox_ast::Pattern::rust_text)
      .collect();
    let arm = match row.as_slice() {
      [one] => one.clone(),
      many => format!("({})", many.join(", ")),
    };

    emit_arm(out, clause, &arm, base + RUST_INDENT * 2, tail);
  }

  pad(out, base + RUST_INDENT);
  out.push_str("}\n");
}
