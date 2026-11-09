use crate::lint::allow_pragma::allow_pragma;
use crate::lint::line_of::line_of;
use redox_diag::Span;
use redox_lexer::TokenKind;

/// Every unsuppressed `unwrap`/`expect` call site in a source file — doc 03
/// §12.7, plan 38.
///
/// # Why this scans tokens rather than walking the AST
///
/// §12.7: *"it costs nothing because no analysis is involved."* Taken
/// literally, the lint is **lexical** — a `.` followed by the name `unwrap` or
/// `expect` — and scanning tokens says exactly that and nothing more.
///
/// The alternative was an `Expr` walker, and it is worse here for a reason
/// worth recording: a walker needs an arm for every expression variant that
/// can contain a sub-expression, and one that forgets a variant **silently
/// misses call sites**. Nothing in the suite would notice, because a lint's
/// absence looks like a clean program. A token scan cannot miss one, and it
/// gains that property by knowing less.
///
/// It also gives an exact span — the name's own — where a walker would have to
/// reconstruct it from the call's.
///
/// The cost is lexing twice, once here and once inside `parse_source`. Named
/// rather than hidden: it is a few microseconds on a file that is about to be
/// handed to rustc.
///
/// # What is matched
///
/// The whole name, not a prefix. `unwrap_or`, `unwrap_or_else`,
/// `unwrap_or_default` and `expect_err` are the ergonomic path, and a prefix
/// match would flag the cure as the disease.
///
/// A `.` before it, so a free function called `unwrap` is not a method call
/// and is not matched.
///
/// # Suppression is per line
///
/// Plan 38 §1.4 said the unit is the statement; scanning tokens makes it the
/// **line**, which is the same thing for every program anyone writes and is
/// what the pragma's placement implies anyway. Two call sites on one line are
/// suppressed together, and splitting the line is the way to suppress one.
pub(crate) fn unwrap_call_sites(source: &str) -> Vec<Span> {
  let Ok(tokens) = redox_lexer::lex(source) else {
    // A file that does not lex has no call sites worth reporting: the lexical
    // error is the diagnostic, and this pass only ever runs after a successful
    // parse anyway.
    return Vec::new();
  };

  // Lines carrying the pragma, from the trivia on every token.
  let suppressed: Vec<usize> = tokens
    .iter()
    .flat_map(|t| t.leading.iter().chain(t.trailing.as_ref()))
    .filter(|trivium| allow_pragma(trivium))
    .map(|trivium| line_of(source, trivium.span.0))
    .collect();

  tokens
    .windows(2)
    .filter(|pair| pair[0].kind == TokenKind::Dot)
    .filter_map(|pair| match &pair[1].kind {
      TokenKind::Ident(name) if name == "unwrap" || name == "expect" => Some(pair[1].span),
      _ => None,
    })
    .filter(|span| !suppressed.contains(&line_of(source, span.0)))
    .collect()
}
