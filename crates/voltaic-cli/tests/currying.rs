//! The claims about plan 56's lowering that a golden states but cannot
//! *assert*.
//!
//! Plan 56 (`docs/50 - 56 - Currying.md`) criteria 1, 2, 5, 6 and 7.
//! `tests/golden/currying.xrs` already pins every byte of
//! `examples/currying.sdx`, so none of this is about whether the text is
//! right — `examples.rs` says that, and `run.rs` says the program prints the
//! right numbers.
//!
//! What a golden cannot say is *which* bytes carry the criterion. Criterion 6
//! asks that an operator section and a one-of-two partial application share
//! one lowering path, and a golden that happens to contain both closures
//! would still be green on the day they stopped matching. Criterion 5's two
//! halves are the same shape: *this* name became a closure and *that* one
//! stayed a call, in the same file, for a reason a reader has to be told.
//!
//! So these are substring assertions about **shape**, which is the one form
//! `no_literal_expectations.rs` explicitly allows (*"a substring assertion
//! about shape is not an expectation-as-data"*) — the expectation itself is a
//! file, and it is the golden.

mod support;

/// The generated Rust for `examples/currying.sdx`, transpiled once per test.
fn generated() -> String {
  support::transpile("currying.sdx").rust.clone()
}

/// Criterion 6: one lowering path, so one closure shape — and **no `move`**
/// on either side.
///
/// §1.3 asked for `move |__rdx_1| add(1, __rdx_1)` and §11.2 measured that
/// sections carry no capture annotation at all, so the two halves of the
/// criterion could not both hold. Sections are the side that does not change:
/// `move` on a closure capturing a borrowed argument turns rustc's borrow
/// error into a move error, and `tests/check/borrowed_capture.sdx` is the
/// fixture that wants the borrow error.
#[test]
fn a_section_and_a_partial_application_write_the_same_closure() {
  let rust = generated();
  assert!(
    rust.contains(".map(|__rdx_1| __rdx_1 + 1)"),
    "the operator section's closure:\n{rust}"
  );
  assert!(
    rust.contains(".map(|__rdx_1| add(1, __rdx_1))"),
    "the partial application's closure, the same shape in the same position:\n{rust}"
  );
  assert!(
    !rust.contains("move |"),
    "neither side carries `move` (§5 criterion 6):\n{rust}"
  );
}

/// Criterion 1's second half, and §11.6: full application is not currying and
/// does not pay for it.
///
/// Three spellings, one emitted call each. `add 1 2` is the saturated call,
/// `add 1 $ 2` is the same thing written loosely — flattened back rather than
/// allocating a closure that is immediately applied — and `two 2 3` is a call
/// of an unknown-arity local, which is what makes the flat closure the right
/// shape rather than a nested one.
#[test]
fn a_saturated_call_is_still_one_call() {
  let rust = generated();
  assert_eq!(
    rust.matches("add(1, 2)").count(),
    2,
    "`add 1 2` and `add 1 $ 2` should both be one call:\n{rust}"
  );
  assert!(
    !rust.contains("add(1, 2)("),
    "a flattened `$` must not leave a closure behind to call:\n{rust}"
  );
}

/// Criterion 2: n-of-m for m ≥ 3 binds **every** parameter still missing.
///
/// The case a single-level implementation passes criterion 1 on and fails
/// here, and the reason `SECTION_PARAM` had to become an indexed generator
/// (§11.2).
#[test]
fn one_of_three_binds_both_missing_parameters() {
  let rust = generated();
  assert!(
    rust.contains("|__rdx_1, __rdx_2| tagged(\"sum\", __rdx_1, __rdx_2)"),
    "two parameters missing, two bound, in one flat list:\n{rust}"
  );
}

/// Criterion 5's second half: an arity that is **not** knowable is emitted as
/// a call exactly as it always was, and rustc judges.
///
/// `op`, `inc` and `two` are all `where` locals, so no signature in the file
/// names any of them — §11.1's finding, and the whole of the row's safety
/// argument. The same `.sdx` holds both halves, which is what "asserted both
/// ways" asks for.
#[test]
fn an_unknowable_arity_stays_a_call() {
  let rust = generated();
  for call in ["op(7, 8)", "inc(5)", "two(2, 3)"] {
    assert!(
      rust.contains(call),
      "`{call}` should be emitted as a call, not curried:\n{rust}"
    );
  }
}

/// Criterion 7: a `Path` callee is passable, never curried.
///
/// Already true at HEAD, which is why this is an assertion rather than an
/// implementation — but it is the one a reader who has just learned that Voltaic
/// functions curry will get wrong, so §6 makes it a criterion.
#[test]
fn a_rust_path_is_passed_and_never_curried() {
  let rust = generated();
  assert!(
    rust.contains(".map(String::from)"),
    "a path in value position is already a `fn` item:\n{rust}"
  );
}
