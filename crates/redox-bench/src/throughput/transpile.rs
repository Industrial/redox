/// Parse and emit, which is what `redox build` does between reading a file and
/// writing one.
///
/// Deliberately **not** `redox build` itself: that writes a file and consults
/// a cache, so timing it would time the filesystem. The claim the report makes
/// is about the compiler, and this is the compiler.
///
/// `None` for a source that does not parse. The report excludes `lexical.sdx`
/// for that reason — transpiling a file that cannot parse measures the error
/// path — and this returns rather than panicking so a future caller can decide
/// differently.
pub(crate) fn transpile(source: &str) -> Option<String> {
  let program = redox_parser::parse_source(source).ok()?;
  Some(redox_emit::emit(&program))
}
