use redox_ast::Expr;

/// Whether an ascription on `inner` chooses what to **build** rather than
/// annotating what `inner` already is.
///
/// Plan 62 (`docs/56 - 62 - Ascription Completion.md`) leaf 1. Doc 18 §6's
/// third disclosed lowering reads the container off the ascription —
/// `[a, b] : C<_>` is `C::from([a, b])`, which is one spelling for the set and
/// map literals doc 03 had and doc 18 deleted. For those the ascription is an
/// input to the lowering, so moving it to a `let` annotation emits the bare
/// array and earns `E0308`.
///
/// A **comprehension** is deliberately absent. Its container also comes from
/// the ascription, but it reaches it through `collect`, and a `let`
/// annotation pins `collect`'s target just as well — so hoisting is safe
/// there and shortens the output. The test is therefore *which lowering reads
/// the type*, not *which construct looks like a collection*.
pub(crate) fn selects_a_container(inner: &Expr) -> bool {
  matches!(inner, Expr::List { .. } | Expr::ArrayRepeat { .. })
}
