use crate::expr_text::expr_text;
use crate::pattern_text::pattern_text;
use redox_ast::Binding;

/// A binding — `n = 0`, `mut n = 0`, `n : u64 = 0`.
///
/// # The separator is one colon, everywhere — and §3 row 8 is now moot
///
/// Row 8 read *"a binding's `::` type annotation → spaced — `n : u64 = 0`"*,
/// and `n : u64 = 0` never parsed: `try_binding` has always read a single
/// `TokenKind::Colon`. The local binding was the one form spelled with `:`
/// while a signature, a `const` item and an ascription took a spaced `::`.
///
/// Plan 52 collapsed that: `:` annotates a type in every position and `::`
/// joins a path and nothing else, so the three forms below differ only in
/// what surrounds them.
///
/// | Form | Separator | |
/// |---|---|---|
/// | A local binding | `:` | `n : u64 = 0` — this function |
/// | A `const` or `static` item | `:` | `threshold : f64 = 0.5` — leaf 2's |
/// | An expression ascription | `:` | `(1 + 2) : u64` — leaf 3's |
///
/// So row 8's *rule* — never tighten a spaced `::`, because tightening it
/// changed an ascription into a path — no longer has anything to protect: the
/// two are different tokens now. §10 records the correction.
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
