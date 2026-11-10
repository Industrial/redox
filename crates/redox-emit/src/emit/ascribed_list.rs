use crate::emit::expr_list::expr_list;
use crate::emit::unparen::unparen;
use redox_ast::{Expr, TypeExpr};

/// Doc 18 §6's third disclosed lowering: `[a, b] : C<_>` becomes
/// `C::from([a, b])`.
///
/// ```text
/// seen = [1, 2] : HashSet<i32>    HashSet::from([1, 2])
/// xs   = [1, 2] : Vec<i32>        Vec::from([1, 2])
/// ```
///
/// Type-free, and §6 says why: **the container is named in the ascription**,
/// so the lowering reads the name it was given rather than inferring one.
/// That is what covers the set and map literals doc 03 had and doc 18
/// deletes — `C::from` is one spelling for all of them.
///
/// The head is the ascribed type up to its first `<`, which is the whole of
/// the extraction: a type is verbatim text (doc 03 §14.1), and `HashSet` is
/// what precedes the generic arguments of `HashSet<i32>`.
///
/// Returns `None` when the ascribed expression is not a list literal, so the
/// ordinary ascription block handles everything else — including a
/// *comprehension*, which already ends in `.collect()` and needs the block to
/// give that collect its type.
///
/// **This is the ascription's value, not a replacement for it.** §6's table
/// writes the whole lowering as `C::from([a, b])`, and that drops the type's
/// generic arguments: `[] : Vec<i32>` becomes `Vec::from([])` with nothing
/// to infer the element from, and `[1, 2] : Vec<i64>` becomes a `Vec<i32>`
/// because an unsuffixed integer literal defaults. Emitting it *inside* the
/// ascription block keeps the type exactly as written and still converts —
/// `{ let __rdx_t: Vec<i64> = Vec::from([1, 2]); __rdx_t }` — so the two
/// lowerings compose instead of one replacing the other.
pub(crate) fn ascribed_list(inner: &Expr, ty: &TypeExpr, indent: usize) -> Option<String> {
  let Expr::List { items, .. } = unparen(inner) else {
    return None;
  };
  let head = ty.text.split('<').next().unwrap_or(&ty.text).trim();
  Some(format!("{head}::from([{}])", expr_list(items, indent)))
}
