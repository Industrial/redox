use redox_ast::Expr;

/// Render an expression as Rust.
pub(crate) fn emit_expr(e: &Expr) -> String {
  match e {
    Expr::StrLit { value, .. } => value.clone(),
    Expr::Ident(i) => i.text.clone(),
    Expr::Call { callee, args, .. } => {
      let rendered: Vec<String> = args.iter().map(emit_expr).collect();
      let joined = rendered.join(", ");
      // `print` is the one sanctioned sugar of its kind (doc 01 §3.3,
      // doc 03 §10.9) and it is a **dated loan**: it is exactly Emerald's
      // hardcoded `puts` keyword, which papered over a general parenless-call
      // ambiguity for 89 plans. Plan 19 resolves the general case and this
      // special case then dissolves.
      if let Expr::Ident(name) = callee.as_ref()
        && name.text == "print"
      {
        return format!("println!({joined})");
      }
      format!("{}({joined})", emit_expr(callee))
    }
  }
}
