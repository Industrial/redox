use crate::emit::double_braces::double_braces;
use crate::emit::expr::emit_expr;
use crate::emit::expr_list::expr_list;
use crate::emit::rust_escapes::rust_escapes;
use crate::emit::rust_indent::RUST_INDENT;
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
    // `print` and `print ()` are the same call, for the reason `expr_list`
    // gives: a lone unit argument is the empty argument list.
    [] | [Expr::Unit { .. }] => "println!()".to_string(),
    _ => {
      let holes = vec!["{}"; args.len()].join(" ");
      let one_line = expr_list(args, indent);

      // **rustfmt overflows a block-like last argument and spreads anything
      // else.** `println!("{}", { … })` keeps the braces hanging off the
      // call; `println!("{}", match … )` is rewritten one argument per line.
      // Both were measured against rustfmt's defaults outside the
      // repository, and the first version of this got it backwards in each
      // direction before the ascription and `??` examples found it.
      //
      // So: a multi-line argument that *opens with a brace* is emitted at the
      // call's own column and left to hang; anything else multi-line is
      // spread. Neither form is a preference — it is which one rustfmt leaves
      // alone, which is plan 04's guarantee.
      if one_line.contains('\n') && !one_line.starts_with('{') {
        let inner_pad = " ".repeat(indent + RUST_INDENT);
        let close_pad = " ".repeat(indent);
        let spread = args
          .iter()
          .map(|e| emit_expr(unparen(e), indent + RUST_INDENT))
          .collect::<Vec<_>>()
          .join(&format!(",\n{inner_pad}"));
        return format!("println!(\n{inner_pad}\"{holes}\",\n{inner_pad}{spread}\n{close_pad})");
      }

      format!("println!(\"{holes}\", {one_line})")
    }
  }
}
