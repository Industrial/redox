use crate::emit::binding_kind::BindingKind;
use crate::emit::binding_scope::BindingScope;
use crate::emit::class::emit_class;
use crate::emit::data::emit_data;
use crate::emit::emit_const::emit_const;
use crate::emit::emit_fn::emit_fn;
use crate::emit::errors::emit_errors;
use crate::emit::extern_block::emit_extern_block;
use crate::emit::import::emit_import;
use crate::emit::inline_rust::emit_inline_rust;
use crate::emit::instance::emit_instance;
use crate::emit::module::emit_module;
use crate::emit::shared_const::shared_const;
use crate::emit::sink::Sink;
use crate::emit::type_alias::emit_type_alias;
use redox_ast::Item;

/// Emit one declaration at `indent`.
///
/// Extracted from `emit_items` by plan 55 leaf 5, which gave declarations a
/// second home: a block element. Both callers read this one dispatch, so a
/// local `data` and a module-scope one cannot drift — which matters more than
/// it sounds, because the two are the same parse and a reader who sees them
/// differ will reasonably assume the difference is deliberate.
///
/// `Item::Test` is unreachable here. Tests are **gathered** rather than
/// emitted in place — `emit_items` collects them into one `#[cfg(test)]`
/// module — and a local one never reaches the emitter at all, because
/// `RDX0112` refuses it in the parser (leaf 6).
///
/// The match is exhaustive with no wildcard, for the reason `emit_expr` gives:
/// a new `Item` must fail to compile here rather than be silently dropped.
pub(crate) fn emit_item(out: &mut Sink<'_>, item: &Item, indent: usize) {
  match item {
    // Plan 59 leaf 2. A parameterless binding with a literal body and an
    // upper-case name is a value, so it is written as the `const` doc 03
    // §13.4 makes the default — through plan 16's own emission, not a second
    // path to the same Rust. Everything else is still a function, and leaf 3
    // restores the call at the use site.
    Item::Fn(f) if BindingScope::kind_of(&f.name.text) == Some(BindingKind::Shared) => {
      match shared_const(f) {
        Some(c) => emit_const(out, &c, indent),
        None => emit_fn(out, f, indent),
      }
    }
    Item::Fn(f) => emit_fn(out, f, indent),
    Item::Data(d) => emit_data(out, d, indent),
    Item::Type(a) => emit_type_alias(out, a, indent),
    Item::Class(c) => emit_class(out, c, indent),
    Item::Instance(i) => emit_instance(out, i, indent),
    Item::Errors(e) => emit_errors(out, e, indent),
    Item::Import(i) => emit_import(out, i, indent),
    Item::Module(m) => emit_module(out, m, indent),
    Item::Const(c) => emit_const(out, c, indent),
    Item::InlineRust(r) => emit_inline_rust(out, r, indent),
    Item::Extern(e) => emit_extern_block(out, e, indent),
    Item::Test(_) => unreachable!("tests are gathered, not emitted in place"),
  }
}
