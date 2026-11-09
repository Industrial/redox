use crate::emit::emit_stmts::emit_stmts;
use crate::emit::pad::pad;
use crate::emit::rust_indent::RUST_INDENT;
use crate::emit::sink::Sink;
use crate::emit::test_slug::test_slug;
use crate::emit::trivia::emit_trivia;
use redox_ast::TestDecl;

/// Emit every `test` block as one attributed module — doc 03 §15.
///
/// ```text
/// test "adds two numbers"      #[cfg(test)]
///   assert add 1 2 == 3        mod __rdx_tests {
///                          →       use super::*;
///
///                                  #[test]
///                                  fn adds_two_numbers() {
///                                      assert_eq!(add(1, 2), 3);
///                                  }
///                              }
/// ```
///
/// **One module per file, emitted only when a `test` block is present** —
/// §15's rule, and the reason the tests are gathered here rather than emitted
/// where they appear. The module name uses the `__rdx_` prefix doc 02
/// reserves, so it cannot collide with a module the author wrote.
///
/// `use super::*;` is what makes a test see the file it tests. It is the one
/// `use` Redox emits, and it names nothing: a glob of the enclosing module is
/// true whatever the file contains.
///
/// A slug collision appends an index, as §15 specifies. Two tests whose
/// descriptions differ only in punctuation are the realistic case, and
/// without this they would be two functions of one name — which rustc
/// reports, but about a line the author did not write.
pub(crate) fn emit_tests_module(out: &mut Sink<'_>, tests: &[&TestDecl], indent: usize) {
  if tests.is_empty() {
    return;
  }

  out.unmark();
  pad(out, indent);
  out.push_str("#[cfg(test)]\n");
  pad(out, indent);
  out.push_str("mod __rdx_tests {\n");
  pad(out, indent + RUST_INDENT);
  out.push_str("use super::*;\n");

  let mut used: Vec<String> = Vec::new();
  for t in tests {
    let base = test_slug(&t.description);
    let mut name = base.clone();
    let mut index = 2;
    while used.contains(&name) {
      name = format!("{base}_{index}");
      index += 1;
    }
    used.push(name.clone());

    out.unmark();
    out.push('\n');
    emit_trivia(out, &t.leading, indent + RUST_INDENT);
    out.mark(t.span);
    pad(out, indent + RUST_INDENT);
    out.push_str("#[test]\n");
    pad(out, indent + RUST_INDENT);
    out.push_str(&format!("fn {name}() {{\n"));
    // A test returns nothing, so its last statement is a statement and not a
    // tail expression.
    emit_stmts(out, &t.body.stmts, indent + RUST_INDENT * 2, false);
    out.unmark();
    pad(out, indent + RUST_INDENT);
    out.push_str("}\n");
  }

  pad(out, indent);
  out.push_str("}\n");
}
