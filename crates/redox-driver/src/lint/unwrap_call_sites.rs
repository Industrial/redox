use crate::lint::allow_pragma::allow_pragma;
use crate::lint::line_of::line_of;
use crate::lint::suppressed_spans::suppressed_spans;
use redox_ast::Program;
use redox_diag::{Lint, Span};
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
/// **That argument is about detection and not about suppression**, which is
/// why `program` is here — see
/// [`suppressed_spans`](super::suppressed_spans::suppressed_spans) for the
/// other half, and for why consulting the tree is safe there and not here.
/// The `Program` is optional because the language server lints a document
/// that does not parse; without one only the line rule below applies, and a
/// pragma that fails to suppress is loud.
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
/// # Suppression is per line, and per construct
///
/// Plan 38 §1.4 said the unit is the statement; scanning tokens made it the
/// **line**, which is the same thing for a trailing comment —
/// `port = raw.unwrap ()  -- redox: allow unwrap` — and nothing at all for an
/// attribute, which can only sit on the line *above*. Plan 53 §11.6 measured
/// that: a pragma one line up did not suppress.
///
/// So there are two keys now and both are consulted. The **line** key is
/// plan 38's, unchanged, and covers the trailing comment. The **span** key is
/// the span of the construct a pragma leads, which is what makes the
/// attribute work — and incidentally makes a *leading comment* pragma work
/// where it silently did not. Two call sites on one line are still suppressed
/// together, and splitting the line is still the way to suppress one.
pub(crate) fn unwrap_call_sites(source: &str, program: Option<&Program>) -> Vec<Span> {
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
    .filter(|trivium| allow_pragma(trivium, Lint::Unwrap))
    .map(|trivium| line_of(source, trivium.span.0))
    .collect();

  // Spans of the constructs a pragma leads, which the lines above cannot
  // reach.
  let ranges = program
    .map(|program| suppressed_spans(program, Lint::Unwrap))
    .unwrap_or_default();

  tokens
    .windows(2)
    .filter(|pair| pair[0].kind == TokenKind::Dot)
    .filter_map(|pair| match &pair[1].kind {
      TokenKind::Ident(name) if name == "unwrap" || name == "expect" => Some(pair[1].span),
      _ => None,
    })
    .filter(|span| !suppressed.contains(&line_of(source, span.0)))
    .filter(|span| !ranges.iter().any(|r| r.0 <= span.0 && span.1 <= r.1))
    .collect()
}
