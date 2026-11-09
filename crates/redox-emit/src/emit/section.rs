use crate::emit::expr::emit_expr;
use crate::emit::section_param::SECTION_PARAM;
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
/// (|> name)   |__rdx_1| __rdx_1.name()
/// ```
///
/// **A comparison uses the method form**, which is doc 18 §4's own table and
/// §1.1's reason: an iterator adaptor hands its closure a *reference*, so
/// `&i32 > 0` does not compile and `(&i32).gt(&0)` does. The borrow on the
/// operand is part of that — `gt` takes `&Self`.
///
/// A left comparison section is **flipped into a right one** rather than
/// lowered separately: `(0 <)` and `(> 0)` are the same predicate written
/// from either side, which is why §4 gives them one lowering.
pub(crate) fn emit_section(section: &Section, indent: usize) -> String {
  match section {
    Section::Field(name) => format!("|{SECTION_PARAM}| {SECTION_PARAM}.{}", name.text),
    Section::Method(name) => format!("|{SECTION_PARAM}| {SECTION_PARAM}.{}()", name.text),

    Section::Right { op, operand } => {
      let value = emit_expr(unparen(operand), indent);
      if let Some(method) = op.compare_method() {
        return format!("|{SECTION_PARAM}| {SECTION_PARAM}.{method}(&{value})");
      }
      let spelling = op.rust_text().unwrap_or("/* unsupported */");
      format!("|{SECTION_PARAM}| {SECTION_PARAM} {spelling} {value}")
    }

    Section::Left { op, operand } => {
      let value = emit_expr(unparen(operand), indent);
      if let Some(method) = op.flipped().and_then(redox_ast::BinaryOp::compare_method) {
        return format!("|{SECTION_PARAM}| {SECTION_PARAM}.{method}(&{value})");
      }
      let spelling = op.rust_text().unwrap_or("/* unsupported */");
      format!("|{SECTION_PARAM}| {value} {spelling} {SECTION_PARAM}")
    }
  }
}
