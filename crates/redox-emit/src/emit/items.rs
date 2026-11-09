use crate::emit::class::emit_class;
use crate::emit::data::emit_data;
use crate::emit::emit_fn::emit_fn;
use crate::emit::errors::emit_errors;
use crate::emit::import::emit_import;
use crate::emit::instance::emit_instance;
use crate::emit::module::emit_module;
use crate::emit::sink::Sink;
use crate::emit::tests_module::emit_tests_module;
use crate::emit::type_alias::emit_type_alias;
use redox_ast::{Item, TestDecl};

/// Emit a list of items at `indent` spaces, one blank line between them.
///
/// Lifted out of `emit_with_map` when plan 14's `module` body arrived,
/// because a module body is a list of items and wants everything this does:
/// the blank-line rule, source order, and **its own** gathered test module.
///
/// One blank line between items, driven by position in the tree rather than
/// by how many blank lines the source happened to have — so the output is a
/// function of structure and therefore byte-stable.
///
/// **Items keep source order, imports included**, and the blank line between
/// them is what makes that possible. Plan 14 decision 3 is why the order is
/// kept — gathering imports to the top would reorder the author's file for no
/// gain, which is the opposite of doc 01 §3.4 — and rustfmt is why the blank
/// line stays.
///
/// rustfmt's `reorder_imports` defaults to **on** and sorts `use` statements
/// **within a contiguous block**. Emitting two adjacent imports without a
/// blank line between them, which is Rust's own convention and was tried
/// first, therefore produced output rustfmt reorders — so keeping the author's
/// order would have meant the emitter replicating rustfmt's sort, which is
/// not plain lexicographic and would be a fourth width-rule-shaped liability.
/// A blank line between every item costs one line each and makes the question
/// not arise.
///
/// The one exception is doc 03 §15's `test` block, which §15 *forces*: one
/// module per file, so the tests are gathered and written after the rest.
/// Gathering them per list rather than per file is what puts a test written
/// inside a `module` into **that** module's test module, which is plan 14's
/// acceptance criterion 5.
pub(crate) fn emit_items(out: &mut Sink<'_>, items: &[Item], indent: usize) {
  let tests: Vec<&TestDecl> = items
    .iter()
    .filter_map(|item| match item {
      Item::Test(t) => Some(t),
      _ => None,
    })
    .collect();

  let mut written = 0usize;
  for item in items {
    if matches!(item, Item::Test(_)) {
      continue;
    }
    if written > 0 {
      // The separator belongs to no construct, so it is a synthetic line.
      out.unmark();
      out.push('\n');
    }
    written += 1;

    match item {
      Item::Fn(f) => emit_fn(out, f, indent),
      Item::Data(d) => emit_data(out, d, indent),
      Item::Type(a) => emit_type_alias(out, a, indent),
      Item::Class(c) => emit_class(out, c, indent),
      Item::Instance(i) => emit_instance(out, i, indent),
      Item::Errors(e) => emit_errors(out, e, indent),
      Item::Import(i) => emit_import(out, i, indent),
      Item::Module(m) => emit_module(out, m, indent),
      Item::Test(_) => unreachable!("tests are gathered, not emitted in place"),
    }
  }

  if !tests.is_empty() {
    if written > 0 {
      out.unmark();
      out.push('\n');
    }
    emit_tests_module(out, &tests, indent);
  }
}
