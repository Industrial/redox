use redox_ast::{Item, leading};

/// Where an item's text begins — its first leading trivium, or the item
/// itself — plan 34, leaf 1.
///
/// A comment or attribute above a declaration travels with it, which is the
/// attachment plans 17 and 18 built: trivia belongs to the construct below it.
/// The formatter inherits that decision rather than re-making it.
pub(crate) fn item_start(item: &Item) -> usize {
  leading(item)
    .first()
    .map_or(item.span().0, |trivium| trivium.span.0)
}
