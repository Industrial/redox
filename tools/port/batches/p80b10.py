# -*- coding: utf-8 -*-
"""Plan 80 batch 10: `lint` and the rustc-message parser.

`lint`'s `warning` closure is the first ported body to meet doc 69 §10.4's
finding — **a lambda parameter is a bare name** — on a closure whose Rust
*annotated* two of its four. Dropping the annotations needs one inferred type
per parameter across three call sites, so `message` and `help` are `String`
where the Rust took `&str`. Both were already allocated by `to_string` inside
the closure, so the only change is where.

`RustcMessage::parse` is five `split_once` pairs in a row, each destructured in
the Rust, and each half rebinding a name the previous line bound — so it is
also the densest `RDX1003` site in the crate.
"""
import sys

sys.path.insert(0, '/home/tom/Code/rust/voltaic/tools/port')
from write import write  # noqa: E402

write('remap/rustc_message', '''
instance RustcMessage
  -- | Parse one line, or `None` if it is not a positioned diagnostic.
  -- |
  -- | Splitting is done from the **left on `:`** for the three position fields
  -- | and then left alone, because the message routinely contains colons
  -- | (`expected \\`i32\\`, found \\`&str\\`` does not, but `cannot add \\`&str\\` to
  -- | \\`i32\\`: no implementation` does). A greedy split on the whole line would
  -- | truncate the message at rustc's own punctuation.
  -- |
  -- | A Windows path (`C:\\…`) would defeat a naive left split too, which is why
  -- | the line and column are found by taking the *last two* colon-separated
  -- | fields before the level rather than the second and third overall.
  export parse : &str -> Option<Self>
  parse line =
    -- `path:line:col: level...` — find the `: ` that ends the position.
    --
    -- Each `split_once` is bound and projected, where the Rust destructured,
    -- and each half takes its own name rather than rebinding `path`:
    -- `RDX1003` reports a shadow.
    head = (.split_once ": " line)?
    position = head.0
    rest = head.1

    -- The position's last two `:` fields are the line and column; everything
    -- before them is the path, colons and all.
    tail = (.rsplit_once ':' position)?
    col : usize = (.ok (.parse tail.1))?
    stem = (.rsplit_once ':' tail.0)?
    line_no : usize = (.ok (.parse stem.1))?

    -- `level[code]: message`, or `level: message`.
    labelled = (.split_once ": " rest)?
    label = labelled.0
    message = labelled.1
    bracketed = .split_once '[' label
    level = .map_or label (\\parts -> parts.0) (.as_ref bracketed)
    code = .map (\\parts -> .to_string (.trim_end_matches ']' parts.1)) (.as_ref bracketed)
    if level != "error" && level != "warning"
      return None

    Some (RustcMessage { path = .to_string stem.0, line = line_no, col, level = .to_string level, code, message = .to_string message })

  export is_error : &self -> bool
  is_error self = self.level == "error"

-- | The exact line rustc produced for `plan 07 §2`'s measurement.
test "parses a coded error"
  m = .expect "should parse" (RustcMessage::parse ".tmp/typeerr.xrs:6:24: error[E0308]: mismatched types: expected `i32`, found `&str`")
  assert_eq! \\-> m.path \\-> ".tmp/typeerr.xrs"
  assert_eq! \\-> (m.line, m.col) \\-> (6, 24)
  assert_eq! \\-> m.level \\-> "error"
  assert_eq! \\-> .as_deref m.code \\-> Some "E0308"
  assert_eq! \\-> m.message \\-> "mismatched types: expected `i32`, found `&str`" \\-> "the message keeps rustc's own colons"
  assert! \\-> .is_error m

-- | The other measured line, whose message contains a colon.
test "keeps a message containing a colon"
  m = .expect "should parse" (RustcMessage::parse ".tmp/tail.xrs:2:7: error[E0277]: cannot add `&str` to `i32`: no implementation for `i32 + &str`")
  assert_eq! \\-> m.message \\-> "cannot add `&str` to `i32`: no implementation for `i32 + &str`"

test "parses an uncoded warning"
  m = .expect "should parse" (RustcMessage::parse "main.xrs:3:9: warning: unused variable: `x`")
  assert_eq! \\-> m.level \\-> "warning"
  assert_eq! \\-> m.code \\-> None
  assert_eq! \\-> m.message \\-> "unused variable: `x`"
  assert! \\-> !.is_error m

-- | rustc's summary line is not a diagnostic.
test "declines the summary line"
  assert_eq! \\-> RustcMessage::parse "error: aborting due to 1 previous error" \\-> None

test "declines an unpositioned note"
  assert_eq! \\-> RustcMessage::parse "note: run with `RUST_BACKTRACE=1`" \\-> None
  assert_eq! \\-> RustcMessage::parse "" \\-> None

-- | A path containing a colon must not be mistaken for a position.
test "survives a colon in the path"
  m = .expect "should parse" (RustcMessage::parse "C:\\\\out\\\\main.xrs:6:24: error[E0308]: mismatched types")
  assert_eq! \\-> m.path \\-> "C:\\\\out\\\\main.xrs"
  assert_eq! \\-> (m.line, m.col) \\-> (6, 24)

-- | A level rustc does not use is not a diagnostic Voltaic should relay.
test "declines an unknown level"
  assert_eq! \\-> RustcMessage::parse "main.xrs:1:1: help: try this" \\-> None
''')

