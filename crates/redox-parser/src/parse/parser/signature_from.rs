use crate::ParseError;
use redox_ast::{Param, Pattern, Signature, TypeExpr};
use redox_diag::{Code, Span};

/// Build a [`Signature`] from a signature line's spine and an equation's
/// binders.
///
/// Doc 18 §2.2. The spine's last type is the return type and the rest are the
/// parameter types, matched positionally against the binders — so
/// `load :: &str -> Result<Config, Error>` plus `load p =` gives
/// `fn load(p: &str) -> Result<Config, Error>`.
///
/// **A unit return type is omitted**, not emitted as `-> ()`. `main :: ()`
/// becomes `fn main()`, which is what Rust writes and what rustfmt would
/// produce; emitting the arrow would be byte-stable noise that plan 04's
/// rustfmt-clean guarantee would then have to defend.
///
/// An arity mismatch is reported here, with both counts, because it is the
/// one thing the two-line form can get wrong that the one-line form could
/// not. Doc 18 §2.3 notes that hoisting the signature reduces `RDX0045` from
/// "the clauses disagree" to exactly this.
pub(crate) fn signature_from(
  spine: &[TypeExpr],
  bound: &[Pattern],
  span: Span,
) -> Result<Signature, ParseError> {
  let (ret, params) = spine
    .split_last()
    .expect("parse_spine returns at least one type");

  if params.len() != bound.len() {
    let arrows = params.len();
    let names = bound.len();
    let detail = if names < arrows {
      format!("the signature takes {arrows}, but this equation binds {names}")
    } else {
      format!("this equation binds {names}, but the signature takes {arrows}")
    };
    return Err(ParseError::with(
      Code::ExpectedName,
      format!("the signature and the equation disagree about arity: {detail}"),
      span,
    ));
  }

  Ok(Signature {
    params: bound
      .iter()
      .zip(params)
      .enumerate()
      .map(|(i, (pattern, ty))| Param {
        // A pattern that binds lends its name to the parameter; one that
        // *tests* cannot, so the parameter gets a generated name and the
        // pattern moves into the `match` the emitter builds. That is doc 18
        // §2.3's `__rdx_a0`, and the reason the generated-identifier prefix
        // exists.
        name: match pattern {
          Pattern::Binding(ident) => ident.clone(),
          _ => redox_ast::Ident {
            text: format!("__rdx_a{i}"),
            span: pattern.span(),
          },
        },
        ty: Some(ty.clone()),
        is_mut: false,
        span: pattern.span(),
      })
      .collect(),
    ret: (ret.text != "()").then(|| ret.clone()),
    generics: Vec::new(),
    is_const: false,
    bounds: Vec::new(),
    span,
  })
}

#[cfg(test)]
mod tests {
  use super::signature_from;
  use redox_ast::{Ident, Pattern, TypeExpr};

  fn ty(text: &str) -> TypeExpr {
    TypeExpr {
      text: text.to_string(),
      span: (0, 0),
    }
  }

  /// A binding pattern, which is what an ordinary parameter name is.
  fn id(text: &str) -> Pattern {
    Pattern::Binding(Ident {
      text: text.to_string(),
      span: (0, 0),
    })
  }

  #[test]
  fn pairs_the_spine_with_the_binders() {
    let sig = signature_from(
      &[ty("i32"), ty("i32"), ty("i32")],
      &[id("a"), id("b")],
      (0, 0),
    )
    .expect("should build");
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.params[0].name.text, "a");
    assert_eq!(sig.params[0].ty.as_ref().expect("a type").text, "i32");
    assert_eq!(sig.ret.expect("a return type").text, "i32");
  }

  /// `main :: ()` is `fn main()`, not `fn main() -> ()`.
  #[test]
  fn a_unit_return_is_omitted() {
    let sig = signature_from(&[ty("()")], &[], (0, 0)).expect("should build");
    assert!(sig.params.is_empty());
    assert_eq!(sig.ret, None);
  }

  #[test]
  fn too_few_binders_is_an_error_naming_both_counts() {
    let err = signature_from(&[ty("i32"), ty("i32"), ty("i32")], &[id("a")], (0, 0))
      .expect_err("should fail");
    assert!(err.message.contains("arity"), "{}", err.message);
    assert!(err.message.contains("takes 2"), "{}", err.message);
    assert!(err.message.contains("binds 1"), "{}", err.message);
  }

  #[test]
  fn too_many_binders_is_also_an_error() {
    let err = signature_from(&[ty("i32")], &[id("a")], (0, 0)).expect_err("should fail");
    assert!(err.message.contains("binds 1"), "{}", err.message);
    assert!(err.message.contains("takes 0"), "{}", err.message);
  }

  /// A non-unit zero-parameter function keeps its return type.
  #[test]
  fn a_zero_parameter_function_can_still_return_something() {
    let sig = signature_from(&[ty("u32")], &[], (0, 0)).expect("should build");
    assert!(sig.params.is_empty());
    assert_eq!(sig.ret.expect("a return type").text, "u32");
  }
}
