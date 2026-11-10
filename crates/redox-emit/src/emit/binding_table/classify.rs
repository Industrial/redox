use crate::emit::binding_kind::BindingKind;
use crate::emit::binding_table::is_upper_case::is_upper_case;
use crate::emit::is_shareable::is_shareable;
use redox_ast::{ClauseBody, FnDecl};

/// `decl`'s kind, or `None` when it is not a parameterless binding at all.
///
/// Plan 59 (`docs/53 - 59 - Value Bindings.md`) leaf 1. A binding is **one
/// clause, no patterns, no `where` locals and a single expression**. A
/// multi-equation or guard-laddered declaration is a function however few
/// parameters it has, because a `const` has nowhere to put a second equation
/// and no way to spell a guard.
///
/// The three tests are each syntactic and each decidable from the source text
/// alone, which is doc 03 §2's admission test 1 — the shape of the body
/// ([`is_shareable`]), the case of the name ([`is_upper_case`]), and the
/// absence of parameters. Nothing here evaluates anything, because doc 03
/// §13.3 declines transpile-time evaluation permanently.
pub(crate) fn classify(decl: &FnDecl) -> Option<BindingKind> {
  if !decl.sig.params.is_empty() {
    return None;
  }
  let [clause] = decl.clauses.as_slice() else {
    return None;
  };
  if !clause.patterns.is_empty() {
    return None;
  }
  let ClauseBody::Plain(block) = &clause.body else {
    return Some(BindingKind::Called);
  };
  let shared = clause.wheres.is_empty()
    && block.stmts.len() == 1
    && is_shareable(&block.stmts[0].expr)
    && is_upper_case(&decl.name.text);
  Some(if shared {
    BindingKind::Shared
  } else {
    BindingKind::Called
  })
}
