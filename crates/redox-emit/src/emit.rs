pub(crate) mod arm;
pub(crate) mod case;
pub(crate) mod data;
pub(crate) mod double_braces;
pub(crate) mod emit_body;
pub(crate) mod emit_clauses;
pub(crate) mod emit_fn;
pub(crate) mod emit_stmts;
pub(crate) mod expr;
pub(crate) mod expr_list;
pub(crate) mod fields;
pub(crate) mod guards;
pub(crate) mod inline_comment;
pub(crate) mod one_tail_expression;
pub(crate) mod pad;
pub(crate) mod pipeline;
pub(crate) mod print_call;
pub(crate) mod rust_escapes;
pub(crate) mod rust_indent;
pub(crate) mod signature;
pub(crate) mod sink;
pub(crate) mod stmt;
pub(crate) mod struct_variant_width;
pub(crate) mod trivia;
pub(crate) mod type_alias;
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
