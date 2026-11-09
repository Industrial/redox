use crate::ParseError;
use crate::parse::parser::Parser;
use crate::parse::parser::constructor_path::constructor_path;
use crate::parse::parser::pattern::pattern;
use crate::parse::parser::starts_a_pattern_argument::starts_a_pattern_argument;
use redox_ast::Pattern;

/// A constructor applied to sub-patterns, or a single pattern — plan 22.
///
/// ```text
/// Some v          Some(v)
/// Ok raw      →   Ok(raw)
/// Move x _        Move(x, _)
/// Close           Close          -- nullary: a `Binding`, not an `Apply`
/// ```
///
/// Applied by **juxtaposition**, like every other application in Redox: doc 18
/// §2.1 made that the call form, doc 18 §2.4's own `data` example says *"a
/// constructor is applied by juxtaposition, like any other function"*, and a
/// pattern is where that sentence finally has a second meaning.
///
/// # Why this is greedy, and where it therefore may not be used
///
/// Juxtaposition in a pattern is ambiguous wherever a *row* of patterns is
/// read: in `f Some v = …`, the row could be `f (Some v)` or the three
/// patterns `f`, `Some`, `v`. Haskell resolves it by requiring parentheses in
/// a row, and so does Redox — so this function is called from the positions
/// that hold **exactly one** pattern (a `case` arm, a condition, a `guard`),
/// and an equation's row calls [`pattern`] which reads one atom. `(Some v)`
/// in a row works, because [`pattern`] reads a parenthesised group.
///
/// # Why an uppercase head decides it
///
/// Doc 18 §5.3 makes case meaningful — constructors uppercase, values
/// lowercase — so `Some v` applies and `f v` would not be a pattern at all.
/// Deciding it from the source text alone is doc 03 §2's first admission test,
/// and the reason no name resolution is involved.
pub(crate) fn pattern_app(p: &mut Parser) -> Result<Pattern, ParseError> {
  let start = p.peek_span().0;

  // Only an uppercase identifier can head an application. Anything else is a
  // single pattern, read and returned unchanged — which is the path every
  // pattern took before this plan.
  let Some(path) = constructor_path(p) else {
    return pattern(p);
  };

  let mut args = Vec::new();
  while starts_a_pattern_argument(p) {
    args.push(pattern(p)?);
  }

  // A bare constructor stays a `Binding`, as it was before this plan: the
  // emitter writes the bare path, and doc 18 §2.4 emits `use T::*;` beside the
  // enum so the path resolves. Emitting `Close()` for `Close` would be wrong
  // for a unit variant, so the two cases must stay distinct.
  if args.is_empty() {
    return Ok(Pattern::Binding(redox_ast::Ident {
      text: path,
      span: (start, p.prev_span().1),
    }));
  }

  Ok(Pattern::Apply {
    path,
    args,
    span: (start, p.prev_span().1),
  })
}
