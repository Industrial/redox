use crate::lint::allow_pragma::allow_pragma;
use crate::lint::program_walk::ProgramWalk;
use redox_ast::{Item, Stmt};
use redox_diag::{Lint, Span};

/// Collects the byte ranges one lint is suppressed over — plan 53 leaf 4.
///
/// One range per construct that carries a suppression pragma in its leading
/// trivia: a statement's own span, or a declaration's whole span. A site
/// inside any of them is silenced. The lint is a field rather than a
/// hard-coded name, so `#[redox::allow(unwrap)]` silences `RDX1001` and
/// nothing else.
pub(crate) struct PragmaWalk {
  /// The lint whose suppressions are being collected.
  pub(crate) lint: Lint,
  /// The spans it is suppressed over, one per pragma-carrying construct.
  pub(crate) found: Vec<Span>,
}

impl ProgramWalk for PragmaWalk {
  /// A pragma leading a declaration suppresses the declaration's **whole
  /// span** — `FnDecl::span` runs from the signature line to the end of the
  /// last clause, so one attribute above it covers every site inside.
  fn item(&mut self, item: &Item) {
    if redox_ast::leading(item)
      .iter()
      .any(|t| allow_pragma(t, self.lint))
    {
      self.found.push(item.span());
    }
  }

  /// A pragma leading a statement suppresses that statement's **own span**,
  /// which runs to the end of everything the statement owns — so an attribute
  /// above an `if` covers its branches, and one above a `for` covers its
  /// body.
  fn stmts(&mut self, stmts: &[Stmt]) {
    for stmt in stmts {
      if stmt.leading.iter().any(|t| allow_pragma(t, self.lint)) {
        self.found.push(stmt.span);
      }
    }
  }
}
