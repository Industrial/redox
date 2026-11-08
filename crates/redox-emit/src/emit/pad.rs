/// Append `n` spaces to `out`.
pub(crate) fn pad(out: &mut String, n: usize) {
  for _ in 0..n {
    out.push(' ');
  }
}
