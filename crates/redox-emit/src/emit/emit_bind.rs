use crate::emit::bind_mode::BindMode;
use crate::emit::bind_scope::BindScope;
use crate::emit::receiver::receiver;
use crate::emit::unparen::unparen;
use redox_ast::{Expr, Pattern};

/// Lower `<-`, monadic bind, as the **enclosing block modifier** reads it.
///
/// ```text
/// async fn / async block        try block
/// ─────────────────────────     ──────────────────────
/// user <- fetch id              a <- parse x
///   let user = fetch(id).await;   let a = parse(x)?;
///
/// <- flush ()                   <- check ()
///   flush().await;                check()?;
/// ```
///
/// Plan 60 (`docs/54 - 60 - The Bind Arrow.md`) leaf 3. This function was
/// `emit_await` and the token meant `await` unconditionally, which is why it
/// emitted `.await` inside a `try` block's non-async closure — rust rejects
/// that, and no corpus file had a `<-` inside a `try` to catch it. The name
/// changed with the meaning: a function called `emit_await` that sometimes
/// writes `?` is the stale-name hazard this project keeps recording.
///
/// The value takes the **receiver** treatment, because `.await` is a postfix
/// and binds tighter than every operator — so `n <- a ?? b` must be
/// `(match a { … }).await` and not `a ?? b.await`. That is the rule every
/// other postfix in the emitter already uses, and reaching it through
/// `receiver` is what keeps this from being a second answer to the same
/// question.
///
/// No binder emits a statement rather than a `let` — doc 30 §7's decision 6.
/// `let _ = f().await;` would bind a discard where the author wrote nothing to
/// bind, and Rust spells the effect-only form `f().await;`.
pub(crate) fn emit_bind(pattern: &Option<Pattern>, value: &Expr, indent: usize) -> String {
  let postfix = match BindScope::mode() {
    BindMode::Await => ".await",
    BindMode::Question => "?",
  };
  let bound = format!("{}{postfix}", receiver(unparen(value), indent));
  match pattern {
    Some(pattern) => format!("let {} = {bound};", pattern.rust_text()),
    None => format!("{bound};"),
  }
}
