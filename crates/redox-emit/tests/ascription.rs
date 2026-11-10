//! Doc 18 §6's ascription, with plan 52's one-token marker.
//!
//! `e : T` lowers to a block, because Rust has no expression-level
//! ascription and a `let` is the only way to state a type without naming a
//! path:
//!
//! ```text
//! n = 1 : u64         let n = {
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
//! Doc 18 §5.2 used to tell an ascription from a path by its *whitespace* — a
//! spaced `::` ascribed, a tight one was a path — which made this the only
//! place in Redox where a stray space changed meaning, and which doc 18 §10.1
//! item 3 named as the thing to watch. Plan 52 retired the rule by giving
//! ascription its own token: `:` ascribes, `::` joins a path. These tests are
//! what watch that the two never cross.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

fn body(src: &str) -> String {
  // The trailing `()` is plan 55 §1.6's unit tail, which `main` now needs.
  xrs(&format!("main : ()\nmain =\n  {src}\n  ()\n"))
}

#[test]
fn a_colon_ascribes() {
  let out = body("n = 1 : u64");
  assert!(
    out.contains("let n = {\n        let __rdx_t: u64 = 1;\n        __rdx_t\n    };"),
    "{out}"
  );
}

/// The other half of plan 52, and the one that would break every existing
/// program if it went wrong.
#[test]
fn a_colon_colon_is_still_a_path() {
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
  let out = body("total = 1 + 2 : u64");
  assert!(
    out.contains("let __rdx_t: u64 = 1 + 2;"),
    "the sum is ascribed, not the `2`:\n{out}"
  );
}

/// A generic type survives verbatim — doc 03 §14.1's guarantee, which doc 18
/// §8 declines type-application-by-juxtaposition specifically to keep.
#[test]
fn a_generic_type_survives_verbatim() {
  let out = body("set = xs : HashSet<i32>");
  assert!(out.contains("let __rdx_t: HashSet<i32> = xs;"), "{out}");
}

/// A path *inside* an ascribed type must not end the type — doc 03 §14.1's
/// verbatim capture is what carries it through.
#[test]
fn a_path_inside_an_ascribed_type_is_kept() {
  let out = body("e = x : fmt::Result");
  assert!(out.contains("let __rdx_t: fmt::Result = x;"), "{out}");
}

/// `::` no longer ascribes, however it is spaced — plan 52 criterion 2.
///
/// With a literal on the left there is nothing for `::` to join, so the line
/// is **refused** rather than reinterpreted. That is the outcome the criterion
/// asks for. The remaining gap — `x :: u64`, where `::` does join two names
/// and so is still consumed as a path — is asserted in
/// `redox-parser/tests/parse.rs`, because it is a parse question rather than
/// an emit one.
#[test]
fn a_colon_colon_no_longer_ascribes() {
  let failure = parse_source("main : ()\nmain =\n  n = 1 :: u64\n")
    .expect_err("`::` joins a path, and `1` is not a path segment");
  let message = format!("{failure:?}");
  assert!(
    message.contains("expected"),
    "it should be a parse error, not a misreading: {message}"
  );
}
