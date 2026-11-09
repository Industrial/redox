use crate::emit::expr::emit_expr;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// Lower `catch` — doc 03 §12.2, kept unchanged by doc 18 §8.
///
/// ```text
/// n = parse s catch 0          let n = match parse(s) {
///                                  Ok(__rdx_v) => __rdx_v,
///                          →       Err(_) => 0,
///                              };
///
/// n = parse s catch (e) ->     let n = match parse(s) {
///   log e                          Ok(__rdx_v) => __rdx_v,
///   0                              Err(e) => { log(e); 0 },
///                              };
/// ```
///
/// `Result`-only, mirroring `??` — doc 03 §12 keeps `Option` and `Result`
/// *"visually distinct and non-interchangeable"*. A `match` rather than
/// `unwrap_or_else` for §12.1's three reasons, the decisive one being that
/// `unwrap_or_else`'s closure arity differs between the two types, so a method
/// lowering would be type-dependent and fail doc 03 §2's first admission test.
///
/// # The binder comes from a lambda, not from a second syntax
///
/// §12.2's second form writes the fallback as `(e) -> …`, which is doc 18 §4's
/// lambda with its own parameter list — so a one-parameter lambda on the right
/// contributes its parameter to the `Err` pattern and its body to the arm.
/// Plan 28 §1.5: a second syntax for a shape the language already has is a
/// second thing to keep consistent.
///
/// A lambda of any other arity is **not** unwrapped. It stays the arm's value,
/// which is a closure where a value was wanted, and rustc reports it — the
/// alternative is Redox deciding what the author meant.
pub(crate) fn catch_fallback(subject: &Expr, fallback: &Expr, indent: usize) -> String {
  let arm_pad = " ".repeat(indent + RUST_INDENT);
  let close_pad = " ".repeat(indent);

  let (binding, value) = match unparen(fallback) {
    Expr::Lambda { params, body, .. } if params.len() == 1 => (
      params[0].name.text.clone(),
      emit_expr(unparen(body), indent + RUST_INDENT),
    ),
    other => ("_".to_string(), emit_expr(other, indent + RUST_INDENT)),
  };

  format!(
    "match {} {{\n{arm_pad}Ok(__rdx_v) => __rdx_v,\n{arm_pad}Err({binding}) => {value},\n{close_pad}}}",
    emit_expr(unparen(subject), indent)
  )
}
