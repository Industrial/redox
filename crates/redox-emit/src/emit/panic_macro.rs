/// The Rust macro a bare panic-family name spells — doc 03 §12.6.
///
/// ```text
/// todo            todo!()
/// unimplemented   unimplemented!()
/// unreachable     unreachable!()
/// ```
///
/// §12.6 adds these spellings because "partial code and invariant assertions
/// are routine" and the macros were reachable only through raw macro syntax.
///
/// `panic "message"` is not here: it takes an argument, so it is a *call* and
/// is lowered where calls are. These three are bare names, which is why they
/// need a lookup at all — nothing else distinguishes `todo` from a function
/// the author wrote and then never defined.
///
/// Returns `None` for every other name, so an ordinary identifier passes
/// through untouched.
pub(crate) fn panic_macro(name: &str) -> Option<&'static str> {
  Some(match name {
    "todo" => "todo!()",
    "unimplemented" => "unimplemented!()",
    "unreachable" => "unreachable!()",
    _ => return None,
  })
}
