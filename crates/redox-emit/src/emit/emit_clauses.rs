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
pub(crate) fn emit_clauses(out: &mut Sink<'_>, f: &FnDecl) {
  let tail = f.sig.ret.is_some();

  let simple = f.clauses.len() == 1
    && f.clauses[0]
      .patterns
      .iter()
      .all(redox_ast::Pattern::is_irrefutable);
  if simple {
    emit_stmts(out, &f.clauses[0].body.stmts, RUST_INDENT, tail);
    return;
  }

  let names: Vec<&str> = f.sig.params.iter().map(|p| p.name.text.as_str()).collect();
  let scrutinee = match names.as_slice() {
    [one] => (*one).to_string(),
    many => format!("({})", many.join(", ")),
  };

  pad(out, RUST_INDENT);
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

    pad(out, RUST_INDENT * 2);
    out.push_str(&format!("{arm} => {{\n"));
    // Every arm is a tail position: a `match` arm's value is the arm's value,
    // whether or not the function returns anything.
    emit_stmts(out, &clause.body.stmts, RUST_INDENT * 3, tail);
    pad(out, RUST_INDENT * 2);
    out.push_str("}\n");
  }

  pad(out, RUST_INDENT);
  out.push_str("}\n");
}
