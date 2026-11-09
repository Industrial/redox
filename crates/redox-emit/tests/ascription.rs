//! Doc 18 §6's ascription, and §5.2's spaced-versus-tight `::` rule.
//!
//! `e :: T` lowers to a block, because Rust has no expression-level
//! ascription and a `let` is the only way to state a type without naming a
//! path:
//!
//! ```text
//! n = 1 :: u64        let n = {
//!                         let __rdx_t: u64 = 1;
//!                 →       __rdx_t
//!                     };
//! ```
//!
//! Three lines rather than one. The one-line form was what the lowering
//! emitted until the indent threading landed, and rustfmt rewrites it — so
//! the lowering had been breaking plan 04's guarantee from the day it shipped,
//! unnoticed because no example used ascription.
//!
//! The rule that tells it from a path is the **only** place in Redox where a
//! stray space changes meaning, which doc 18 §10.1 item 3 names as the thing
//! to watch. These tests are that watch.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

fn body(src: &str) -> String {
  xrs(&format!("main :: ()\nmain =\n  {src}\n"))
}

#[test]
fn a_spaced_colon_colon_ascribes() {
  let out = body("n = 1 :: u64");
  assert!(
    out.contains("let n = {\n        let __rdx_t: u64 = 1;\n        __rdx_t\n    };"),
    "{out}"
  );
}

/// The other half of §5.2, and the one that would break every existing
/// program if it went wrong.
#[test]
fn a_tight_colon_colon_is_still_a_path() {
  let out = body("kind = Error::Malformed");
  assert!(out.contains("let kind = Error::Malformed;"), "{out}");
  assert!(
    !out.contains("__rdx_t"),
    "a path must not be ascribed:\n{out}"
  );
}

/// Ascription is the loosest thing in an expression, so it takes the whole
/// sum rather than the last operand.
#[test]
fn ascription_binds_looser_than_every_operator() {
  let out = body("total = 1 + 2 :: u64");
  assert!(
    out.contains("let __rdx_t: u64 = 1 + 2;"),
    "the sum is ascribed, not the `2`:\n{out}"
  );
}

/// A generic type survives verbatim — doc 03 §14.1's guarantee, which doc 18
/// §8 declines type-application-by-juxtaposition specifically to keep.
#[test]
fn a_generic_type_survives_verbatim() {
  let out = body("set = xs :: HashSet<i32>");
  assert!(out.contains("let __rdx_t: HashSet<i32> = xs;"), "{out}");
}

/// A path *inside* an ascribed type is tight, and must not end the type.
#[test]
fn a_path_inside_an_ascribed_type_is_kept() {
  let out = body("e = x :: fmt::Result");
  assert!(out.contains("let __rdx_t: fmt::Result = x;"), "{out}");
}

/// Asymmetric spacing is neither form, and is **refused** rather than
/// reinterpreted.
///
/// `1 ::u64` is not an ascription by §5.2's rule and not a path either, so it
/// fails to parse. That is the right outcome for the one whitespace-sensitive
/// rule in the language: a reader who gets the spacing wrong is told, rather
/// than handed a program that means something else. Doc 18 §10.1 item 3 asks
/// for the diagnostic to name the spacing explicitly — it does not yet, and
/// that is recorded there rather than asserted here.
#[test]
fn asymmetric_spacing_is_refused() {
  let failure = parse_source("main :: ()\nmain =\n  n = 1 ::u64\n")
    .expect_err("`::u64` is neither an ascription nor a path");
  let message = format!("{failure:?}");
  assert!(
    message.contains("expected"),
    "it should be a parse error, not a misreading: {message}"
  );
}
