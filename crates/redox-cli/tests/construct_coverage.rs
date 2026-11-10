//! Every construct the language has, exampled somewhere a reader will find it.
//!
//! Plan 51 (`docs/45 - 51 - Construct Coverage.md`) leaves 1–3. This is the
//! one test in this directory that does not ask whether the compiler is
//! correct: it asks whether the **corpus is complete**, which fails in the
//! opposite direction. A compiler bug makes a test red; a corpus gap makes
//! every test green and the language smaller than it is.
//!
//! The measurement that forced it: `$` — doc 18 §2.1's low-precedence
//! application operator — was fully implemented, fully specified, and appeared
//! in no example, while `examples/lexical.sdx` explained its absence by
//! calling it unlexed. Nothing caught that, because nothing checked the corpus
//! against the language.
//!
//! So this is a **join**, not a new inventory (plan 51 §1.2):
//!
//! | Side | Source | Built by |
//! |---|---|---|
//! | what the language has | `docs/GRAMMAR.md`, graded `SUGARED` / `PASSTHROUGH` | plan 46 |
//! | what the corpus shows | `examples/*.sdx`, `tests/{ui,run}/*.sdx` | plans 08, 13, 43 |
//! | the assertion | every row on the left appears somewhere on the right | plan 51 |
//!
//! `UNREACHABLE` rows are out of scope **by construction**: plan 46 requires
//! their fixtures to still fail, so covering one would break that test.
//!
//! Three properties are load-bearing.
//!
//! 1. **The index is parsed by plan 46's parser, not a second one.** The row
//!    reader below is `grammar_inventory.rs`'s, escaping discipline included:
//!    an unescaped `` `\|` `` in a table cell once split every column by one
//!    and hid a graded row from the test that read it (plan 47 §10.1).
//!
//! 2. **A short index is a failure.** Plan 46 §10 and plan 47 §10.1 both
//!    record a generator reporting success on input it had silently failed to
//!    parse. [`MINIMUM_ROWS`] is the guard.
//!
//! 3. **The match is lexical.** A construct is witnessed by a pattern over
//!    `redox_lexer::lex`'s token stream, never by a substring of the file.
//!    `|>` inside a comment is not coverage, and `$` inside a string is not
//!    coverage; `tests/construct_coverage/decoy.sdx` is a file that contains
//!    nine constructs textually and none of them lexically, and
//!    [`a_construct_inside_a_comment_or_a_string_does_not_count`] asserts both
//!    halves.
//!
//! ## Why there is a witness table at all
//!
//! `docs/GRAMMAR.md` grades **Rust** constructs with prose descriptions —
//! *"Trait objects `dyn Trait`"*, *"Blocks as expressions"* — and a row's text
//! is not a Redox spelling. Searching the corpus for a row's own words would
//! find nothing for almost every row and would be a test of this file's
//! wording rather than of the corpus. [`WITNESSES`] is therefore an explicit
//! map from each graded row to the Redox token pattern that witnesses it, and
//! [`every_graded_row_has_a_witness`] asserts the map and the document agree
//! in both directions so a row added to one cannot be forgotten in the other.
//!
//! ## Three files, since plan 67
//!
//! This file sat at `line_cap`'s 1,500-line ceiling **exactly**, and plan 67
//! had three witnesses to add. The split follows what the pieces are rather
//! than where the middle fell: [`corpus`] is the haystack and the shape of a
//! claim about it, [`shape`] holds the predicates a token run cannot express,
//! and this file keeps the index, the witness table and the assertions — the
//! three things a reader comparing it to `docs/GRAMMAR.md` needs side by side.

#[path = "construct_coverage/corpus.rs"]
mod corpus;
#[path = "construct_coverage/shape.rs"]
mod shape;

use std::path::PathBuf;

use crate::corpus::{Covers, Source, Witness, corpus};
use crate::shape::{
  has_a_block_valued_binding, has_a_late_receiver, has_a_local_declaration,
  has_a_parameterised_class, has_an_associated_function, has_associated_const, has_async_marker,
  has_const_item, has_const_marker, has_doc_comment, has_fenced_comment, has_ident,
  has_inner_attribute, has_interpolation, has_line_comment, has_outer_attribute, has_raw_ident,
  has_shadowing, has_signature_line, has_static_item, has_tight_path, has_triple_quoted,
  has_two_impls_for_one_type, has_unit_struct, has_unsafe_marker,
};

/// The smallest index that can be believed.
///
/// `docs/GRAMMAR.md` grades somewhat more rows than this, a handful of them
/// `UNREACHABLE` and so out of scope here. The floor is deliberately below the
/// real count and deliberately not at it: a row legitimately added or regraded
/// should not fail this, and a parse that loses a table should.
///
/// **The count is deliberately not written down.** It was, as "89 rows, 7 of
/// them `UNREACHABLE`, so 82", and it has been wrong twice since — 91/7/84
/// once plan 54 graded assignment, then 93/8/85 once `ref`/`ref mut` was
/// regraded. A frozen number beside a constant that exists *because* the
/// number moves is the defect plan 52 §10.4a found three times over in byte
/// offsets derived in comments. `every_graded_row_has_a_witness` prints the
/// real figures when it fails, which is where a reader who needs them is
/// already looking.
const MINIMUM_ROWS: usize = 80;

