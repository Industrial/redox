use crate::emit::double_braces::double_braces;
use crate::emit::expr::emit_expr;
use crate::emit::expr_list::expr_list;
use crate::emit::fn_call_width::FN_CALL_WIDTH;
use crate::emit::format_template::format_template;
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
    // An interpolated string is `println!` with **its own** template — doc 03
    // §11.1, plan 23. The alternative, `println!("{}", format!(…))`, is two
    // macros and one allocation to say what one says, and is not what a
    // person would write.
    //
    // This arm is also where plan 23's §1.2 defect lived: before the node
    // existed, an interpolated string arrived as a `StrLit` whose raw text
    // still held `#{name}`, took the arm above, and `double_braces` turned it
    // into `println!("hi #{{name}}")` — valid Rust printing `hi #{name}`,
    // with no diagnostic anywhere.
    [Expr::StrInterp { parts, .. }] => {
      let (template, args) = format_template(parts, indent);
      if args.is_empty() {
        format!("println!(\"{template}\")")
      } else {
        format!("println!(\"{template}\", {})", args.join(", "))
      }
    }
    // `print` and `print ()` are the same call, for the reason `expr_list`
    // gives: a lone unit argument is the empty argument list.
    [] | [Expr::Unit { .. }] => "println!()".to_string(),
    _ => {
      let holes = vec!["{}"; args.len()].join(" ");
      let one_line = expr_list(args, indent);

      // **rustfmt overflows a block-like last argument and spreads anything
      // else.** `println!("{}", { … })` keeps the braces hanging off the
      // call whatever its width; `println!("{}", match … )` is rewritten one
      // argument per line, and so is any argument list wider than
      // `fn_call_width`.
      //
      // Every clause of that was measured against rustfmt's defaults outside
      // the repository, and each arrived from an example that caught the
      // previous version being wrong: the ascription example found the
      // overflow, the `??` example found the spread, and the `errors` example
      // found the width — which is three rounds of getting it wrong in a
      // different direction each time, and the reason it is spelled out here
      // rather than left to read off the code.
      let last = unwrapped
        .last()
        .map(|e| emit_expr(e, indent))
        .unwrap_or_default();
      let block_like = last.starts_with('{');
      let arguments = format!("\"{holes}\", {one_line}");
      if !block_like && (one_line.contains('\n') || arguments.len() > FN_CALL_WIDTH) {
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
