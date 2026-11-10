use crate::program::item::class_member::ClassMember;
use crate::program::item::constraint::Constraint;
use crate::program::item::type_app::TypeApp;
use redox_lexer::{Span, Trivia};

/// An `instance` declaration — doc 18 §2.5's `impl`.
///
/// ```text
/// instance Display t => Describe (Wrapper t)   impl<T: Display> Describe for Wrapper<T> {
///   describe self = "…"                    →       fn describe(&self) -> String { "…" }
///                                              }
///
/// instance Counter                             impl Counter {
///   increment : &mut self -> ()           →       fn increment(&mut self) {
///   increment self = self.value += 1                   self.value += 1;
///                                                  }
///                                              }
/// ```
///
/// `instance` means `impl`, **with or without a class name**, which is what
/// lets one keyword cover both of Rust's forms. Which one is written is
/// decided by how many names precede the target: `instance Counter` is
/// inherent, `instance Describe (Wrapper t)` is a trait impl.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceDecl {
  /// The class being implemented, with its own arguments, or `None` for an
  /// inherent `impl`.
  ///
  /// A [`TypeApp`] rather than a bare name since plan 67:
  /// `instance Convert f64 Meters` is `impl Convert<f64> for Meters`, so the
  /// class carries arguments of its own and the **last** name in the head is
  /// the implementing type.
  pub class: Option<TypeApp>,
  pub target: TypeApp,
  /// `Display t =>` — bounds on the generic parameters.
  pub constraints: Vec<Constraint>,
  pub members: Vec<ClassMember>,
  pub span: Span,
  pub leading: Vec<Trivia>,
}
