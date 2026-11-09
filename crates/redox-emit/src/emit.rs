pub(crate) mod double_braces;
pub(crate) mod emit_fn;
pub(crate) mod emit_stmts;
pub(crate) mod expr;
pub(crate) mod expr_list;
pub(crate) mod inline_comment;
pub(crate) mod pad;
pub(crate) mod print_call;
pub(crate) mod rust_escapes;
pub(crate) mod rust_indent;
pub(crate) mod signature;
pub(crate) mod sink;
pub(crate) mod stmt;
pub(crate) mod trivia;
pub(crate) mod unparen;
pub(crate) mod with_map;

use crate::emit::with_map::emit_with_map;
use redox_ast::Program;

/// Emit `program` as Rust source text.
///
/// Kept at its plan 01 signature. Plan 04 made this function's output a
/// *measured* property — `rustfmt` leaves it unchanged, and the goldens
/// compare exact bytes — so plan 06 added [`emit_with_map`] beside it rather
/// than threading a map through it and putting that property at risk.
pub fn emit(program: &Program) -> String {
  emit_with_map(program, "", "").0
}
