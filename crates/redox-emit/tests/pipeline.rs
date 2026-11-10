//! Doc 18 §4's receiver pipeline.
//!
//! `a |> f` becomes `a.f()`. It was parsed from plan 03 onward and emitted a
//! `compile_error!` placeholder until now — an honest hole rather than
//! something that merely looked plausible, which is why this could sit
//! unimplemented without anyone shipping wrong code against it.

use redox_emit::emit;
use redox_parser::parse_source;

fn body(src: &str) -> String {
  // The trailing `()` is plan 55 §1.6's unit tail, which `main` now needs.
  emit(&parse_source(&format!("main : ()\nmain =\n  {src}\n  ()\n")).expect("should parse"))
}

/// A bare name on the right is a no-argument method call. This is the case
/// that makes a pipeline read like a chain.
#[test]
fn a_bare_name_is_a_method_call() {
  assert!(
    body("r = xs |> iter").contains("let r = xs.iter();"),
    "{}",
    body("r = xs |> iter")
  );
}

#[test]
fn a_chain_nests_left_to_right() {
  let out = body("r = numbers |> iter |> map f |> sum");
  assert!(
    out.contains("let r = numbers.iter().map(f).sum();"),
    "{out}"
  );
}

/// An application on the right contributes its head as the method name and
/// its arguments as the call's — one method call, not a call of a call.
#[test]
fn an_application_becomes_one_method_call() {
  let out = body("r = xs |> fold 0 f");
  assert!(out.contains("xs.fold(0, f)"), "{out}");
  assert!(!out.contains("fold(0, f)("), "not a call of a call:\n{out}");
}

/// A lambda argument survives, which is the shape every doc 18 §4 example
/// uses.
#[test]
fn a_lambda_argument_survives() {
  let out = body("r = xs |> map (\\n -> n * n)");
  assert!(out.contains("xs.map(|n| n * n)"), "{out}");
}

/// **No operator emits the placeholder any more.**
///
/// It was the honest hole for `|>` and then for `??`, and both now lower, so
/// this asserts the emitter makes no such claim about either. The arm itself
/// stays in `emit_expr`: a *new* operator with no lowering must fail loudly
/// in the generated file rather than quietly produce something plausible,
/// which is what the placeholder is for.
#[test]
fn no_operator_emits_the_placeholder() {
  for src in ["r = xs |> iter", "r = a ?? b"] {
    assert!(
      !body(src).contains("compile_error!"),
      "`{src}` is lowered now:\n{}",
      body(src)
    );
  }
}

/// Doc 18 §4 explains why the spelling is `|>` and not Haskell's: composition
/// is `.`, which in Redox is field access. So a field access after a pipeline
/// must still be a field access.
#[test]
fn a_field_access_is_not_a_pipeline_stage() {
  let out = body("r = u.name");
  assert!(out.contains("let r = u.name;"), "{out}");
}
