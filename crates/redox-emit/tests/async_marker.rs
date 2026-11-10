//! Doc 03 §14.3's async — the marker and the block. Plan 39.
//!
//! Await is doc 30's `x <- f`, already shipped. This row supplies the two
//! things that let one be written: `async` on a signature and `async` as a
//! block expression. Doc 03 §14.3 provides no executor, so the running proof
//! is `examples/async.sdx`, which brings its own in nine lines.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented.
fn body(src: &str) -> String {
  xrs(&format!("main : ()\nmain =\n{src}"))
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect::<Vec<_>>()
    .join("\n")
}

// --- §5.1 and §5.2: the marker --------------------------------------------

#[test]
fn the_marker_emits_async_fn() {
  let out = xrs("f : async i32 -> i32\nf n = n\n");
  assert!(out.starts_with("async fn f(n: i32) -> i32 {"), "{out}");
}

/// **Plan 16's defect, asserted here because the mechanism is shared.** A
/// type in Redox is verbatim text, so a marker left inside one would make the
/// first parameter's type `async i32` — which is not a Rust type, and is
/// exactly what happened to `const` before plan 16 consumed it.
#[test]
fn the_marker_does_not_leak_into_the_first_type() {
  let out = xrs("f : async i32 -> i32\nf n = n\n");
  assert!(!out.contains("async i32"), "{out}");
  assert!(!out.contains("ASYNC"), "{out}");
}

/// It composes with everything else a signature carries, because it is
/// consumed before the spine is read and touches nothing after.
#[test]
fn the_marker_composes_with_the_rest_of_a_signature() {
  let out = xrs("export f : Ord t => async Option<t> -> t\nf xs = xs.unwrap ()\n");
  assert!(out.contains("pub async fn f"), "{out}");
  // Doc 18 §5.3 uppercases a lowercase type variable, so the bound reads
  // `T: Ord` — and the marker survives in front of `fn` rather than being
  // swept into the generic list, which is the thing this asserts.
  assert!(out.contains("T: Ord"), "{out}");
}

/// Rust has no `const async fn`, and that is **rustc's** to say: both are
/// written and rustc objects. AGENTS.md's rule, and the alternative would be a
/// check Redox cannot make better than rustc does.
#[test]
fn both_markers_emit_and_rustc_objects() {
  let out = xrs("f : const async i32 -> i32\nf n = n\n");
  assert!(out.starts_with("const async fn f"), "{out}");
}

// --- §5.3 and §5.4: the block ---------------------------------------------

/// An expression, reached from `prefix` as plan 20's `do` and plan 28's `try`
/// are — the third of that shape, and the reason none needed a statement form.
#[test]
fn an_async_block_is_an_expression() {
  assert_eq!(body("  w = async\n    1\n"), "let w = async {\n    1\n};");
}

/// Its statements may include an await, which is the only reason to write one.
#[test]
fn an_async_block_holds_an_await() {
  assert_eq!(
    body("  w = async\n    n <- f ()\n    n\n"),
    "let w = async {\n    let n = f().await;\n    n\n};"
  );
}

/// `try` wraps its block in `(|| … )()` and this prefixes `async`, which is
/// the whole difference — both asserted together so neither can drift into
/// the other's shape.
#[test]
fn an_async_block_is_not_a_try_block() {
  assert_eq!(body("  w = async\n    1\n"), "let w = async {\n    1\n};");
  assert_eq!(body("  w = try\n    1\n"), "let w = (|| {\n    1\n})();");
}

// --- §5.4: an await needs the marker around it ---------------------------

/// A `<-` in a **non**-`async` function emits the `.await` and lets rustc
/// object — doc 03 §14.3's division of labour, and the reason this row had to
/// exist before doc 30's conversion could be proved by running.
#[test]
fn an_await_outside_async_still_emits_and_rustc_objects() {
  let out = xrs("f : i32 -> i32\nf n =\n  m <- g n\n  m\n");
  assert!(out.starts_with("fn f(n: i32) -> i32 {"), "{out}");
  assert!(out.contains("let m = g(n).await;"), "{out}");
}
