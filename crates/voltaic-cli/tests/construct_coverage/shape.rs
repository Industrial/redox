//! The claims a token run cannot express.
//!
//! Split out of `construct_coverage.rs` at plan 67 — see
//! [`corpus`](super::corpus)'s module docs for the division. Every function
//! here is a `Witness::Shape` predicate, and each one's doc comment records
//! what made a token pattern insufficient, because several of them replaced a
//! pattern that was green and hollow.

use std::collections::BTreeMap;

use voltaic_lexer::{StrPart, TokenKind, TriviaKind};

use crate::corpus::Source;

/// A string literal carrying an interpolation.
pub(crate) fn has_interpolation(file: &Source) -> bool {
  file.tokens.iter().any(|token| match &token.kind {
    TokenKind::Str { parts, .. } => parts
      .iter()
      .any(|part| matches!(part, StrPart::Interp { .. })),
    _ => false,
  })
}

/// A `"""` block string, told from an ordinary one by its own delimiters.
pub(crate) fn has_triple_quoted(file: &Source) -> bool {
  file.tokens.iter().any(|token| {
    matches!(token.kind, TokenKind::Str { .. }) && file.span_text(token).starts_with("\"\"\"")
  })
}

pub(crate) fn has_ident(file: &Source) -> bool {
  file
    .tokens
    .iter()
    .any(|token| matches!(token.kind, TokenKind::Ident(_)))
}

/// `r#type` — an identifier whose spelling is a Rust keyword.
pub(crate) fn has_raw_ident(file: &Source) -> bool {
  file.tokens.iter().any(|token| {
    matches!(token.kind, TokenKind::Ident(_)) && file.span_text(token).starts_with("r#")
  })
}

fn has_trivia(file: &Source, kinds: &[TriviaKind]) -> bool {
  file.trivia().any(|kind| kinds.contains(kind))
}

pub(crate) fn has_line_comment(file: &Source) -> bool {
  has_trivia(file, &[TriviaKind::LineComment])
}

pub(crate) fn has_fenced_comment(file: &Source) -> bool {
  has_trivia(file, &[TriviaKind::FencedComment])
}

pub(crate) fn has_doc_comment(file: &Source) -> bool {
  has_trivia(file, &[TriviaKind::DocComment, TriviaKind::InnerDocComment])
}

pub(crate) fn has_outer_attribute(file: &Source) -> bool {
  has_trivia(file, &[TriviaKind::Attribute])
}

pub(crate) fn has_inner_attribute(file: &Source) -> bool {
  has_trivia(file, &[TriviaKind::InnerAttribute])
}

/// Whether the token at `index` opens a line.
fn starts_a_line(file: &Source, index: usize) -> bool {
  index == 0
    || matches!(
      file.tokens[index - 1].kind,
      TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
    )
}

/// The two spellings of the type colon.
///
/// `::` is doc 18 §5.2's, and plan 52 is replacing it with a single `:` — the
/// paths keep `::`. Both are accepted here on purpose: this gate asserts a
/// property of the *corpus*, and making it also assert which side of plan 52's
/// migration the tree is on would make it fail for a reason that has nothing
/// to do with coverage. The spacing, not the character count, is what tells a
/// type colon from a path in either spelling.
const TYPE_COLON: &[&str] = &[":", "::"];

/// A signature line: a name, then a **spaced** type colon.
///
/// The spacing is the whole distinction. Doc 18 §5.2 makes a tight `::` a path
/// and a spaced colon a signature or an ascription, which is why the lexer
/// records `ws_before` at all — so `i32::max` is not read as a declaration.
pub(crate) fn has_signature_line(file: &Source) -> bool {
  (1..file.tokens.len()).any(|index| {
    TYPE_COLON.contains(&file.spellings[index].as_str())
      && file.tokens[index].ws_before
      && matches!(file.tokens[index - 1].kind, TokenKind::Ident(_))
      && starts_a_line(file, index - 1)
  })
}

/// A marker consumed between the type colon and the spine — `const`, `async`
/// or `unsafe` in `name : const u32 -> u32`.
///
/// Doc 18 §2.2 puts it there rather than inside the first type, because a type
/// is verbatim text and `const u32` is not a Rust type.
fn has_signature_marker(file: &Source, marker: &str) -> bool {
  (1..file.spellings.len()).any(|index| {
    file.spellings[index] == marker && TYPE_COLON.contains(&file.spellings[index - 1].as_str())
  })
}

pub(crate) fn has_const_marker(file: &Source) -> bool {
  has_signature_marker(file, "const")
}

