use redox_ast::Signature;

/// Render a signature as `(a: i32, b: i32) -> i32`, with each part omitted
/// when absent.
///
/// Types are emitted from their captured verbatim text (plan 03 §1.1), so
/// every Rust type form round-trips without the emitter knowing any type
/// grammar.
pub(crate) fn signature(sig: &Signature) -> String {
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

  let mut out = format!("({})", params.join(", "));
  if let Some(ret) = &sig.ret {
    out.push_str(" -> ");
    out.push_str(&ret.text);
  }
  out
}
