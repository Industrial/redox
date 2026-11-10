use crate::emit::closure::closure;
use crate::emit::closure_param::closure_param;
use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::Section;

/// Lower doc 18 §4's section to a one-parameter closure.
///
/// ```text
/// (+ 1)       |__rdx_1| __rdx_1 + 1
/// (1 +)       |__rdx_1| 1 + __rdx_1
/// (> 0)       |__rdx_1| __rdx_1.gt(&0)
/// (0 <)       |__rdx_1| __rdx_1.gt(&0)
/// (.name)     |__rdx_1| __rdx_1.name
/// ```
///
/// `(|> name)` was a fifth row, emitting `|__rdx_1| __rdx_1.name()`. Plan 57
/// retires it: a bare `.name` is a method awaiting its receiver and lowers to
/// that same text, so `emit/method_ref.rs` holds the lowering and this holds
/// one row fewer.
///
/// **A comparison uses the method form**, which is doc 18 §4's own table and
/// §1.1's reason: an iterator adaptor hands its closure a *reference*, so
/// `&i32 > 0` does not compile and `(&i32).gt(&0)` does. The borrow on the
/// operand is part of that — `gt` takes `&Self`.
///
/// A left comparison section is **flipped into a right one** rather than
/// lowered separately: `(0 <)` and `(> 0)` are the same predicate written
/// from either side, which is why §4 gives them one lowering.
///
/// This builds a **body**, and [`closure`] writes the closure around it — the
/// same `closure` plan 56's partial application uses, which is that row's
/// criterion 6: `(+ 1)` and a one-of-two `add` differ in the table consulted
/// and nowhere else. The text is byte-identical to what this emitted before
/// the two were unified, and `tests/golden/section.xrs` is what says so.
pub(crate) fn emit_section(section: &Section, indent: usize) -> String {
  let param = closure_param(0);
  let body = match section {
    Section::Field(name) => format!("{param}.{}", name.text),

    Section::Right { op, operand } => {
      let value = emit_expr(unparen(operand), indent);
      match op.compare_method() {
        Some(method) => format!("{param}.{method}(&{value})"),
        None => {
          let spelling = op.rust_text().unwrap_or("/* unsupported */");
          format!("{param} {spelling} {value}")
        }
      }
    }

    Section::Left { op, operand } => {
      let value = emit_expr(unparen(operand), indent);
      match op.flipped().and_then(redox_ast::BinaryOp::compare_method) {
        Some(method) => format!("{param}.{method}(&{value})"),
        None => {
          let spelling = op.rust_text().unwrap_or("/* unsupported */");
          format!("{value} {spelling} {param}")
        }
      }
    }
  };
  closure(&[param], &body)
}
