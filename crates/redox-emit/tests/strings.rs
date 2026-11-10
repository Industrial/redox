//! Doc 03 §11's strings — interpolation, the four literal kinds and the `"""`
//! block's dedent. Plan 23.
//!
//! The defect this plan fixes was **silent**: `print "hi #{name}"` emitted
//! `println!("hi #{{name}}")`, which is valid Rust that prints the literal
//! text `hi #{name}` with no diagnostic. Doc 02 §5.2's worst failure mode, in
//! the most common thing a string does — so most of these assertions are
//! about text that compiled before and meant the wrong thing.

use redox_emit::emit;
use redox_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented.
fn body(src: &str) -> String {
  xrs(&format!("main : ()\nmain =\n{src}"))
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect::<Vec<_>>()
    .join("\n")
}

// --- §5.1 and §5.2: interpolation -----------------------------------------

#[test]
fn an_interpolation_becomes_a_format_hole() {
  assert_eq!(
    body("  s = \"hello #{name}\"\n"),
    "let s = format!(\"hello {}\", name);"
  );
}

/// An interpolation holds an **expression**. The lexer lexes the body with the
/// ordinary scanners and the parser runs the ordinary expression parser over
/// those tokens, so none of these is a special case — and Emerald accepted
/// only an identifier here for eleven plans.
#[test]
fn an_interpolation_holds_an_expression() {
  for (sdx, want) in [
    ("#{a + b}", "format!(\"{}\", a + b)"),
    ("#{user.name}", "format!(\"{}\", user.name)"),
    ("#{f x}", "format!(\"{}\", f(x))"),
    ("#{xs.len ()}", "format!(\"{}\", xs.len())"),
    (
      "#{if a then 1 else 2}",
      "format!(\"{}\", if a { 1 } else { 2 })",
    ),
  ] {
    assert_eq!(
      body(&format!("  s = \"{sdx}\"\n")),
      format!("let s = {want};"),
      "{sdx}"
    );
  }
}

/// A string literal inside an interpolation. `lex_fragment` claimed this
/// worked from plan 02 and it did not: the **outer** string's scanner stopped
/// at the inner quote. Plan 23 §10 records it.
#[test]
fn an_interpolation_may_hold_a_string_literal() {
  assert_eq!(
    body("  s = \"outer #{String::from(\"inner\")} done\"\n"),
    "let s = format!(\"outer {} done\", String::from(\"inner\"));"
  );
}

// --- §5.3: format specs ----------------------------------------------------

/// Spec text is pasted **uninterpreted** — doc 03 §11.1, and AGENTS.md's rule
/// applied to a grammar Rust already owns. Five specs that are not all width,
/// because "every Rust format spec is reachable" has to mean something.
#[test]
fn a_format_spec_is_copied_verbatim() {
  for (sdx, want) in [
    ("#{x:.2}", "{:.2}"),
    ("#{x:?}", "{:?}"),
    ("#{x:>8}", "{:>8}"),
    ("#{x:05}", "{:05}"),
    ("#{x:#x}", "{:#x}"),
    ("#{x:+e}", "{:+e}"),
  ] {
    assert_eq!(
      body(&format!("  s = \"{sdx}\"\n")),
      format!("let s = format!(\"{want}\", x);"),
      "{sdx}"
    );
  }
}

/// Redox does not validate a spec, so a nonsense one reaches rustc — which
/// reports it against the interpolation's own location through plan 06's map.
#[test]
fn a_nonsense_spec_is_passed_to_rustc() {
  assert_eq!(
    body("  s = \"#{x:nonsense}\"\n"),
    "let s = format!(\"{:nonsense}\", x);"
  );
}

// --- §5.4: braces and holes in one template -------------------------------

/// **The two things the old code confused.** Doubling a brace the author wrote
/// is right; doubling the interpolation's was the defect. Asserted together,
/// because fixing either one alone would have passed a test that checked only
/// the other.
#[test]
fn an_authors_brace_doubles_and_an_interpolation_does_not() {
  assert_eq!(
    body("  s = \"{} is not a hole, but #{n} is\"\n"),
    "let s = format!(\"{{}} is not a hole, but {} is\", n);"
  );
}

#[test]
fn several_interpolations_fill_several_holes() {
  assert_eq!(
    body("  s = \"#{a}-#{b}-#{c}\"\n"),
    "let s = format!(\"{}-{}-{}\", a, b, c);"
  );
}

// --- §5.5: `print` of an interpolation ------------------------------------

/// `println!` with the interpolation's **own** template, not
/// `println!("{}", format!(…))` — one macro and one allocation instead of two,
/// and what a person would write.
#[test]
fn print_of_an_interpolation_is_println_with_that_template() {
  assert_eq!(
    body("  print \"hi #{name}\"\n"),
    "println!(\"hi {}\", name);"
  );
}

/// The regression test for §1.2's silent defect, stated as the thing that must
/// **not** happen.
#[test]
fn print_does_not_double_an_interpolations_braces() {
  let out = body("  print \"hi #{name}\"\n");
  assert!(!out.contains("#{{"), "{out}");
  assert!(!out.contains("#{name}"), "{out}");
}

