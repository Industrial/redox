use crate::expr_text::expr_text;
use crate::pattern_text::pattern_text;
use redox_ast::Binding;

/// A binding — `n = 0`, `mut n = 0`, `n : u64 = 0`.
///
/// # The separator is one colon, and §3 row 8 is about something else
///
/// Row 8 reads *"a binding's `::` type annotation → spaced — `n :: u64 = 0`"*,
/// and `n :: u64 = 0` does not parse. `try_binding` reads a single
/// `TokenKind::Colon`; `::` is its own token, so a `mut` binding written with
/// it fails with *"expected `=` in a binding"*.
///
/// Three forms share the look and none of them is the others:
///
/// | Form | Separator | |
/// |---|---|---|
/// | A local binding | `:` | `n : u64 = 0` — this function |
/// | A `const` or `static` item | `::` | `threshold :: f64 = 0.5` — leaf 2's |
/// | An expression ascription | `::` | `(1 + 2) :: u64` — leaf 3's |
///
/// So row 8's *rule* holds — a spaced `::` ascribes and a tight one is a path,
/// so the formatter must never tighten one — and its *example* belongs to one
/// of the other two rows. §10 records the correction.
pub(crate) fn binding_text(binding: &Binding, indent: usize, source: &str) -> String {
  let prefix = if binding.is_mut { "mut " } else { "" };
  let ty = match &binding.ty {
    Some(ty) => format!(" : {}", ty.text),
    None => String::new(),
  };
  format!(
    "{prefix}{}{ty} = {}",
    pattern_text(&binding.pattern),
    expr_text(&binding.value, 0, indent, source)
  )
}
