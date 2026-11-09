use crate::emit::expr_list::expr_list;
use crate::emit::print_call::print_call;
use crate::emit::rust_escapes::rust_escapes;
use redox_ast::Expr;

/// Render an expression as Rust.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) leaf 6 grew this to every node
/// the new grammar produces. The match is deliberately **exhaustive with no
/// wildcard**: an AST node with no lowering is a `todo!()` with a delay fuse,
/// so a new node must fail to compile here rather than fail at runtime.
pub(crate) fn emit_expr(e: &Expr) -> String {
  match e {
    // Doc 18 §4. A Rust closure, with `move` and types carried through.
    Expr::Lambda {
      params,
      body,
      is_move,
      ..
    } => {
      let binders: Vec<String> = params
        .iter()
        .map(|param| match &param.ty {
          Some(ty) => format!("{}: {}", param.name.text, ty.text),
          None => param.name.text.clone(),
        })
        .collect();
      format!(
        "{}|{}| {}",
        if *is_move { "move " } else { "" },
        binders.join(", "),
        emit_expr(body)
      )
    }
    // Doc 18 §6. A block, because Rust has no expression-level ascription:
    // `{ let __rdx_t: T = e; __rdx_t }` states the type and yields the value.
    // The `__rdx_` prefix is reserved for generated identifiers (doc 02), so
    // this cannot collide with anything the author wrote.
    Expr::Ascribe { inner, ty, .. } => format!(
      "{{ let __rdx_t: {} = {}; __rdx_t }}",
      ty.text,
      emit_expr(inner)
    ),
    Expr::StrLit { value, .. } => rust_escapes(value),
    Expr::Num { text, .. } => text.clone(),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::Char { value, .. } => format!("{value:?}"),
    Expr::Ident(i) => i.text.clone(),
    Expr::Path { text, .. } => text.clone(),
    // Doc 01 §16: `@field` is `self.field` and `@@name` is `Self::name`.
    Expr::SelfField { name, .. } => format!("self.{}", name.text),
    Expr::SelfType { name, .. } => format!("Self::{}", name.text),
    Expr::Paren { inner, .. } => format!("({})", emit_expr(inner)),
    Expr::Unary { op, operand, .. } => format!("{}{}", op.rust_text(), emit_expr(operand)),
    Expr::Binary { op, lhs, rhs, .. } => match op.rust_text() {
      Some(text) => format!("{} {text} {}", emit_expr(lhs), emit_expr(rhs)),
      // `??` and `|>` need a lowering rather than a spelling — plans 28 and
      // 27. Until then the parser accepts them and this emits a hard error
      // rather than something that merely looks plausible.
      None => format!("compile_error!(\"`{op:?}` is parsed but not yet lowered\")"),
    },
    Expr::Field { recv, name, .. } => format!("{}.{}", emit_expr(recv), name.text),
    Expr::Index { recv, index, .. } => format!("{}[{}]", emit_expr(recv), emit_expr(index)),
    Expr::MethodCall {
      recv, name, args, ..
    } => format!("{}.{}({})", emit_expr(recv), name.text, expr_list(args)),
    Expr::Call { callee, args, .. } => {
      if let Expr::Ident(name) = callee.as_ref()
        && name.text == "print"
      {
        return print_call(args);
      }
      format!("{}({})", emit_expr(callee), expr_list(args))
    }
  }
}
