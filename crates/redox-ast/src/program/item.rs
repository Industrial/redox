pub(crate) mod class_decl;
pub(crate) mod class_member;
pub(crate) mod clause;
pub(crate) mod clause_body;
pub(crate) mod constraint;
pub(crate) mod constructor;
pub(crate) mod constructor_fields;
pub(crate) mod data_decl;
pub(crate) mod error_variant;
pub(crate) mod errors_decl;
pub(crate) mod field_decl;
pub(crate) mod fn_decl;
pub(crate) mod guard;
pub(crate) mod instance_decl;
pub(crate) mod param;
pub(crate) mod pattern;
pub(crate) mod signature;
pub(crate) mod test_decl;
pub(crate) mod type_alias;
pub(crate) mod type_app;
pub(crate) mod visibility;

use crate::FnDecl;
use crate::program::item::class_decl::ClassDecl;
use crate::program::item::data_decl::DataDecl;
use crate::program::item::errors_decl::ErrorsDecl;
use crate::program::item::instance_decl::InstanceDecl;
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
}

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
    }
  }
}
