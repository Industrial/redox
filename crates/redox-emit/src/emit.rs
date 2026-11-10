/// A binding's type annotation and the value to emit under it.
pub(crate) mod annotated_binding;
/// Lower an application: a saturated call, or the closure an under-applied one becomes.
pub(crate) mod application;
/// The arity table in scope, empty outside an [`arity_scope::ArityScope`].
pub(crate) mod arity_scope;
/// The declared arity of every function a file names, by name.
pub(crate) mod arity_table;
/// Emit one `match` arm for a clause.
pub(crate) mod arm;
/// The `let` that gives an arm's ascription somewhere to live, or none.
pub(crate) mod ascribed_arm;
/// Lower `[a, b] : C<_>` to `C::from([a, b])`.
pub(crate) mod ascribed_list;
/// Lower an `assert` statement.
pub(crate) mod assert_call;
/// What `<-` means in the block being emitted.
pub(crate) mod bind_mode;
/// What `<-` means right here, awaiting outside any modifier.
pub(crate) mod bind_scope;
/// Which Rust item a parameterless binding becomes.
pub(crate) mod binding_kind;
/// The binding table in scope, empty outside a [`binding_scope::BindingScope`].
pub(crate) mod binding_scope;
/// Every parameterless binding a file declares, and which Rust item it becomes.
pub(crate) mod binding_table;
/// How a bare name or a path reads at a use site.
pub(crate) mod binding_text;
/// Lower `case … of` to Rust's `match`.
pub(crate) mod case;
/// Lower `catch`.
pub(crate) mod catch_fallback;
/// Assemble a method chain the way rustfmt writes it.
pub(crate) mod chain;
/// rustfmt's `chain_width` default.
pub(crate) mod chain_width;
/// Emit a `class` as a Rust `trait`.
pub(crate) mod class;
/// Write a closure: `|params| body`.
pub(crate) mod closure;
/// The name a generated closure binds for its nth parameter.
pub(crate) mod closure_param;
/// Lower `??`.
pub(crate) mod coalesce;
/// Lower composition — `f >>> g` and `f <<< g` — as a closure.
pub(crate) mod compose;
/// The functions a composition chain applies, in application order.
pub(crate) mod compose_chain;
/// Lower a comprehension to an iterator chain.
pub(crate) mod comprehension;
/// A control-flow statement's keyword, condition and opening brace.
pub(crate) mod condition_head;
/// A control-flow form's Rust as text, for an expression position.
pub(crate) mod control_text;
/// Emit a `data` declaration as a `struct` or an `enum`.
pub(crate) mod data;
/// Escape a string literal's braces for use as a `format!` string.
pub(crate) mod double_braces;
/// Lower `<-`, monadic bind, as the enclosing block modifier reads it.
pub(crate) mod emit_bind;
/// Emit an indented block as a Rust block expression.
pub(crate) mod emit_block;
/// Emit a clause's right-hand side.
pub(crate) mod emit_body;
/// Emit a function's body from its clauses.
pub(crate) mod emit_clauses;
/// The Rust text of an `if`, `while` or arm-guard condition.
pub(crate) mod emit_condition;
/// Emit a module-scope `const` or `static`.
pub(crate) mod emit_const;
/// Emit a control-flow statement.
pub(crate) mod emit_control;
/// An `else if`'s nested `if`, written without its leading pad.
pub(crate) mod emit_control_inline;
/// Emit a function declaration.
pub(crate) mod emit_fn;
/// Emit a run of statements, flattening `with` callbacks.
pub(crate) mod emit_stmts;
/// A `where` group's entries, emitted in dependency order.
pub(crate) mod emit_where_locals;
/// Split an error variant's message into a format string and its arguments.
pub(crate) mod error_message;
/// Emit an `errors` declaration: the enum, `Display`, `Error`, and its `From` rows.
pub(crate) mod errors;
/// Render an expression as Rust.
pub(crate) mod expr;
/// A comma-separated argument list.
pub(crate) mod expr_list;
/// Emit an `extern` block.
pub(crate) mod extern_block;
/// A record field's value text, with punning expanded.
pub(crate) mod field_value;
/// Emit a constructor's fields.
pub(crate) mod fields;
/// rustfmt's `fn_call_width` default.
pub(crate) mod fn_call_width;
/// An interpolated string's Rust format template and argument list.
pub(crate) mod format_template;
/// The generic parameter list an `instance` needs, or an empty string.
pub(crate) mod generics;
/// Emit one guard ladder as an `if`/`else if`/`else` chain.
pub(crate) mod guards;
/// Emit an `import` as a Rust `use`.
pub(crate) mod import;
/// Append a trailing `// …` comment to the current line, if there is one.
pub(crate) mod inline_comment;
/// Emit inline Rust.
pub(crate) mod inline_rust;
/// Emit an `instance` as a Rust `impl`.
pub(crate) mod instance;
/// Whether a body is a literal, and so may be shared as a `const`.
pub(crate) mod is_shareable;
/// Emit one declaration.
pub(crate) mod item;
/// Emit a list of items, one blank line between them.
pub(crate) mod items;
/// A loop's or block's label, as the `'name: ` Rust writes before the keyword.
pub(crate) mod label_prefix;
/// Lower a macro invocation.
pub(crate) mod macro_call;
/// Emit one member of a `trait` or `impl`.
pub(crate) mod member;
/// Lower method-as-function: `.name` and its application.
pub(crate) mod method_ref;
/// How many parameters an application has still to be given.
pub(crate) mod missing_arity;
/// Emit a `module` as Rust's `mod`.
pub(crate) mod module;
/// Whether a record literal's head names a constructor rather than a value.
pub(crate) mod names_a_constructor;
/// An index chain, as a receiver and its segments.
pub(crate) mod nest;
/// One field of a record literal: `name: value`, or `name` when punned.
pub(crate) mod one_field;
/// An `if … else …` on one line, when rustfmt would put it there.
pub(crate) mod one_line_if;
/// A clause's body as one tail expression statement, when that is what it is.
pub(crate) mod one_tail_expression;
/// Append a number of spaces to the sink.
pub(crate) mod pad;
/// Append a number of spaces to a `String`.
pub(crate) mod pad_to;
/// Lower `|>` as reverse application: `x |> f` is `f x`.
pub(crate) mod pipeline;
/// Render an expression as a method call's receiver, parenthesised when `.` would bind tighter.
pub(crate) mod receiver;
/// Lower a record literal.
pub(crate) mod record;
/// Re-encode a Redox string literal's text as a Rust string literal's text.
pub(crate) mod rust_escapes;
/// A `rust` escape hatch in expression position, at its indentation.
pub(crate) mod rust_hatch_text;
/// Indentation of emitted Rust, in spaces.
pub(crate) mod rust_indent;
/// Lower a section to a one-parameter closure.
pub(crate) mod section;
/// The `const` item a shared binding is emitted as.
pub(crate) mod shared_const;
/// Render a signature, with each part omitted when absent.
pub(crate) mod signature;
/// rustfmt's `single_line_if_else_max_width` default.
pub(crate) mod single_line_if_else_max_width;
/// The emitter's output, recording where each generated line came from.
pub(crate) mod sink;
/// A guard row's body as one expression's text, if that is what it is.
pub(crate) mod sole_expression;
/// Emit one block element.
pub(crate) mod stmt;
/// The widest a record literal's fields may be and still go on one line.
pub(crate) mod struct_lit_width;
/// The widest an enum's record variant may be and still go on one line.
pub(crate) mod struct_variant_width;
/// A deterministic function name for a test's description.
pub(crate) mod test_slug;
/// Emit every `test` block as one attributed module.
pub(crate) mod tests_module;
/// Emit leading trivia.
pub(crate) mod trivia;
/// Lower a `try` block.
pub(crate) mod try_block;
/// A two-armed guard ladder as one line, when rustfmt would write it that way.
pub(crate) mod two_armed;
/// Emit a `type` alias, which is already Rust.
pub(crate) mod type_alias;
/// The refusal for a form that has no Rust expression spelling at all.
pub(crate) mod unlowerable;
/// Strip a grouping parenthesis that the output does not need.
pub(crate) mod unparen;
/// Emit Rust's `unsafe { … }`.
pub(crate) mod unsafe_block;
/// A `where` group's entries in dependency order.
pub(crate) mod where_order;
/// Emit a program, and record where each generated line came from.
pub(crate) mod with_map;
/// Whether an expression needs parentheses as a receiver or callee.
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
