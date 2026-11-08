//! An ordinary Rust entry point over Redox-generated code.
//!
//! The `include!` is the whole mechanism. `build.rs` wrote
//! `$OUT_DIR/greeting.rs` from `src/greeting.sdx`, and this pulls it in — so
//! `greet` is an ordinary Rust function and the rest of the crate cannot tell
//! the difference.
//!
//! `include!` is what makes doc 02 §6.6 option (a) work rather than merely
//! compile: it keeps the path to the generated file, so rustc's spans point
//! into `$OUT_DIR/greeting.rs` and plan 06's map can carry them back to
//! `src/greeting.sdx`.
//!
//! **Why at the crate root and not inside `mod greeting`.** A module would be
//! the nicer shape, and it does not work yet: Redox emits `fn greet(…)` with
//! no visibility, so inside `mod greeting { include!(…) }` every item is
//! private to that module and unreachable from here. Doc 03 §9.6 specifies the
//! spelling that fixes it — `export` lowers to `pub`, `export(crate)` to
//! `pub(crate)` — and **plan 24** (`data-declarations`) implements it. Until
//! then, include at the root, where private items are still crate-visible.
//!
//! Plan 09 found this by building this crate, which is the argument for a
//! worked project over a unit test: nothing else would have noticed that
//! Redox cannot yet produce a reachable item inside a module.
//!
//! A crate can hold both kinds of source. This file is plain Rust.

include!(concat!(env!("OUT_DIR"), "/greeting.rs"));

fn main() {
  println!("{}", greet("world"));
  println!("{}", twice(21));
}
