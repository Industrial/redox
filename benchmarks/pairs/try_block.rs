pub fn attempt(r: Result<i32, String>) -> Result<i32, String> {
  let a = r?;
  Ok(a + 1)
}
