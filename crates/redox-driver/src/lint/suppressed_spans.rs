use crate::lint::walk_item_for_pragmas::walk_item_for_pragmas;
use redox_ast::Program;
use redox_diag::Span;

/// The byte ranges `RDX1001` is suppressed over — plan 53 leaf 4.
///
/// One range per construct that carries a suppression pragma in its leading
/// trivia: a statement's own span, or a declaration's whole span. A call site
/// inside any of them is silenced.
///
/// # Why detection scans tokens and suppression walks the tree
///
/// [`unwrap_call_sites`](super::unwrap_call_sites::unwrap_call_sites) argues
/// at length against walking an AST, and that argument stands for
/// **detection**: a walker that forgets an `Expr` variant *silently misses
/// call sites*, nothing in the suite notices, and a lint's absence looks
/// exactly like a clean program.
///
/// This walk fails the other way. A suppression range it fails to compute
/// makes a warning **fire that should have been silenced** — in the author's
/// face, on the line they annotated, with the pragma visible beside it. That
/// is loud rather than silent, so the reason not to walk does not apply here,
/// and nothing cheaper will do: an attribute can only sit on the line *above*
/// what it applies to, so the line the pragma is written on is not the line
/// it is about, and a declaration-level pragma covers a range rather than a
/// line at all.
///
/// Spans make it cheap. Every AST node carries a byte span (`redox-ast`'s
/// module doc records what Emerald paid for finding that out late), so a
/// construct's extent is read rather than reconstructed.
///
/// # The pragma's own line still suppresses too
///
/// This is *added* to plan 38's line rule, not a replacement for it. A
/// **trailing** comment pragma — `port = raw.unwrap ()  -- redox: allow
/// unwrap` — is not any construct's leading trivia, so it appears in no range
/// here and is still keyed on its line by the caller.
pub(crate) fn suppressed_spans(program: &Program) -> Vec<Span> {
  let mut found = Vec::new();
  for item in &program.items {
    walk_item_for_pragmas(item, &mut found);
  }
  found
}
