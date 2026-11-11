pub fn shifted(xs: Vec<i32>) -> i32 {
  xs.into_iter().map(|n| sub(100, n)).sum()
}

pub fn sub(a: i32, b: i32) -> i32 {
  a - b
}
