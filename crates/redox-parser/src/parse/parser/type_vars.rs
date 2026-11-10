use crate::parse::parser::is_word::is_word;
use redox_ast::primitive_type;

/// The type variables and lifetimes a signature's types mention, in order of
/// first appearance — doc 18 §§2.2, 5.3.
///
/// > Lowercase type names are type variables, as in Haskell; the generic list
/// > is the order of their first appearance … lifetimes included.
///
/// ```text
/// &[t] -> &t            ["t"]
/// (t, u) -> (u, t)      ["t", "u"]
/// &'a str -> &'a str    ["'a"]
/// i32 -> i32            []
/// ```
///
/// The names are the **source's**, not Rust's: a caller needs both, to write
/// `<T>` in the generic list and to replace `t` with `T` in the types.
///
/// **Lifetimes come first**, which is rustc's requirement rather than a
/// style: a type parameter before a lifetime is an error, not a lint. Within
/// each group the order is the source's, because the emitter's output has to
/// be byte-stable.
///
/// Five things are excluded, each for a reason worth stating:
///
/// - **primitives** — `i32` is lowercase and concrete; see
///   [`primitive_type`];
/// - **`self`** — plan 67's receiver marker. `area : &self -> f64` would
///   otherwise synthesise `fn area<Self_>(…)`, and the failure mode is a
///   program that still compiles in simple cases. Checked here as its own
///   condition rather than added to `primitive_type`, because `self` is not a
///   primitive type and a list whose name stops describing its contents is
///   how a reader learns to stop trusting the names — plan 67 §9 decision 4;
/// - **`'static`** — a lifetime that needs no declaration;
/// - **the type keywords** `dyn`, `impl`, `mut` and `as`, which are lowercase
///   words inside a type and name nothing;
/// - **path segments** — `fmt` in `fmt::Result` is a module. A type variable
///   is never part of a path, so an identifier adjacent to `::` on either
///   side is skipped. Found by `fmt::Result` in a signature emitting
///   `fn write_ok<FMT>(…) -> FMT::Result`.
///
/// Reads the captured type **text**, which is a token sequence (doc 03
/// §14.1), so an identifier in it is delimited by characters that cannot
/// appear in one. That makes a word scan exact without parsing the type.
pub(crate) fn type_vars(types: &[&str]) -> Vec<String> {
  let mut lifetimes: Vec<String> = Vec::new();
  let mut vars: Vec<String> = Vec::new();

  for text in types {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
      if !is_word(chars[i]) {
        i += 1;
        continue;
      }
      let start = i;
      while i < chars.len() && is_word(chars[i]) {
        i += 1;
      }
      let word: String = chars[start..i].iter().collect();
      let is_lifetime = start > 0 && chars[start - 1] == '\'';

      // **A path segment is not a type variable.** `fmt::Result` has a
      // lowercase `fmt` and `fmt` is a *module*; making it a parameter emitted
      // `fn write_ok<FMT>(…) -> FMT::Result`, which rustc rejects with
      // "associated type `Result` not found". Doc 18 §5.3's rule is about a
      // type variable, and a variable is never part of a path — so an
      // identifier adjacent to `::` on either side is skipped.
      let after_colons = start >= 2 && chars[start - 2] == ':' && chars[start - 1] == ':';
      let before_colons = chars.get(i) == Some(&':') && chars.get(i + 1) == Some(&':');
      if after_colons || before_colons {
        continue;
      }

      if is_lifetime {
        if word != "static" {
          let named = format!("'{word}");
          if !lifetimes.contains(&named) {
            lifetimes.push(named);
          }
        }
        continue;
      }

      if !word.starts_with(|c: char| c.is_lowercase())
        || primitive_type(&word)
        // Plan 67's receiver marker, and not a type at all — see the doc
        // comment's fifth exclusion for why it is not in `primitive_type`.
        || word == "self"
        || matches!(word.as_str(), "dyn" | "impl" | "mut" | "as")
      {
        continue;
      }
      if !vars.contains(&word) {
        vars.push(word);
      }
    }
  }

  lifetimes.extend(vars);
  lifetimes
}
