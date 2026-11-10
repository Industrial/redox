use crate::emit::double_braces::double_braces;
use crate::emit::expr::emit_expr;
use crate::emit::format_template::format_template;
use crate::emit::rust_escapes::rust_escapes;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// One Redox macro argument as the Rust arguments it supplies — plan 64
/// leaf 3.
///
/// ```text
/// \-> "no route for #{id}"     "no route for {}", id
/// \-> "a {} b"                 "a {{}} b"
/// \-> f                        f
/// ```
///
/// One argument in, **one or more** out, which is the whole of leaf 3: an
/// interpolated string is a format template and its spliced values, and
/// `format_template` already returns exactly that pair. Its doc comment says
/// it is a pair *"because two callers need it"* — `format!` and `println!`.
/// This is the third, and a third caller is a parameter rather than a second
/// implementation (§11.5).
///
/// # A string literal is a format string here
///
/// `{` in a Redox string is a literal brace — `#{` is the hole — so a string
/// literal reaching a macro is brace-doubled, exactly as `print_call` does
/// it. Without that, `println! \-> "a {} b"` hands Rust a hole with no value
/// and the author's braces become an error about argument count.
///
/// The cost is disclosed rather than hidden: a macro that wants a `{}` to
/// reach it *as* a hole cannot be written this way, and must interpolate
/// instead. Nothing is silently wrong — a string with no brace in it is
/// unaffected, which is every string in the corpus.
pub(crate) fn macro_argument(arg: &Expr, indent: usize) -> Vec<String> {
  match unparen(arg) {
    Expr::StrLit { value, .. } => vec![double_braces(&rust_escapes(value))],
    Expr::StrInterp { parts, .. } => {
      let (template, args) = format_template(parts, indent);
      let mut supplied = vec![format!("\"{template}\"")];
      supplied.extend(args);
      supplied
    }
    other => vec![emit_expr(other, indent)],
  }
}
