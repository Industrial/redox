use crate::emit::binding_kind::BindingKind;
use crate::emit::binding_scope::BindingScope;

/// How a bare name or a path reads at a use site.
///
/// Plan 59 (`docs/53 - 59 - Value Bindings.md`) leaf 3. The author writes the
/// name alone; which Rust that becomes depends on what the name was declared
/// as in **this file**:
///
/// ```text
/// MAX                   ->  MAX              a literal body: plan 16's `const`
/// sizes                 ->  sizes()          a body that does work: still a `fn`
/// inner::hello          ->  inner::hello()
/// std::f64::consts::PI  ->  unchanged        not ours
/// ```
///
/// Everything the table does not know is returned verbatim, which is plan 56
/// §11.1's safety argument holding: a name reached through `import`, a `where`
/// local, an equation binder, a lambda parameter and a bodiless `module`'s
/// path are all absent from the table, and absent means *exactly as before*.
///
/// # Why the call is restored here rather than removed at the declaration
///
/// A `Called` binding keeps emitting `fn name() -> T`, so the use site is the
/// only place the elided call can come back. The alternative — rewriting the
/// declaration into something callable without parentheses — does not exist
/// in Rust.
///
/// And §11.10 is why a bare name is the **value** rather than the function
/// item: the two readings are indistinguishable from syntax, so the one left
/// unspelled has to be the rarer one. A factory is the nullary lambda, which
/// already parses and already emits a closure.
pub(crate) fn binding_text(name: &str) -> String {
  match BindingScope::kind_of(name) {
    Some(BindingKind::Called) => format!("{name}()"),
    Some(BindingKind::Shared) | None => name.to_string(),
  }
}