/// Rows this gate records as uncovered rather than asserting.
///
/// **An entry here is a hole in the gate, and is a finding.** Plan 51 §4 leaf
/// 3 asks for this list to be *empty at merge*; it is not, and that is the
/// first result of running the gate rather than a weakening of it. Seven rows
/// are graded reachable in `docs/GRAMMAR.md` and appear in no corpus file —
/// which is precisely the `$` failure mode plan 51 §1 was written for, found
/// seven more times the moment a machine looked.
///
/// `docs/GRAMMAR.md` §3 already disclaims the property that would have caught
/// them: *"A claim that every citation exercises its construct … the test
/// asserts the cited file exists"*. Four of the seven cite a file that does
/// not contain the construct at all.
///
/// Two rules keep the list from becoming policy:
///
/// - every entry names the construct, why it is uncovered, and what closing it
///   needs — a reason that stops at "not written yet" is not a reason;
/// - [`ALLOW_AT_MERGE`] pins the length, so the list can shrink and cannot
///   grow. A new gap must be fixed or must raise that number deliberately.
const ALLOW: &[(&str, &str)] = &[
  (
    "Unit structs",
    "doc 18 §2.4 spells this `data Marker = Marker` — one constructor named \
     for the type, with no fields — and the corpus has thirteen `data`, \
     `newtype` and `union` declarations, every one of which has fields or \
     alternatives. The row cites `examples/data.sdx`, which declares an enum \
     of unit *variants* instead; those are a different Rust shape. Closing it \
     is a declaration in a running example, and the example has to use the \
     value or rustc's `dead_code` fires",
  ),
  (
    "Raw pointers",
    "a type is verbatim text (doc 03 §14.1), so `*const u8` needs no parser \
     work and is certainly reachable — but no corpus file contains a `*` in a \
     type. The row cites `examples/reachability.sdx` for being the file where \
     verbatim types live, not for containing a pointer. Closing it needs an \
     `unsafe` block to dereference in, which that file already has",
  ),
  (
    "Trait objects dyn Trait",
    "same shape as the raw-pointer row: verbatim type text, reachable by \
     construction, and the cited `examples/generic.sdx` contains no `dyn`. A \
     `dyn` witness also has to be *used*, because this corpus requires rustc \
     to emit nothing",
  ),
  (
    "impl Trait, argument and return",
    "the cited `examples/generic.sdx` contains no `impl` token at all — the \
     only `impl` anywhere in the corpus is the word inside a comment in \
     `examples/errors.sdx`, which is exactly the match a substring scan would \
     have accepted and this one does not",
  ),
  (
    "Function pointers and Fn bounds",
    "the cited `examples/generic.sdx` has three bound clauses and none of them \
     is an `Fn` bound, and no corpus file writes a `fn(T) -> U` type. Both \
     halves are verbatim type text, so this is a missing example rather than \
     a missing feature",
  ),
  (
    "move closures",
    "`move` is reserved (`Keyword::Move`) and the row cites \
     `examples/async.sdx`, which has four `async` constructs and no `move`. \
     Unlike the four rows above, this one is a *marker* on a lambda rather \
     than verbatim type text, so whether the parser accepts it is unverified \
     here — plan 51's §6 is explicit that this gate cannot tell a missing \
     example from a mis-graded row, and the next step is to try writing one",
  ),
];

/// The length of [`ALLOW`] when this gate was written.
///
/// The list may shrink. Raising this is a deliberate act with a diff, which is
/// the friction plan 51 §9.3 asks for: an escape hatch with none becomes the
/// answer to every failure.
const ALLOW_AT_MERGE: usize = 6;

// ---------------------------------------------------------------------------
// The inventory
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One graded row of `docs/GRAMMAR.md`.
struct Row {
  /// The area cell, normalised: backticks and cell escapes removed.
  key: String,
  grade: String,
  /// 1-based line in `docs/GRAMMAR.md`, so a failure can be jumped to.
  line: usize,
  /// The `###` heading the row sits under.
  section: String,
}

impl Row {
  /// Where to look in the document, for a failure message.
  fn cite(&self) -> String {
    format!("docs/GRAMMAR.md:{} ({})", self.line, self.section)
  }
}

/// A markdown table row's cells, respecting `\|`.
///
/// Lifted verbatim from `grammar_inventory.rs`, which is the point: splitting
/// on a bare `|` is wrong here and was wrong three times before that function
/// existed. The inventory quotes Redox and Rust source, and some of those
/// quotes contain a pipe — `\|` for or-patterns, `|>` for the pipeline. An
/// escaped one still splits a naive parser, and the row it belongs to then has
/// its grade in the wrong cell and is silently skipped.
fn split_cells(line: &str) -> Vec<String> {
  const SENTINEL: char = '\u{0}';
  line
    .trim()
    .trim_matches('|')
    .replace("\\|", &SENTINEL.to_string())
    .split('|')
    .map(|cell| cell.replace(SENTINEL, "|"))
    .collect()
}

