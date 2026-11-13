// Deliberately *not* equivalent: this clones, and the Voltaic side does not.
// Criterion 4 asserts the comparison notices.
pub fn unequal(xs: Vec<String>) -> usize {
  let copied = xs.clone();
  copied.len()
}
