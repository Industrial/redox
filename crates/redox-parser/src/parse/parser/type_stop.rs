/// How far a type capture should read.
///
/// Doc 18 §2.1 reads a constructor's and an error variant's payload by
/// **juxtaposition** — `Invalid String u32` is two fields — while a
/// parameter or a return type runs to its own terminator. The same scan
/// serves both, and this says which is wanted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypeStop {
  /// Read to the first terminator: `,`, `)`, `}`, `|`, `=`, `as`, a string
  /// literal, a line break, or a signature's `->`.
  ///
  /// What a parameter, a field after `::`, a return type and an alias want.
  Greedy,
  /// Read **one** type and stop at the next whitespace-separated one.
  ///
  /// What a juxtaposed payload list wants. Without it `Invalid String u32`
  /// is a single field spelled `String u32` — which rustc rejects, but only
  /// after the author has been told nothing useful.
  Atom,
}
