//! `redox-emit` — the Rust source emitter.
//!
//! Plan 01 (`docs/05 - 01 - Vertical Slice.md`), leaf 5. Turns a
//! [`Program`] into Rust source text, written to a `.xrs` file.
//!
//! Three disciplines, each established here because each is expensive to
//! introduce later:
//!
//! - **Deterministic and byte-stable.** No `HashMap` iteration anywhere;
//!   output is a straight fold over the tree in source order. Doc 01 §3.4
//!   requires it, and it is what makes exact-text golden tests possible at
//!   all (acceptance criterion 7).
//! - **No rustfmt in the contract.** Doc 02 §5.3: shelling out would make
//!   output depend on an external tool's version, break determinism, and
//!   destroy the span map plan 06 needs.
//! - **One output line per input line wherever the construct permits**, which
//!   at this slice's size is everywhere except a block's closing brace. This
//!   is doc 02 §6.5 item 1's line-preserving discipline; establishing it now
//!   is what keeps plan 06's span map cheap.
//!
//! Emitted Rust uses **four-space indentation**, matching rustfmt's default so
//! generated code does not read as foreign. That is independent of this
//! repository's own two-space source style (`rustfmt.toml`).

use redox_ast::{Expr, FnDecl, Item, Program, Stmt, StmtKind, Trivia, TriviaKind};
use std::fmt::Write as _;

/// Indentation of emitted Rust, in spaces. rustfmt's default.
const RUST_INDENT: usize = 4;

/// Emit `program` as Rust source text.
pub fn emit(program: &Program) -> String {
  let mut out = String::new();
  for (i, item) in program.items.iter().enumerate() {
    // One blank line between items. Driven by position in the tree, not by
    // how many blank lines the source happened to have, so output is a
    // function of structure and therefore stable.
    if i > 0 {
      out.push('\n');
    }
    match item {
      Item::Fn(f) => emit_fn(&mut out, f),
    }
  }
  emit_trivia(&mut out, &program.trailing, 0);
  // A source file's final newline arrives here as a trailing `BlankLines`
  // trivium, which would otherwise append a stray blank line to every
  // generated file. Normalising to exactly one terminating newline is a
  // deterministic rule that also guards against any other trailing-whitespace
  // drift, which matters because golden tests compare exact bytes.
  while out.ends_with("\n\n") {
    out.pop();
  }
  if !out.is_empty() && !out.ends_with('\n') {
    out.push('\n');
  }
  out
}

fn emit_fn(out: &mut String, f: &FnDecl) {
  emit_trivia(out, &f.leading, 0);
  // Parameters and return types are plan 03; this slice emits a bare
  // signature.
  out.push_str("fn ");
  out.push_str(&f.name.text);
  out.push_str("() {");
  emit_inline_comment(out, f.trailing.as_ref());
  out.push('\n');
  for stmt in &f.body.stmts {
    emit_stmt(out, stmt, RUST_INDENT);
  }
  out.push_str("}\n");
}

fn emit_stmt(out: &mut String, stmt: &Stmt, indent: usize) {
  emit_trivia(out, &stmt.leading, indent);
  pad(out, indent);
  match &stmt.kind {
    StmtKind::Expr(e) => {
      out.push_str(&expr(e));
      out.push(';');
    }
  }
  emit_inline_comment(out, stmt.trailing.as_ref());
  out.push('\n');
}

/// Render an expression.
fn expr(e: &Expr) -> String {
  match e {
    Expr::StrLit { value, .. } => value.clone(),
    Expr::Ident(i) => i.text.clone(),
    Expr::Call { callee, args, .. } => {
      let rendered: Vec<String> = args.iter().map(expr).collect();
      let joined = rendered.join(", ");
      // `print` is the one sanctioned sugar of its kind (doc 01 §3.3,
      // doc 03 §10.9) and it is a **dated loan**: it is exactly Emerald's
      // hardcoded `puts` keyword, which papered over a general
      // parenless-call ambiguity for 89 plans. Plan 19 resolves the general
      // case and this special case then dissolves.
      if let Expr::Ident(name) = callee.as_ref()
        && name.text == "print"
      {
        return format!("println!({joined})");
      }
      format!("{}({joined})", expr(callee))
    }
  }
}

