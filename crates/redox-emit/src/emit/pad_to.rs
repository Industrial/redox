/// Append `n` spaces to a `String`.
///
/// `pad` writes into a [`crate::emit::sink::Sink`], which records a source
/// position per line; the expression emitters build plain strings and have no
/// position to record, so they need this instead. Two functions rather than
/// one because the two have genuinely different jobs — the `Sink` is where
/// plan 06's span map comes from.
pub(crate) fn pad_to(out: &mut String, n: usize) {
  for _ in 0..n {
    out.push(' ');
  }
}