pub(crate) fn has_async_marker(file: &Source) -> bool {
  has_signature_marker(file, "async")
}

pub(crate) fn has_unsafe_marker(file: &Source) -> bool {
  has_signature_marker(file, "unsafe")
}

/// A module-scope `const` or `static`, which is a keyword, a name and a type
/// colon.
fn has_typed_item(file: &Source, keyword: &str) -> bool {
  (0..file.spellings.len().saturating_sub(2)).any(|index| {
    file.spellings[index] == keyword
      && matches!(file.tokens[index + 1].kind, TokenKind::Ident(_))
      && TYPE_COLON.contains(&file.spellings[index + 2].as_str())
  })
}

pub(crate) fn has_const_item(file: &Source) -> bool {
  has_typed_item(file, "const")
}

pub(crate) fn has_static_item(file: &Source) -> bool {
  has_typed_item(file, "static")
}

/// A tight `::`, which is a path.
pub(crate) fn has_tight_path(file: &Source) -> bool {
  (1..file.tokens.len()).any(|index| file.spellings[index] == "::" && !file.tokens[index].ws_before)
}

/// `data Marker = Marker` — one constructor, named for the type, no fields.
///
/// Doc 18 §2.4 reads a *shape* off the constructors, so a unit struct has no
/// keyword of its own and cannot be matched by one: it is the absence of
/// fields and of alternatives that makes it one.
pub(crate) fn has_unit_struct(file: &Source) -> bool {
  file.spellings.windows(5).any(|window| {
    window[0] == "data"
      && window[2] == "="
      && window[1] == window[3]
      && (window[4] == "«nl»" || window[4] == "«eof»")
  })
}

/// A `const` declared inside a `class` or `instance` body.
///
/// An associated constant is spelled exactly as a module-scope one — doc 18
/// §2.5 gives it no marker — so only its position tells them apart, and the
/// layout tokens are what make the position readable.
pub(crate) fn has_associated_const(file: &Source) -> bool {
  let mut depth = 0usize;
  let mut inside = false;
  for (index, spelling) in file.spellings.iter().enumerate() {
    match file.tokens[index].kind {
      TokenKind::Indent if inside => depth += 1,
      TokenKind::Dedent if inside => {
        depth = depth.saturating_sub(1);
        if depth == 0 {
          inside = false;
        }
      }
      _ => {}
    }
    if !inside && (spelling == "class" || spelling == "instance") {
      inside = true;
      depth = 0;
    } else if inside && depth > 0 && spelling == "const" {
      return true;
    }
  }
  false
}

/// The same name bound twice in one file, which is what shadowing is.
pub(crate) fn has_shadowing(file: &Source) -> bool {
  let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
  for index in 0..file.tokens.len().saturating_sub(1) {
    if !matches!(file.tokens[index].kind, TokenKind::Ident(_))
      || !starts_a_line(file, index)
      || file.spellings[index + 1] != "="
    {
      continue;
    }
    *seen.entry(file.spellings[index].as_str()).or_default() += 1;
  }
  seen.values().any(|count| *count > 1)
}

/// Whether any declaration keyword begins a line inside a **function body**.
///
/// Plan 55 leaf 5's witness, and it took three tries, each hollow in a
/// different way and each proved so by moving `examples/local.sdx` aside and
/// watching this gate stay green:
///
/// 1. `Toks(["import", "«id»"])` — matched a **module-scope** `import`.
/// 2. "a declaration keyword at depth above zero" — matched an inline
///    `module`'s items, which the inventory grades as *Modules, inline and
///    nested*.
/// 3. the same, excluding `module` bodies — matched `examples/class.sdx`'s
///    `const SIDES : u32`, an **associated constant**, graded as *Associated
///    types and constants*.
///
/// The lesson is in the pattern: "a declaration keyword, indented" is not what
/// distinguishes a local declaration. What distinguishes it is **what opened
/// the block**. A function body is opened by an equation, whose line begins
/// with a *name*; every other indented block in the language is opened by a
/// line beginning with a declaration keyword. So this is one test rather than
/// a growing list of exclusions — which is also what makes it survive a row
/// that adds a new kind of block.
///
/// Depth comes from `Indent`/`Dedent` rather than from the text, for the
/// reason `indentation_round_trip` records: a continuation line is indented
/// text at its parent's token depth.
///
/// `test` and `module` are absent from the keyword list, matching the parser.
/// `RDX0112` refuses a local `test`, because rustc does not collect a nested
/// `#[test]`; a bodiless `module` loads a file, which a block cannot do.
pub(crate) fn has_a_local_declaration(file: &Source) -> bool {
  const KEYWORDS: &[&str] = &[
    "data", "newtype", "union", "type", "class", "instance", "errors", "import", "const", "static",
  ];
  any_line_head(file, |index, in_a_body| {
    in_a_body && KEYWORDS.contains(&file.spellings[index].as_str())
  })
}

