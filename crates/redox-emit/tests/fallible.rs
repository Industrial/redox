//! Doc 03 §12's `catch`, `?.` and `try` blocks — plan 28.
//!
//! All three lower by handing the work to rustc, and two by the same
//! mechanism: `?` inside a closure that returns from itself rather than from
//! the enclosing function. Doc 18 §8 keeps all three despite having no
//! Haskell analogue.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented.
fn body(src: &str) -> String {
  // Plan 55 §1.6 requires a `()`-returning body to end in `()`, so the wrapper
  // writes one and the `();` it emits is dropped again — these assertions are
  // about `src`.
  let out = xrs(&format!("main : ()\nmain =\n{src}  ()\n"));
  let mut lines: Vec<&str> = out
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect();
  assert_eq!(lines.pop(), Some("();"), "{out}");
  lines.join("\n")
}

// --- §5.1 to §5.4: `catch` -------------------------------------------------

/// Doc 03 §12.2's lowering, character for character. A `match` and not
/// `unwrap_or_else`, because that method's closure arity differs between
/// `Option` and `Result` — so a method lowering would be type-dependent and
/// fail doc 03 §2's first admission test.
#[test]
fn catch_emits_a_match_on_result() {
  assert_eq!(
    body("  n = parse s catch 0\n"),
    "let n = match parse(s) {\n    Ok(__rdx_v) => __rdx_v,\n    Err(_) => 0,\n};"
  );
}

/// The fallback may bind the error, and the binder comes from doc 18 §4's
/// lambda rather than from a second syntax.
#[test]
fn a_lambda_fallback_binds_the_error() {
  assert_eq!(
    body("  n = parse s catch (\\e -> e)\n"),
    "let n = match parse(s) {\n    Ok(__rdx_v) => __rdx_v,\n    Err(e) => e,\n};"
  );
}

/// A lambda of any other arity stays the arm's **value** — a closure where a
/// value was wanted, which rustc reports. The alternative is Redox deciding
/// what the author meant.
#[test]
fn a_two_parameter_lambda_is_not_unwrapped() {
  let out = body("  n = parse s catch (\\a b -> a)\n");
  assert!(out.contains("Err(_) => |a, b| a,"), "{out}");
}

/// **§1.4, verified rather than assumed.** Doc 03 §12.2 says `catch`
/// terminates a parenless argument list by §5.1.1's terminator set — and that
/// set dissolved with plan 19. The property survives because `catch` is a
/// keyword and `starts_argument`'s allow-list holds no keyword but `true` and
/// `false`. Two tokens have already had to leave `continues_line` for this
/// class of reason, so it is checked.
#[test]
fn catch_needs_no_parentheses_around_a_parenless_call() {
  let out = body("  n = parse s t catch 0\n");
  assert!(out.starts_with("let n = match parse(s, t) {"), "{out}");
}

/// Doc 03 §4.5 places `catch` at `??`'s level, so the two group left to right
/// at equal precedence rather than one swallowing the other.
#[test]
fn catch_sits_at_coalesces_precedence() {
  let out = body("  n = a catch b ?? c\n");
  // `(a catch b) ?? c` — the `match` on `a` is the `??`'s subject.
  assert!(out.starts_with("let n = match match a {"), "{out}");
}

// --- §5.5 to §5.7: the `?.` chain, retired by plan 57 ---------------------

/// `?.` is **retired** — `RDX0117`, plan 57 leaf 4.
///
/// The token is gone from the lexer, so refusing it is an *adjacency* check
/// and not a branch. That is the only thing standing between the deletion and
/// a silent reinterpretation: without it, `user?.profile` becomes `user?`
/// followed by `.profile`, which is valid Rust meaning something else.
#[test]
fn an_optional_chain_is_retired() {
  let refused = parse_source("main : ()\nmain =\n  n = user?.profile?.name\n  ()\n")
    .expect_err("`?.` is retired");
  assert!(
    format!("{refused:?}").contains("OptionalChaining"),
    "{refused:?}"
  );
}

