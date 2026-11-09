use crate::emit::expr::emit_expr;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// Lower `??` — doc 03 §12.1, confirmed unchanged by doc 18's opening table.
///
/// ```text
/// name = opt ?? "anonymous"     let name = match opt {
///                                   Some(__rdx_v) => __rdx_v,
///                           →       None => "anonymous",
///                               };
/// ```
///
/// `Option`-only, and a **`match` rather than `unwrap_or_else`**. Doc 03
/// §12.1 records three reasons, because the choice is not obvious:
///
/// - `unwrap_or_else`'s closure arity differs between `Option` and `Result`,
///   so a method lowering would be *type-dependent* — and Redox has no types;
/// - a user type carrying an inherent `unwrap_or_else` could otherwise hijack
///   the meaning;
/// - the fallback's laziness is explicit in the generated code.
///
/// The first of those is decisive rather than merely persuasive: doc 03 §2's
/// admission test 1 requires a lowering fixed by the source text alone, and a
/// method call whose shape depends on the operand's type fails it.
///
/// `__rdx_v` is the reserved generated-identifier prefix (doc 02), so the
/// binding cannot shadow anything the author wrote. `??=` is not here: doc 18
/// §7 retires it in favour of its expanded form.
pub(crate) fn coalesce(lhs: &Expr, rhs: &Expr, indent: usize) -> String {
  let arm_pad = " ".repeat(indent + RUST_INDENT);
  let close_pad = " ".repeat(indent);
  format!(
    "match {} {{\n{arm_pad}Some(__rdx_v) => __rdx_v,\n{arm_pad}None => {},\n{close_pad}}}",
    emit_expr(unparen(lhs), indent),
    emit_expr(unparen(rhs), indent + RUST_INDENT)
  )
}