write('lint', '''
import voltaic_diag::DiagnosticCode

export lint : &Path -> &str -> Option<&Program> -> Vec<Diagnostic>
lint path source program =
  -- `shown`, not a rebinding of `path`: `RDX1003` reports a shadow.
  shown = .to_string (.display path)
  -- No annotations, because a lambda parameter is a bare name (doc 69 §10.4),
  -- so one inferred type per parameter has to serve all three call sites:
  -- `message` and `help` are `String` where the Rust took `&str`. Both were
  -- already allocated by `to_string` inside, so only the place changed.
  --
  -- And the `Code`-to-`DiagnosticCode` conversion moved **out** of the
  -- closure for the same reason: `code.into()` on an unannotated parameter is
  -- `E0282`, because a closure body is checked before any call site is seen.
  -- Naming the target at each call site resolves `code` from the struct field
  -- instead, which is the general answer whenever an annotation would have
  -- carried the inference.
  warning = \\code message span help -> Diagnostic { code, severity = Severity::Warning, message, span, help = Some help, path = .clone shown, source = .to_string source }

  unwraps = unwrap_call_sites source program
    |> .into_iter
    |> .map (\\span -> warning (DiagnosticCode::from Code::UnwrapUsed) (.to_string UNWRAP_MESSAGE) span (.to_string "if a panic is intended here, say so on the line above: `\\#[voltaic::allow(unwrap)]`"))

  pragmas = comment_pragma_sites source
    |> .into_iter
    |> .map (\\pair ->
    span = pair.0
    named = pair.1
    code_text = .text (.code named)
    help = format! \\-> "the comment still suppresses `#{code_text}` for this milestone; the attribute is what plan 64 keeps"
    warning (DiagnosticCode::from Code::CommentAllowPragma) (comment_pragma_message named) span help)

  mut found : Vec<Diagnostic> = .collect (.chain pragmas unwraps)

  -- `RDX1003` is the one lint that needs the tree rather than the text, so it
  -- is the one with nothing to say when there is no tree — which is the right
  -- degradation: a language server lints a document mid-edit, and a shadow
  -- reported from a half-parsed statement list would be a guess.
  if program is Some tree
    shadows = shadow_sites tree
      |> .into_iter
      |> .map (\\pair ->
      span = pair.0
      name = pair.1
      warning (DiagnosticCode::from Code::ShadowedBinding) (shadow_message (&name)) span (.to_string SHADOW_HELP))
    .extend shadows found

  -- One file's warnings are rendered in source order whichever lint found
  -- them, because a recorded `.stderr` is compared as exact text and two lints
  -- concatenated are not sorted.
  .sort_by_key (\\d -> d.span) found
  found
''')

print('plan 80 batch 10: two files')
