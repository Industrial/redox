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
/// **what follows `instance`**, in readings that between them cover
/// everything Haskell can write and the one thing it cannot:
///
/// | Source | Reading |
/// |---|---|
/// | `instance Counter` | inherent, on `Counter` |
/// | `instance Show Point` | `Show` for `Point` |
/// | `instance Convert f64 Meters` | `Convert<f64>` for `Meters` |
/// | `instance Describe (Wrapper t)` | `Describe` for `Wrapper<T>` |
/// | `instance (Wrapper t)` | inherent, on `Wrapper<T>` |
///
/// **The last name is the implementing type and the rest are the class's own
/// arguments** — plan 67 §9 decision 3, chosen so
/// `instance Convert f64 Meters` and `impl Convert<f64> for Meters` read in
/// the same order. That reverses the previous split, which took the *first*
/// argument as the target and so put the implementing type between the trait
/// and its arguments; a two-name head reads the same under both, which is why
/// nothing in the corpus observed the change.
///
/// The parentheses are grouping, as they are in an expression — they say
/// where the target begins, which is the only thing juxtaposition alone
/// cannot. The last row is the case Haskell has no need for and Rust does:
/// an inherent `impl` on a generic type.
///
/// The impl's own generic parameter is the first type variable of the
/// **target**. An inherent instance on a concrete type has none.
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
    // `instance Describe (Wrapper t)`, and `instance Convert f64 (Wrapper t)`
    // — the whole juxtaposition before the group is the class.
    (Some(first), type_app(p)?)
  } else if first.args.is_empty() {
    (None, first)
  } else {
    // `instance Show Point` and `instance Convert f64 Meters` — juxtaposition
    // put the target in the argument list, and it is the **last** one.
    let mut args = first.args.clone();
    let head = args.pop().expect("the argument list is not empty");
    let class_end = args.last().map_or(first.head.span.1, |a| a.span.1);
    (
      Some(TypeApp {
        head: first.head.clone(),
        args,
        span: (first.head.span.0, class_end),
      }),
      TypeApp {
        head: head.clone(),
        args: Vec::new(),
        span: head.span,
      },
    )
  };

  let var = target.vars().first().map(|v| v.to_string());
  // Inside an `instance` a target's type variable is the impl's generic
  // parameter, so it is that parameter's uppercase name — doc 18 §5.3.
  let subject = var.as_deref().map_or_else(String::new, TypeApp::rust_name);
  let mut ctx = MethodContext::for_instance(var.as_deref(), &subject);
  // Plan 63 leaf 1: the class this instance implements is where an elided
  // member's signature comes from. `None` for an inherent `instance`, which
  // has no class and so cannot elide — the diagnostic in `class_member` says
  // exactly that.
  ctx.implementing = class.as_ref().map(|c| c.head.text.clone());
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
