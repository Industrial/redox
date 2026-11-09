use crate::emit::chain_width::CHAIN_WIDTH;
use crate::emit::pad_to::pad_to;
use crate::emit::rust_indent::RUST_INDENT;

/// Assemble a method chain the way rustfmt writes it.
///
/// ```text
/// xs.into_iter().map(|n| n * 2).collect()        segments total 36 — one line
///
/// xs                                             segments total 76 — broken
///     .into_iter()
///     .filter_map(|n| if n % 2 == 1 { Some(n) } else { None })
///     .collect()
/// ```
///
/// The decision is rustfmt's `chain_width`, measured on the **segments** —
/// everything after the receiver. That is what makes it computable here: the
/// emitter knows the segments it is about to write without knowing what
/// column the line has already reached, which is the information a
/// line-width rule would need and the emitter does not have.
///
/// A segment that is already multi-line forces the break too, because
/// rustfmt will not leave a line break inside a joined chain.
///
/// `base` is the indent of the statement the chain belongs to; broken
/// segments sit one level in from it, which is where rustfmt puts them.
pub(crate) fn chain(receiver: &str, segments: &[String], base: usize) -> String {
  let joined: String = segments.concat();
  if joined.len() <= CHAIN_WIDTH && !joined.contains('\n') {
    return format!("{receiver}{joined}");
  }

  let mut out = receiver.to_string();
  for segment in segments {
    out.push('\n');
    pad_to(&mut out, base + RUST_INDENT);
    out.push_str(segment);
  }
  out
}
