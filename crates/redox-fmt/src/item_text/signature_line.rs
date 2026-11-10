use redox_ast::FnDecl;

/// A function's signature line, verbatim from the source — or `None` when
/// the declaration has none.
///
/// # Why this is sliced and not printed
///
/// `parser::generalise` rewrites a signature's types **in place**: for each
/// type variable it replaces every occurrence with the Rust spelling, so
/// `largest : Ord t => Option<t> -> t` is stored as
/// `Ord T => Option<T> -> T` and the author's lowercase `t` is gone. Doc 18
/// §5.3 makes the case of the first letter the whole difference between a
/// variable and a concrete type, which is why the parser does it and why it
/// cannot be undone from the tree: `T` is a legal concrete name.
///
/// So printing a signature from the AST would rewrite every generic
/// declaration in the corpus into Rust's convention — `generic.sdx`,
/// `class.sdx`, `module.sdx` and `async.sdx` all have one. That is a change to
/// the **language's surface**, not a formatting choice, so criterion 7 applies
/// and the line is passed through.
///
/// §10 records what would lift this: the AST keeping the source spelling and
/// the emitter applying `rust_generic` when it writes, which moves
/// `generalise`'s third step from parse time to emit time.
///
/// # Finding the line
///
/// The declaration's span starts at the signature — at `export` when there is
/// one — and the signature is one line. The first clause's span starts *after*
/// the function's name on the equation line, which is enough to tell the two
/// shapes apart: if the clause begins past the first line's end, the first
/// line is a signature.
pub(crate) fn signature_line(decl: &FnDecl, source: &str) -> Option<String> {
  let start = decl.span.0;
  let end = source[start..]
    .find('\n')
    .map_or(source.len(), |at| start + at);
  let line = source[start..end].trim_end().to_string();
  let Some(clause) = decl.clauses.first() else {
    // A signature with **no equation**: a `class` member's declaration, like
    // `area : &t -> f64` above a default-less method. The whole declaration
    // is the signature line.
    //
    // The first version bailed here, because it read the first clause's span
    // to find out where the signature ended. That dropped the member
    // entirely — `examples/class.sdx` lost `area : &t -> f64` and emitted a
    // line of bare indentation in its place.
    return Some(line);
  };
  if clause.span.0 <= end {
    // The equation is on the first line, so there is no separate signature.
    return None;
  }
  Some(line)
}
