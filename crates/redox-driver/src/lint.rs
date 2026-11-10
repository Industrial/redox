/// Whether one trivium suppresses a lint, in either spelling.
pub(crate) mod allow_pragma;
/// The lint one trivium suppresses as an attribute.
pub(crate) mod attribute_pragma;
/// The one name a binding contributes to `RDX1003`, if it contributes one.
pub(crate) mod binding_name;
/// The lint one trivium suppresses as a deprecated comment.
pub(crate) mod comment_pragma;
/// The message `RDX1002` carries.
pub(crate) mod comment_pragma_message;
/// Every comment-form suppression pragma in a source file, for `RDX1002`.
pub(crate) mod comment_pragma_sites;
/// The 1-based line a byte offset falls on.
pub(crate) mod line_of;
/// Collects the byte ranges one lint is suppressed over.
pub(crate) mod pragma_walk;
/// What a lint wants to see as the walk descends a program.
pub(crate) mod program_walk;
/// The help `RDX1003` carries.
pub(crate) mod shadow_help;
/// The message `RDX1003` carries.
pub(crate) mod shadow_message;
/// Every unsuppressed shadowing `=` in a program — `RDX1003`.
pub(crate) mod shadow_sites;
/// Collects every bare `=` that rebinds a name bound earlier in the same statement list.
pub(crate) mod shadow_walk;
/// The byte ranges a lint is suppressed over.
pub(crate) mod suppressed_spans;
/// Every unsuppressed `unwrap`/`expect` call site in a source file.
pub(crate) mod unwrap_call_sites;
/// The message `RDX1001` carries.
pub(crate) mod unwrap_message;
/// Walk one item.
pub(crate) mod walk_item;
/// Walk one class or instance member.
pub(crate) mod walk_member;
/// Walk a whole program.
pub(crate) mod walk_program;
/// Walk a statement run and the blocks its statements own.
pub(crate) mod walk_stmts;

use crate::lint::comment_pragma_message::comment_pragma_message;
use crate::lint::comment_pragma_sites::comment_pragma_sites;
use crate::lint::shadow_help::SHADOW_HELP;
use crate::lint::shadow_message::shadow_message;
use crate::lint::shadow_sites::shadow_sites;
use crate::lint::unwrap_call_sites::unwrap_call_sites;
use crate::lint::unwrap_message::UNWRAP_MESSAGE;
use redox_ast::Program;
use redox_diag::{Code, Diagnostic, Severity};
use std::path::Path;

/// Every lint, as warnings — doc 03 §12.7, plan 38, plan 53 leaves 4–5 and
/// plan 54 leaf 3.
///
/// Three lints. `RDX1001` is `unwrap` discouragement, which opened the
/// `RDX1000`+ band; `RDX1002` is its suppression pragma written as a comment,
/// which plan 53 deprecates in favour of `#[redox::allow(unwrap)]`; `RDX1003`
/// is a bare `=` rebinding a name already bound in the same statement list,
/// which plan 54 §1.1 measured as the one defect in the language that nothing
/// caught. The second one's shape is the first one's, as plan 38 predicted it
/// would be.
///
/// `RDX1003` is the first lint that is **not** lexical, and the first that
/// cannot be: it is two bindings *in one statement list*, so the list is the
/// unit and a token scan has no notion of one. It is also the lint that made
/// the suppression pragma take a lint name — see
/// [`allow_pragma`](allow_pragma::allow_pragma).
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
    .chain(
      comment_pragma_sites(source)
        .into_iter()
        .map(|(span, lint)| {
          warning(
            Code::CommentAllowPragma,
            &comment_pragma_message(lint),
            span,
            &format!(
              "the comment still suppresses `{}` for this milestone; the attribute is what \
           plan 64 keeps",
              lint.code().text()
            ),
          )
        }),
    )
    .collect();

  // `RDX1003` is the one lint that needs the tree rather than the text, so it
  // is the one with nothing to say when there is no tree — which is the right
  // degradation: a language server lints a document mid-edit, and a shadow
  // reported from a half-parsed statement list would be a guess.
  if let Some(program) = program {
    found.extend(shadow_sites(program).into_iter().map(|(span, name)| {
      warning(
        Code::ShadowedBinding,
        &shadow_message(&name),
        span,
        SHADOW_HELP,
      )
    }));
  }

  // One file's warnings are rendered in source order whichever lint found
  // them, because a blessed `.stderr` is compared as exact text and two lints
  // concatenated are not sorted.
  found.sort_by_key(|d| d.span);
  found
}
