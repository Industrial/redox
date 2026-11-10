/// One Redox macro argument as the Rust arguments it supplies.
pub(crate) mod macro_argument;

use crate::emit::assert_call::assert_call;
use crate::emit::fn_call_width::FN_CALL_WIDTH;
use crate::emit::macro_call::macro_argument::macro_argument;
use crate::emit::rust_indent::RUST_INDENT;
use redox_ast::Expr;

/// Lower plan 64's macro invocation.
///
/// ```text
/// println! \-> "#{a} #{b}"        println!("{} {}", a, b)
/// write!   \-> f \-> "#{n} rows"   write!(f, "{} rows", n)
/// todo!                           todo!()
/// assert!  \-> a == b              assert_eq!(a, b)
/// ```
///
/// Each argument contributes one *or more* Rust arguments, which is leaf 3:
/// an interpolated string is a format template plus its spliced values, so
/// `bail! \-> "no route for #{id}"` reaches the macro as two arguments where
/// the author wrote one. [`macro_argument`] is where that happens.
///
/// # `assert!` keeps its rewrite
///
/// `assert! \-> a == b` emits `assert_eq!(a, b)` rather than `assert!(a ==
/// b)`, and `!=` likewise. That is not a leftover from the retired `assert`
/// keyword — it is the reason plan 64 §11.2 kept the rewrite when the keyword
/// went: `assert_eq!` prints **both values** when they differ and `assert!`
/// prints only the expression's text, so retiring the keyword into the
/// general macro path without this would be a regression in diagnostic
/// quality. The condition is narrow on purpose — one argument, a comparison —
/// and every other `assert!` goes the general way.
pub(crate) fn macro_call(name: &str, args: &[Expr], indent: usize) -> String {
  if name == "assert"
    && let [only] = args
    && let rewritten = assert_call(only, indent)
    && !rewritten.starts_with("assert!")
  {
    return rewritten;
  }

  let supplied: Vec<String> = args
    .iter()
    .flat_map(|a| macro_argument(a, indent))
    .collect();
  // Measured on the **arguments**, not on the rendered call: that is what
  // `FN_CALL_WIDTH` means, and counting `println!(` and `)` with them put
  // every argument list nine characters nearer the threshold than rustfmt
  // puts it. `examples/errors.sdx` spread a line rustfmt keeps whole.
  let arguments = supplied.join(", ");
  let one_line = format!("{name}!({arguments})");
  if !arguments.contains('\n') && arguments.len() <= FN_CALL_WIDTH {
    return one_line;
  }

  // The same shape `print_call` spreads to, for the same reason: rustfmt
  // overflows a block-like last argument and spreads anything else. A
  // multi-line callback body is block-like, so it hangs off the call — which
  // is why the test is on the *rendered* argument rather than on its arity.
  if supplied.last().is_some_and(|a| a.starts_with('{')) {
    return one_line;
  }
  let inner_pad = " ".repeat(indent + RUST_INDENT);
  let close_pad = " ".repeat(indent);
  let spread = args
    .iter()
    .flat_map(|a| macro_argument(a, indent + RUST_INDENT))
    .collect::<Vec<_>>()
    .join(&format!(",\n{inner_pad}"));
  format!("{name}!(\n{inner_pad}{spread}\n{close_pad})")
}
