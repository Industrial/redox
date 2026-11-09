pub struct Point {
  pub x: i32,
  pub y: i32,
}

pub fn optional_chain(p: &Option<Point>) -> Option<&i32> {
  p.as_ref().map(|inner| &inner.x)
}
