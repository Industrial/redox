pub fn total(xs: Vec<i32>) -> i32 {
  xs.into_iter().filter(|n| *n > 1).sum()
}