/// The replacement, and it is the **lowering** rather than a workaround: doc
/// 02 §6.3 always put a `?` run inside a closure returning `Option`, and a
/// `try` block *is* that closure. §5 criterion 6.
///
/// The `&`s are the whole cost of the deletion, and §11.4 is why they cannot
/// be dropped: a field access **moves**, so `map (.tag)` over a borrow is
/// `E0507`. Written at the front, where a reader sees them — which is the
/// property that chose this spelling over two lambdas and an `and_then`.
#[test]
fn a_try_over_a_question_run_is_the_replacement() {
  assert_eq!(
    body("  n = try\n    p = &(.as_ref user)? .profile\n    Some (&(.as_ref p)? .name)\n"),
    "let n = (|| {\n    let p = &user.as_ref()?.profile;\n    Some(&p.as_ref()?.name)\n})();"
  );
}

/// **The correction to doc 03 §12.3's published lowering, re-sited.** §12.3
/// writes `.as_ref()` on every link including the last, which does not
/// compile when the final field is not an `Option` — and is *type-dependent*,
/// meaning `Option::as_ref` on one and `AsRef::as_ref` on the other. Plan 28
/// §10 made the final link take no method and the whole path a leading `&`;
/// plan 57 makes both the author's, so the property is now that the emitter
/// adds **neither**.
#[test]
fn the_final_link_takes_no_method() {
  let out = body("  n = try\n    Some (&(.as_ref user)? .name)\n");
  assert!(!out.contains("name.as_ref()"), "{out}");
  assert!(out.contains("Some(&user.as_ref()?.name)"), "{out}");
}

/// The head is parenthesised exactly where `.` would bind tighter, by the
/// same receiver rule every other postfix chain uses.
#[test]
fn a_chains_head_is_parenthesised_when_it_must_be() {
  let out = body("  n = try\n    Some (&(.as_ref (f x))? .name)\n");
  assert!(out.contains("Some(&f(x).as_ref()?.name)"), "{out}");
}

// --- §5.8: `try` -----------------------------------------------------------

/// Doc 03 §12.4's immediately-invoked closure. The difference from plan 20's
/// `do` is exactly the `(|| … )()` wrapper — a `try` lowered to a bare block
/// compiles, and its `?` then returns from the enclosing function.
#[test]
fn try_emits_an_immediately_invoked_closure() {
  assert_eq!(
    body("  r = try\n    a = (parse x)?\n    Ok a\n"),
    "let r = (|| {\n    let a = parse(x)?;\n    Ok(a)\n})();"
  );
}

/// A `try` whose body is one expression works too — the block is a statement
/// list like any other.
#[test]
fn a_one_statement_try_works() {
  assert_eq!(
    body("  r = try\n    Ok 1\n"),
    "let r = (|| {\n    Ok(1)\n})();"
  );
}

// --- §5.9: plan 15's token is untouched -----------------------------------

/// `?.` was **one token**, which plan 15 §10.1 records; plan 57 deleted it,
/// and this is the test that the deletion did not quietly reinterpret the
/// spelling instead.
///
/// The two halves are the whole rule, and the second is what keeps the first
/// from being a hollow refusal: `?` and a field *are* two postfix steps, so a
/// check that refused the spaced form too would have deleted a construct
/// nobody asked to delete.
#[test]
fn question_dot_is_refused_and_a_spaced_one_is_not() {
  let refused = parse_source("main : ()\nmain =\n  n = user?.profile\n  ()\n")
    .expect_err("the glued spelling is `RDX0117`");
  assert!(
    format!("{refused:?}").contains("OptionalChaining"),
    "{refused:?}"
  );
  // Separated by a space, so these are two postfixes: `?` then a field.
  assert_eq!(
    body("  n = (parse x)? .inner\n"),
    "let n = parse(x)?.inner;"
  );
}

/// And the plain `?` postfix is unaffected.
#[test]
fn the_try_postfix_still_works() {
  assert_eq!(body("  n = (parse x)?\n"), "let n = parse(x)?;");
}
