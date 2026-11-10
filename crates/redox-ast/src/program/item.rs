/// A `class` declaration — Rust's `trait`.
pub(crate) mod class_decl;

/// One member of a `class` or `instance` body.
pub(crate) mod class_member;

/// One equation of a function.
pub(crate) mod clause;

/// A clause's right-hand side.
pub(crate) mod clause_body;

/// A module-scope `const` or `static`.
pub(crate) mod const_decl;

/// One `Trait t` constraint, from a `=>` clause.
pub(crate) mod constraint;

/// One constructor of a `data` declaration.
pub(crate) mod constructor;

/// What a constructor carries.
pub(crate) mod constructor_fields;

/// A `data` or `newtype` declaration.
pub(crate) mod data_decl;

/// One row of an `errors` declaration.
pub(crate) mod error_variant;

/// An `errors` declaration.
pub(crate) mod errors_decl;

/// An `extern` block.
pub(crate) mod extern_block;

/// One `name : Type` field of a record constructor.
pub(crate) mod field_decl;

/// A signature line and the equations that define it.
pub(crate) mod fn_decl;

/// One declaration inside an `extern` block.
pub(crate) mod foreign_fn;

/// One `| cond = body` row of a guard ladder.
pub(crate) mod r#guard;

/// An `import` declaration.
pub(crate) mod import_decl;

/// The inline Rust escape hatch.
pub(crate) mod inline_rust;

/// An `instance` declaration — Rust's `impl`.
pub(crate) mod instance_decl;

/// An item's leading trivia — its comments and attributes.
pub(crate) mod leading;

/// A `module` declaration.
pub(crate) mod module_decl;

/// One parameter in a signature.
pub(crate) mod param;

/// A pattern in an equation's binder position.
pub(crate) mod pattern;

/// Whether a lowercase name is one of Rust's primitive type names.
pub(crate) mod primitive_type;

/// One field of a record pattern — `x` or `x = 0`.
pub(crate) mod record_pattern_field;

/// A function's parameters and return type.
pub(crate) mod signature;

/// A `test` block.
pub(crate) mod test_decl;

/// A `type` alias.
pub(crate) mod type_alias;

/// A type written by juxtaposition: `Wrapper t`, `Pair k v`.
pub(crate) mod type_app;

/// An item's or field's visibility.
pub(crate) mod visibility;

use crate::FnDecl;

use crate::program::item::class_decl::ClassDecl;

use crate::program::item::const_decl::ConstDecl;

use crate::program::item::data_decl::DataDecl;

use crate::program::item::errors_decl::ErrorsDecl;

use crate::program::item::extern_block::ExternBlock;

use crate::program::item::import_decl::ImportDecl;

use crate::program::item::inline_rust::InlineRust;

use crate::program::item::instance_decl::InstanceDecl;

use crate::program::item::module_decl::ModuleDecl;

use crate::program::item::test_decl::TestDecl;

use crate::program::item::type_alias::TypeAlias;

use redox_lexer::Span;

/// A top-level declaration.
///
/// Doc 18 §§2.4–2.5 collapsed six of doc 03's declaration keywords into four.
/// `struct`, `enum`, `union` and the tuple-struct spelling are all `data`;
/// `class` means `trait` rather than doc 01 §17.7's "struct plus impl"; and
/// `instance` means `impl`, trait or inherent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
  Fn(FnDecl),
  Data(DataDecl),
  Type(TypeAlias),
  Class(ClassDecl),
  Instance(InstanceDecl),
  /// Doc 03 §12.5's `errors`, kept by doc 18 §8 — the one declaration that
  /// generates more than one Rust item.
  Errors(ErrorsDecl),
  /// Doc 03 §15's `test`, kept by doc 18 §8. The only item that is **not**
  /// emitted where it appears: §15 asks for one module per file, so the
  /// tests are gathered and written together.
  Test(TestDecl),
  /// Doc 03 §13.1's `import` — plan 14.
  Import(ImportDecl),
  /// Doc 18 §7's `module`, which is Rust's `mod` — plan 14. The only item
  /// that **contains** items, and so the only one the emitter recurses on.
  Module(ModuleDecl),
  /// Doc 03 §13.4's module-scope `const` or `static` — plan 16.
  Const(ConstDecl),
  /// Doc 01 §23.1's inline Rust — plan 45. The only item whose contents are
  /// never parsed.
  InlineRust(InlineRust),
  /// An `extern` block — plan 45 leaf 3, and doc 01 §26's FFI requirement.
  Extern(ExternBlock),
}
#[allow(unused_imports)]
use Item::*;

impl Item {
  pub fn span(&self) -> Span {
    match self {
      Item::Fn(f) => f.span,
      Item::Data(d) => d.span,
      Item::Type(a) => a.span,
      Item::Class(c) => c.span,
      Item::Instance(i) => i.span,
      Item::Errors(e) => e.span,
      Item::Test(t) => t.span,
      Item::Import(i) => i.span,
      Item::Module(m) => m.span,
      Item::Const(c) => c.span,
      Item::InlineRust(r) => r.span,
      Item::Extern(e) => e.span,
    }
  }
}
