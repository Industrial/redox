use crate::ParseError;
use redox_diag::Code;
use redox_lexer::Span;

/// Refuse Rust's method-call syntax — `RDX0115`, plan 57 leaf 3.
///
/// A method is a function whose **last** argument is its receiver, so
/// `s.trim ()` is `.trim s`, `m.insert k v` is `.insert k v m`, and a
/// pipeline writes the receiver first: `s |> .trim`. §1.1 measured four
/// separate defects in generated Rust that all came from a receiver-bearing
/// call form colliding with juxtaposition, and none of them is spellable once
/// `.` means field access and nothing else.
///
/// # Two shapes, one code
///
/// §11.2's finding, and the reason this is a function rather than a branch:
/// the retired form has **two** AST shapes and deleting one does not retire
/// it.
///
/// - `x.f (a)` was `Expr::MethodCall`, built by [`postfix`](super::postfix)
///   when a `(` followed the name.
/// - `x.f a` is an application over an `Expr::Field`, because `postfix` has
///   no paren-call step at all (doc 18 §2.1 makes parentheses grouping only).
///
/// So refusing only the first would leave `x.f (a)` re-parsing as the second
/// and emitting the identical `x.f(a)` silently, while `x.f ()` would degrade
/// to `x.f(())` rather than to an error. Both callers raise this, which is why
/// the span is a parameter: `postfix` has the name and
/// [`application`](super::application) has the whole head.
pub(crate) fn method_call_syntax(name: &str, span: Span) -> ParseError {
  ParseError::with_help(
    Code::MethodCallSyntax,
    "method-call syntax is retired",
    format!(
      "a method is a function whose last argument is its receiver, so write `.{name} args recv` \
       or `recv |> .{name} args`. `.` is field access and nothing else (plan 57 §1.1)"
    ),
    span,
  )
}