// --- criterion 6: the old path is unchanged -------------------------------

/// A string with no interpolation is still a `StrLit` passed through raw,
/// which is what plan 02 carried `raw` for — and why one node for both would
/// have made every existing golden a diff.
#[test]
fn a_string_without_interpolation_is_unchanged() {
  assert_eq!(body("  s = \"plain\"\n"), "let s = \"plain\";");
  assert_eq!(body("  print \"plain\"\n"), "println!(\"plain\");");
}

/// And a brace in such a string still doubles only where it is a format
/// string, which is the distinction `double_braces` calls load-bearing.
#[test]
fn a_brace_in_a_plain_string_doubles_only_in_a_template() {
  assert_eq!(body("  s = \"{}\"\n"), "let s = \"{}\";");
  assert_eq!(body("  print \"{}\"\n"), "println!(\"{{}}\");");
}

// --- §5.7: the four literal kinds -----------------------------------------

/// Doc 03 §11.2 calls all four *"pure passthrough"*. `r"…"` is also what makes
/// §11.3's refusal to add regex literals honest.
#[test]
fn every_literal_kind_passes_through() {
  for sdx in [
    "r\"^\\d+$\"",
    "r#\"he said \"yes\"\"#",
    "b\"bytes\"",
    "c\"ffi\"",
    "b'x'",
    "'R'",
  ] {
    let out = body(&format!("  s = {sdx}\n"));
    assert_eq!(out, format!("let s = {sdx};"), "{sdx}");
  }
}

/// A raw string's `#` count is re-emitted, which is why the token carries
/// `hashes`.
#[test]
fn a_raw_strings_hash_count_survives() {
  assert_eq!(body("  s = r##\"a\"#b\"##\n"), "let s = r##\"a\"#b\"##;");
}

/// A byte literal that is not printable ASCII takes `\xNN` — the only form
/// that round-trips every byte, and what rustc itself prints.
#[test]
fn a_non_printable_byte_takes_a_hex_escape() {
  assert_eq!(body("  s = b'\\n'\n"), "let s = b'\\x0A';");
  assert_eq!(body("  s = b'\\x00'\n"), "let s = b'\\x00';");
}

/// `\x` takes **exactly two** characters, as Rust's does. Scanning greedily
/// for hex digits consumed the `E` of `\x7fELF` as a third and rejected a
/// literal rustc accepts — plan 23 §10.
#[test]
fn a_hex_escape_stops_after_two_characters() {
  assert_eq!(body("  s = b\"\\x7fELF\"\n"), "let s = b\"\\x7FELF\";");
}

// --- §5.8: the `"""` block -------------------------------------------------

/// Rust has no triple-quoted string, so the lexer's **dedented** value becomes
/// one ordinary escaped literal. The common indentation goes and the relative
/// indentation stays — doc 01 §7.2.
#[test]
fn a_triple_quoted_block_emits_one_dedented_rust_string() {
  assert_eq!(
    body("  s = \"\"\"\n    first\n      indented\n    last\n    \"\"\"\n"),
    "let s = \"first\\n  indented\\nlast\";"
  );
}

/// Interpolation works inside a `"""` block for free, because the block is
/// dedented and then handed to the same part splitter.
#[test]
fn a_triple_quoted_block_interpolates() {
  assert_eq!(
    body("  s = \"\"\"\n    hello #{name}\n    \"\"\"\n"),
    "let s = format!(\"hello {}\", name);"
  );
}

/// The block's span still covers **the whole block**, because doc 03 §11.2
/// ties the form to line numbering the source map depends on. A span that
/// collapsed to the opening delimiter would be three characters wide; this one
/// has to reach the closing `"""`.
#[test]
fn a_triple_quoted_blocks_span_covers_it() {
  let src = "main : ()\nmain =\n  s = \"\"\"\n    a\n    b\n    \"\"\"\n  print s\n";
  let program = parse_source(src).expect("should parse");

  let redox_ast::Item::Fn(main) = &program.items[0] else {
    panic!("expected a function");
  };
  let block = main.clauses[0]
    .body
    .plain()
    .expect("a plain body, not a guard ladder");
  let redox_ast::Expr::Binding(binding) = &block.stmts[0].expr else {
    panic!("expected the binding");
  };
  let redox_ast::Expr::StrLit { span, value } = &binding.value else {
    panic!("a dedented block is a StrLit, not {:?}", binding.value);
  };

  assert_eq!(value, "\"a\\nb\"");
  // The source form spans from `\"\"\"` on line 3 to `\"\"\"` on line 6 — far
  // more than the dedented value's own length, which is the point: the token
  // keeps the block's extent so every later line maps to the right number.
  assert!(
    span.1 - span.0 > value.len(),
    "span {span:?} does not cover the block"
  );
  assert!(src[span.0..span.1].starts_with("\"\"\""), "{span:?}");
  assert!(src[span.0..span.1].ends_with("\"\"\""), "{span:?}");
}
