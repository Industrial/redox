use crate::item_text::visibility_text::visibility_text;
use crate::stmt_text::indent_text::indent_text;
use redox_ast::ErrorsDecl;

/// An `errors` declaration — a table of variants and their messages.
///
/// ```text
/// errors ConfigError
///   Missing String                        "missing key: #{0}"
///   Invalid String u32                    "invalid value #{0} at line #{1}"
///   Closed                                "closed"
///   from std::num::ParseIntError as Parse  "not a number: #{0}"
/// ```
///
/// # The messages are aligned, and §3 row 7 says alignment is wrong
///
/// Row 7 retires alignment for `case` arms, and the reason it gives — that
/// alignment breaks on the next edit — applies here too: add a long variant
/// and every message moves.
///
/// This aligns anyway, and the distinction is worth stating because it is the
/// one place in the plan where the two arguments point different ways. A
/// `case`'s arms are **heterogeneous**: each is a different pattern of a
/// different length, and padding them is decoration. An `errors` block is a
/// **table**: every row is a variant and a message, the columns mean the same
/// thing on every line, and that is the shape where alignment is doing work
/// rather than looking tidy.
///
/// And §1.5 decides it either way: `examples/errors.sdx` is aligned, and the
/// corpus is the style guide rather than the thing measured against one.
///
/// A `from` variant is a conversion — `from <path> as <Name>` — and sits in
/// the same column as an ordinary one, which is what makes the block read as
/// one table instead of two.
pub(crate) fn errors_text(decl: &ErrorsDecl, indent: usize) -> String {
  let pad = indent_text(indent + 1);
  let header = format!(
    "{}errors {}",
    visibility_text(&decl.visibility),
    decl.name.text
  );
  if decl.variants.is_empty() {
    return header;
  }

  let heads: Vec<String> = decl
    .variants
    .iter()
    .map(|variant| {
      let mut head = if variant.from {
        // The path is the first field's text and the name is the variant's,
        // which is how the parser stores `from P as N`.
        let path = variant
          .fields
          .first()
          .map_or(String::new(), |ty| ty.text.clone());
        format!("from {path} as {}", variant.name.text)
      } else {
        variant.name.text.clone()
      };
      if !variant.from {
        for ty in &variant.fields {
          head.push(' ');
          head.push_str(&ty.text);
        }
      }
      head
    })
    .collect();

  let width = heads.iter().map(String::len).max().unwrap_or(0);
  let rows: Vec<String> = heads
    .iter()
    .zip(&decl.variants)
    .map(|(head, variant)| {
      format!(
        "{pad}{head}{}  {}",
        " ".repeat(width - head.len()),
        // Stored **with** its quotes, like every other literal the lexer
        // keeps verbatim. Adding a pair gave `""missing key: #{0}""`.
        variant.message
      )
    })
    .collect();
  format!("{header}\n{}", rows.join("\n"))
}
