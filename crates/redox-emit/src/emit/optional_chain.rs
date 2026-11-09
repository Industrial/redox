use crate::emit::receiver::receiver;
use redox_ast::Expr;

/// Lower a `?.` chain — doc 01 §13.1 with doc 02 §6.3's lowering, plan 28.
///
/// ```text
/// n = user?.profile?.name   →   let n = (|| Some(
///                                   user.as_ref()?.profile.as_ref()?.name.as_ref()
///                               ))();
/// ```
///
/// # The entire unification is rustc's
///
/// Doc 02 §6.3 *"delegates the entire `and_then`/`map` unification to rustc
/// via `?` inside a closure returning `Option`"*. Hand-written combinators
/// would need `and_then` for a link that yields an `Option` and `map` for one
/// that does not — a distinction only a type checker can make, and Redox has
/// none. The closure needs no such distinction: every link is `?`.
///
/// For the same reason **nested options are not flattened**, which doc 01
/// §13.1 requires: `?` does not flatten, so there is no flattening code to
/// write and none to get wrong.
///
/// # It borrows, and that is the design claim
///
/// Doc 03 §12.3 resolves doc 02 §13's question 7 outright — *"the chain
/// borrows and never clones"* — and doc 01 §25.1's no-silent-clone rule is
/// absolute. Owning the result is written explicitly:
///
/// ```text
/// n = user?.profile?.name.clone!
/// ```
///
/// §12.3 admits the cost — *"more verbose in the common `String` case"* — and
/// takes it, *"visible at the call site rather than hidden in the lowering."*
///
/// # One correction to §12.3's published lowering
///
/// §12.3 writes `.as_ref()` on **every** link, including the last:
///
/// ```rust,ignore
/// (|| Some(user.as_ref()?.profile.as_ref()?.name.as_ref()))()
/// ```
///
/// That does not compile when the final field is not an `Option`. For a
/// `String` field it resolves to `AsRef::as_ref`, which `String` does not
/// implement for `String`:
///
/// ```text
/// the trait bound `String: AsRef<String>` is not satisfied
/// ```
///
/// Worse than not compiling, it is **type-dependent**: `.as_ref()` means
/// `Option::as_ref` on an `Option` field and `AsRef::as_ref` on anything
/// else. Doc 03 §2's first admission test requires a lowering fixed by the
/// source text alone, and §12.1 calls that requirement *decisive rather than
/// merely persuasive* when it rejected `unwrap_or_else` for exactly this. The
/// published lowering fails its own test.
///
/// So the final link takes **no** method and the whole path takes a `&`:
///
/// ```rust,ignore
/// (|| Some(&user.as_ref()?.profile.as_ref()?.name))()
/// ```
///
/// `&` borrows whatever is there, whatever its type, which is what §12.3
/// wanted and says. It also keeps the no-flattening property doc 01 §13.1
/// requires: a final field of type `Option<String>` yields
/// `Option<&Option<String>>`, nested exactly once, because `?` does not
/// flatten and `&` does not unwrap.
pub(crate) fn optional_chain(head: &Expr, segments: &[String], indent: usize) -> String {
  // `&` binds looser than `?` and `.` in Rust, so it covers the whole place
  // expression without parentheses.
  let mut body = format!("&{}.as_ref()?", receiver(head, indent));
  for (i, name) in segments.iter().enumerate() {
    body.push('.');
    body.push_str(name);
    // Every link but the last unwraps its `Option`; the last is the value the
    // leading `&` borrows.
    if i + 1 < segments.len() {
      body.push_str(".as_ref()?");
    }
  }
  format!("(|| Some({body}))()")
}