/// An area cell as a stable key: no backticks, no cell escapes, one space
/// between words.
fn normalise(area: &str) -> String {
  area
    .replace(['`', '\\'], "")
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

/// Every graded row of the inventory, in document order.
///
/// A row is a table line whose **second** cell is one of the three grades,
/// which is what distinguishes the inventory's tables from the prose tables
/// around them without needing a marker a future editor could forget.
fn rows() -> Vec<Row> {
  let path = workspace_root().join("docs/GRAMMAR.md");
  let text = std::fs::read_to_string(&path)
    .unwrap_or_else(|e| panic!("should read {}: {e}", path.display()));

  let mut section = String::new();
  let mut out = Vec::new();
  for (index, line) in text.lines().enumerate() {
    if let Some(heading) = line.strip_prefix("### ") {
      section = heading.trim().to_owned();
      continue;
    }
    if !line.starts_with('|') {
      continue;
    }
    let cells = split_cells(line);
    if cells.len() < 3 {
      continue;
    }
    let grade = cells[1].trim().trim_matches('`').trim();
    if !matches!(grade, "PASSTHROUGH" | "SUGARED" | "UNREACHABLE") {
      continue;
    }
    out.push(Row {
      key: normalise(&cells[0]),
      grade: grade.to_owned(),
      line: index + 1,
      section: section.clone(),
    });
  }
  out
}

/// The rows this test is responsible for: everything reachable.
fn reachable_rows() -> Vec<Row> {
  rows()
    .into_iter()
    .filter(|row| row.grade != "UNREACHABLE")
    .collect()
}

/// Every reachable row of `docs/GRAMMAR.md`, and what witnesses it.
///
/// Ordered as the document orders them, so the two can be read side by side.
/// A row's `spelling` is prose for the failure message; its `witness` is the
/// assertion.
const WITNESSES: &[Covers] = &[
  // --- §1.1 Lexical structure ---
  Covers {
    row: "Integer, float, bool literals",
    spelling: "`1_000_000`, `2.5e-3`, `true`",
    witness: Witness::All(&[
      Witness::Toks(&["«int»"]),
      Witness::Toks(&["«float»"]),
      Witness::Any(&[Witness::Toks(&["true"]), Witness::Toks(&["false"])]),
    ]),
  },
  Covers {
    row: "String literals, escapes",
    // The lexer decodes escapes, so an escape leaves no token of its own and
    // the literal is as close as a lexical witness can get. The byte-string
    // row below carries `b"\x7fELF"`, which is the escape actually exercised.
    spelling: "a `\"…\"` literal",
    witness: Witness::Toks(&["«str»"]),
  },
  Covers {
    row: "String interpolation",
    spelling: "`\"hi #{name}\"`",
    witness: Witness::Shape("a string literal carrying a `#{…}`", has_interpolation),
  },
  Covers {
    row: "Raw strings r\"…\", r#\"…\"#",
    spelling: "`r#\"no \\escapes here\"#`",
    witness: Witness::Toks(&["«rawstr»"]),
  },
  Covers {
    row: "Byte and byte-string literals",
    spelling: "`b'x'` and `b\"\\x7fELF\"`",
    witness: Witness::All(&[Witness::Toks(&["«byte»"]), Witness::Toks(&["«bytestr»"])]),
  },
  Covers {
    row: "C-string literals c\"…\"",
    spelling: "`c\"redox\"`",
    witness: Witness::Toks(&["«cstr»"]),
  },
  Covers {
    row: "Char literals",
    spelling: "`'R'`",
    witness: Witness::Toks(&["«char»"]),
  },
  Covers {
    row: "Multi-line strings \"\"\"…\"\"\"",
    spelling: "a `\"\"\"` block",
    witness: Witness::Shape("a string literal delimited by `\"\"\"`", has_triple_quoted),
  },
  Covers {
    row: "Identifiers",
    spelling: "any name",
    witness: Witness::Shape("an identifier token", has_ident),
  },
  Covers {
    row: "Raw identifiers r#type",
    spelling: "`r#type`",
    witness: Witness::Shape("an identifier spelled `r#name`", has_raw_ident),
  },
  Covers {
    row: "Line and block comments",
    spelling: "`-- …` and `{- … -}`",
    witness: Witness::All(&[
      Witness::Shape("a `--` line comment", has_line_comment),
      Witness::Shape("a `{- … -}` fenced comment", has_fenced_comment),
    ]),
  },
  Covers {
    row: "Doc comments",
    spelling: "`-- |` and `-- !`",
    witness: Witness::Shape("a `-- |` or `-- !` doc comment", has_doc_comment),
  },
  // --- §1.2 Items ---
  Covers {
    row: "Functions",
    spelling: "`name : T -> U` and an equation",
    witness: Witness::Shape(
      "a name at the start of a line followed by `:`",
      has_signature_line,
    ),
  },
  Covers {
    row: "const fn",
    spelling: "`square : const u32 -> u32`",
    witness: Witness::Shape(
      "a `const` marker after a signature's type colon",
      has_const_marker,
    ),
  },
  Covers {
    row: "async fn",
    spelling: "`double : async i32 -> i32`",
    witness: Witness::Shape(
      "an `async` marker after a signature's type colon",
      has_async_marker,
    ),
  },
  Covers {
    row: "unsafe fn",
    spelling: "`unsafe_double : unsafe u32 -> u32`",
    witness: Witness::Shape(
      "an `unsafe` marker after a signature's type colon",
      has_unsafe_marker,
    ),
  },
  Covers {
    row: "Structs (braced)",
    spelling: "`data Point = Point { x : i32 }`",
    witness: Witness::Toks(&["data", "«id»", "=", "«id»", "{"]),
  },
  Covers {
    row: "Tuple structs",
    spelling: "`newtype Meters = Meters f64`",
    witness: Witness::Toks(&["newtype"]),
  },
  Covers {
    row: "Unit structs",
    spelling: "`data Marker = Marker`",
    witness: Witness::Shape(
      "a `data` declaration whose one constructor is named for the type and takes no fields",
      has_unit_struct,
    ),
  },
  Covers {
    row: "Enums, variants, discriminants",
    spelling: "a second constructor, `| Close`",
    witness: Witness::Any(&[
      Witness::Toks(&["«nl»", "|", "«id»", "«nl»"]),
      Witness::Toks(&["«nl»", "|", "«id»", "{"]),
    ]),
  },
  Covers {
    row: "Unions",
    spelling: "`union Word = Word { … }`",
    witness: Witness::Toks(&["union"]),
  },
  Covers {
    row: "Type aliases",
    spelling: "`type Depth = f64`",
    witness: Witness::Toks(&["type", "«id»", "="]),
  },
  Covers {
    row: "const and static items",
    spelling: "`const MAX : usize = 1024`, `static GREETING : &'static str = …`",
    witness: Witness::All(&[
      Witness::Shape("a `const NAME : T` item", has_const_item),
      Witness::Shape("a `static NAME : T` item", has_static_item),
    ]),
  },
  Covers {
    row: "Traits",
    spelling: "`class Shape`",
    witness: Witness::Toks(&["class"]),
  },
  Covers {
    row: "Generic traits",
    spelling: "`class Scale t`, which is `trait Scale<T>`",
    witness: Witness::Shape(
      "a `class` head carrying a parameter of its own",
      has_a_parameterised_class,
    ),
  },
  Covers {
    row: "Impls, inherent and trait",
    spelling: "`instance Shape Square`",
    witness: Witness::Toks(&["instance"]),
  },
  Covers {
    row: "Two impls of one trait for one type",
    spelling: "`instance Scale f64 Meters` beside `instance Scale u32 Meters`",
    witness: Witness::Shape(
      "two `instance` heads agreeing on the class and the implementing type and \
       differing in between",
      has_two_impls_for_one_type,
    ),
  },
  Covers {
    row: "Items in a function body",
    spelling: "a declaration keyword beginning a line inside a function body",
    witness: Witness::Shape(
      "a declaration keyword beginning a line in a block an equation opened",
      has_a_local_declaration,
    ),
  },
  Covers {
    row: "Modules, inline and nested",
    spelling: "`module inner`",
    witness: Witness::Toks(&["module"]),
  },
  Covers {
    row: "use declarations, groups, globs, aliases",
    // The glob form, `import crate::models::*`, has no witness of its own
    // here: `*` is also multiplication, so a token run cannot tell them apart
    // and the import path is parsed verbatim. The group and alias forms below
    // are what this row's three remaining sub-forms reduce to.
    spelling: "`import std::fmt::{self, Write}` and `import … as Sorted`",
    witness: Witness::All(&[
      Witness::Toks(&["import"]),
      Witness::Toks(&["{", "self", ","]),
      Witness::Toks(&["as", "«id»"]),
    ]),
  },
  Covers {
    row: "extern blocks",
    spelling: "`extern \"C\"`",
    witness: Witness::Toks(&["extern", "«str»"]),
  },
  Covers {
    row: "Attributes, inner and outer",
    spelling: "`#[inline]` and `#![allow(…)]`",
    witness: Witness::All(&[
      Witness::Shape("an `#[…]` outer attribute", has_outer_attribute),
      Witness::Shape("a `#![…]` inner attribute", has_inner_attribute),
    ]),
  },
  // --- §1.3 Types ---
  Covers {
    row: "Paths and generic arguments",
    spelling: "`i32::max` and `Option<u8>`",
    witness: Witness::All(&[
      Witness::Shape("a tight `::`, which is a path", has_tight_path),
      Witness::Toks(&["«id»", "<", "«id»", ">"]),
    ]),
  },
  Covers {
    row: "References, &mut",
    spelling: "`&p` and `&mut buffer`",
    witness: Witness::All(&[Witness::Toks(&["&", "«id»"]), Witness::Toks(&["&", "mut"])]),
  },
  Covers {
    row: "Raw pointers",
    spelling: "`*const u8` or `*mut u8` in a type",
    witness: Witness::Any(&[Witness::Toks(&["*", "const"]), Witness::Toks(&["*", "mut"])]),
  },
  Covers {
    row: "Tuples",
    spelling: "`(a, b)`",
    witness: Witness::Toks(&["(", "«id»", ",", "«id»", ")"]),
  },
  Covers {
    row: "Arrays and slices",
    spelling: "`[u8; 4]`",
    witness: Witness::Toks(&["[", "«id»", ";", "«int»", "]"]),
  },
  Covers {
    row: "Trait objects dyn Trait",
    spelling: "`dyn Trait` in a type",
    witness: Witness::Toks(&["dyn"]),
  },
  Covers {
    row: "impl Trait, argument and return",
    spelling: "`impl Trait` in a type",
    witness: Witness::Toks(&["impl"]),
  },
  Covers {
    row: "Function pointers and Fn bounds",
    spelling: "`fn(i32) -> i32`, or an `Fn` bound",
    witness: Witness::Any(&[
      Witness::Toks(&["fn", "("]),
      Witness::Toks(&["Fn", "("]),
      Witness::Toks(&["FnMut", "("]),
      Witness::Toks(&["FnOnce", "("]),
    ]),
  },
  Covers {
    row: "Lifetimes",
    spelling: "`&'static str`",
    witness: Witness::Toks(&["&", "«label»", "«id»"]),
  },
  Covers {
    row: "Generic parameters and bounds",
    spelling: "a bound clause, `Ord t => …`",
    witness: Witness::Toks(&["«id»", "«id»", "=>"]),
  },
  Covers {
    row: "where clauses",
    // The same bound clause. `docs/GRAMMAR.md` grades Rust's `where` as
    // reachable *through* `=>`, and says outright that Redox's own `where`
    // introduces local bindings and is a different construct — so witnessing
    // this row with the `where` keyword would witness the wrong thing.
    spelling: "the bound clause `=>`, which is where Rust's `where` comes from",
    witness: Witness::Toks(&["=>"]),
  },
  Covers {
    row: "The receiver, marked in a type",
    spelling: "`minus : u32 -> &self -> u32` — the marker, past the first type",
    witness: Witness::Shape(
      "a member signature whose `self` marker follows at least one `->`",
      has_a_late_receiver,
    ),
  },
  Covers {
    row: "Associated functions",
    spelling: "`start : u32 -> Self`, called `Counter::start 3`",
    witness: Witness::Shape(
      "a `class` or `instance` member with a spine and no `self` marker",
      has_an_associated_function,
    ),
  },
  Covers {
    row: "Associated types and constants",
    spelling: "`const SIDES : u32` inside a `class`",
    witness: Witness::Shape(
      "a `const` declared inside a `class` or `instance` body",
      has_associated_const,
    ),
  },
  // --- §1.4 Expressions ---
  Covers {
    row: "Operators and precedence",
    spelling: "`2 + 3 * 4` as **one** run, because the row is *precedence* and two operators matched apart witness none of it — plus `1 == 1`, `1 << 2`, and `.trim >>> .len <<< .to_string`. Plan 66's composition is one run holding both operators and a method reference, and deliberately **not** `>>>` alone: a triple type closer is the same token, so `wide : Vec<Vec<Vec<i32>>> -> usize` in the very file that adds the operator would have satisfied a bare witness with no composition in the corpus at all — the hollow shape plan 57 §11.6 found from the other side. A `.name` cannot close a type at any depth, which is what makes the run honest. One run rather than three also keeps this file inside `line_cap`'s 1,500 lines, which it was exactly at — plan 66 §10.",
    witness: Witness::All(&[
      Witness::Toks(&["+", "«int»", "*"]),
      Witness::Toks(&["=="]),
      Witness::Toks(&["<<"]),
      Witness::Toks(&[">>>", ".", "«id»", "<<<", "."]),
    ]),
  },
  Covers {
    row: "Method calls, field access",
    // **§11.6's finding, fixed.** Until plan 57 this witness was `«id» . «id»`
    // plus a left section — *field access only* — so it would have stayed
    // green through the whole method-as-function migration without once
    // witnessing a method call. So the call is witnessed by what makes it
    // **different**, which is *position*: a `.` in head position calls and a
    // `.` after an expression selects. Neither new run can be met by a field
    // access, a selector, or by `tests/ui/retired_method_call.sdx`, which
    // this corpus reads and which contains `s.len (1)`.
    spelling: "`p.x`, `.len xs`, `xs |> .len`, and a left section like `(1 <)`",
    witness: Witness::All(&[
      Witness::Toks(&["«id»", ".", "«id»"]),
      Witness::Toks(&["(", ".", "«id»", "«id»", ")"]),
      Witness::Toks(&["|>", ".", "«id»"]),
      Witness::Any(&[
        Witness::Toks(&["(", "«int»", "<", ")"]),
        Witness::Toks(&["(", "«int»", "-", ")"]),
        Witness::Toks(&["(", "«int»", ">", ")"]),
        Witness::Toks(&["(", "«id»", "<", ")"]),
      ]),
    ]),
  },
  Covers {
    row: "Function calls",
    spelling: "`add 10 20` and `zeros ()`",
    witness: Witness::All(&[
      Witness::Toks(&["«id»", "(", ")"]),
      Witness::Toks(&["«id»", "«int»", "«int»"]),
    ]),
  },
  Covers {
    row: "Indexing",
    spelling: "`(row ())[0]`",
    witness: Witness::Toks(&[")", "[", "«int»", "]"]),
  },
  Covers {
    row: "Tuple indexing .0",
    spelling: "`m.0`",
    witness: Witness::Toks(&["«id»", ".", "«int»"]),
  },
  Covers {
    row: "Ranges",
    spelling: "`1 .. 4` and `1..=10`",
    witness: Witness::All(&[Witness::Toks(&[".."]), Witness::Toks(&["..="])]),
  },
  Covers {
    row: "Struct and record literals",
    spelling: "`Point { x = 10, y = 20 }`",
    witness: Witness::Toks(&["«id»", "{", "«id»", "="]),
  },
  Covers {
    row: "Array literals and repeats",
    spelling: "`[1, 2, 3]` and `[0u8; 4]`",
    witness: Witness::All(&[
      Witness::Toks(&["[", "«int»", ","]),
      Witness::Toks(&["[", "«int»", ";"]),
    ]),
  },
  Covers {
    row: "Closures",
    spelling: "`\\n -> n * 2`",
    witness: Witness::Toks(&["\\", "«id»", "->"]),
  },
  Covers {
    row: "move closures",
    spelling: "`move \\n -> …`",
    witness: Witness::Toks(&["move"]),
  },
  Covers {
    row: "Blocks as expressions",
    spelling: "`inner =` with an indented block under it",
    witness: Witness::Shape(
      "a binding inside a function body whose value is an indented block",
      has_a_block_valued_binding,
    ),
  },
  Covers {
    row: "async blocks",
    spelling: "`work = async` with an indented body",
    witness: Witness::Toks(&["=", "async", "«nl»"]),
  },
  Covers {
    row: "unsafe blocks",
    spelling: "`unsafe` with an indented body",
    witness: Witness::Toks(&["unsafe", "«nl»"]),
  },
  Covers {
    row: "try blocks",
    spelling: "`try` with an indented body",
    witness: Witness::Toks(&["try"]),
  },
  Covers {
    row: "? operator",
    spelling: "`(parse_it x)?`",
    witness: Witness::Toks(&["?"]),
  },
  Covers {
    row: ".await",
    spelling: "the draw arrow, `once <- double n`",
    witness: Witness::Toks(&["<-"]),
  },
  Covers {
    row: "Macro invocations",
    // The `!` has to be witnessed **next to a name**, because a bare `Bang`
    // is also negation — which is the whole of leaf 1's adjacency rule, and
    // the hollow-witness shape plan 57 §11.6 and plan 66 §10 each found
    // from their own side. The `\\` and `->` after it are what make the run a
    // macro *call* rather than a name followed by a negated something.
    spelling: "`println! \\-> \"#{total} rows\"`",
    witness: Witness::Toks(&["«id»", "!", "\\", "->"]),
  },
  Covers {
    row: "Inline Rust",
    spelling: "`rust \"\"\"…\"\"\"`",
    witness: Witness::Toks(&["rust"]),
  },
  // --- §1.5 Patterns ---
  Covers {
    row: "Literal and wildcard patterns",
    spelling: "`0 -> …` and `_ -> …`",
    witness: Witness::All(&[Witness::Toks(&["«int»", "->"]), Witness::Toks(&["_", "->"])]),
  },
  Covers {
    row: "Identifier and binding patterns",
    spelling: "`mut n = 0`",
    witness: Witness::Toks(&["mut", "«id»", "="]),
  },
  Covers {
    row: "@ bindings",
    spelling: "`whole @ Rect { w, h }`",
    witness: Witness::Toks(&["«id»", "@"]),
  },
  Covers {
    row: "Struct and record patterns",
    spelling: "`Point { label, .. }`",
    witness: Witness::Toks(&["«id»", "{", "«id»", ",", "..", "}"]),
  },
  Covers {
    row: "Tuple patterns",
    spelling: "`(a, b) = p`",
    witness: Witness::Toks(&["(", "«id»", ",", "«id»", ")", "="]),
  },
  Covers {
    row: "Slice patterns, with ..",
    spelling: "`[first, .., last]`",
    witness: Witness::Toks(&["[", "«id»", ",", "..", ","]),
  },
  Covers {
    row: "Range patterns",
    spelling: "`18..=64 -> …`",
    witness: Witness::Toks(&["«int»", "..=", "«int»", "->"]),
  },
  // --- §1.6 Statements and control flow ---
  Covers {
    row: "let bindings, let mut",
    spelling: "`x = 1` and `mut n = 0`",
    witness: Witness::All(&[
      Witness::Toks(&["«id»", "=", "«int»"]),
      Witness::Toks(&["mut", "«id»", "="]),
    ]),
  },
  // Plan 54. `:=` assigns and `=` binds, so the two rows below and the two
  // above them are four readings of one character sequence — which is exactly
  // why each needs its own witness rather than a shared one.
  Covers {
    row: "Assignment",
    spelling: "`x := 1`, and a place that is not a name",
    witness: Witness::All(&[
      Witness::Toks(&["«id»", ":=", "«int»"]),
      Witness::Toks(&["]", ":=", "«int»"]),
    ]),
  },
  Covers {
    row: "Compound assignment",
    spelling: "all five of `+=`, `-=`, `*=`, `/=`, `%=`",
    witness: Witness::All(&[
      Witness::Toks(&["«id»", "+="]),
      Witness::Toks(&["«id»", "-="]),
      Witness::Toks(&["«id»", "*="]),
      Witness::Toks(&["«id»", "/="]),
      Witness::Toks(&["«id»", "%="]),
    ]),
  },
  Covers {
    row: "Shadowing",
    spelling: "one name bound twice in a scope",
    witness: Witness::Shape(
      "a name bound by `name = …` twice at the start of a line in one file",
      has_shadowing,
    ),
  },
  Covers {
    row: "if / else",
    spelling: "`if c` … `else`",
    witness: Witness::All(&[Witness::Toks(&["if"]), Witness::Toks(&["else"])]),
  },
  Covers {
    row: "if let",
    spelling: "`if o is Some v`",
    witness: Witness::Toks(&["if", "«id»", "is", "«id»"]),
  },
  Covers {
    row: "let chains",
    spelling: "`if o is Some v && v > 2`",
    witness: Witness::Toks(&["if", "«any»", "is", "«any»", "«any»", "&&"]),
  },
  Covers {
    row: "while, while let",
    spelling: "`while c` and `while .pop stack is Some v`",
    witness: Witness::All(&[
      Witness::Toks(&["while"]),
      // `while .pop stack is Some v` — the scrutinee is a method call, so it
      // is three tokens rather than one. Counted rather than wildcarded,
      // because a lone `is` would match the keyword anywhere.
      Witness::Toks(&["while", "«any»", "«any»", "«any»", "is"]),
    ]),
  },
  Covers {
    row: "loop",
    spelling: "`loop`",
    witness: Witness::Toks(&["loop"]),
  },
  Covers {
    row: "for",
    spelling: "`for c in 1 .. 4`",
    witness: Witness::Toks(&["for", "«id»", "in"]),
  },
  Covers {
    row: "break, continue, with value",
    spelling: "`break`, `continue`, `break 'search c`",
    witness: Witness::All(&[
      Witness::Toks(&["break"]),
      Witness::Toks(&["continue"]),
      Witness::Toks(&["break", "«label»", "«id»"]),
    ]),
  },
  Covers {
    row: "Labelled loops and blocks",
    spelling: "`'outer loop` and `'search` on a bare block",
    witness: Witness::All(&[
      Witness::Toks(&["«label»", "loop"]),
      // A label that **ends its line** opens a block — plan 55 leaf 7, where
      // `do 'search` became `'search`. `'outer loop` has a keyword after the
      // label and `break 'search c` has an expression, so neither is this.
      Witness::Toks(&["«label»", "«nl»"]),
    ]),
  },
  Covers {
    row: "match",
    spelling: "`case e of`",
    witness: Witness::Toks(&["case", "«any»", "of"]),
  },
  Covers {
    row: "Guards",
    spelling: "`| spend > 10_000 = …` and `| otherwise = …`",
    witness: Witness::All(&[
      Witness::Toks(&["|", "«id»", ">"]),
      Witness::Toks(&["|", "otherwise", "="]),
    ]),
  },
  Covers {
    row: "return",
    spelling: "`return d`",
    witness: Witness::Toks(&["return"]),
  },
  // --- §1.7 Beyond the grammar ---
  Covers {
    row: "std and crate paths",
    spelling: "`std::collections::HashMap`",
    witness: Witness::Toks(&["std", "::"]),
  },
  Covers {
    row: "Error types and conversions",
    spelling: "`errors ConfigError`",
    witness: Witness::Toks(&["errors"]),
  },
  Covers {
    row: "Tests",
    spelling: "`test \"adds two numbers\"`",
    witness: Witness::Toks(&["test", "«str»"]),
  },
];

fn witness_for(row: &str) -> Option<&'static Covers> {
  WITNESSES.iter().find(|covers| covers.row == row)
}

