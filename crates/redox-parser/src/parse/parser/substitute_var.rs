use crate::parse::parser::rewritten::rewritten;
use redox_ast::Signature;

/// Replace the class variable in a method's remaining types — doc 18 §5.3.
///
/// "`t` in a signature is a generic parameter, emitted as `T`." Inside a
/// `class` the class variable *is* the implementing type, so it becomes
/// `Self`; inside an `instance` it is the impl's generic parameter, so it
/// becomes that parameter's uppercase name. `subject` is whichever applies.
///
/// ```text
/// class Combine t                  trait Combine {
///   combine : &t -> t -> t    →       fn combine(&self, __rdx_a1: Self) -> Self;
/// ```
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
