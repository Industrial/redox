pub(crate) mod class_text;
pub(crate) mod clause_text;
pub(crate) mod const_text;
pub(crate) mod constructor_text;
pub(crate) mod data_text;
pub(crate) mod errors_text;
pub(crate) mod fn_text;
pub(crate) mod instance_text;
pub(crate) mod leading_text;
pub(crate) mod member_text;
pub(crate) mod same_line;
pub(crate) mod signature_line;
pub(crate) mod visibility_text;

use crate::item_text::class_text::class_text;
use crate::item_text::const_text::const_text;
use crate::item_text::data_text::data_text;
use crate::item_text::errors_text::errors_text;
use crate::item_text::fn_text::fn_text;
use crate::item_text::instance_text::instance_text;
use crate::item_text::leading_text::leading_text;
use crate::item_text::visibility_text::visibility_text;
use crate::stmt_text::block_text::block_text;
use crate::stmt_text::indent_text::indent_text;
use redox_ast::Item;

/// One item in canonical Redox, with its leading trivia — plan 34, leaf 2.
///
/// `indent` is the item's depth: 0 at the top of a file, deeper inside a
/// `module`. Leaf 4's `indent_text` is the only place a level becomes spaces.
///
/// # `export`, not `pub`
///
/// `Visibility::rust_text` gives `pub `, `pub(crate) ` and so on. Redox spells
/// the same thing `export` and `export(crate)`, which is why
/// `visibility_text` exists beside it rather than reusing it — the third
/// printer in this plan to need a Redox spelling where the AST held only the
/// Rust one.
///
/// # A function's signature is sliced, and that is a recorded defect
///
/// Every item here is printed from the tree except a function's **signature
/// line**, which is sliced verbatim out of the source. The reason is in §10:
/// `parser::generalise` rewrites a signature's types in place, replacing each
/// type variable with its uppercase Rust spelling, so the AST no longer knows
/// the author wrote `Ord t => Option<t> -> t` rather than
/// `Ord T => Option<T> -> T`. Printing from it would rewrite every generic
/// signature in the corpus into Rust's convention, which is the opposite of
/// doc 18 §5.3's surface.
///
/// Criterion 7 exists for exactly this, so the slice is correct rather than a
/// shortcut — but it does mean the formatter cannot normalise a signature's
/// spacing, and the fix is a change to the AST, not to this file.
pub fn item_text(item: &Item, indent: usize, source: &str) -> String {
  let mut out = leading_text(item, indent);
  let pad = indent_text(indent);
  out.push_str(&pad);
  out.push_str(&match item {
    Item::Fn(decl) => fn_text(decl, indent, source),
    Item::Data(decl) => data_text(decl, indent, source),
    Item::Type(decl) => format!(
      "{}type {} = {}",
      visibility_text(&decl.visibility),
      decl.name.text,
      decl.ty.text
    ),
    Item::Class(decl) => class_text(decl, indent, source),
    Item::Instance(decl) => instance_text(decl, indent, source),
    Item::Errors(decl) => errors_text(decl, indent),
    // The description keeps its quotes, as an error variant's message does.
    Item::Test(decl) => format!(
      "test {}\n{}",
      decl.description,
      block_text(&decl.body, indent + 1, source)
    ),
    Item::Import(decl) => {
      let alias = decl
        .alias
        .as_ref()
        .map_or(String::new(), |name| format!(" as {}", name.text));
      format!(
        "{}import {}{alias}",
        visibility_text(&decl.visibility),
        decl.path
      )
    }
    // A `module` with no items is a file module — `module name` and nothing
    // else — so the brace-less form is the absence of a body rather than an
    // empty one.
    Item::Module(decl) => {
      let header = format!(
        "{}module {}",
        visibility_text(&decl.visibility),
        decl.name.text
      );
      match &decl.items {
        None => header,
        Some(items) => {
          let printed: Vec<String> = items
            .iter()
            .map(|inner| item_text(inner, indent + 1, source))
            .collect();
          format!("{header}\n{}", printed.join("\n"))
        }
      }
    }
    Item::Const(decl) => const_text(decl, source),
    // Plan 45 leaf 1. The formatter reproduces the hatch as written: the
    // body is Rust that Redox never parsed, so there is nothing here that
    // could be canonicalised without implementing a Rust formatter — and
    // `rustfmt` is the tool for that, on the `.xrs`.
    // Plan 45 leaf 1. The body is re-indented one level inside the block,
    // which is what makes the construct canonical and round-trip: the lexer
    // *dedents* a `"""` block, so emitting `decl.code` as-is writes it at
    // column 0 and the file changes every time it is formatted.
    Item::InlineRust(decl) => {
      let body: String = decl
        .code
        .lines()
        .map(|line| {
          if line.trim().is_empty() {
            "\n".to_string()
          } else {
            format!("{}{}\n", indent_text(indent + 1), line)
          }
        })
        .collect();
      format!("rust \"\"\"\n{}{}\"\"\"", body, indent_text(indent))
    }
    // Plan 45 leaf 3. The declarations are reproduced from the source slice
    // for the reason this module's header gives: a signature's own spacing is
    // not in the AST, so re-rendering one would change text the formatter has
    // no mandate to change.
    Item::Extern(decl) => {
      let body: String = decl
        .decls
        .iter()
        .map(|d| {
          format!(
            "{}{}\n",
            indent_text(indent + 1),
            source[d.span.0..d.span.1].trim()
          )
        })
        .collect();
      format!("extern {}\n{}", decl.abi, body.trim_end())
    }
  });
  out
}