/// Ask `test` at the first token of every line, with whether the enclosing
/// block is a **function body**.
///
/// The walk [`has_a_local_declaration`] argues for, shared since plan 55 leaf
/// 7 gave it a second caller. Depth comes from `Indent`/`Dedent` rather than
/// from the text, for the reason `indentation_round_trip` records: a
/// continuation line is indented text at its parent's token depth.
fn any_line_head(file: &Source, mut test: impl FnMut(usize, bool) -> bool) -> bool {
  /// Heads of every line that opens a block which is *not* a function body.
  const OPENS_A_NON_BODY: &[&str] = &[
    "module", "class", "instance", "errors", "data", "newtype", "union", "extern", "test",
  ];

  // One entry per open block: true when the block is a function body.
  let mut is_body: Vec<bool> = Vec::new();
  let mut line_head: Option<&str> = None;

  for index in 0..file.tokens.len() {
    match file.tokens[index].kind {
      TokenKind::Indent => {
        let opener = line_head.unwrap_or("");
        is_body.push(!OPENS_A_NON_BODY.contains(&opener));
      }
      TokenKind::Dedent => {
        is_body.pop();
      }
      _ => {
        if starts_a_line(file, index) {
          line_head = Some(file.spellings[index].as_str());
          if test(index, is_body.last() == Some(&true)) {
            return true;
          }
        }
      }
    }
  }
  false
}

/// Whether a **binding's value** is an indented block — plan 55 leaf 7.
///
/// `Toks(["do"])` witnessed *Blocks as expressions* until the leaf retired the
/// keyword, and it had to go for a second reason worth recording: this gate
/// reads `tests/ui/` too, so `tests/ui/reserved_do.sdx` — the fixture that
/// asserts `do` is **refused** — kept the old witness green. A token run
/// cannot tell a construct from its own refusal fixture.
///
/// Nor can one tell a block-valued binding from a clause body: `inner =` and
/// `tier n =` with a block under each are the same four tokens. What separates
/// them is **what opened the enclosing block**, which is
/// [`has_a_local_declaration`]'s finding one leaf earlier, and that walk is
/// [`any_line_head`] — shared rather than copied.
pub(crate) fn has_a_block_valued_binding(file: &Source) -> bool {
  any_line_head(file, |index, in_a_body| {
    in_a_body
      && matches!(file.tokens[index].kind, TokenKind::Ident(_))
      && file.spellings.get(index + 1).is_some_and(|s| s == "=")
      && file
        .tokens
        .get(index + 2)
        .is_some_and(|t| t.kind == TokenKind::Newline)
      && file
        .tokens
        .get(index + 3)
        .is_some_and(|t| t.kind == TokenKind::Indent)
  })
}

/// An **associated function** declared in a `class` or `instance` body — a
/// member with a spine and no receiver at all.
///
/// Plan 67 leaf 4. What makes it different is the *absence* of the marker,
/// which is why this cannot be a token run: `Toks(["::"])` would be satisfied
/// by any path, and a call such as `i32::max 1 9` or `f64::from n` reaches a
/// library's associated function rather than one this corpus declares. So the
/// witness is the declaration — a member signature carrying `->` and no
/// `self` — which is the half only this language can supply.
///
/// `->` is what separates it from an associated **constant**: `const SIDES :
/// u32` is also a self-less member, and it has its own row.
pub(crate) fn has_an_associated_function(file: &Source) -> bool {
  let mut inside = false;
  let mut depth = 0usize;
  let mut index = 0usize;
  while index < file.tokens.len() {
    match file.tokens[index].kind {
      TokenKind::Indent if inside => depth += 1,
      TokenKind::Dedent if inside => {
        depth = depth.saturating_sub(1);
        if depth == 0 {
          inside = false;
        }
      }
      _ => {}
    }
    let spelling = file.spellings[index].as_str();
    if !inside && (spelling == "class" || spelling == "instance") {
      inside = true;
      depth = 0;
      index += 1;
      continue;
    }
    // A member's signature line: a name at the head of the line, then a
    // spaced type colon. Read to the line break, and ask what the spine held.
    if inside
      && depth > 0
      && starts_a_line(file, index)
      && matches!(file.tokens[index].kind, TokenKind::Ident(_))
      && file
        .spellings
        .get(index + 1)
        .is_some_and(|s| TYPE_COLON.contains(&s.as_str()))
      && file.tokens[index + 1].ws_before
    {
      let mut arrow = false;
      let mut receiver = false;
      let mut at = index + 2;
      while at < file.tokens.len()
        && !matches!(
          file.tokens[at].kind,
          TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
        )
      {
        match file.spellings[at].as_str() {
          "->" => arrow = true,
          "self" => receiver = true,
          _ => {}
        }
        at += 1;
      }
      if arrow && !receiver {
        return true;
      }
      index = at;
      continue;
    }
    index += 1;
  }
  false
}

