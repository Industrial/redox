use crate::emit::error_message::error_message;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::trivia::emit_trivia;
use redox_ast::ErrorsDecl;

/// Emit doc 03 §12.5's `errors` declaration: the enum, `Display`, `Error`,
/// and a `From` per `from` row.
///
/// ```text
/// errors ConfigError                    #[derive(Debug)]
///   Missing String  "missing: #{0}"     pub enum ConfigError {
///   from io::Error as Io "io: #{0}"  →      Missing(String),
///                                           Io(io::Error),
///                                       }
///                                       impl std::fmt::Display for … { … }
///                                       impl std::error::Error for … {}
///                                       impl From<io::Error> for … { … }
/// ```
///
/// Four items from one declaration, which is the most any Redox construct
/// generates — and the reason doc 03 §12.5 is careful to say it "generates
/// items for a user-declared type, as `derive` does; it is not runtime
/// scaffolding". The distinction is what admits it: every name it emits is
/// the author's or `std`'s.
///
/// Paths are written in full — `std::fmt::Display`, `std::error::Error` —
/// because Redox emits no `use`, so a short path would depend on what the
/// author happened to import. Doc 03 §12.5's own Rust does the same.
pub(crate) fn emit_errors(out: &mut Sink<'_>, d: &ErrorsDecl) {
  out.unmark();
  emit_trivia(out, &d.leading, 0);
  out.mark(d.span);
  let name = &d.name.text;

  out.push_str("#[derive(Debug)]\n");
  out.push_str(&format!("pub enum {name} {{\n"));
  for v in &d.variants {
    out.mark(v.span);
    pad(out, RUST_INDENT);
    out.push_str(&v.name.text);
    if !v.fields.is_empty() {
      let texts: Vec<&str> = v.fields.iter().map(|t| t.text.as_str()).collect();
      out.push_str(&format!("({})", texts.join(", ")));
    }
    out.push_str(",\n");
  }
  out.unmark();
  out.push_str("}\n\n");

  out.push_str(&format!("impl std::fmt::Display for {name} {{\n"));
  pad(out, RUST_INDENT);
  out.push_str("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
  pad(out, RUST_INDENT * 2);
  out.push_str("match self {\n");
  for v in &d.variants {
    out.mark(v.span);
    let (format, args) = error_message(&v.message);
    let binders: Vec<String> = (0..v.fields.len()).map(|i| format!("_{i}")).collect();
    pad(out, RUST_INDENT * 3);
    out.push_str(&format!("{name}::{}", v.name.text));
    if !binders.is_empty() {
      out.push_str(&format!("({})", binders.join(", ")));
    }
    out.push_str(&format!(" => write!(f, \"{format}\""));
    for arg in &args {
      out.push_str(&format!(", {arg}"));
    }
    out.push_str("),\n");
  }
  out.unmark();
  pad(out, RUST_INDENT * 2);
  out.push_str("}\n");
  pad(out, RUST_INDENT);
  out.push_str("}\n}\n\n");

  out.push_str(&format!("impl std::error::Error for {name} {{}}\n"));

  for v in &d.variants {
    if !v.from {
      continue;
    }
    let ty = &v.fields[0].text;
    out.mark(v.span);
    out.push('\n');
    out.push_str(&format!("impl From<{ty}> for {name} {{\n"));
    pad(out, RUST_INDENT);
    out.push_str(&format!("fn from(e: {ty}) -> Self {{\n"));
    pad(out, RUST_INDENT * 2);
    out.push_str(&format!("{name}::{}(e)\n", v.name.text));
    pad(out, RUST_INDENT);
    out.push_str("}\n}\n");
  }
  out.unmark();
}