/// Emit leading trivia at `indent`. A comment becomes `//`; a blank-line run
/// collapses to a single blank line.
fn emit_trivia(out: &mut String, trivia: &[Trivia], indent: usize) {
  for t in trivia {
    match t.kind {
      TriviaKind::LineComment => {
        pad(out, indent);
        if t.text.is_empty() {
          out.push_str("//\n");
        } else {
          let _ = writeln!(out, "// {}", t.text);
        }
      }
      TriviaKind::BlankLines => {
        // Never open a file or a block with a blank line; it would be noise
        // and would break byte-stability against a blessed golden file.
        if !out.is_empty() && !out.ends_with("{\n") && !out.ends_with("\n\n") {
          out.push('\n');
        }
      }
    }
  }
}

fn emit_inline_comment(out: &mut String, trivia: Option<&Trivia>) {
  if let Some(t) = trivia
    && t.kind == TriviaKind::LineComment
  {
    if t.text.is_empty() {
      out.push_str(" //");
    } else {
      let _ = write!(out, " // {}", t.text);
    }
  }
}

fn pad(out: &mut String, n: usize) {
  for _ in 0..n {
    out.push(' ');
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use redox_parser::parse_source;

  fn xrs(src: &str) -> String {
    emit(&parse_source(src).expect("should parse"))
  }

  /// Plan 01 §3's concrete proof, at the unit level.
  #[test]
  fn emits_the_hello_slice() {
    let out = xrs("# The smallest Redox program.\nmain = ->\n  print \"hello\"\n");
    assert_eq!(
      out,
      "// The smallest Redox program.\nfn main() {\n    println!(\"hello\");\n}\n"
    );
  }

  /// Acceptance criterion 5.
  #[test]
  fn a_comment_survives_into_the_generated_rust() {
    let out = xrs("# kept\nmain = ->\n  print \"x\"\n");
    assert!(out.starts_with("// kept\n"), "{out}");
  }

  #[test]
  fn an_inline_comment_survives_into_the_generated_rust() {
    let out = xrs("main = ->\n  print \"x\" # why\n");
    assert!(out.contains("println!(\"x\"); // why\n"), "{out}");
  }

  #[test]
  fn a_comment_at_end_of_file_survives() {
    let out = xrs("main = ->\n  print \"x\"\n# the end\n");
    assert!(out.ends_with("// the end\n"), "{out}");
  }

  /// Acceptance criterion 7, at the unit level: emission is a pure function
  /// of the tree.
  #[test]
  fn emission_is_deterministic() {
    let src = "# a\nmain = ->\n  print \"x\"\n\nother = ->\n  print \"y\"\n";
    assert_eq!(xrs(src), xrs(src));
  }

  #[test]
  fn separates_items_with_one_blank_line() {
    let out = xrs("a = ->\n  print \"1\"\nb = ->\n  print \"2\"\n");
    assert_eq!(
      out,
      "fn a() {\n    println!(\"1\");\n}\n\nfn b() {\n    println!(\"2\");\n}\n"
    );
  }

  #[test]
  fn emits_four_space_indentation() {
    let out = xrs("main = ->\n  print \"x\"\n");
    assert!(out.contains("\n    println!"), "{out}");
  }

  #[test]
  fn a_non_print_call_keeps_its_name() {
    let out = xrs("main = ->\n  log \"x\"\n");
    assert!(out.contains("log(\"x\");"), "{out}");
  }

  /// Regression: the fixture's own final newline used to append a stray blank
  /// line to every generated file. Found by the end-to-end test comparing
  /// against plan 01 §3's stated output, not by inspection.
  #[test]
  fn ends_with_exactly_one_newline() {
    for src in [
      "main = ->\n  print \"x\"\n",
      "main = ->\n  print \"x\"",
      "main = ->\n  print \"x\"\n\n\n",
      "main = ->\n  print \"x\"\n# tail\n",
    ] {
      let out = xrs(src);
      assert!(out.ends_with('\n'), "{out:?}");
      assert!(!out.ends_with("\n\n"), "{out:?}");
    }
  }

  /// Line-preserving discipline: source lines 1-3 map to output lines 1-3.
  #[test]
  fn preserves_line_positions_for_the_slice() {
    let src = "# one\nmain = ->\n  print \"three\"\n";
    let out = xrs(src);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "// one");
    assert_eq!(lines[1], "fn main() {");
    assert_eq!(lines[2], "    println!(\"three\");");
  }
}
