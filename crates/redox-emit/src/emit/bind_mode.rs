/// What `<-` means in the block being emitted.
///
/// Plan 60 (`docs/54 - 60 - The Bind Arrow.md`) leaf 3. `<-` is monadic bind
/// and the **enclosing block modifier** says what binding is:
///
/// ```text
/// async    ->  await the sub-effect
/// try      ->  the `?`
/// ```
///
/// That is do-notation earned rather than imitated, and the property worth
/// having is that it generalises: a future modifier for another effect type
/// gets the token for free.
///
/// Before this row the token meant `await` unconditionally, whatever block it
/// sat in — so `<-` inside a `try` emitted `.await` inside a non-async
/// closure, which rustc rejects. The mode is what fixes that, and it is a
/// property of the *context* rather than of the token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum BindMode {
  /// Inside an `async` function or block: `<-` awaits.
  ///
  /// The default, because that is what `<-` meant before this row and what it
  /// still means in the position doc 30 gave it — five of
  /// `examples/async.sdx`'s six uses are in an `async fn` body with no block
  /// at all, which §11.1 is the finding for.
  #[default]
  Await,
  /// Inside a `try` block: `<-` is the `?`.
  Question,
}
