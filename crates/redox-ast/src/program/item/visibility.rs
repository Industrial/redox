/// An item's or field's visibility — doc 03 §9.6.
///
/// | Redox | Rust |
/// |---|---|
/// | (absent) | private |
/// | `export` | `pub` |
/// | `export(crate)` | `pub(crate)` |
/// | `export(super)` | `pub(super)` |
/// | `export(in path::to)` | `pub(in path::to)` |
///
/// §9.6 extends doc 01 §22, which "offers only `export` → `pub`, which cannot
/// express Rust's scoped visibilities or per-field visibility, and so cannot
/// support real library work". Doc 18 §8 declines Haskell's module header for
/// the same reason — `module M (f, g) where` cannot say any of this.
///
/// Private is the default and has no spelling, which is Rust's rule and
/// Haskell's opposite: an export list makes everything public by omission of
/// the list. Doc 03 §9.6 keeps Rust's, because the generated code has to mean
/// what Rust says it means.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Visibility {
  #[default]
  Private,
  Public,
  Crate,
  Super,
  /// `export(in path::to)` — the path carried verbatim, as every path in
  /// Redox is (doc 03 §13.1).
  In(String),
}

impl Visibility {
  /// The Rust spelling, **including its trailing space** when there is one.
  ///
  /// The space is part of it because the caller writes `{vis}fn name` and a
  /// private item must not acquire a leading space — which is the kind of
  /// difference the golden corpus notices and a reader does not.
  pub fn rust_text(&self) -> String {
    match self {
      Self::Private => String::new(),
      Self::Public => "pub ".to_string(),
      Self::Crate => "pub(crate) ".to_string(),
      Self::Super => "pub(super) ".to_string(),
      Self::In(path) => format!("pub(in {path}) "),
    }
  }
}
