use crate::lexer::dedent_block::dedent_block;

/// A `"""` block's body: delimiter lines dropped, common indentation removed.
///
/// Doc 01 §7.2's rule, factored out of `scan_string` by plan 45 so the escape
/// hatch can ask for the same body *undecoded*. One implementation rather
/// than two, because two would be free to disagree about where a block's body
/// begins.
pub(crate) fn dedent_triple_body(body: &str) -> String {
  let inner = body.strip_prefix('\n').unwrap_or(body);
  let inner = inner.trim_end_matches(' ');
  dedent_block(inner.strip_suffix('\n').unwrap_or(inner))
}
