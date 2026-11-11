/// A function type's grouping parentheses, removed — `(impl Fn(char) -> bool)`
/// is `impl Fn(char) -> bool`.
///
/// They are needed to *capture* the type: a type is source text, and the arrow
/// inside `Fn(char) -> bool` would otherwise split the signature at the wrong
/// place. They are the same grouping as doc 18 §2.5's `instance … (Wrapper t)`
/// — for the reader of the signature, carrying no meaning of their own — and
/// `clippy::unnecessary_parens` is right that they do not belong in the Rust.
/// Plan 74 found it on `Cursor::eat_while`, the first ported signature to take
/// a function.
///
/// **Only a single balanced group whose content has a top-level arrow.** That
/// is the one shape the parentheses exist for, and the test is what keeps
/// `(i32, u32)`, `()` and `(dyn Display)` intact — each of those is also a
/// single group, and none of them has the arrow.
pub(crate) fn ungrouped(text: &str) -> String {
  let Some(inner) = text.strip_prefix('(').and_then(|t| t.strip_suffix(')')) else {
    return text.to_string();
  };

  // One group, not two side by side: `(a) -> (b)` must not lose its arrow.
  let chars: Vec<char> = inner.chars().collect();
  let mut depth = 0i32;
  let mut top_level_arrow = false;
  let mut top_level_comma = false;
  let mut i = 0;
  while i < chars.len() {
    // **`->` is one token, and its `>` is not a closing angle bracket.**
    // Counting it as one took the depth negative on the very shape this
    // exists for, so the strip never fired.
    if chars[i] == '-' && chars.get(i + 1) == Some(&'>') {
      top_level_arrow |= depth == 0;
      i += 2;
      continue;
    }
    match chars[i] {
      '(' | '[' | '<' => depth += 1,
      ')' | ']' | '>' => depth -= 1,
      // **A top-level comma means a tuple**, and a tuple keeps its
      // parentheses however its elements are spelled. That is not the same
      // test as the arrow: `(&str, fn(char) -> bool)` has both, and the first
      // version of this stripped it — which the very next file to use one
      // caught, as `expected = in a binding`.
      ',' if depth == 0 => top_level_comma = true,
      _ => {}
    }
    // A closer with no opener means the outer parentheses did not match each
    // other — `(a) -> (b)` is two groups, not one — so the text stands.
    if depth < 0 {
      return text.to_string();
    }
    i += 1;
  }
  if depth != 0 || !top_level_arrow || top_level_comma {
    return text.to_string();
  }
  inner.to_string()
}

#[cfg(test)]
mod ungrouped_tests {
  use super::ungrouped;

  #[test]
  fn a_function_types_grouping_goes() {
    assert_eq!(
      ungrouped("(impl Fn(char) -> bool)"),
      "impl Fn(char) -> bool"
    );
  }

  /// Each of these is also a single balanced group, and none has a top-level
  /// arrow — which is the whole of why the test is the arrow and not the
  /// brackets.
  #[test]
  fn a_tuple_a_unit_and_a_trait_object_keep_theirs() {
    assert_eq!(ungrouped("(i32, u32)"), "(i32, u32)");
    assert_eq!(ungrouped("()"), "()");
    assert_eq!(ungrouped("(dyn Display)"), "(dyn Display)");
  }

  /// **A tuple whose element is a function type has both marks**, so the
  /// comma has to be tested as well as the arrow. `scan_number`'s
  /// `(&str, fn(char) -> bool)` is the type that caught the first version.
  #[test]
  fn a_tuple_holding_a_function_type_keeps_its_parentheses() {
    assert_eq!(
      ungrouped("(&str, fn(char) -> bool)"),
      "(&str, fn(char) -> bool)"
    );
  }

  /// Two groups side by side are not one group, so the arrow between them is
  /// the type's own and the text is left alone.
  #[test]
  fn two_groups_are_not_one() {
    assert_eq!(ungrouped("(a) -> (b)"), "(a) -> (b)");
  }

  #[test]
  fn an_unparenthesised_type_is_untouched() {
    assert_eq!(ungrouped("Vec<Vec<u8>>"), "Vec<Vec<u8>>");
    assert_eq!(ungrouped("&'a str"), "&'a str");
  }
}
