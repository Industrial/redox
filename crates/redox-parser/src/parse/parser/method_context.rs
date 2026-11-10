/// What a `class` or `instance` body needs to know about its own subject.
///
/// Doc 18 §2.5 derives two things from the enclosing declaration, and both
/// need the same information, so they travel together rather than as four
/// loose arguments:
///
/// - **which parameter becomes `self`** — "the first parameter typed by the
///   class variable". More than one spelling names that type, so
///   `receiver_names` is a list: `Self` always, the class variable when there
///   is one, and the target's own name in an `instance`, because
///   `instance Shape Square` writes `area : &Square -> f64`.
/// - **what the class variable means in Rust** — `Self` inside a class, where
///   the variable *is* the implementing type, and the impl's generic
///   parameter inside an instance, where it is a type parameter.
pub(crate) struct MethodContext {
  /// The class variable, if the declaration has one.
  pub var: Option<String>,
  /// What the class variable becomes in Rust.
  pub subject: String,
  /// Every spelling of the implementing type, for receiver detection.
  pub receiver_names: Vec<String>,
}

impl MethodContext {
  /// A `class`'s context: the variable is the implementing type, so it is
  /// `Self`.
  pub(crate) fn for_class(var: &str) -> Self {
    Self {
      var: Some(var.to_string()),
      subject: "Self".to_string(),
      receiver_names: vec!["Self".to_string(), var.to_string()],
    }
  }

  /// An `instance`'s context. `var` is the target's first type variable, if
  /// it has one, and `target` is the target's head name — `Square` in
  /// `instance Shape Square`, which is how that instance's methods spell
  /// their receiver.
  pub(crate) fn for_instance(var: Option<&str>, target: &str, subject: &str) -> Self {
    let mut receiver_names = vec!["Self".to_string(), target.to_string()];
    if let Some(var) = var {
      receiver_names.push(var.to_string());
    }
    Self {
      var: var.map(str::to_string),
      subject: subject.to_string(),
      receiver_names,
    }
  }
}
