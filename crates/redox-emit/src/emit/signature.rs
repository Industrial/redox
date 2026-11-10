use redox_ast::{Param, Signature};

/// Render a signature as `(a: i32, b: i32) -> i32`, with each part omitted
/// when absent.
///
/// With generic parameters it is `<T>(xs: &[T]) -> &T`, and with bounds a
/// `where` clause follows on its own lines — doc 18 §2.2's lowering, which
/// puts the bounds there rather than inline in the generic list. `indent` is
/// the column the enclosing `fn` starts at, which the `where` lines need and
/// nothing else does.
///
/// Types are emitted from their captured verbatim text (plan 03 §1.1), so
/// every Rust type form round-trips without the emitter knowing any type
/// grammar.
///
/// # The receiver is written first here, and nowhere else
///
/// Plan 67 **marks** the receiver — `self` in a type — so an author may write
/// `area : &self -> f64` or `scaled : f64 -> &self -> Self`, and Rust needs
/// the receiver first either way. The tree keeps the source's order and this
/// function floats it, which is a deliberate split from plan 67 leaf 1's
/// "move it to the front" in `parser::receiver`:
///
/// - `emit_clauses` pairs `sig.params` with each clause's **pattern row** by
///   index to build its `match` scrutinee, so moving one and not the other
///   misaligns the arms — silently, into Rust that can still compile;
/// - moving the pattern row too is a tree rewrite `redox fmt` prints, because
///   `clause_text` prints an equation's patterns from the tree where
///   `signature_line` slices the signature from the source. The formatter
///   would emit a signature in the author's order above an equation in
///   Rust's, which does not re-parse to the same program.
///
/// So plan 61 §11.6's two-consumers rule binds for the pattern row even
/// though plan 67 §1.7 measured it not binding for the signature line, and
/// the ordering belongs to the one consumer that wants it.
///
/// The receiver is the parameter with **no type**: the whole of it lives in
/// the name, which is the shape `parser::receiver` leaves behind and the only
/// way a `Param` is ever built without one.
pub(crate) fn signature(sig: &Signature, indent: usize) -> String {
  let generics = if sig.generics.is_empty() {
    String::new()
  } else {
    format!("<{}>", sig.generics.join(", "))
  };

  let mut ordered: Vec<&Param> = sig.params.iter().collect();
  if let Some(at) = ordered.iter().position(|p| p.ty.is_none()) {
    let marked = ordered.remove(at);
    ordered.insert(0, marked);
  }

  let params: Vec<String> = ordered
    .iter()
    .map(|p| {
      let mut text = String::new();
      if p.is_mut {
        text.push_str("mut ");
      }
      text.push_str(&p.name.text);
      if let Some(ty) = &p.ty {
        text.push_str(": ");
        text.push_str(&ty.text);
      }
      text
    })
    .collect();

  let mut out = format!("{generics}({})", params.join(", "));
  // **A unit return is omitted**, not written `-> ()`: rustfmt removes the
  // arrow, so writing it would be output rustfmt rewrites. The decision lives
  // here rather than in the parser since plan 55 leaf 4 — the tree records
  // what the author declared, and §1.6's unit rule needs that.
  if let Some(ret) = &sig.ret
    && !sig.returns_unit()
  {
    out.push_str(" -> ");
    out.push_str(&ret.text);
  }

  // Doc 18 §2.2's lowering puts the bounds in a `where` clause rather than
  // inline in the generic list. One bound per line with a trailing comma,
  // which is the form rustfmt leaves alone.
  //
  // No trailing newline: the caller decides whether the body's `{` follows a
  // space or a line break, and it has to be a line break after a `where`
  // clause. Ending the clause here with a newline put a stray space before
  // the brace, which rustfmt removes — so the emitter was producing output
  // rustfmt rewrites.
  if !sig.bounds.is_empty() {
    let pad = " ".repeat(indent);
    out.push_str(&format!("\n{pad}where"));
    for (param, bound) in &sig.bounds {
      out.push_str(&format!("\n{pad}    {param}: {bound},"));
    }
  }
  out
}
