use crate::emit::sink::Sink;
/// Append `n` spaces to `out`.
pub(crate) fn pad(out: &mut Sink<'_>, n: usize) {
  for _ in 0..n {
    out.push(' ');
  }
}
