pub(crate) mod application;
pub(crate) mod arity_scope;
pub(crate) mod arity_table;
pub(crate) mod arm;
pub(crate) mod ascribed_list;
pub(crate) mod assert_call;
pub(crate) mod case;
pub(crate) mod catch_fallback;
pub(crate) mod chain;
pub(crate) mod chain_width;
pub(crate) mod class;
pub(crate) mod closure;
pub(crate) mod closure_param;
pub(crate) mod coalesce;
pub(crate) mod comprehension;
pub(crate) mod condition_head;
pub(crate) mod control_text;
pub(crate) mod data;
pub(crate) mod double_braces;
pub(crate) mod emit_await;
pub(crate) mod emit_block;
pub(crate) mod emit_body;
pub(crate) mod emit_clauses;
pub(crate) mod emit_condition;
pub(crate) mod emit_const;
pub(crate) mod emit_control;
pub(crate) mod emit_control_inline;
pub(crate) mod emit_fn;
pub(crate) mod emit_stmts;
pub(crate) mod error_message;
pub(crate) mod errors;
pub(crate) mod expr;
pub(crate) mod expr_list;
pub(crate) mod extern_block;
pub(crate) mod field_value;
pub(crate) mod fields;
pub(crate) mod fn_call_width;
pub(crate) mod format_template;
pub(crate) mod generics;
pub(crate) mod guards;
pub(crate) mod import;
pub(crate) mod inline_comment;
pub(crate) mod inline_rust;
pub(crate) mod instance;
pub(crate) mod item;
pub(crate) mod items;
pub(crate) mod label_prefix;
pub(crate) mod member;
pub(crate) mod missing_arity;
pub(crate) mod module;
pub(crate) mod names_a_constructor;
pub(crate) mod nest;
pub(crate) mod one_field;
pub(crate) mod one_line_if;
pub(crate) mod one_tail_expression;
pub(crate) mod optional_chain;
pub(crate) mod pad;
pub(crate) mod pad_to;
pub(crate) mod panic_macro;
pub(crate) mod pipeline;
pub(crate) mod print_call;
pub(crate) mod receiver;
pub(crate) mod record;
pub(crate) mod rust_escapes;
pub(crate) mod rust_indent;
pub(crate) mod section;
pub(crate) mod signature;
pub(crate) mod single_line_if_else_max_width;
pub(crate) mod sink;
pub(crate) mod sole_expression;
pub(crate) mod stmt;
pub(crate) mod struct_lit_width;
pub(crate) mod struct_variant_width;
pub(crate) mod test_slug;
pub(crate) mod tests_module;
pub(crate) mod trivia;
pub(crate) mod try_block;
pub(crate) mod two_armed;
pub(crate) mod type_alias;
pub(crate) mod unlowerable;
pub(crate) mod unparen;
pub(crate) mod unsafe_block;
pub(crate) mod with_map;
pub(crate) mod wraps_as_receiver;

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
