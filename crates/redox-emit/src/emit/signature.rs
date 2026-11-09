use redox_ast::Signature;

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
pub(crate) fn signature(sig: &Signature, indent: usize) -> String {
  let generics = if sig.generics.is_empty() {
    String::new()
  } else {
    format!("<{}>", sig.generics.join(", "))
  };

  let params: Vec<String> = sig
    .params
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
  if let Some(ret) = &sig.ret {
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
