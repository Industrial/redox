use crate::program::item::Item;
use redox_lexer::Trivia;

/// An item's leading trivia — its comments and attributes.
///
/// `Item::span` already answers *where does this item start*, and every
/// variant carries a `leading` beside it; plan 34 needs both to slice an item
/// out of the source with its comments attached, so the second gets an
/// accessor like the first.
///
/// `Fn` is the only variant whose trivia lives one level down in some shapes —
/// a clause carries its own — and this returns the **item's** leading trivia
/// only. A clause's belongs to the clause.
pub fn leading(item: &Item) -> &[Trivia] {
  match item {
    Item::Fn(f) => &f.leading,
    Item::Data(d) => &d.leading,
    Item::Type(a) => &a.leading,
    Item::Class(c) => &c.leading,
    Item::Instance(i) => &i.leading,
    Item::Errors(e) => &e.leading,
    Item::Test(t) => &t.leading,
    Item::Import(i) => &i.leading,
    Item::Module(m) => &m.leading,
    Item::Const(c) => &c.leading,
    Item::InlineRust(r) => &r.leading,
    Item::Extern(e) => &e.leading,
  }
}
