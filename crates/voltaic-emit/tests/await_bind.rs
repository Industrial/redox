//! Doc 30's await-bind — `<-` means `await` and nothing else.
//!
//! Haskell's monadic bind for the monad Voltaic has: `do` binds an effect's
//! result to a name and this binds a future's. Doc 30 §1.1 is the argument —
//! doc 18 §3.1 had given `<-` four jobs and kept Haskell's meaning in only
//! one of them, so a reader's first guess was wrong three times in four.
//!
//! This file replaces `await_postfix.rs`. Postfix `.await` is deleted: every
//! await binds, which is doc 30 §2.3's trade and §7's decision 2.

use voltaic_emit::emit;
use voltaic_parser::parse_source;

fn xrs(src: &str) -> String {
  emit(&parse_source(src).expect("should parse"))
}

/// The body of `fn main`, dedented. `main` is not `async`, so rustc would
/// reject these — which is doc 30 §2.2's division of labour and not this
/// file's business: these assert the **lowering**, and the `async` marker is
/// row 39's.
fn body(src: &str) -> String {
  // Plan 55 §1.6 requires a `()`-returning body to end in `()`, so the wrapper
  // writes one and the `();` it emits is dropped again — these assertions are
  // about `src`.
  let out = xrs(&format!("main : ()\nmain =\n{src}  ()\n"));
  let mut lines: Vec<&str> = out
    .lines()
    .skip(1)
    .take_while(|l| *l != "}")
    .map(|l| l.strip_prefix("    ").unwrap_or(l))
    .collect();
  assert_eq!(lines.pop(), Some("();"), "{out}");
  lines.join("\n")
}

// --- the three forms ------------------------------------------------------

#[test]
fn an_await_bind_emits_a_let_and_a_dot_await() {
  assert_eq!(body("  user <- fetch id\n"), "let user = fetch(id).await;");
}

/// The effect-only form binds nothing, and emits a statement rather than a
/// `let` — doc 30 §7's decision 6. `let _ = f().await;` would bind a discard
/// where the author wrote nothing to bind.
#[test]
fn an_await_with_no_binder_emits_a_statement() {
  assert_eq!(body("  <- flush ()\n"), "flush().await;");
}

/// The left side is a **pattern**, because plan 33 made a binding's left side
/// one and this is a binding.
#[test]
fn the_binder_is_a_pattern() {
  assert_eq!(
    body("  (a, b) <- load id\n"),
    "let (a, b) = load(id).await;"
  );
  assert_eq!(
    body("  Point { x, y } <- fetch ()\n"),
    "let Point { x, y } = fetch().await;"
  );
}

// --- the receiver rule ----------------------------------------------------

/// `.await` is a postfix and binds tighter than every operator, so an
/// operator-valued future is parenthesised — the same rule every other
/// postfix uses, reached through `receiver` rather than answered twice.
#[test]
fn an_operator_valued_future_keeps_its_parentheses() {
  let out = body("  n <- count () ?? 0\n");
  assert!(out.starts_with("let n = (match count() {"), "{out}");
  assert!(out.ends_with("}).await;"), "{out}");
}

/// And a call needs none, which is the common case.
#[test]
fn a_call_needs_no_parentheses() {
  assert_eq!(body("  n <- f x y\n"), "let n = f(x, y).await;");
}

// --- what `<-` is not -----------------------------------------------------

/// A binding is still a binding. The two can both open with a pattern and are
/// told apart by the token that ends it, which is why `try_await`'s scan stops
/// at an `=` at depth zero.
#[test]
fn a_binding_is_not_an_await() {
  assert_eq!(body("  x = f y\n"), "let x = f(y);");
  assert_eq!(body("  (a, b) = p\n"), "let (a, b) = p;");
}

/// Postfix `.await` is **deleted**. Asserted as a parse failure, because the
/// spelling worked for fifteen plans and a reader may still try it.
#[test]
fn postfix_await_no_longer_parses() {
  assert!(
    parse_source("main : ()\nmain =\n  r = (fetch id).await\n").is_err(),
    "doc 30 deletes postfix `.await`: every await binds"
  );
}

/// Plan 15's surviving test, moved here from `await_postfix.rs`.
///
/// `?.` is one token — doc 03 §12.3's optional chain — so `x?.await` was never
/// `?` then `.await`. Doc 30 §2.4 is why this matters *more* now, not less:
/// deleting `.await` puts `?` beside an awaited value far more often, and the
/// shape that avoids the collision is a second binding.
#[test]
fn question_dot_is_one_token_and_not_two_postfixes() {
  assert!(
    parse_source("main : ()\nmain =\n  r = (fetch id)?.await\n").is_err(),
    "`?.` is doc 03 §12.3's operator, not `?` then `.`"
  );
}
