use crate::emit::ascribed_list::ascribed_list;
use crate::emit::case::emit_case;
use crate::emit::catch_fallback::catch_fallback;
use crate::emit::coalesce::coalesce;
use crate::emit::comprehension::emit_comprehension;
use crate::emit::emit_condition::emit_condition;
use crate::emit::emit_do::emit_do;
use crate::emit::expr_list::expr_list;
use crate::emit::format_template::format_template;
use crate::emit::optional_chain::optional_chain;
use crate::emit::panic_macro::panic_macro;
use crate::emit::pipeline::pipeline;
use crate::emit::print_call::print_call;
use crate::emit::receiver::receiver;
use crate::emit::record::emit_record;
use crate::emit::rust_escapes::rust_escapes;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::section::emit_section;
use crate::emit::try_block::try_block;
use crate::emit::unparen::unparen;
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
      // Doc 18 §6's third lowering: an ascribed list literal names its
      // container, so its value is `C::from([…])`. Inside the block, not
      // instead of it — see `ascribed_list`.
      let value = ascribed_list(inner, ty, indent + RUST_INDENT)
        .unwrap_or_else(|| emit_expr(inner, indent + RUST_INDENT));
      format!(
        "{{\n{inner_pad}let __rdx_t: {} = {value};\n{inner_pad}__rdx_t\n{close_pad}}}",
        ty.text
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
    // Doc 01 §8's conditional expression — plan 20. One line, because Rust's
    // own `if` expression is one when its arms are: see
    // `SINGLE_LINE_IF_ELSE_MAX_WIDTH` for when rustfmt disagrees.
    Expr::If {
      condition,
      then,
      otherwise,
      ..
    } => format!(
      "if {} {{ {} }} else {{ {} }}",
      emit_condition(condition, indent),
      emit_expr(unparen(then), indent),
      emit_expr(unparen(otherwise), indent)
    ),
    // Doc 03 §7.2's `do` block — plan 20. A block expression, labelled when
    // the author labelled it.
    Expr::Do { label, body, .. } => emit_do(label, body, indent),
    Expr::Unit { .. } => "()".to_string(),
    // Rust's try operator, unchanged. The receiver needs parentheses for the
    // same reason a method call's does: `?` binds tighter than any operator.
    Expr::Try { inner, .. } => format!("{}?", receiver(inner, indent)),
    // Doc 03 §14.3's postfix await, verbatim passthrough. The receiver is
    // parenthesised on the same rule as a method call's, which is why doc 18
    // §2.1 names `(get u).await` among its surviving required-parens cases.
    Expr::Await { inner, .. } => format!("{}.await", receiver(inner, indent)),
    // Rust's array literal, unchanged — a spelling, not a lowering.
    Expr::List { items, .. } => format!("[{}]", expr_list(items, indent)),
    // Doc 18 §3.2. An iterator chain, built inside out.
    Expr::Comprehension {
      element,
      qualifiers,
      ..
    } => emit_comprehension(element, qualifiers, indent),
    Expr::StrLit { value, .. } => rust_escapes(value),
    // Doc 03 §11.1's interpolation — plan 23. `format!` here; `print_call`
    // takes the same template to `println!` instead.
    Expr::StrInterp { parts, .. } => {
      let (template, args) = format_template(parts, indent);
      if args.is_empty() {
        format!("format!(\"{template}\")")
      } else {
        format!("format!(\"{template}\", {})", args.join(", "))
      }
    }
    // Doc 01 §13.1's optional chain and doc 03 §12.4's `try` block — plan 28.
    Expr::OptionalChain { head, segments, .. } => optional_chain(head, segments, indent),
    Expr::TryBlock { body, .. } => try_block(body, indent),
    // Doc 03 §10.4's array repeat — plan 29. **Verbatim, never `vec!`**: a
    // repeat expression is a fixed-size array and `vec!` is a heap
    // allocation. Both compile, so only this line tells them apart.
    Expr::ArrayRepeat { value, count, .. } => format!(
      "[{}; {}]",
      emit_expr(unparen(value), indent),
      emit_expr(unparen(count), indent)
    ),
    // Doc 03 §11.2's four literal kinds — verbatim, never a format string.
    Expr::RawLit { text, .. } => text.clone(),
    Expr::Num { text, .. } => text.clone(),
    Expr::Bool { value, .. } => value.to_string(),
    Expr::Char { value, .. } => format!("{value:?}"),
    // Doc 03 §12.6's bare panic-family spellings. A lookup rather than a
    // keyword, because nothing else distinguishes `todo` from a function the
    // author wrote and has not defined yet — which is, after all, what
    // `todo` is for.
    Expr::Ident(i) => panic_macro(&i.text).map_or_else(|| i.text.clone(), str::to_string),
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
    // Doc 03 §12.1, which doc 18 leaves unchanged: `??` is `Option`-only and
    // lowers to a `match`. A *lowering*, not a spelling, which is why it
    // sits beside the pipeline rather than in the operator table.
    Expr::Binary {
      op: BinaryOp::Coalesce,
      lhs,
      rhs,
      ..
    } => coalesce(lhs, rhs, indent),
    // Doc 03 §12.2's `catch` — `??`'s mirror for `Result`, and the same
    // `match` lowering for §12.1's reasons. Plan 28.
    Expr::Binary {
      op: BinaryOp::Catch,
      lhs,
      rhs,
      ..
    } => catch_fallback(lhs, rhs, indent),
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
      // Both operators that need a lowering rather than a spelling — `??`
      // and `|>` — are matched above, so this arm is unreachable through any
      // source. Kept as a hard error rather than removed: a *new* operator
      // with no lowering must fail loudly in the generated file, not quietly
      // produce something that looks plausible.
      None => format!("compile_error!(\"`{op:?}` is parsed but not yet lowered\")"),
    },
    // `receiver` rather than `emit_expr` for all three: it strips a grouping
    // paren the author wrote and adds one where `.` or `[` would otherwise
    // bind tighter, which is the same question asked in both directions.
    Expr::Field { recv, name, .. } => format!("{}.{}", receiver(recv, indent), name.text),
    Expr::Index { recv, index, .. } => {
      format!("{}[{}]", receiver(recv, indent), emit_expr(index, indent))
    }
    Expr::MethodCall {
      recv, name, args, ..
    } => format!(
      "{}.{}({})",
      receiver(recv, indent),
      name.text,
      expr_list(args, indent)
    ),
    Expr::Call { callee, args, .. } => {
      if let Expr::Ident(name) = callee.as_ref()
        && name.text == "print"
      {
        return print_call(args, indent);
      }
      // `panic "message"` — doc 03 §12.6. A call rather than a bare name,
      // because it takes an argument, which is why it is lowered here and
      // the other three are lowered as identifiers.
      if let Expr::Ident(name) = callee.as_ref()
        && name.text == "panic"
      {
        return format!("panic!({})", expr_list(args, indent));
      }
      format!("{}({})", emit_expr(callee, indent), expr_list(args, indent))
    }
  }
}
