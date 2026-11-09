use crate::emit::chain::chain;
use crate::emit::pad_to::pad_to;
use crate::emit::receiver::receiver;
use crate::emit::rust_indent::RUST_INDENT;
use redox_ast::Expr;

/// The chain for `draws[0..]`, as a receiver and its segments.
///
/// Recursive rather than a loop because each generator's `flat_map` argument
/// is the whole of the chain below it, already assembled and possibly
/// multi-line — which is the one thing a loop over segments cannot carry.
pub(crate) fn nest(
  draws: &[(&str, &Expr)],
  guards: &[String],
  item: &str,
  base: usize,
  nested: bool,
) -> (String, Vec<String>) {
  let ((name, source), rest) = draws
    .split_first()
    .expect("the caller checked that there is at least one generator");
  // `move` is needed exactly when an inner closure captures an outer
  // generator's binder, which is exactly when there is more than one.
  let mv = if nested { "move " } else { "" };

  if rest.is_empty() {
    let last = if guards.is_empty() {
      format!(".map({mv}|{name}| {item})")
    } else {
      format!(
        ".filter_map({mv}|{name}| if {} {{ Some({item}) }} else {{ None }})",
        guards.join(" && ")
      )
    };
    return (
      receiver(source, base),
      vec![".into_iter()".to_string(), last],
    );
  }

  let inner_base = base + RUST_INDENT * 2;
  let (inner_head, inner_segments) = nest(rest, guards, item, inner_base, nested);
  let inner = chain(&inner_head, &inner_segments, inner_base);

  // rustfmt braces a closure body that spans lines, and leaves a one-line
  // body bare.
  let arg = if inner.contains('\n') {
    let mut text = format!("move |{name}| {{\n");
    pad_to(&mut text, inner_base);
    text.push_str(&inner);
    text.push('\n');
    pad_to(&mut text, base + RUST_INDENT);
    text.push('}');
    text
  } else {
    format!("move |{name}| {inner}")
  };

  (
    receiver(source, base),
    vec![".into_iter()".to_string(), format!(".flat_map({arg})")],
  )
}
