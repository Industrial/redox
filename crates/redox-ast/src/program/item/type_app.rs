use crate::Ident;
use crate::program::item::primitive_type::primitive_type;
use redox_lexer::Span;

/// A type written by **juxtaposition**: `Wrapper t`, `Pair k v`, `Counter`.
///
/// Doc 18 §2.1 reads application the same way everywhere, types included, so
/// an `instance` target needs no notation of its own. The lowering adds the
/// angle brackets Rust wants and nothing else:
///
/// ```text
/// Counter        Counter
/// Wrapper t   →  Wrapper<T>
/// Pair k v       Pair<K, V>
/// ```
///
/// Structured rather than captured verbatim, unlike every other type in Redox
/// (doc 03 §14.1). The reason is that the *generic list* has to be
/// synthesised from it — doc 18 §5.3's "the generic list is synthesised from
/// first appearance" — and a string cannot say which of its words were type
/// variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeApp {
  pub head: Ident,
  /// The arguments, in source order. A lowercase argument is a type
  /// *variable* (doc 18 §5.3) and is emitted uppercase.
  pub args: Vec<Ident>,
  pub span: Span,
}

impl TypeApp {
  /// The Rust spelling: `Wrapper<T>`, or just `Counter` with no arguments.
  pub fn rust_text(&self) -> String {
    if self.args.is_empty() {
      return self.head.text.clone();
    }
    let args: Vec<String> = self.args.iter().map(|a| Self::rust_name(&a.text)).collect();
    format!("{}<{}>", self.head.text, args.join(", "))
  }

  /// The type variables, in order of first appearance.
  ///
  /// Doc 18 §5.3: "type variables need no declaration … the generic list is
  /// synthesised from first appearance". Order matters and duplicates do not,
  /// so this is a vector filtered against itself rather than a set — a set
  /// would make the output depend on iteration order, which the emitter's
  /// byte-stability forbids.
  pub fn vars(&self) -> Vec<&str> {
    let mut seen: Vec<&str> = Vec::new();
    for a in &self.args {
      if Self::is_var(&a.text) && !seen.contains(&a.text.as_str()) {
        seen.push(&a.text);
      }
    }
    seen
  }

  /// Whether a name is a type variable, which doc 18 §5.3 makes the case of
  /// its first letter — with [`primitive_type`]'s correction, because `f64`
  /// is lowercase and concrete.
  ///
  /// The correction arrived with plan 67: `instance Convert f64 Meters` is
  /// `impl Convert<f64> for Meters`, and the case of the first letter alone
  /// made it `Convert<F64>`.
  pub fn is_var(text: &str) -> bool {
    text.chars().next().is_some_and(char::is_lowercase) && !primitive_type(text)
  }

  /// A type argument's Rust spelling: a variable uppercased, a concrete type
  /// left alone.
  pub fn rust_name(text: &str) -> String {
    if Self::is_var(text) {
      return text.to_uppercase();
    }
    text.to_string()
  }
}
