//! Doc 03 §12.5's `errors` declaration, kept by doc 18 §8.
//!
//! The one Redox construct that generates more than one Rust item: the enum,
//! `Display`, `Error`, and a `From` per `from` row. §12.5 is careful that it
//! "generates items for a user-declared type, as `derive` does; it is not
//! runtime scaffolding" — which is what admits it at all.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The visibility is doc 03 §9.6's, which this declaration obeys like any
/// other — §12.5's example shows `pub enum` because it reads as having been
/// written `export errors`.
#[test]
fn the_enum_derives_debug_and_takes_its_marker() {
  let private = xrs("errors E\n  Missing String \"missing: #{0}\"\n");
  assert!(
    private.starts_with("#[derive(Debug)]\nenum E {\n"),
    "{private}"
  );
  assert!(private.contains("    Missing(String),"), "{private}");

  let public = xrs("export errors E\n  Missing String \"missing: #{0}\"\n");
  assert!(
    public.starts_with("#[derive(Debug)]\npub enum E {\n"),
    "{public}"
  );
}

/// The payload is **juxtaposed**, doc 18 §2.1 — so `Invalid String u32` is
/// two fields. `capture_type` reads greedily everywhere else, and read
/// `String u32` as one field until the juxtaposed lists asked for an atom.
#[test]
fn a_juxtaposed_payload_is_several_fields() {
  let out = xrs("errors E\n  Invalid String u32 \"bad #{0} at #{1}\"\n");
  assert!(out.contains("    Invalid(String, u32),"), "{out}");
}

#[test]
fn a_variant_with_no_payload_has_no_parentheses() {
  let out = xrs("errors E\n  Closed \"closed\"\n");
  assert!(out.contains("    Closed,\n"), "{out}");
}

/// The holes are positional and become `{}` with the field passed alongside,
/// rather than Rust's `{0}`, because the arguments are supplied in hole order.
#[test]
fn the_message_holes_become_format_arguments() {
  let out = xrs("errors E\n  Invalid String u32 \"bad #{0} at line #{1}\"\n");
  assert!(
    out.contains("E::Invalid(_0, _1) => write!(f, \"bad {} at line {}\", _0, _1),"),
    "{out}"
  );
}

#[test]
fn a_message_with_no_holes_takes_no_arguments() {
  let out = xrs("errors E\n  Closed \"closed\"\n");
  assert!(out.contains("E::Closed => write!(f, \"closed\"),"), "{out}");
}

/// Paths are written in full, because Redox emits no `use` — a short path
/// would depend on what the author happened to import.
#[test]
fn the_display_impl_uses_full_paths() {
  let out = xrs("errors E\n  Closed \"closed\"\n");
  assert!(out.contains("impl std::fmt::Display for E {"), "{out}");
  assert!(
    out.contains("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {"),
    "{out}"
  );
}

#[test]
fn the_error_impl_is_empty() {
  let out = xrs("errors E\n  Closed \"closed\"\n");
  assert!(out.contains("impl std::error::Error for E {}"), "{out}");
}

/// A `from` row additionally generates the `From` impl, which is the whole
/// reason the row exists: it is what makes `?` work across the boundary.
#[test]
fn a_from_row_generates_a_from_impl() {
  let out = xrs("errors E\n  from std::io::Error as Io \"io: #{0}\"\n");
  assert!(out.contains("    Io(std::io::Error),"), "{out}");
  assert!(out.contains("impl From<std::io::Error> for E {"), "{out}");
  assert!(
    out.contains("fn from(e: std::io::Error) -> Self {"),
    "{out}"
  );
  assert!(out.contains("E::Io(e)"), "{out}");
}

/// A row without `from` gets no `From` impl, which is what makes the keyword
/// mean something.
#[test]
fn a_plain_row_generates_no_from_impl() {
  let out = xrs("errors E\n  Missing String \"missing: #{0}\"\n");
  assert!(!out.contains("impl From<"), "{out}");
}

/// `e?` is Rust's try operator unchanged, and it is what gives the generated
/// `From` impl a caller.
#[test]
fn the_try_operator_passes_through() {
  let out =
    xrs("parse_it : &str -> Result<u32, E>\nparse_it s = Ok n\n  where\n    n = s.parse ()?\n");
  assert!(out.contains("let n = s.parse()?;"), "{out}");
}

/// Criterion 1 of plan 53 — the variants come into scope under their own
/// names, as `data`'s do.
///
/// Before this row, `errors` was the one declaration that introduced
/// constructors and then hid them. `data Event = Click | Close` gave you
/// `Click`; `errors ConfigError` with a `Missing` row gave you
/// `ConfigError::Missing` and nothing shorter, and writing the short form got
/// `E0425: cannot find function, tuple struct or tuple variant` against a
/// line that reads like ordinary Haskell. Neither doc 03 §12.5 nor doc 18 §8
/// asks for the difference — it was an omission.
///
/// The glob sits after the enum and before the `Display` impl, and is a plain
/// `use` rather than `pub use`: `emit_data`'s is plain, and re-exporting the
/// variants from the enclosing module would be a visibility decision nobody
/// made.
#[test]
fn the_variants_come_into_scope_under_their_own_names() {
  let out = xrs("errors E\n  Gone  \"gone\"\n");
  assert!(out.contains("use E::*;\n"), "{out}");
  let glob = out.find("use E::*;").expect("the glob is emitted");
  let display = out
    .find("impl std::fmt::Display")
    .expect("Display is emitted");
  assert!(
    out.find("enum E").expect("the enum is emitted") < glob && glob < display,
    "the glob sits between the enum and its Display impl:\n{out}"
  );
}

/// And the qualified form still works, which is the half that must not be
/// traded away: a program with two error types in scope needs it.
///
/// Asserted on the generated `Display` arms rather than on user code, because
/// those are written fully qualified by the emitter and would break first if
/// the glob had somehow replaced the long form rather than added to it.
#[test]
fn the_display_arms_stay_qualified() {
  let out = xrs("errors E\n  Gone  \"gone\"\n");
  assert!(out.contains("E::Gone =>"), "{out}");
}
