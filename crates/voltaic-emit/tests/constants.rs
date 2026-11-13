//! Doc 03 §13.4's module-scope bindings and doc 18 §2.2's `const fn` —
//! plan 16.
//!
//! §13.4 corrects doc 01 §4.3 twice: Rust forbids `let` at module scope, so
//! every top-level binding is `const` or `static`; and both require an
//! explicit type, so §4.3's `PI = 3.14159` *"cannot be lowered at all"*.

use voltaic_emit::emit;
use voltaic_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

#[test]
fn a_const_takes_its_type_from_the_signature_colons() {
  assert_eq!(
    xrs("const MAX : usize = 1024\n"),
    "const MAX: usize = 1024;\n"
  );
}

/// One keyword apart, which is why one node carries both.
#[test]
fn a_static_differs_by_one_keyword() {
  assert_eq!(xrs("static COUNT : u32 = 0\n"), "static COUNT: u32 = 0;\n");
}

#[test]
fn every_visibility_scope_applies_to_a_binding() {
  for (marker, want) in [
    ("export", "pub const LIMIT: usize = 8;"),
    ("export(crate)", "pub(crate) const LIMIT: usize = 8;"),
    ("export(super)", "pub(super) const LIMIT: usize = 8;"),
    (
      "export(in crate::deep)",
      "pub(in crate::deep) const LIMIT: usize = 8;",
    ),
  ] {
    let out = xrs(&format!("{marker} const LIMIT : usize = 8\n"));
    assert_eq!(out, format!("{want}\n"), "{marker}");
  }
}

/// §13.4: *"the uppercase-name convention is a lint, not semantics"*.
///
/// Asserted here and **not** in the example corpus, which requires rustc to
/// emit nothing: a lowercase `const` is legal Voltaic and legal Rust, and
/// rustc *warns* about it. So the corpus cannot demonstrate anything whose
/// correct behaviour is a warning, and this is where that belongs.
#[test]
fn a_lowercase_name_is_legal() {
  assert_eq!(
    xrs("const threshold : u32 = 2\n"),
    "const threshold: u32 = 2;\n"
  );
}

/// Doc 18 §2.2's `const fn`: a marker between `::` and the spine.
#[test]
fn a_const_marker_makes_a_const_fn() {
  let out = xrs("square : const u32 -> u32\nsquare n = n * n\n");
  assert!(out.contains("const fn square(n: u32) -> u32 {"), "{out}");
}

/// **No parameter type contains `const`**, which is the whole reason the
/// marker is consumed before `parse_spine`: a type is verbatim text (doc 03
/// §14.1), and `const u32` is not a Rust type. It *was* one until plan 16.
#[test]
fn the_const_marker_never_reaches_a_type() {
  let out = xrs("square : const u32 -> u32\nsquare n = n * n\n");
  assert!(!out.contains("const u32"), "{out}");
  assert!(!out.contains("n: const"), "{out}");
}

/// It composes with a visibility marker and with a constraint clause, because
/// it is read between them and the spine.
#[test]
fn a_const_fn_composes_with_the_rest_of_a_signature() {
  let out = xrs("export square : const u32 -> u32\nsquare n = n * n\n");
  assert!(out.contains("pub const fn square("), "{out}");
}

/// A module-scope binding must be initialised. A class's associated constant
/// may be declared without a value and supplied by an instance; this has
/// nowhere else for one to come from.
#[test]
fn a_binding_without_a_value_is_rejected() {
  assert!(parse_source("const MAX : usize\n").is_err());
}

/// And it must have a type, which is the second of §13.4's two corrections.
#[test]
fn a_binding_without_a_type_is_rejected() {
  assert!(parse_source("const MAX = 1024\n").is_err());
}

/// Doc 01 §4.3's own spelling gets its own message. The author who writes it
/// followed a published specification, and `expected \`::\`` does not say
/// what replaced it.
#[test]
fn doc_01_s_spelling_names_the_correction() {
  let failure = parse_source("PI = 3.14159\n").expect_err("should fail");
  let rendered = format!("{failure:?}");
  assert!(rendered.contains("ModuleScopeBinding"), "{rendered}");
  assert!(rendered.contains("const"), "{rendered}");
}