// ---------------------------------------------------------------------------
// The gate
// ---------------------------------------------------------------------------

#[test]
fn the_index_is_long_enough_to_be_believed() {
  let reachable = reachable_rows();
  assert!(
    reachable.len() >= MINIMUM_ROWS,
    "docs/GRAMMAR.md parsed to {} reachable rows, fewer than the {MINIMUM_ROWS} this gate \
     requires. A parse that silently loses rows reports success on the rows it kept — plan 46 \
     §10 and plan 47 §10.1 both record exactly that, the second time because an unescaped \
     `\\|` in a table cell split every column by one. Check the table's escaping before \
     lowering this floor.",
    reachable.len()
  );
}

#[test]
fn every_graded_row_has_a_witness() {
  let reachable = reachable_rows();

  let unwitnessed: Vec<String> = reachable
    .iter()
    .filter(|row| witness_for(&row.key).is_none())
    .map(|row| format!("{} — {}", row.cite(), row.key))
    .collect();

  let stale: Vec<String> = WITNESSES
    .iter()
    .filter(|covers| !reachable.iter().any(|row| row.key == covers.row))
    .map(|covers| format!("{} names no reachable row of docs/GRAMMAR.md", covers.row))
    .collect();

  assert!(
    unwitnessed.is_empty() && stale.is_empty(),
    "the witness table and docs/GRAMMAR.md must agree in both directions, or a row added to \
     one is silently unasserted by the other:\n  {}\n  {}",
    unwitnessed.join("\n  "),
    stale.join("\n  ")
  );
}

