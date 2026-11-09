/// Append a pending identifier run to `out`, substituted if it is the type
/// variable — the inner step of `rewritten`.
pub(crate) fn flush_word(out: &mut String, word: &mut String, var: &str, subject: &str) {
  if word.is_empty() {
    return;
  }
  if word == var {
    out.push_str(subject);
  } else {
    out.push_str(word);
  }
  word.clear();
}
