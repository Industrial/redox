use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::class_body::class_body;
use crate::parse::parser::constraints::constraints;
use crate::parse::parser::method_context::MethodContext;
use crate::parse::parser::type_app::type_app;
use redox_ast::{InstanceDecl, TypeApp};
use redox_lexer::{Keyword, TokenKind, Trivia};

/// Parse doc 18 §2.5's `instance` — Rust's `impl`, trait or inherent.
///
/// ```text
/// instance Display t => Describe (Wrapper t)   impl<T: Display> Describe for Wrapper<T> {
///   describe self = "…"                    →       fn describe(&self) -> String { "…" }
///                                              }
///
/// instance Counter                             impl Counter {
///   increment self = self.value += 1       →       fn increment(&mut self) { … }
///                                              }
/// ```
///
/// One keyword covers both of Rust's forms. Which is written is decided by
/// **what follows `instance`**, in three readings that between them cover
/// everything Haskell can write and the one thing it cannot:
///
/// | Source | Reading |
/// |---|---|
/// | `instance Counter` | inherent, on `Counter` |
/// | `instance Show Point` | `Show` for `Point` |
/// | `instance Describe (Wrapper t)` | `Describe` for `Wrapper<T>` |
/// | `instance (Wrapper t)` | inherent, on `Wrapper<T>` |
///
/// The parentheses are grouping, as they are in an expression — they say
/// where the target begins, which is the only thing juxtaposition alone
/// cannot. The last row is the case Haskell has no need for and Rust does:
/// an inherent `impl` on a generic type.
///
/// The class variable is the first type variable of the **target**, which is
/// what makes a method's `self` derivable here as it is in a `class`. An
/// inherent instance on a concrete type has none, and its methods name `Self`
/// instead — as §2.5's own `instance Counter` example does.
pub(crate) fn instance_decl(
  p: &mut Parser,
  leading: Vec<Trivia>,
  start: usize,
) -> Result<InstanceDecl, ParseError> {
  p.expect(&TokenKind::Kw(Keyword::Instance), "expected `instance`")?;
  let constraints = constraints(p)?;

  // Parentheses on the *first* type mean it is the target and there is no
  // class — nothing else would be worth grouping there.
  let grouped = p.check(&TokenKind::LParen);
  let first = type_app(p)?;

  let (class, target) = if grouped {
    (None, first)
  } else if p.check(&TokenKind::LParen) {
    (Some(first.head.clone()), type_app(p)?)
  } else if first.args.is_empty() {
    (None, first)
  } else {
    // `instance Show Point` — juxtaposition put the target in the argument
    // list, so the head is the class and the first argument is the target.
    let mut args = first.args.clone();
    let head = args.remove(0);
    (
      Some(first.head.clone()),
      TypeApp {
        head,
        args,
        span: first.span,
      },
    )
  };

  let var = target.vars().first().map(|v| v.to_string());
  // Inside an `instance` the class variable is the impl's generic parameter,
  // so it is that parameter's uppercase name — doc 18 §5.3.
  let subject = var.as_deref().map_or_else(String::new, TypeApp::rust_name);
  let ctx = MethodContext::for_instance(var.as_deref(), &target.head.text, &subject);
  let members = class_body(p, &ctx)?;

  Ok(InstanceDecl {
    class,
    target,
    constraints,
    members,
    span: (start, p.prev_span().1),
    leading,
  })
}
