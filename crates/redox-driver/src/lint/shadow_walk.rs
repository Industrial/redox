use crate::lint::allow_pragma::allow_pragma;
use crate::lint::binding_name::binding_name;
use crate::lint::program_walk::ProgramWalk;
use redox_ast::{Expr, Item, Stmt};
use redox_diag::{Lint, Span};

/// Collects every bare `=` that rebinds a name bound earlier in the **same
/// statement list** — `RDX1003`, plan 54 leaf 3.
///
/// One pass per list, carrying the names seen so far. That is the whole of
/// the analysis, and it is deliberately not more: a list is handed over by
/// [`walk_stmts`](super::walk_stmts::walk_stmts) on its own, so there is no
/// outer list to consult and no scope to track. Plan 54 §1.3 refused the
/// `=`-assigns design because it needed a scope table AGENTS.md forbids
/// Redox growing; this is the same question answered within one list, where
/// it costs a `Vec` of names and no resolution.
pub(crate) struct ShadowWalk {
  pub(crate) found: Vec<(Span, String)>,
}

impl ProgramWalk for ShadowWalk {
  /// Nothing. A shadow is always a statement rebinding another statement's
  /// name, so a declaration carries no site of its own — and two *items*
  /// sharing a name is rustc's `E0428`, which is a check rustc makes.
  fn item(&mut self, _item: &Item) {}

  fn stmts(&mut self, stmts: &[Stmt]) {
    let mut bound: Vec<&str> = Vec::new();
    for stmt in stmts {
      // Only a binding statement, because only a binding statement has an
      // alternative spelling to suggest. `for x <- xs` scopes its binder to
      // the loop body and is not a shadow at all; `n <- f`, `guard P = e`
      // and a draw arrow do bind into the rest of the list, but none of them
      // has a `:=` form — so a warning on one would name a problem and no
      // fix. Plan 54 §6's residue, stated rather than widened.
      let Expr::Binding(binding) = &stmt.expr else {
        continue;
      };
      let Some(ident) = binding_name(binding) else {
        continue;
      };

      // A **trailing** pragma is not any construct's leading trivia, so
      // `suppressed_spans` cannot see it — the same split plan 38's line rule
      // covers for `RDX1001`, and keyed on the statement here because a
      // shadow site always has a statement of its own to hang it on.
      let suppressed = stmt
        .trailing
        .as_ref()
        .is_some_and(|t| allow_pragma(t, Lint::Shadow));

      if bound.contains(&ident.text.as_str()) && !suppressed {
        self.found.push((ident.span, ident.text.clone()));
      }
      // Recorded whether or not it warned: a suppressed shadow still binds,
      // and a third `=` on the same name is a fresh site.
      bound.push(&ident.text);
    }
  }
}