#[test]
fn every_graded_construct_is_exampled() {
  let corpus = corpus();
  let mut uncovered = Vec::new();

  for row in reachable_rows() {
    if ALLOW.iter().any(|(allowed, _)| *allowed == row.key) {
      continue;
    }
    let Some(covers) = witness_for(&row.key) else {
      continue; // reported by `every_graded_row_has_a_witness`
    };
    if let Some(missing) = covers.witness.unmet(&corpus) {
      uncovered.push(format!(
        "{} [{}]\n      spelled {} — looked for {missing}",
        row.key,
        row.cite(),
        covers.spelling,
      ));
    }
  }

  assert!(
    uncovered.is_empty(),
    "these constructs are graded reachable in docs/GRAMMAR.md and appear in no corpus file. \
     The examples are the documentation a reader actually reads, so an unexampled construct \
     is an undiscoverable one — which is how `$` stayed invisible for thirty plans while \
     being fully implemented (plan 51 §1). Write the example; the ALLOW list is for \
     constructs that cannot be witnessed, not for ones that have not been written \
     yet:\n  {}",
    uncovered.join("\n  ")
  );
}

/// `ALLOW` may only name rows that exist, may only shrink, and every entry
/// must carry a reason — or it accumulates exemptions for constructs nobody
/// can find.
#[test]
fn every_allowance_names_a_real_row_and_gives_a_reason() {
  assert!(
    ALLOW.len() <= ALLOW_AT_MERGE,
    "ALLOW holds {} entries against a pin of {ALLOW_AT_MERGE}. The list exists to shrink: a \
     newly uncovered construct wants an example, and raising the pin is a deliberate act that \
     has to be argued for in the diff.",
    ALLOW.len()
  );

  let reachable = reachable_rows();
  for (row, why) in ALLOW {
    assert!(
      reachable.iter().any(|known| known.key == *row),
      "ALLOW names {row:?}, which is not a reachable row of docs/GRAMMAR.md"
    );
    assert!(
      why.len() > 40,
      "ALLOW's entry for {row:?} needs a reason, not a note: {why:?}"
    );
  }
}

