use crate::emit::emit_stmts::emit_stmts;
use crate::emit::guards::emit_guards;
use crate::emit::sink::Sink;
use redox_ast::ClauseBody;

/// Emit a clause's right-hand side at `indent` spaces.
///
/// Doc 18 §2.3 gave a clause two shapes — `= body` and a `|` guard ladder —
/// and both appear in the two places a clause body is emitted: a function with
/// one irrefutable clause, and each arm of the `match` the other clauses
/// become. This is the one place that knows the difference, so neither caller
/// has to.
pub(crate) fn emit_body(out: &mut Sink<'_>, body: &ClauseBody, indent: usize, tail: bool) {
  match body {
    ClauseBody::Plain(block) => emit_stmts(out, &block.stmts, indent, tail),
    ClauseBody::Guarded(guards) => emit_guards(out, guards, indent, tail),
  }
}
