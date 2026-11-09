use redox_ast::Pattern;

/// Whether this pattern is safe beside another without parentheses.
///
/// An `Apply` with **no** arguments — `None`, `Close` — is a bare name and
/// needs nothing, which is why the arity is tested rather than the variant.
pub(crate) fn stands_alone(pattern: &Pattern) -> bool {
  match pattern {
    Pattern::Apply { args, .. } => args.is_empty(),
    Pattern::Binding(_)
    | Pattern::Literal { .. }
    | Pattern::Wildcard(_)
    | Pattern::Tuple { .. }
    | Pattern::Slice { .. }
    | Pattern::Rest(_)
    | Pattern::NamedRest { .. } => true,
    Pattern::Range { .. } | Pattern::Record { .. } | Pattern::As { .. } | Pattern::Or(_) => false,
  }
}
