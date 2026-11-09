use crate::emit::double_braces::double_braces;
use crate::emit::expr::emit_expr;
use crate::emit::unparen::unparen;
use redox_ast::StrInterpPart;

/// An interpolated string's Rust format template and argument list — doc 03
/// §11.1, plan 23.
///
/// ```text
/// "hello #{name}"           ("hello {}",      ["name"])
/// "total: #{amount:.2}"  →  ("total: {:.2}",  ["amount"])
/// "{} is #{n}"              ("{{}} is {}",    ["n"])
/// ```
///
/// Returned as a pair rather than a finished macro call because two callers
/// need it: `format!` for an interpolated string in expression position, and
/// `println!` for `print` of one. Doc 03 §10.9's `print` sugar would otherwise
/// have to wrap a `format!` in a `println!("{}", …)`, which is two macros and
/// one allocation to say what one says.
///
/// # The two transformations, and why they must happen here
///
/// Literal text arrives escaped from the parser and is brace-doubled here: a
/// brace the author wrote has to print as a brace instead of naming a hole. An interpolation's `{spec}` is
/// written after that doubling, so it survives — which is exactly the bug this
/// plan fixes. Before it, an interpolated string reached `print_call` as a
/// `StrLit` whose *raw source* still held `#{name}`, and `double_braces`
/// turned it into `#{{name}}`: valid Rust that prints `#{name}` with no
/// diagnostic. Plan 23 §1.2.
pub(crate) fn format_template(parts: &[StrInterpPart], indent: usize) -> (String, Vec<String>) {
  let mut template = String::new();
  let mut args = Vec::new();

  for part in parts {
    match part {
      // Already escaped by the parser; only the brace doubling is the
      // emitter's, because it is a *format-string* concern and not an
      // escaping one — the distinction `double_braces` calls load-bearing.
      StrInterpPart::Text(text) => template.push_str(&double_braces(text)),
      StrInterpPart::Interp { expr, spec } => {
        // The spec is pasted uninterpreted — doc 03 §11.1, and AGENTS.md's
        // rule applied to a grammar Rust already owns. `#{x:nonsense}` emits
        // `{:nonsense}` and rustc reports it against the interpolation's own
        // location through plan 06's map.
        match spec {
          Some(spec) => template.push_str(&format!("{{:{spec}}}")),
          None => template.push_str("{}"),
        }
        args.push(emit_expr(unparen(expr), indent));
      }
    }
  }

  (template, args)
}
