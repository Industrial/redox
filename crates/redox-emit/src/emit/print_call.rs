use crate::emit::double_braces::double_braces;
use crate::emit::expr_list::expr_list;
use crate::emit::rust_escapes::rust_escapes;
use crate::emit::unparen::unparen;
use redox_ast::Expr;

/// Lower a `print` call.
///
/// `print` is the one sanctioned sugar of its kind (doc 01 §3.3, doc 03
/// §10.9) and it is a **dated loan**: it is exactly Emerald's hardcoded `puts`
/// keyword, which papered over a general parenless-call ambiguity for 89
/// plans. Plan 19 resolves the general case and this special case dissolves.
///
/// A single string literal becomes `println!(lit)`; anything else becomes
/// `println!("{}", …)`, because a bare `println!(expr)` is only valid when the
/// argument *is* a literal format string.
///
/// That literal is then a **format string**, so it needs both of the emitter's
/// literal transformations: `rust_escapes` to be valid Rust at all, and
/// `double_braces` so a brace the author wrote prints as a brace instead of
/// naming a variable. Plan 08's corpus found this path emitting neither —
/// `print_call` reads `StrLit`'s value directly rather than going through
/// `emit_expr`, so it had quietly missed both.
pub(crate) fn print_call(args: &[Expr], indent: usize) -> String {
  // Doc 18 §2.1: a grouping paren around the argument carries no meaning, so
  // `print ("x")` must take the same arm as `print "x"`.
  let unwrapped: Vec<Expr> = args.iter().map(|a| unparen(a).clone()).collect();
  match unwrapped.as_slice() {
    [Expr::StrLit { value, .. }] => {
      format!("println!({})", double_braces(&rust_escapes(value)))
    }
    [] => "println!()".to_string(),
    _ => {
      let holes = vec!["{}"; args.len()].join(" ");
      // The arguments are emitted at `indent`, the column the `println!`
      // itself starts at, which is what makes a multi-line argument come out
      // the way rustfmt writes it: `println!("{}", {` with the block's body
      // one level in and its closing brace back at `indent`. Spreading the
      // arguments one per line instead was tried first and is *wrong* —
      // rustfmt reformats that back to the hanging form, so the emitter would
      // have been producing output rustfmt rewrites, which is exactly what
      // plan 04's guarantee forbids.
      format!("println!(\"{holes}\", {})", expr_list(args, indent))
    }
  }
}
