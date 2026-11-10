pub struct Point {
  pub x: i32,
  pub y: i32,
}

pub fn borrowed_chain(p: &Option<Point>) -> Option<&i32> {
  p.as_ref().map(|inner| &inner.x)
}
