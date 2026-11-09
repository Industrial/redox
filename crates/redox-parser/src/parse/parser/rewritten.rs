use crate::parse::parser::flush_word::flush_word;

/// A captured type's text with every whole-word occurrence of a type
/// variable replaced — doc 18 §5.3.
///
/// ```text
/// t          → Self        inside a class
/// &[t]       → &[T]        inside an instance, or a constrained function
/// Vec<t>     → Vec<Self>
/// &'static t → &'static T
/// ```
///
/// **Whole-word, not whole-type.** The first version replaced a type only
/// when the entire type was the variable, which left `Vec<t>` alone and so
/// left a `t` rustc could not resolve. A type's text is a token sequence
/// (doc 03 §14.1), and an identifier in it is delimited by characters that
/// cannot appear in one — so a word scan is exact here without parsing the
/// type, which §14.1 exists to avoid having to do.
///
/// A leading `'` is part of the preceding run's delimiter, so `'static` is
/// the word `static` and never matches a variable named `t`. That falls out
/// rather than being special-cased, which is why it is worth saying.
pub(crate) fn rewritten(text: &str, var: &str, subject: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut word = String::new();
  for ch in text.chars() {
    if ch.is_alphanumeric() || ch == '_' {
      word.push(ch);
      continue;
    }
    flush_word(&mut out, &mut word, var, subject);
    out.push(ch);
  }
  flush_word(&mut out, &mut word, var, subject);
  out
}
