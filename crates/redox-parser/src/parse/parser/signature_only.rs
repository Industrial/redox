use redox_ast::{Ident, Param, Signature, TypeExpr};
use redox_diag::Span;

/// Build a [`Signature`] from a spine **with no equation** — doc 18 §2.5.
///
/// A required method is a signature and nothing else:
/// `area :: &t -> f64` becomes `fn area(&self) -> f64;`. There is no pattern
/// row to take parameter names from, so each parameter takes the generated
/// name doc 02 reserves — `__rdx_a0` — exactly as a *testing* pattern's
/// parameter does in `signature_from`.
///
/// Separate from `signature_from` rather than a special case inside it,
/// because the two disagree about what an empty row means: there it is an
/// arity error worth reporting with both counts, here it is the normal case.
/// Folding them together would have made a genuinely wrong equation —
/// `area = …` under `area :: &t -> f64` — silently acquire a parameter.
pub(crate) fn signature_only(spine: &[TypeExpr], span: Span) -> Signature {
  let (ret, params) = spine
    .split_last()
    .expect("parse_spine returns at least one type");

  Signature {
    params: params
      .iter()
      .enumerate()
      .map(|(i, ty)| Param {
        name: Ident {
          text: format!("__rdx_a{i}"),
          span: ty.span,
        },
        ty: Some(ty.clone()),
        is_mut: false,
        span: ty.span,
      })
      .collect(),
    // A unit return type is omitted, not written `-> ()`, for the reason
    // `signature_from` gives.
    ret: (ret.text != "()").then(|| ret.clone()),
    span,
  }
}
