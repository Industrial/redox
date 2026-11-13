//! Every palette pairing the site and the themes actually use clears WCAG AA.
//!
//! Identity and documentation site
//! (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 10.
//!
//! The palette was chosen for the identity study, where every pairing was
//! large display type on its own swatch. The book and the editor themes use it
//! for body text, line numbers, strings and punctuation, which is a far
//! stricter requirement — and three pairings did not survive the move:
//!
//! | pairing | was | needed |
//! |---|---|---|
//! | `ink-faint` on `ground-2`, light | 2.69 | 4.5 |
//! | `ink-faint` on `ground-2`, dark | 3.65 | 4.5 |
//! | `string` as `zinc`, light | 2.93 | 4.5 |
//!
//! `ink-faint` was darkened and lightened to clear it; `string` was remapped
//! off `zinc` entirely, because solving for a readable zinc produced a value
//! within one hex of `ink-faint` and would have made strings and punctuation
//! the same colour. Zinc is the mark's second metal and is not a text colour.
//!
//! This test exists so that is a standing property rather than a thing that
//! was true once. A palette edit that darkens a ground or brightens a role
//! fails here.

use std::path::PathBuf;

/// sRGB channel to linear, per WCAG 2.1 relative luminance.
fn channel(c: u8) -> f64 {
  let c = f64::from(c) / 255.0;
  if c <= 0.03928 {
    c / 12.92
  } else {
    ((c + 0.055) / 1.055).powf(2.4)
  }
}

fn luminance(hex: &str) -> f64 {
  let h = hex.trim_start_matches('#');
  let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).expect("a hex pair");
  0.2126 * channel(byte(0)) + 0.7152 * channel(byte(2)) + 0.0722 * channel(byte(4))
}

fn ratio(a: &str, b: &str) -> f64 {
  let (la, lb) = (luminance(a), luminance(b));
  let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
  (hi + 0.05) / (lo + 0.05)
}

fn tokens() -> serde_json::Value {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../design/tokens/tokens.json");
  serde_json::from_str(&std::fs::read_to_string(path).expect("tokens.json")).expect("valid JSON")
}

fn role(t: &serde_json::Value, name: &str, theme: &str) -> String {
  t["roles"][name][theme]
    .as_str()
    .unwrap_or_else(|| panic!("role {name}.{theme} should exist"))
    .to_string()
}

/// Criterion: every text pairing in the book's chrome clears 4.5:1.
///
/// Hairlines are deliberately absent from this list. WCAG 1.4.11 covers UI
/// components and meaningful graphics, not dividers — asserting 3:1 on
/// `rule` was my own error during the audit, and the palette was right.
#[test]
fn the_pages_text_pairings_clear_wcag_aa() {
  let t = tokens();
  // (foreground, surface, what it is)
  for (fg, bg, what) in [
    ("ink", "ground", "body text"),
    ("ink-2", "ground", "secondary text"),
    ("ink-mute", "ground", "captions"),
    ("ink-faint", "ground", "line numbers and the quietest text"),
    ("arc", "ground", "links"),
    ("copper", "ground", "accents and the active line number"),
    ("ink", "ground-2", "text on a code block"),
  ] {
    for theme in ["light", "dark"] {
      let r = ratio(&role(&t, fg, theme), &role(&t, bg, theme));
      assert!(
        r >= 4.5,
        "{what}: {fg} on {bg} in the {theme} theme is {r:.2}:1, below AA's 4.5:1"
      );
    }
  }
}

/// Criterion: every syntax class clears 4.5:1 on *both* code surfaces.
///
/// Code sits on `ground-2` in the book and on `ground` in an editor, so the
/// stricter of the two is the bar. Testing only one surface is how the first
/// pass passed while punctuation was at 2.69 on the other.
#[test]
fn every_syntax_class_clears_wcag_aa_on_both_code_surfaces() {
  let t = tokens();
  let classes = t["syntax"].as_object().expect("a syntax block");
  assert!(
    classes.len() >= 10,
    "the syntax block should map every class"
  );

  for (class, spec) in classes {
    let r = spec["role"].as_str().expect("a role");
    for theme in ["light", "dark"] {
      let fg = role(&t, r, theme);
      let worst =
        ratio(&fg, &role(&t, "ground", theme)).min(ratio(&fg, &role(&t, "ground-2", theme)));
      assert!(
        worst >= 4.5,
        "syntax class {class} uses {r}, which is {worst:.2}:1 in the {theme} \
         theme against the stricter of the two code surfaces"
      );
    }
  }
}

/// Criterion: zinc is not used as a text colour anywhere.
///
/// It is 2.93:1 on white — the mark's second metal, legible as a 42x7 plate
/// and not as six-point punctuation. This is the finding that is easiest to
/// undo by accident, because zinc is the obvious choice for "the other
/// Voltaic colour" and reads fine in a swatch.
#[test]
fn zinc_is_never_used_for_text() {
  let t = tokens();
  for (class, spec) in t["syntax"].as_object().expect("a syntax block") {
    assert_ne!(
      spec["role"].as_str(),
      Some("zinc"),
      "syntax class {class} maps to zinc, which is 2.93:1 on white; zinc is \
       for the mark, not for text"
    );
  }
}
