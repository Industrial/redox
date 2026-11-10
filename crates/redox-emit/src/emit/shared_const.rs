use redox_ast::{ClauseBody, ConstDecl, FnDecl};

/// The `const` item a shared binding is emitted as.
///
/// Plan 59 (`docs/53 - 59 - Value Bindings.md`) leaf 2, and the shape of this
/// function *is* the leaf's decision: a shared binding reuses plan 16's
/// emission rather than growing a second path to the same Rust, so what this
/// does is build the node `emit_const` already knows how to write and hand it
/// over.
///
/// `is_static` is always `false`. Doc 03 §13.4 states no rule selecting
/// `static` by type, so the keyword keeps its explicit spelling, and inventing
/// a rule here would make its meaning conditional on something the author did
/// not write.
///
/// `None` only for a declaration the binding table would not have classified
/// as shared in the first place; the two agree by construction, and the
/// duplicated shape test is the price of not threading the classification
/// through the item dispatch.
pub(crate) fn shared_const(decl: &FnDecl) -> Option<ConstDecl> {
  let ty = decl.sig.ret.clone()?;
  let [clause] = decl.clauses.as_slice() else {
    return None;
  };
  let ClauseBody::Plain(block) = &clause.body else {
    return None;
  };
  let [stmt] = block.stmts.as_slice() else {
    return None;
  };
  Some(ConstDecl {
    name: decl.name.clone(),
    ty,
    value: stmt.expr.clone(),
    is_static: false,
    visibility: decl.visibility.clone(),
    span: decl.span,
    leading: decl.leading.clone(),
  })
}