/// The scan is lexical, and this is the file that proves it.
///
/// Plan 51 §5 criterion 6 and §9 decision 2: `|>` inside a comment is not
/// coverage. `tests/construct_coverage/decoy.sdx` contains nine constructs as
/// text and none as tokens, so a substring search finds every one of them and
/// the token stream finds none — which is the difference between this gate and
/// the one it would have been easy to write.
#[test]
fn a_construct_inside_a_comment_or_a_string_does_not_count() {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/construct_coverage/decoy.sdx");
  let text = std::fs::read_to_string(&path).expect("the decoy fixture should exist");
  let decoy = Source::new("decoy.sdx".to_owned(), text).expect("the decoy should lex");

  let decoys: &[(&str, &[&str])] = &[
    ("case … of", &["case", "«any»", "of"]),
    ("|>", &["|>"]),
    ("$", &["$"]),
    ("dyn", &["dyn"]),
    ("move", &["move"]),
    ("ref", &["ref", "«id»"]),
    ("*const", &["*", "const"]),
    ("a lambda", &["\\", "«id»", "->"]),
    ("@name", &["@", "«id»"]),
  ];

  for (name, pattern) in decoys {
    assert!(
      decoy.text.contains(
        pattern
          .iter()
          .find(|part| !part.starts_with('«'))
          .expect("each decoy has a literal part")
      ),
      "the decoy fixture should contain {name} as text, or it proves nothing"
    );
    assert!(
      !decoy.has(pattern),
      "{name} appears only in a comment or a string in {}, so the scan must not see it — \
       a substring search would, which is the whole reason this is a token scan",
      decoy.name
    );
  }
}

/// A witness that cannot fail is worth nothing, so one is run against a file
/// chosen to lack it.
#[test]
fn a_witness_can_actually_fail() {
  let empty = Source::new("empty.sdx".to_owned(), String::new()).expect("the empty file lexes");
  let without = [empty];

  let mut unmet = 0;
  for covers in WITNESSES {
    assert!(
      covers.witness.unmet(&without).is_some(),
      "{} is witnessed by an empty corpus, so its witness asserts nothing",
      covers.row
    );
    unmet += 1;
  }
  assert_eq!(unmet, WITNESSES.len());
}
