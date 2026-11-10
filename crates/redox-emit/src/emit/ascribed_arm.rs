use crate::emit::ascribed_arm::sole_binder::sole_binder;
use redox_ast::Pattern;

pub(crate) mod sole_binder;

/// The `let` that gives an arm's ascription somewhere to live, or `None`.
///
/// Plan 62 (`docs/56 - 62 - Ascription Completion.md`) leaf 3. rustc accepts a
/// type in a pattern in exactly three places — `let PAT: T = e`, let-else, and
/// a function or closure parameter — and a `match` arm is not one of them
/// (*"specifying the type of a pattern isn't supported"*). §11.3 measured that
/// and found the only workable lowering is a `let` at the top of the arm body:
///
/// ```text
/// Err e : VersionError -> "error: #{e}"
///
///   Err(e) => {
///       let e: VersionError = e;
///       format!("error: {}", e)
///   }
/// ```
///
/// # Why the pattern's sole binder
///
/// `Err e : VersionError` pins **`e`**, not the `Result`. §11.3 is explicit
/// that annotating the scrutinee instead *cannot express* this, because it
/// would require naming the `Ok` type too — which the author deliberately did
/// not write, and which is the whole convenience of writing it at the arm.
///
/// An arm that binds nothing, or binds several names, gives the type nowhere
/// to go: those yield `None` and the ascription is dropped rather than guessed
/// at. A wrong binder would compile and annotate something else, which is the
/// one outcome worse than declining — and no syntax says which of several the
/// author meant. §10 records it as a disclosed limit rather than a silent one.
pub(crate) fn ascribed_arm(pattern: &Pattern, ty: &str) -> Option<String> {
  let binder = sole_binder(pattern)?;
  Some(format!("let {binder}: {ty} = {binder};"))
}
