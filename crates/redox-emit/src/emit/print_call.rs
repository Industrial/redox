use crate::emit::expr_list::expr_list;
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
pub(crate) fn print_call(args: &[Expr]) -> String {
  match args {
    [Expr::StrLit { value, .. }] => format!("println!({value})"),
    [] => "println!()".to_string(),
    _ => {
      let holes = vec!["{}"; args.len()].join(" ");
      format!("println!(\"{holes}\", {})", expr_list(args))
    }
  }
}
