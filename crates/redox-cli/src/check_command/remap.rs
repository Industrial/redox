pub(crate) mod char_offset;
pub(crate) mod generated_offset;
pub(crate) mod narrow;

use crate::check_command::remap::generated_offset::generated_offset;
use crate::check_command::remap::narrow::narrow;
use crate::check_command::rustc_message::RustcMessage;
use redox_diag::{Diagnostic, DiagnosticCode, SourceMap};

/// Turn one of rustc's diagnostics into one against the `.sdx`.
///
/// Plan 07 (`docs/12 - 07 - Rustc Remapping.md`) leaf 3. The message, the code
/// and the level stay rustc's verbatim; only the position changes (§1.1).
///
/// Three outcomes, and telling them apart is the whole plan:
///
/// 1. **The line maps and the shifted column lands inside the construct's
///    span** — [`narrow`] reduces the span to that character. The exact case,
///    which holds wherever the emitter copied the source's content verbatim.
///
/// 2. **The line maps but the column does not land inside the span** — the
///    whole span is underlined, because the error is somewhere in that
///    statement and Redox does not know where.
///
/// 3. **The line does not map** — it is one the emitter synthesised, a closing
///    brace or a blank. The diagnostic is reported against the `.xrs`, with a
///    note saying so. Relocating it to the nearest mapped line would be a
///    guess presented as a fact.
pub(crate) fn remap(
  message: &RustcMessage,
  map: &SourceMap,
  sdx_path: &str,
  sdx_source: &str,
  xrs_path: &str,
  xrs_source: &str,
) -> Diagnostic {
  // rustc gives most diagnostics a code and some — a plain lint, say — none.
  // Falling back to the level keeps the rendered output from printing an empty
  // `error[]`, and is still rustc's own word rather than one of Redox's.
  let code = DiagnosticCode::Foreign(
    message
      .code
      .clone()
      .unwrap_or_else(|| message.level.clone()),
  );

  match map.origin(message.line) {
    Some(origin) => {
      let shifted = origin.map_col(message.col);
      Diagnostic {
        code,
        message: message.message.clone(),
        span: narrow(origin.span, sdx_source, origin.src_line, shifted),
        help: None,
        path: sdx_path.to_string(),
        source: sdx_source.to_string(),
      }
    }
    None => {
      let offset = generated_offset(xrs_source, message.line, message.col);
      Diagnostic {
        code,
        message: message.message.clone(),
        span: (offset, offset + 1),
        help: Some(format!(
          "this is on a line Redox generated, so it has no place in your source; \
           see {xrs_path}:{}:{}",
          message.line, message.col
        )),
        path: xrs_path.to_string(),
        source: xrs_source.to_string(),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::remap;
  use crate::check_command::rustc_message::RustcMessage;
  use redox_diag::{Origin, SourceMap};

  const SDX: &str = "add = (a: i32, b: i32): i32 ->\n  a + \"x\"\n";
  const XRS: &str = "fn add(a: i32, b: i32) -> i32 {\n    a + \"x\"\n}\n";

  /// `  a + "x"` spans bytes 33..40.
  fn map() -> SourceMap {
    SourceMap::new(
      "p.sdx",
      vec![
        Some(Origin {
          span: (0, 30),
          src_line: 1,
          src_col: 1,
          gen_col: 1,
        }),
        Some(Origin {
          span: (33, 40),
          src_line: 2,
          src_col: 3,
          gen_col: 5,
        }),
        None,
      ],
    )
  }

  fn message(line: usize, col: usize) -> RustcMessage {
    RustcMessage {
      path: "p.xrs".to_string(),
      line,
      col,
      level: "error".to_string(),
      code: Some("E0277".to_string()),
      message: "cannot add `&str` to `i32`".to_string(),
    }
  }

  fn run(m: &RustcMessage) -> redox_diag::Diagnostic {
    remap(m, &map(), "p.sdx", SDX, "p.xrs", XRS)
  }

  /// Case 1, and the measured one: rustc's `.xrs:2:7` is the `+`, and the
  /// `.sdx`'s `+` is at line 2 column 5.
  #[test]
  fn narrows_to_the_exact_character_on_a_verbatim_line() {
    let d = run(&message(2, 7));
    assert_eq!(d.path, "p.sdx");
    assert_eq!(&SDX[d.span.0..d.span.1], "+");
    assert_eq!(d.code.text(), "E0277");
    assert_eq!(
      d.message, "cannot add `&str` to `i32`",
      "rustc's message passes through"
    );
    assert!(d.help.is_none());
  }

  /// Case 2: a column past the source line underlines the whole statement
  /// rather than clamping to a plausible-looking character.
  #[test]
  fn underlines_the_whole_span_when_the_column_misses() {
    let d = run(&message(2, 99));
    assert_eq!(d.span, (33, 40));
    assert_eq!(&SDX[d.span.0..d.span.1], "a + \"x\"");
  }

  /// Case 3: a synthetic line is reported against the `.xrs`, with a note.
  #[test]
  fn reports_a_synthetic_line_against_the_generated_file() {
    let d = run(&message(3, 1));
    assert_eq!(d.path, "p.xrs");
    assert_eq!(d.source, XRS);
    let help = d.help.expect("a synthetic line should explain itself");
    assert!(help.contains("Redox generated"), "{help}");
    assert!(help.contains("p.xrs:3:1"), "{help}");
  }

  /// A line past the map's end is not relocated either.
  #[test]
  fn reports_an_unmapped_line_against_the_generated_file() {
    let d = run(&message(99, 1));
    assert_eq!(d.path, "p.xrs");
  }

  /// A diagnostic rustc gave no code for still carries its level, so the
  /// rendered output never prints an empty `error[]`.
  #[test]
  fn falls_back_to_the_level_when_rustc_gave_no_code() {
    let mut m = message(2, 7);
    m.code = None;
    m.level = "warning".to_string();
    assert_eq!(run(&m).code.text(), "warning");
  }
}
