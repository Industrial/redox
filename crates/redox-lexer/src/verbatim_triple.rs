pub(crate) mod dedent_triple_body;

use crate::verbatim_triple::dedent_triple_body::dedent_triple_body;

/// The body of a `"""…"""` literal, **undecoded**.
///
/// Plan 45 (`docs/39 - 45 - Reachability Residue.md`) leaf 1. Doc 01 §23.1's
/// inline Rust escape hatch needs the text a user wrote, not the text a string
/// literal means, and the difference is not cosmetic: escape decoding turns
///
/// ```text
/// rust """
///   println!("a\nb");
/// """
/// ```
///
/// into Rust carrying a **real newline inside a string literal**, which does
/// not compile. So the hatch reads the block through here rather than through
/// [`crate::TokenKind::Str`]'s decoded parts.
///
/// `scan_string` calls this too, so there is one implementation of what a
/// `"""` block's body is rather than two free to disagree. It takes the
/// literal's `raw` text — delimiters included, exactly as
/// [`crate::TokenKind::Str::raw`] stores it — and returns `None` for anything
/// that is not a triple-quoted literal.
pub fn verbatim_triple(raw: &str) -> Option<String> {
  let body = raw.strip_prefix(r#"""""#)?.strip_suffix(r#"""""#)?;
  Some(dedent_triple_body(body))
}
