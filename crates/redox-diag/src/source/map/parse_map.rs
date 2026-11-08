use crate::source::map::SourceMap;
use crate::source::map::version::VERSION;
use crate::source::origin::Origin;

/// Read a `.xrs.map` sidecar back.
///
/// Plan 06 (`docs/11 - 06 - Source Map.md`) leaf 4. Every failure is an `Err`
/// with a sentence, never a partial map: a map that silently dropped records
/// would resolve some lines and decline others, and the declined ones are
/// indistinguishable from genuinely synthetic lines. Refusing the whole file
/// makes the caller fall back to reporting against the `.xrs`, which is worse
/// output but not *wrong* output.
///
/// The span is not stored in the sidecar — the consumer (plan 07) needs line
/// and column, and a byte range it would have to recompute the text for adds
/// a field that can disagree with the other three. A parsed origin therefore
/// carries an empty span, which is the one way a parsed map differs from the
/// map the emitter produced.
pub fn parse_map(text: &str) -> Result<SourceMap, String> {
  let mut lines = text.lines();
  let header = lines.next().ok_or("the map is empty")?;

  let mut parts = header.split(' ');
  if parts.next() != Some("redox-map") {
    return Err("not a Redox source map: the header does not begin `redox-map`".to_string());
  }
  let version = parts
    .next()
    .ok_or("the map header has no version")?
    .parse::<u32>()
    .map_err(|_| "the map header's version is not a number".to_string())?;
  if version != VERSION {
    return Err(format!(
      "this map is version {version}; this Redox writes version {VERSION}"
    ));
  }
  let source_path = parts.collect::<Vec<_>>().join(" ");
  if source_path.is_empty() {
    return Err("the map header names no source file".to_string());
  }

  let mut origins = Vec::new();
  for (i, line) in lines.enumerate() {
    if line.is_empty() {
      continue;
    }
    let fields: Vec<&str> = line.split(' ').collect();
    let expected = i + 1;
    let gen_line: usize = fields
      .first()
      .and_then(|f| f.parse().ok())
      .ok_or_else(|| format!("record {expected} does not begin with a line number"))?;
    if gen_line != expected {
      return Err(format!(
        "record {expected} claims to be line {gen_line}; the records must be in order \
         with no gaps"
      ));
    }

    match fields.as_slice() {
      [_, "-"] => origins.push(None),
      [_, src_line, src_col, gen_col] => {
        let num = |f: &str, what: &str| {
          f.parse::<usize>()
            .map_err(|_| format!("record {expected}'s {what} is not a number"))
        };
        let origin = Origin {
          span: (0, 0),
          src_line: num(src_line, "source line")?,
          src_col: num(src_col, "source column")?,
          gen_col: num(gen_col, "generated column")?,
        };
        if origin.src_line == 0 || origin.src_col == 0 || origin.gen_col == 0 {
          return Err(format!(
            "record {expected} has a zero line or column; both are 1-based"
          ));
        }
        origins.push(Some(origin));
      }
      _ => {
        return Err(format!(
          "record {expected} has {} fields; expected 2 (`n -`) or 4 \
           (`n src_line src_col gen_col`)",
          fields.len()
        ));
      }
    }
  }

  Ok(SourceMap::new(source_path, origins))
}

#[cfg(test)]
mod tests {
  use super::parse_map;
  use crate::source::map::SourceMap;
  use crate::source::map::render_map::render_map;
  use crate::source::origin::Origin;

  fn sample() -> SourceMap {
    SourceMap::new(
      "examples/phase1.sdx",
      vec![
        Some(Origin {
          span: (0, 0),
          src_line: 1,
          src_col: 1,
          gen_col: 4,
        }),
        None,
        Some(Origin {
          span: (0, 0),
          src_line: 2,
          src_col: 3,
          gen_col: 5,
        }),
      ],
    )
  }

  #[test]
  fn round_trips() {
    let text = render_map(&sample());
    assert_eq!(parse_map(&text).expect("should parse"), sample());
  }

  #[test]
  fn rejects_a_foreign_file() {
    let err = parse_map("{\"version\":3}\n").expect_err("should fail");
    assert!(err.contains("does not begin"), "{err}");
  }

  #[test]
  fn rejects_a_future_version() {
    let err = parse_map("redox-map 99 a.sdx\n").expect_err("should fail");
    assert!(err.contains("version 99"), "{err}");
  }

  #[test]
  fn rejects_a_header_with_no_source() {
    let err = parse_map("redox-map 1\n").expect_err("should fail");
    assert!(err.contains("names no source"), "{err}");
  }

  /// Records are positional, so a gap would shift every line after it. Better
  /// to refuse the file than to resolve the wrong line.
  #[test]
  fn rejects_records_out_of_order() {
    let err = parse_map("redox-map 1 a.sdx\n1 1 1 1\n3 2 1 1\n").expect_err("should fail");
    assert!(err.contains("must be in order"), "{err}");
  }

  #[test]
  fn rejects_a_malformed_record() {
    let err = parse_map("redox-map 1 a.sdx\n1 2 3\n").expect_err("should fail");
    assert!(err.contains("3 fields"), "{err}");
  }

  #[test]
  fn rejects_a_zero_column() {
    let err = parse_map("redox-map 1 a.sdx\n1 1 0 1\n").expect_err("should fail");
    assert!(err.contains("1-based"), "{err}");
  }

  #[test]
  fn parses_a_map_with_no_records() {
    let map = parse_map("redox-map 1 a.sdx\n").expect("should parse");
    assert!(map.is_empty());
    assert_eq!(map.source_path, "a.sdx");
  }
}
