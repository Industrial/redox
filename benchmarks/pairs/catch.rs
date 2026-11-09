pub fn parse_or(raw: &str) -> i32 {
  match raw.parse::<i32>() {
    Ok(value) => value,
    Err(_) => 0,
  }
}
