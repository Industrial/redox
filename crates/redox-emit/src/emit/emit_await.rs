use crate::emit::receiver::receiver;
use crate::emit::unparen::unparen;
use redox_ast::{Expr, Pattern};

/// Lower doc 30's await-bind.
///
/// ```text
/// user <- fetch id          let user = fetch(id).await;
/// <- flush ()           →   flush().await;
/// (a, b) <- load id         let (a, b) = load(id).await;
/// ```
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
pub(crate) fn emit_await(pattern: &Option<Pattern>, value: &Expr, indent: usize) -> String {
  let awaited = format!("{}.await", receiver(unparen(value), indent));
  match pattern {
    Some(pattern) => format!("let {} = {awaited};", pattern.rust_text()),
    None => format!("{awaited};"),
  }
}
