use crate::parse::parser::rewritten::rewritten;
use redox_ast::Signature;

/// Replace a variable inherited from the enclosing `class` or `instance` head
/// in a method's remaining types — doc 18 §5.3.
///
/// "`t` in a signature is a generic parameter, emitted as `T`." The variable
/// belongs to the head rather than to the method, so `generalise` reserves it
/// and this rewrites it: inside a `class` it is one of the **trait's** own
/// parameters, inside an `instance` it is the impl's generic parameter.
/// `subject` is that parameter's Rust spelling in either case.
///
/// ```text
/// class Combine t                     trait Combine<T> {
///   combine : &self -> t -> t    →       fn combine(&self, __rdx_a1: T) -> T;
/// ```
///
/// Plan 67 is what made `subject` uniform. Before it a class's single variable
/// stood for the *implementing* type and substituted to `Self`; with the
/// receiver marked, `Self` names the implementing type directly and there is
/// nothing left here standing in for it.
///
/// **Only a type that is exactly the variable, or `&`/`&mut ` plus the
/// variable, is substituted.** A nested occurrence — `Vec<t>`,
/// `Result<t, Error>` — is left alone, and that is a real gap rather than an
/// oversight: a type in Redox is captured verbatim text (doc 03 §14.1), and
/// rewriting inside one needs a type parser, which §14.1 exists to avoid
/// having. Rustc reports the unresolved `t`, which is a worse message than a
/// Redox diagnostic would be and a much better outcome than a silently wrong
/// type. Doc 18 §11.5 records it as known.
pub(crate) fn substitute_var(sig: &mut Signature, var: &str, subject: &str) {
  // The receiver has already been rewritten and carries no type, so this
  // walks what is left.
  for param in &mut sig.params {
    if let Some(ty) = &mut param.ty {
      ty.text = rewritten(&ty.text, var, subject);
    }
  }
  if let Some(ret) = &mut sig.ret {
    ret.text = rewritten(&ret.text, var, subject);
  }
}
