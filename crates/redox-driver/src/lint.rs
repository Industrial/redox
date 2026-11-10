pub(crate) mod allow_pragma;
pub(crate) mod attribute_pragma;
pub(crate) mod comment_pragma;
pub(crate) mod comment_pragma_message;
pub(crate) mod comment_pragma_sites;
pub(crate) mod line_of;
pub(crate) mod suppressed_spans;
pub(crate) mod unwrap_call_sites;
pub(crate) mod unwrap_message;
pub(crate) mod walk_item_for_pragmas;
pub(crate) mod walk_member_for_pragmas;
pub(crate) mod walk_stmts_for_pragmas;

use crate::lint::comment_pragma_message::COMMENT_PRAGMA_MESSAGE;
use crate::lint::comment_pragma_sites::comment_pragma_sites;
use crate::lint::unwrap_call_sites::unwrap_call_sites;
use crate::lint::unwrap_message::UNWRAP_MESSAGE;
use redox_ast::Program;
use redox_diag::{Code, Diagnostic, Severity};
use std::path::Path;

/// Every lint, as warnings — doc 03 §12.7, plan 38 and plan 53 leaves 4–5.
///
/// Two lints. `RDX1001` is `unwrap` discouragement, which opened the
/// `RDX1000`+ band; `RDX1002` is its suppression pragma written as a comment,
/// which plan 53 deprecates in favour of `#[redox::allow(unwrap)]`. The
/// second one's shape is the first one's, as plan 38 predicted it would be.
///
/// # Why the driver and not the parser
///
/// Plan 30's `check_callback_tails` is a whole-program pass in the **parser**,
/// and it produces `ParseError`s — which become hard errors, because
/// `ParseFailure` has no notion of a warning. A lint cannot go that way.
///
/// `Diagnostic` is the type that carries a [`Severity`], and the driver is
/// where `Diagnostic`s are built (`diagnose`). So a lint runs here, over an
/// AST that already parsed, and returns warnings the caller renders and does
/// not fail on.
///
/// # Source **and** the program, where the program is optional
///
/// Both lints are detected lexically, which is why the source comes first and
/// why plan 38 took nothing else. What needs the tree is *where a suppression
/// reaches*: plan 53's attribute can only sit on the line above the construct
/// it applies to, so the pragma's own line is not the line it is about, and a
/// declaration-level pragma covers a range. `suppressed_spans` reads those
/// ranges off the AST's byte spans.
///
/// `None` is for the caller that has no tree to give. `redox-lsp` lints a
/// document that does not parse on purpose — a lint is still an answer about
/// the text while the author is mid-edit — and the degradation is the right
/// way round: a suppression that cannot be computed leaves the warning
/// showing.
pub fn lint(path: &Path, source: &str, program: Option<&Program>) -> Vec<Diagnostic> {
  let path = path.display().to_string();
  let warning = |code: Code, message: &str, span, help: &str| Diagnostic {
    code: code.into(),
    severity: Severity::Warning,
    message: message.to_string(),
    span,
    help: Some(help.to_string()),
    path: path.clone(),
    source: source.to_string(),
  };

  let mut found: Vec<Diagnostic> = unwrap_call_sites(source, program)
    .into_iter()
    .map(|span| {
      warning(
        Code::UnwrapUsed,
        UNWRAP_MESSAGE,
        span,
        "if a panic is intended here, say so on the line above: `#[redox::allow(unwrap)]`",
      )
    })
    .chain(comment_pragma_sites(source).into_iter().map(|span| {
      warning(
        Code::CommentAllowPragma,
        COMMENT_PRAGMA_MESSAGE,
        span,
        "the comment still suppresses `RDX1001` for this milestone; the attribute is what \
         plan 64 keeps",
      )
    }))
    .collect();

  // One file's warnings are rendered in source order whichever lint found
  // them, because a blessed `.stderr` is compared as exact text and two lints
  // concatenated are not sorted.
  found.sort_by_key(|d| d.span);
  found
}
