/// One pattern where the next token is a keyword or an arrow.
pub(crate) mod beside_a_keyword;

/// A comma-separated run of patterns, with no trailing comma.
pub(crate) mod joined;

/// One pattern where the next token could extend it, parenthesised unless it cannot be.
pub(crate) mod pattern_argument;

/// One field of a record pattern — `x = 0`, or `y` for the shorthand.
pub(crate) mod record_field_text;

use crate::pattern_text::beside_a_keyword::beside_a_keyword;

use crate::pattern_text::joined::joined;

use crate::pattern_text::pattern_argument::pattern_argument;

use crate::pattern_text::record_field_text::record_field_text;

use redox_ast::Pattern;

/// One pattern in canonical Redox — plan 34, leaf 4, first half.
///
/// # Why this is not `Pattern::rust_text`
///
/// `rust_text` exists on the AST and is close enough to be tempting. Three of
/// the eleven forms differ, and each difference is one a formatter must get
/// right rather than nearly right:
///
/// | Redox | Rust | |
/// |---|---|---|
/// | `Some v` | `Some(v)` | Juxtaposition, doc 18 §2.1 — the same rule as a call |
/// | `Point { x = 0, .. }` | `Point { x: 0, .. }` | `=`, because `:` is half of the `::` that ascribes |
/// | `13 or 14` | `13 \| 14` | `or`, because `\|` is the guard ladder's separator |
///
/// The other eight are spelled identically, including the two that look like
/// they would not be: a slice's rest is `..` and a named rest is `name @ ..`,
/// exactly as in Rust. `examples/destructuring.sdx` settles both. The `...`
/// spread that `lexical.sdx` lexes is a token the pattern grammar does not
/// use.
///
/// # Patterns carry no indentation and no trivia
///
/// Which is why this half of leaf 4 ships before the other: a pattern is
/// always one line, and a comment inside one is attached to the enclosing
/// statement. So there is no `indent` and no `source` here — nothing is
/// sliced, and everything is printed.
pub fn pattern_text(pattern: &Pattern) -> String {
  match pattern {
    Pattern::Binding(ident) => ident.text.clone(),
    Pattern::Literal { text, .. } => text.clone(),
    Pattern::Wildcard(_) => String::from("_"),
    Pattern::Range {
      lo, hi, inclusive, ..
    } => {
      // Bound, where the Rust put the `if` in `format!`'s argument list: a
      // `#{…}` hole takes an expression and the two-arm choice reads better
      // with a name on it anyway.
      let dots = if *inclusive { "..=" } else { ".." };
      format!("{}{}{}", lo, dots, hi)
    }
    Pattern::Apply { path, args, .. } => {
      let mut out = path.clone();
      for arg in args {
        out.push(' ');
        out.push_str(&pattern_argument(arg));
      }
      out
    }
    Pattern::Tuple { items, .. } => {
      let inner = joined(items);
      if items.len() == 1 {
        format!("({},)", inner)
      } else {
        format!("({})", inner)
      }
    }
    Pattern::Slice { items, .. } => format!("[{}]", joined(items)),
    Pattern::Record {
      path, fields, rest, ..
    } => {
      let mut parts: Vec<String> = fields.iter().map(record_field_text).collect();
      if *rest {
        parts.push(String::from(".."));
      }
      if parts.is_empty() {
        format!("{} {{}}", path)
      } else {
        format!("{} {{ {} }}", path, parts.join(", "))
      }
    }
    Pattern::As { name, inner, .. } => format!("{} @ {}", name.text, beside_a_keyword(inner)),
    Pattern::Rest(_) => String::from(".."),
    Pattern::NamedRest { name, .. } => format!("{} @ ..", name.text),
    Pattern::Or(alts) => {
      let printed: Vec<String> = alts.iter().map(beside_a_keyword).collect();
      printed.join(" or ")
    }
  }
}
