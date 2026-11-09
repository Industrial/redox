pub(crate) mod beside_a_keyword;
pub(crate) mod joined;
pub(crate) mod pattern_argument;
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
    Pattern::Wildcard(_) => "_".to_string(),
    // Tight, unlike a range *expression*, which the corpus spaces. Both
    // spellings are the corpus's own: `18..=64 -> "adult"` in a case arm and
    // `for c in 1 .. 4` in a statement. A pattern range cannot be confused
    // with anything, and an expression range sits among spaced operators.
    Pattern::Range {
      lo, hi, inclusive, ..
    } => format!("{lo}{}{hi}", if *inclusive { "..=" } else { ".." }),
    // Juxtaposition, and left-associative, so an `Apply` argument needs its
    // own parentheses: `Some (Some v)` and not `Some Some v`, which is `Some`
    // applied to two arguments. `examples/draw_condition.sdx` says so in a
    // comment and this is the code that keeps it true.
    Pattern::Apply { path, args, .. } => {
      let mut out = path.clone();
      for arg in args {
        out.push(' ');
        out.push_str(&pattern_argument(arg));
      }
      out
    }
    // A one-element tuple keeps its comma, for plan 33 §10.3's reason: `(a)`
    // is a group and `(a,)` is a tuple, so dropping it turns a destructuring
    // of a `(T,)` into a binding of the whole value.
    Pattern::Tuple { items, .. } => {
      let inner = joined(items);
      if items.len() == 1 {
        format!("({inner},)")
      } else {
        format!("({inner})")
      }
    }
    Pattern::Slice { items, .. } => format!("[{}]", joined(items)),
    Pattern::Record {
      path, fields, rest, ..
    } => {
      let mut parts: Vec<String> = fields.iter().map(record_field_text).collect();
      if *rest {
        parts.push("..".to_string());
      }
      if parts.is_empty() {
        format!("{path} {{}}")
      } else {
        format!("{path} {{ {} }}", parts.join(", "))
      }
    }
    // The inner pattern sits beside `->`, `,` or `or` and not beside another
    // pattern, so it needs parentheses only if it is itself an alternation —
    // `whole @ Rect { w, h }` is the corpus's own spelling.
    Pattern::As { name, inner, .. } => {
      format!("{} @ {}", name.text, beside_a_keyword(inner))
    }
    Pattern::Rest(_) => "..".to_string(),
    Pattern::NamedRest { name, .. } => format!("{} @ ..", name.text),
    // `or`, not `|`. An alternative needs no parentheses: `or` is a keyword,
    // so `Some 1 or None` cannot read `or` as an argument. Printing the
    // alternatives as *arguments* produced `(Some 1) or None`, which the
    // fixture caught against the corpus's spelling.
    Pattern::Or(alts) => alts
      .iter()
      .map(beside_a_keyword)
      .collect::<Vec<_>>()
      .join(" or "),
  }
}
