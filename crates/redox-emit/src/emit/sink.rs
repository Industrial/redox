use redox_diag::{Origin, SourceMap, Span};

/// The emitter's output, recording where each generated line came from.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 2. Replaces the `&mut String`
/// the twelve emit helpers took. It implements [`std::fmt::Write`], so a
/// helper built on `write!` changes in its parameter type and nowhere else.
///
/// The recording works by *current mark*: [`Sink::mark`] says "everything from
/// here came from this source span", and every generated line the sink begins
/// after that is attributed to it until the next mark. A construct emitting
/// three lines therefore maps all three, and no helper has to know how many
/// lines it will produce — which it generally cannot know, since a nested
/// construct may add some.
pub(crate) struct Sink<'a> {
  out: String,
  /// The source text, needed to resolve a span to a line and column. Borrowed
  /// rather than copied because the emitter already outlives it.
  source: &'a str,
  current: Option<Span>,
  /// One entry per *completed* line; the line being built is still `current`.
  origins: Vec<Option<Origin>>,
}

impl<'a> Sink<'a> {
  pub(crate) fn new(source: &'a str) -> Self {
    Self {
      out: String::new(),
      source,
      current: None,
      origins: Vec::new(),
    }
  }

  /// Attribute everything emitted from here to `span`.
  pub(crate) fn mark(&mut self, span: Span) {
    self.current = Some(span);
  }

  /// Stop attributing: the next lines are the emitter's own.
  pub(crate) fn unmark(&mut self) {
    self.current = None;
  }

  pub(crate) fn push(&mut self, ch: char) {
    self.write_text(&ch.to_string());
  }

  pub(crate) fn push_str(&mut self, s: &str) {
    self.write_text(s);
  }

  pub(crate) fn pop(&mut self) -> Option<char> {
    let ch = self.out.pop();
    if ch == Some('\n') {
      self.origins.pop();
    }
    ch
  }

  pub(crate) fn ends_with(&self, s: &str) -> bool {
    self.out.ends_with(s)
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.out.is_empty()
  }

  /// Finish, returning the Rust source and its map.
  ///
  /// `source_path` is recorded in the map's header; it is the `.sdx` path as
  /// the caller knows it, which the emitter has no other way to learn.
  pub(crate) fn finish(mut self, source_path: &str) -> (String, SourceMap) {
    // A final line with no terminating newline is still a line.
    if !self.out.is_empty() && !self.out.ends_with('\n') {
      self.close_line();
    }
    (self.out, SourceMap::new(source_path, self.origins))
  }

  /// Record the origin of the line just completed.
  fn close_line(&mut self) {
    let origin = self.current.and_then(|span| self.origin_of(span));
    self.origins.push(origin);
  }

  /// Resolve a span to an [`Origin`], given where the line being built began.
  fn origin_of(&self, span: Span) -> Option<Origin> {
    if span.0 > self.source.len() {
      return None;
    }
    let before = &self.source[..span.0];
    let src_line = before.matches('\n').count() + 1;
    let src_col = before
      .rsplit('\n')
      .next()
      .map_or(1, |l| l.chars().count() + 1);
    Some(Origin {
      span,
      src_line,
      src_col,
      gen_col: self.gen_col_of_current_line(),
    })
  }

  /// The column at which the current line's non-blank content begins.
  ///
  /// This is the generated half of the column shift: content on the generated
  /// line starts after the emitter's indentation, and the source's starts
  /// after the author's, so the difference is what [`Origin::map_col`]
  /// subtracts.
  fn gen_col_of_current_line(&self) -> usize {
    let line_start = self.out.rfind('\n').map_or(0, |i| i + 1);
    let line = &self.out[line_start..];
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
      // Nothing on the line yet: content will begin where the cursor is.
      line.chars().count() + 1
    } else {
      line.chars().count() - trimmed.chars().count() + 1
    }
  }

  /// Append text, closing a line record at every newline it contains.
  fn write_text(&mut self, s: &str) {
    for part in s.split_inclusive('\n') {
      self.out.push_str(part);
      if part.ends_with('\n') {
        // The origin is computed *before* the newline is consumed, so the
        // column it records is the one for the line that just ended. Pop the
        // newline, measure, put it back.
        self.out.pop();
        self.close_line();
        self.out.push('\n');
      }
    }
  }
}

impl std::fmt::Write for Sink<'_> {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    self.write_text(s);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::Sink;
  use std::fmt::Write as _;

  #[test]
  fn attributes_a_line_to_the_current_mark() {
    let source = "main = ->\n  print 1\n";
    let mut sink = Sink::new(source);
    sink.mark((0, 9)); // `main = ->`
    sink.push_str("fn main() {\n");
    sink.mark((12, 19)); // `print 1`
    sink.push_str("    println!();\n");
    sink.unmark();
    sink.push_str("}\n");

    let (text, map) = sink.finish("a.sdx");
    assert_eq!(text, "fn main() {\n    println!();\n}\n");
    assert_eq!(map.len(), 3);
    assert_eq!(map.resolve(1, 1), Some((1, 1)));
    assert_eq!(map.resolve(2, 5), Some((2, 3)), "content column aligns");
    assert_eq!(map.resolve(3, 1), None, "the closing brace is synthetic");
  }

  /// A mark that spans several generated lines maps every one of them.
  #[test]
  fn carries_a_mark_across_lines() {
    let mut sink = Sink::new("a\nb\n");
    sink.mark((0, 1));
    sink.push_str("one\ntwo\nthree\n");
    let (_, map) = sink.finish("a.sdx");
    assert_eq!(map.len(), 3);
    for line in 1..=3 {
      assert_eq!(map.resolve(line, 1), Some((1, 1)), "line {line}");
    }
  }

  #[test]
  fn counts_a_final_line_without_a_newline() {
    let mut sink = Sink::new("a\n");
    sink.mark((0, 1));
    sink.push_str("x");
    let (text, map) = sink.finish("a.sdx");
    assert_eq!(text, "x");
    assert_eq!(map.len(), 1);
  }

  #[test]
  fn write_macro_records_the_same_way() {
    let mut sink = Sink::new("ab\n");
    sink.mark((0, 2));
    let _ = writeln!(sink, "hello");
    let (text, map) = sink.finish("a.sdx");
    assert_eq!(text, "hello\n");
    assert_eq!(map.len(), 1);
  }

  /// `pop` is how the emitter trims trailing blank lines; the map must shrink
  /// with the text or every record after the trim would be off by one.
  #[test]
  fn pop_removes_the_line_it_removes() {
    let mut sink = Sink::new("a\n");
    sink.mark((0, 1));
    sink.push_str("one\n\n");
    assert_eq!(sink.pop(), Some('\n'));
    let (text, map) = sink.finish("a.sdx");
    assert_eq!(text, "one\n");
    assert_eq!(map.len(), 1);
  }
}
