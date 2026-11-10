use crate::lint::shadow_walk::ShadowWalk;
use crate::lint::suppressed_spans::suppressed_spans;
use crate::lint::walk_program::walk_program;
use redox_ast::Program;
use redox_diag::{Lint, Span};

/// Every unsuppressed shadowing `=` in a program, with the name it rebinds —
/// `RDX1003`, plan 54 leaf 3.
///
/// # Why this walks the AST where `RDX1001` scans tokens
///
/// [`unwrap_call_sites`](super::unwrap_call_sites::unwrap_call_sites) argues
/// at length for a token scan, and the argument does not transfer. It rests
/// on the lint being *lexical* — a `.` then a name — so knowing less cannot
/// miss a site. A shadow is not lexical in any sense: it is two bindings
/// **in one statement list**, and a token scan has no notion of a list. It
/// would see `=` inside a nested block, inside a `do`, inside a record
/// literal and inside a lambda as the same thing, which is precisely the
/// distinction criterion 5 exists to hold.
///
/// So the tree is not an optimisation here, it is the definition. The failure
/// mode a walk brings — a statement kind the walk forgets — is why
/// [`walk_stmts`](super::walk_stmts::walk_stmts)'s match carries no wildcard.
///
/// # The span is the name's, not the statement's
///
/// The warning points at the identifier that is bound twice, which is the
/// thing the author has to look at. It is inside the statement's span, so a
/// leading pragma still suppresses it through the ranges below.
pub(crate) fn shadow_sites(program: &Program) -> Vec<(Span, String)> {
  let mut walk = ShadowWalk { found: Vec::new() };
  walk_program(program, &mut walk);

  let ranges = suppressed_spans(program, Lint::Shadow);
  walk
    .found
    .into_iter()
    .filter(|(span, _)| !ranges.iter().any(|r| r.0 <= span.0 && span.1 <= r.1))
    .collect()
}
