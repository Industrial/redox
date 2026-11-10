use crate::emit::chain::chain;
use crate::emit::expr::emit_expr;
use crate::emit::nest::nest;
use crate::emit::unparen::unparen;
use redox_ast::{Expr, Qualifier};

/// Lower doc 18 §3.2's comprehension to an iterator chain.
///
/// ```text
/// [ x * y | x in 1..4, y in 1..4, x != y ]
/// ```
/// ```text
/// (1..4)
///     .into_iter()
///     .flat_map(move |x| {
///         (1..4)
///             .into_iter()
///             .filter_map(move |y| if x != y { Some(x * y) } else { None })
///     })
///     .collect()
/// ```
///
/// Built inside out: the last generator carries the element, and each earlier
/// one wraps what follows in a `flat_map`. The line breaking is rustfmt's own
/// — see `chain` — because plan 04's guarantee is that the emitter writes
/// nothing rustfmt would rewrite, and a comprehension's chain is long by
/// construction.
///
/// **Every guard is applied at the innermost level**, which is a deliberate
/// deviation from §3.2's shown lowering and the thing that makes this
/// type-free. The doc filters at the guard's own position, which puts the
/// guard inside a `filter` — whose closure receives a *reference*, so §3.2's
/// own example has to write `x != *y`. Redox cannot know when to insert that
/// `*`: it has no types, and doc 03 §2's admission test 3 requires the
/// lowering to preserve Rust semantics without them. `filter_map` hands the
/// closure the value, so the guard is written the way the author wrote it.
///
/// Every binder is in scope at the innermost level, so the two placements
/// differ in *when* a guard runs and never in what the comprehension
/// produces. Haskell's own semantics make guard position an efficiency
/// question, not a meaning one.
///
/// **`collect()` carries no turbofish**, so the element type comes from the
/// ascription — `[ c | c in cs ] : HashSet<char>`, which is §3.2's own
/// spelling — or from the use site. Without either, rustc asks for a type
/// annotation, which is the correct error and points at the comprehension.
pub(crate) fn emit_comprehension(
  element: &Expr,
  qualifiers: &[Qualifier],
  indent: usize,
) -> String {
  let draws: Vec<(&str, &Expr)> = qualifiers
    .iter()
    .filter_map(|q| match q {
      Qualifier::Draw { name, source } => Some((name.text.as_str(), source)),
      Qualifier::Guard(_) => None,
    })
    .collect();
  let guards: Vec<String> = qualifiers
    .iter()
    .filter_map(|q| match q {
      Qualifier::Guard(e) => Some(emit_expr(unparen(e), indent)),
      Qualifier::Draw { .. } => None,
    })
    .collect();

  // With no generator there is nothing to iterate, so the brackets are a
  // one-element list. rustc would reject anything else, and this is what the
  // author wrote.
  if draws.is_empty() {
    return format!("[{}]", emit_expr(unparen(element), indent));
  }

  let item = emit_expr(unparen(element), indent);
  let nested = draws.len() > 1;
  let (head, mut segments) = nest(&draws, &guards, &item, indent, nested);
  segments.push(".collect()".to_string());
  chain(&head, &segments, indent)
}
