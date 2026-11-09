pub fn halve(o: Option<i32>) -> i32 {
  let Some(v) = o else {
    return 0;
  };
  v / 2
}
