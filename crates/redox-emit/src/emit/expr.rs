use crate::emit::case::emit_case;
use crate::emit::expr_list::expr_list;
use crate::emit::pipeline::pipeline;
use crate::emit::print_call::print_call;
use crate::emit::record::emit_record;
use crate::emit::rust_escapes::rust_escapes;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::section::emit_section;
use redox_ast::{BinaryOp, Expr};

/// Render an expression as Rust, starting at column `indent`.
///
/// Plan 03 (`docs/08 - 03 - Parser Core.md`) leaf 6 grew this to every node
/// the new grammar produces. The match is deliberately **exhaustive with no
/// wildcard**: an AST node with no lowering is a `todo!()` with a delay fuse,
/// so a new node must fail to compile here rather than fail at runtime.
///
/// `indent` is the column the expression's **first** line begins at, so a
/// lowering that spans lines knows where to put the rest. Most arms only pass
/// it down; the ones that use it are the ones that emit a `\n`.
///
/// It is not a formatting nicety. Plan 04's guarantee is that generated Rust
/// is *already* rustfmt-clean, and a multi-line lowering emitted at the wrong
/// column is not — which doc 18 §6's ascription had been quietly breaking
/// since it landed, unnoticed because no example used it. A guarantee checked
/// only through the example corpus holds exactly as far as the corpus reaches.
pub(crate) fn emit_expr(e: &Expr, indent: usize) -> String {
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
        emit_expr(body, indent)
      )
    }
    // Doc 18 §6. A block, because Rust has no expression-level ascription:
    // `{ let __rdx_t: T = e; __rdx_t }` states the type and yields the value.
    // The `__rdx_` prefix is reserved for generated identifiers (doc 02), so
    // this cannot collide with anything the author wrote.
    // The block spans three lines, so it is the first lowering that has to
    // know its own column.
    Expr::Ascribe { inner, ty, .. } => {
      let inner_pad = " ".repeat(indent + RUST_INDENT);
      let close_pad = " ".repeat(indent);
      format!(
        "{{\n{inner_pad}let __rdx_t: {} = {};\n{inner_pad}__rdx_t\n{close_pad}}}",
        ty.text,
        emit_expr(inner, indent + RUST_INDENT)
      )
    }
    // Doc 18 §4. An operator with one side supplied, as a closure.
    Expr::Section { section, .. } => emit_section(section, indent),
    // Doc 18 §2.4. A record literal: construction, punning, or the update
    // lowering, decided by the case of the head.
    Expr::Record { head, fields, .. } => emit_record(head, fields, indent),
    // Doc 18 §2.3. Rust's `match`, with `->` arms read as `=>`.
    Expr::Case {
      scrutinee, arms, ..
    } => emit_case(scrutinee, arms, indent),
    Expr::StrLit { value, .. } => rust_escapes(value),
    Expr::Num { text, .. } => text.clone(),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::Char { value, .. } => format!("{value:?}"),
    Expr::Ident(i) => i.text.clone(),
    Expr::Path { text, .. } => text.clone(),
    // Doc 01 §16: `@field` is `self.field` and `@@name` is `Self::name`.
    Expr::SelfField { name, .. } => format!("self.{}", name.text),
    Expr::SelfType { name, .. } => format!("Self::{}", name.text),
    Expr::Paren { inner, .. } => format!("({})", emit_expr(inner, indent)),
    Expr::Unary { op, operand, .. } => format!("{}{}", op.rust_text(), emit_expr(operand, indent)),
    // Doc 18 §4's pipeline is a *lowering*, not a spelling: `a |> f` becomes
    // `a.f()`, which needs the right operand taken apart.
    Expr::Binary {
      op: BinaryOp::Pipeline,
      lhs,
      rhs,
      ..
    } => pipeline(lhs, rhs, indent),
    Expr::Binary { op, lhs, rhs, .. } => match op.rust_text() {
      // A range operator is written **tight**, which is Rust's own style and
      // therefore rustfmt's: it rewrites `1 .. 4` to `1..4`, and plan 04's
      // guarantee is that the emitter does not produce output rustfmt
      // rewrites. Every other operator takes its spaces.
      Some(text) if op.is_range() => {
        format!("{}{text}{}", emit_expr(lhs, indent), emit_expr(rhs, indent))
      }
      Some(text) => format!(
        "{} {text} {}",
        emit_expr(lhs, indent),
        emit_expr(rhs, indent)
      ),
      // `??` still needs a lowering rather than a spelling — a later plan.
      // Until then the parser accepts it and this emits a hard error rather
      // than something that merely looks plausible.
      None => format!("compile_error!(\"`{op:?}` is parsed but not yet lowered\")"),
    },
    Expr::Field { recv, name, .. } => format!("{}.{}", emit_expr(recv, indent), name.text),
    Expr::Index { recv, index, .. } => {
      format!("{}[{}]", emit_expr(recv, indent), emit_expr(index, indent))
    }
    Expr::MethodCall {
      recv, name, args, ..
    } => format!(
      "{}.{}({})",
      emit_expr(recv, indent),
      name.text,
      expr_list(args, indent)
    ),
    Expr::Call { callee, args, .. } => {
      if let Expr::Ident(name) = callee.as_ref()
        && name.text == "print"
      {
        return print_call(args, indent);
      }
      format!("{}({})", emit_expr(callee, indent), expr_list(args, indent))
    }
  }
}
