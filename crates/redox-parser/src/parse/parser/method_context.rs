use crate::parse::parser::rust_generic::rust_generic;
use redox_ast::Ident;

/// The type variables a `class` or `instance` body inherits from its own
/// head, each with the Rust spelling it takes.
///
/// Doc 18 §5.3 makes a lowercase name in a signature a *variable of that
/// signature*, and a `class` or `instance` head is the one place that is
/// wrong: the `t` in `class Convert t` belongs to the trait, and the `t` in
/// `instance Describe (Wrapper t)` belongs to the impl. Both facts are needed
/// twice over — once to keep the name **out** of the method's own generic
/// list (`generalise`'s `reserved`), and once to rewrite it to Rust's
/// spelling (`substitute_var`) — so they travel together rather than as loose
/// arguments.
///
/// **It no longer says which parameter becomes `self`.** Plan 67 marks the
/// receiver in the type, so `receiver_names` — `Self`, the class variable and
/// the instance's own target name — is gone, and with it the question of
/// which of three spellings happened to mean the implementing type.
pub(crate) struct MethodContext {
  /// Each inherited variable as the source spells it, with its Rust
  /// spelling.
  pub vars: Vec<(String, String)>,
  /// The `class` being **declared**, whose member heads are recorded as they
  /// are read — plan 63 leaf 1.
  pub recording: Option<String>,
  /// The `class` being **implemented**, whose member heads an elided
  /// signature borrows. `None` for an inherent `instance`, which has no class
  /// to borrow from and so cannot elide.
  pub implementing: Option<String>,
}

impl MethodContext {
  /// A `class`'s context: its parameters are the trait's own generic
  /// parameters, so each takes its uppercase name — `class Convert t` gives
  /// `trait Convert<T>`.
  ///
  /// Plan 67 §1.6 claimed this could only shrink, since the implementing type
  /// substitutes to itself under `Self`. It lost one entry and gained a
  /// dimension: the class variable is gone, and a class may now have more
  /// than one parameter.
  pub(crate) fn for_class(params: &[Ident]) -> Self {
    Self {
      vars: params
        .iter()
        .map(|p| (p.text.clone(), rust_generic(&p.text)))
        .collect(),
      recording: None,
      implementing: None,
    }
  }

  /// An `instance`'s context. `var` is the target's first type variable, if
  /// it has one, and `subject` is that variable's Rust spelling — the impl's
  /// generic parameter, as in `impl<T> Describe for Wrapper<T>`.
  pub(crate) fn for_instance(var: Option<&str>, subject: &str) -> Self {
    Self {
      vars: var
        .map(|var| (var.to_string(), subject.to_string()))
        .into_iter()
        .collect(),
      recording: None,
      implementing: None,
    }
  }

  /// The names a method must **not** make generic parameters of its own.
  pub(crate) fn reserved(&self) -> Vec<String> {
    self.vars.iter().map(|(var, _)| var.clone()).collect()
  }
}
