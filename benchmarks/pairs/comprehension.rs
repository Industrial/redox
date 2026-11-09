pub fn doubled(xs: Vec<i32>) -> Vec<i32> {
  xs.into_iter()
    .filter_map(|n| if n % 2 == 1 { Some(n * 2) } else { None })
    .collect()
}