/// A **marked receiver** that is not the first parameter — plan 67.
///
/// `self` in a type is the receiver at whatever position it is written, and
/// the whole of plan 67 §1.2 is that the position carries no meaning. So the
/// witness is the shape the old positional rule could not express: a signature
/// line whose spine reaches `self` **after** at least one `->`.
///
/// Built from what makes the construct different rather than from its own
/// spelling, which is plan 66's lesson about hollow witnesses: `Toks(["&",
/// "self"])` would be satisfied by `area : &self -> f64`, the one-argument
/// receiver-first shape the corpus has had all along, and the decoy would be
/// the row it is meant to distinguish itself from.
pub(crate) fn has_a_late_receiver(file: &Source) -> bool {
  (1..file.spellings.len()).any(|index| {
    if file.spellings[index] != "self" {
      return false;
    }
    // Walk back to the type colon that opened this signature, counting the
    // arrows on the way. A line break before one means this `self` is in an
    // expression rather than in a type.
    let mut arrows = 0usize;
    for before in (0..index).rev() {
      match file.spellings[before].as_str() {
        "->" => arrows += 1,
        ":" if file.tokens[before].ws_before => return arrows > 0,
        "«nl»" | "«indent»" | "«dedent»" | "=" => return false,
        _ => {}
      }
    }
    false
  })
}

/// A **class with parameters of its own** — plan 67 leaf 2, `trait From<T>`'s
/// shape.
///
/// `class` followed by its name and then one or more further names on the same
/// line. The name itself is not enough to tell this from the old surface,
/// where `class Shape t` spelled the *implementing* type — so what makes it
/// different is that the parameter survives into a member's type as something
/// other than the receiver, and the shape that proves the head is the trait's
/// is **two impls of it for one type**. That is
/// [`has_two_impls_for_one_type`].
pub(crate) fn has_a_parameterised_class(file: &Source) -> bool {
  (0..file.spellings.len().saturating_sub(3)).any(|index| {
    file.spellings[index] == "class"
      && starts_a_line(file, index)
      && matches!(file.tokens[index + 1].kind, TokenKind::Ident(_))
      && matches!(file.tokens[index + 2].kind, TokenKind::Ident(_))
  })
}

/// **Two `instance` heads naming the same class and the same implementing
/// type** — plan 67 §1.4's closed gap.
///
/// This is the shape that `E0049` made unwritable: a class with one head slot
/// spent it on the implementing type, so a second lowercase name became a
/// *method* generic and `impl From<u32>` and `impl From<f64>` for one type
/// could not both exist.
///
/// Matched on the head's **first and last** names, which is where plan 67 leaf
/// 3 put the class and the implementing type. Two heads that agree on both and
/// differ in between are two impls of one trait for one type, and nothing else
/// is.
pub(crate) fn has_two_impls_for_one_type(file: &Source) -> bool {
  let mut heads: Vec<(&str, &str)> = Vec::new();
  for index in 0..file.spellings.len() {
    if file.spellings[index] != "instance" || !starts_a_line(file, index) {
      continue;
    }
    let mut names: Vec<&str> = Vec::new();
    for at in index + 1..file.spellings.len() {
      if !matches!(file.tokens[at].kind, TokenKind::Ident(_)) {
        break;
      }
      names.push(file.spellings[at].as_str());
    }
    // Two names are a trait and its type with no arguments; the gap this
    // witnesses needs at least three, so the pair can differ in between.
    if names.len() >= 3 {
      heads.push((names[0], names[names.len() - 1]));
    }
  }
  heads
    .iter()
    .enumerate()
    .any(|(at, head)| heads[at + 1..].contains(head))
}
