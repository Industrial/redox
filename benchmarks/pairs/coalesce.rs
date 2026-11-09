// What a competent Rust author writes for the same job.
pub fn pick(o: Option<i32>) -> i32 {
  o.unwrap_or(0)
}
