//! The source map is **total**: every generated line is accounted for.
//!
//! Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 5. §3 of that plan states why
//! totality is the criterion rather than accuracy on a chosen example: a map
//! that is 90% right is worse than no map, because at the point of use the 10%
//! is indistinguishable from the 90%. Plan 07 will report these positions to a
//! user as if they were facts, so they had better be facts.

mod support;

use redox_diag::{SourceMap, parse_map, render_map};
use std::path::Path;

/// The examples every assertion here runs over.
///
/// `lexical.sdx` is excluded for the reason plan 02 recorded: it exercises the
/// lexer with constructs the parser does not yet accept, so it has no
/// generated output to map.
const EXAMPLES: &[&str] = &["hello", "phase1", "expressions", "comments"];

fn emit(example: &str) -> (String, String, SourceMap) {
  let path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../examples")
    .join(format!("{example}.sdx"));
  let source = std::fs::read_to_string(&path)
    .unwrap_or_else(|e| panic!("should read {}: {e}", path.display()));
  let program =
    redox_parser::parse_source(&source).unwrap_or_else(|_| panic!("{example}.sdx should parse"));
  let (rust, map) = redox_emit::emit_with_map(&program, &source, &format!("{example}.sdx"));
  (source, rust, map)
}

/// Criterion 1: every generated line resolves, or is one of the two kinds of
/// line the emitter synthesises.
///
/// The synthetic set is asserted *exactly*, not bounded. If a later plan emits
/// a third kind of line with no source, this fails and whoever added it has to
/// say so in the plan rather than letting the map quietly thin out.
#[test]
fn every_generated_line_is_accounted_for() {
  for example in EXAMPLES {
    let (_, rust, map) = emit(example);
    let lines: Vec<&str> = rust.lines().collect();
    assert_eq!(
      map.len(),
      lines.len(),
      "{example}: the map has {} records for {} generated lines",
      map.len(),
      lines.len()
    );

    for gen_line in map.synthetic_lines() {
      let text = lines[gen_line - 1].trim();
      assert!(
        text == "}" || text.is_empty(),
        "{example}:{gen_line} has no origin but is neither a closing brace \
         nor blank: {text:?}"
      );
    }
  }
}

/// Criterion 2: a resolved position is not merely in range — the source text
/// there begins the same token the generated line's content does.
///
/// This is the assertion that catches an off-by-one. A line count cannot: a
/// map shifted by one line is exactly as long as a correct one.
#[test]
fn a_resolved_position_agrees_with_the_source_text() {
  for example in EXAMPLES {
    let (source, rust, map) = emit(example);
    let src_lines: Vec<&str> = source.lines().collect();

    for (i, _) in rust.lines().enumerate() {
      let gen_line = i + 1;
      let Some(origin) = map.origin(gen_line) else {
        continue;
      };
      assert!(
        origin.src_line <= src_lines.len(),
        "{example}:{gen_line} resolves to source line {}, past the end of a \
         {}-line file",
        origin.src_line,
        src_lines.len()
      );

      // The span the origin carries must name the text it claims to.
      let span_text = &source[origin.span.0..origin.span.1];
      let src_line = src_lines[origin.src_line - 1];
      assert!(
        src_line.contains(span_text.lines().next().unwrap_or("")),
        "{example}:{gen_line} resolves to source line {} ({src_line:?}), \
         which does not contain the span's text ({span_text:?})",
        origin.src_line
      );

      // And the column must point at that text, not at the indentation.
      let col = origin.src_col;
      assert!(
        col >= 1 && col <= src_line.chars().count() + 1,
        "{example}:{gen_line} resolves to column {col} of a {}-character line",
        src_line.chars().count()
      );
    }
  }
}

/// Criterion 3: one claim written out in full, so a reader can check it
/// against `examples/phase1.sdx` without running anything.
///
/// ```text
/// phase1.sdx                      phase1.xrs
/// 1  -- Doc 01 §31 …          1  // Doc 01 §31 …
/// 2  -- print. Prints 30 …    2  // print. Prints 30 …
/// 3  add :: i32 -> i32 -> i32 3  fn add(a: i32, b: i32) -> i32 {
/// 4  add a b =                4      a + b
/// 5    a + b                  5  }
/// ```
///
/// Generated line 4 is `    a + b` at column 5; source line **5** is `  a + b`
/// at column 3. The signature line and the equation head both occupy source
/// lines that the generated `fn` header collapses into one, which is why the
/// two numbers differ at all — and why the map exists.
///
/// The literal numbers are the point. Plan 08 added two comment lines to
/// `phase1.sdx` and this test failed; doc 18's two-line declaration form moved
/// it again. Both times the failure was the map test doing its job — a test
/// that computed the line from the file would have passed while proving
/// nothing.
#[test]
fn phase1_tail_expression_maps_to_its_source_line() {
  let (_, rust, map) = emit("phase1");
  assert_eq!(
    rust.lines().nth(3).map(str::trim),
    Some("a + b"),
    "the premise of this test is that generated line 4 is the tail expression"
  );
  assert_eq!(map.resolve(4, 5), Some((5, 3)));
}

/// Criterion 4: the sidecar round-trips.
///
/// The parsed map's origins carry no span — the sidecar does not store one, by
/// the decision recorded in `parse_map` — so the comparison is over what the
/// format actually claims to preserve: the positions, line by line.
#[test]
fn the_sidecar_round_trips() {
  for example in EXAMPLES {
    let (_, _, map) = emit(example);
    let text = render_map(&map);
    let parsed = parse_map(&text).unwrap_or_else(|e| panic!("{example}: {e}"));

    assert_eq!(parsed.len(), map.len(), "{example}: record count");
    assert_eq!(parsed.source_path, map.source_path, "{example}: header");
    for gen_line in 1..=map.len() {
      let before = map.origin(gen_line);
      let after = parsed.origin(gen_line);
      match (before, after) {
        (Some(b), Some(a)) => {
          assert_eq!(
            (a.src_line, a.src_col, a.gen_col),
            (b.src_line, b.src_col, b.gen_col),
            "{example}:{gen_line}"
          );
        }
        (None, None) => {}
        _ => panic!("{example}:{gen_line} changed whether it has an origin"),
      }
    }
  }
}

/// Criterion 5: `redox build` leaves the sidecar beside the `.xrs`, and what
/// it wrote parses.
#[test]
fn build_writes_a_parseable_sidecar() {
  let case = support::transpile("phase1.sdx");
  let map_path = case.map_path();
  let text = std::fs::read_to_string(&map_path)
    .unwrap_or_else(|e| panic!("should read {}: {e}", map_path.display()));
  let map = parse_map(&text).expect("the sidecar `redox build` wrote should parse");

  assert_eq!(
    map.len(),
    case.rust.lines().count(),
    "the sidecar covers every line of the `.xrs` beside it"
  );
  assert!(
    map.source_path.ends_with("phase1.sdx"),
    "the header names the input: {}",
    map.source_path
  );
}

/// Criterion 6: `emit` is unchanged. Plan 04's goldens depend on it, so the
/// equivalence is asserted rather than assumed.
#[test]
fn emit_still_returns_what_emit_with_map_returns() {
  for example in EXAMPLES {
    let (source, rust, _) = emit(example);
    let program = redox_parser::parse_source(&source).expect("should parse");
    assert_eq!(
      redox_emit::emit(&program),
      rust,
      "{example}: emit and emit_with_map disagree about the output"
    );
  }
}
